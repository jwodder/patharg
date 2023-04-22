#![cfg_attr(docsrs, feature(doc_cfg))]
//! Treat "-" (hyphen/dash) arguments as stdin/stdout
//!
//! Most CLI commands that take file paths as arguments follow the convention
//! of treating a path of `-` (a single hyphen/dash) as referring to either
//! standard input or standard output (depending on whether the path is read
//! from or written to).  The `patharg` crate lets your programs follow this
//! convention too: it provides [`InputArg`] and [`OutputArg`] types that wrap
//! command-line arguments, with methods for reading from/writing to either the
//! given path or — if the argument is just a hyphen — the appropriate standard
//! stream.
//!
//! `InputArg` and `OutputArg` implement `From<OsString>` and `From<String>`,
//! so you can use them seamlessly with your favorite Rust source of
//! command-line arguments, be it [`clap`][], [`lexopt`][], plain old
//! [`std::env::args`]/[`std::env::args_os`], or whatever else is out there.
//! The source repository contains examples of two of these:
//!
//! - [`examples/flipcase.rs`][flipcase] and
//!   [`examples/tokio-flipcase.rs`][tokio-flipcase] show how to use this crate
//!   with `clap`.
//! - [`examples/revchars.rs`][revchars] and
//!   [`examples/tokio-revchars.rs`][tokio-revchars] show how to use this crate
//!   with `lexopt`.
//!
//! [`clap`]: https://crates.io/crates/clap
//! [`lexopt`]: https://crates.io/crates/lexopt
//! [flipcase]: https://github.com/jwodder/patharg/blob/master/examples/flipcase.rs
//! [tokio-flipcase]: https://github.com/jwodder/patharg/blob/master/examples/tokio-flipcase.rs
//! [revchars]: https://github.com/jwodder/patharg/blob/master/examples/revchars.rs
//! [tokio-revchars]: https://github.com/jwodder/patharg/blob/master/examples/tokio-revchars.rs
//!
//! Comparison with clio
//! ====================
//!
//! The only other library I am aware of that provides similar functionality to
//! `patharg` is [`clio`][].  Compared to `clio`, `patharg` aims to be a much
//! simpler, smaller library that doesn't try to be too clever.  Major
//! differences between the libraries include:
//!
//! - When a `clio` path instance is created, `clio` will either (depending on
//!   the type used) open the path immediately — which can lead to empty files
//!   being needlessly left behind if an output file is constructed during
//!   argument processing but an error occurs before the file is actually used
//!   — or else check that the path can be opened — which is vulnerable to
//!   TOCTTOU bugs.  `patharg` does no such thing.
//!
//! - `clio` supports reading from & writing to HTTP(S) URLs and has special
//!   treatment for FIFOs.  `patharg` sees no need for such excesses.
//!
//! - `patharg` has a feature for allowing async I/O with [`tokio`].  `clio`
//!   does not.
//!
//! [`clio`]: https://crates.io/crates/clio
//! [`tokio`]: https://crates.io/crates/tokio

use cfg_if::cfg_if;
use either::Either;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io::{self, BufRead, BufReader, Read, StdinLock, StdoutLock, Write};
use std::path::{Path, PathBuf};

cfg_if! {
    if #[cfg(feature = "serde")] {
        use serde::de::Deserializer;
        use serde::ser::Serializer;
        use serde::{Deserialize, Serialize};
    }
}

cfg_if! {
    if #[cfg(feature = "tokio")] {
        use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncBufReadExt};
        use tokio_util::either::Either as AsyncEither;
        use tokio_stream::wrappers::LinesStream;
    }
}

/// An input path that can refer to either standard input or a file system path
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum InputArg {
    /// Refers to standard input.
    ///
    /// This is the variant returned by `InputArg::default()`.
    #[default]
    Stdin,

    /// Refers to a file system path (stored in `.0`)
    Path(PathBuf),
}

impl InputArg {
    /// Construct an `InputArg` from a string, usually one taken from
    /// command-line arguments.  If the string equals `"-"` (i.e., it contains
    /// only a single hyphen/dash), [`InputArg::Stdin`] is returned; otherwise,
    /// an [`InputArg::Path`] is returned.
    ///
    /// # Example
    ///
    /// ```
    /// use patharg::InputArg;
    /// use std::path::PathBuf;
    ///
    /// let p1 = InputArg::from_arg("-");
    /// assert_eq!(p1, InputArg::Stdin);
    ///
    /// let p2 = InputArg::from_arg("./-");
    /// assert_eq!(p2, InputArg::Path(PathBuf::from("./-")));
    /// ```
    pub fn from_arg<S: Into<PathBuf>>(arg: S) -> InputArg {
        let arg = arg.into();
        if arg == Path::new("-") {
            InputArg::Stdin
        } else {
            InputArg::Path(arg)
        }
    }

    /// Returns true if the input arg is the `Stdin` variant of `InputArg`.
    ///
    /// # Example
    ///
    /// ```
    /// use patharg::InputArg;
    ///
    /// let p1 = InputArg::from_arg("-");
    /// assert!(p1.is_stdin());
    ///
    /// let p2 = InputArg::from_arg("file.txt");
    /// assert!(!p2.is_stdin());
    /// ```
    pub fn is_stdin(&self) -> bool {
        self == &InputArg::Stdin
    }

    /// Returns true if the input arg is the `Path` variant of `InputArg`.
    ///
    /// # Example
    ///
    /// ```
    /// use patharg::InputArg;
    ///
    /// let p1 = InputArg::from_arg("-");
    /// assert!(!p1.is_path());
    ///
    /// let p2 = InputArg::from_arg("file.txt");
    /// assert!(p2.is_path());
    /// ```
    pub fn is_path(&self) -> bool {
        matches!(self, InputArg::Path(_))
    }

    /// Retrieve a reference to the inner [`PathBuf`].  If the input arg is
    /// the `Stdin` variant, this returns `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use patharg::InputArg;
    /// use std::path::PathBuf;
    ///
    /// let p1 = InputArg::from_arg("-");
    /// assert_eq!(p1.path_ref(), None);
    ///
    /// let p2 = InputArg::from_arg("file.txt");
    /// assert_eq!(p2.path_ref(), Some(&PathBuf::from("file.txt")));
    /// ```
    pub fn path_ref(&self) -> Option<&PathBuf> {
        match self {
            InputArg::Stdin => None,
            InputArg::Path(p) => Some(p),
        }
    }

    /// Retrieve a mutable reference to the inner [`PathBuf`].  If the input
    /// arg is the `Stdin` variant, this returns `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use patharg::InputArg;
    /// use std::path::PathBuf;
    ///
    /// let mut p1 = InputArg::from_arg("-");
    /// assert_eq!(p1.path_mut(), None);
    ///
    /// let mut p2 = InputArg::from_arg("file.txt");
    /// assert_eq!(p2.path_mut(), Some(&mut PathBuf::from("file.txt")));
    /// ```
    pub fn path_mut(&mut self) -> Option<&mut PathBuf> {
        match self {
            InputArg::Stdin => None,
            InputArg::Path(p) => Some(p),
        }
    }

    /// Consume the input arg and return the inner [`PathBuf`].  If the input
    /// arg is the `Stdin` variant, this returns `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use patharg::InputArg;
    /// use std::path::PathBuf;
    ///
    /// let p1 = InputArg::from_arg("-");
    /// assert_eq!(p1.into_path(), None);
    ///
    /// let p2 = InputArg::from_arg("file.txt");
    /// assert_eq!(p2.into_path(), Some(PathBuf::from("file.txt")));
    /// ```
    pub fn into_path(self) -> Option<PathBuf> {
        match self {
            InputArg::Stdin => None,
            InputArg::Path(p) => Some(p),
        }
    }

    /// Open the input arg for reading.
    ///
    /// If the input arg is the `Stdin` variant, this returns a locked
    /// reference to stdin.  Otherwise, if the path arg is a `Path` variant,
    /// the given path is opened for reading.
    ///
    /// The returned reader implements [`std::io::BufRead`].
    ///
    /// # Errors
    ///
    /// Has the same error conditions as [`std::fs::File::open`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use patharg::InputArg;
    /// use std::env::args_os;
    /// use std::io::{self, Read};
    ///
    /// fn main() -> io::Result<()> {
    ///     let infile = args_os().nth(1)
    ///                           .map(InputArg::from_arg)
    ///                           .unwrap_or_default();
    ///     let mut f = infile.open()?;
    ///     let mut buffer = [0; 16];
    ///     let n = f.read(&mut buffer)?;
    ///     println!("First {} bytes: {:?}", n, &buffer[..n]);
    ///     Ok(())
    /// }
    /// ```
    pub fn open(&self) -> io::Result<InputArgReader> {
        Ok(match self {
            InputArg::Stdin => Either::Left(io::stdin().lock()),
            InputArg::Path(p) => Either::Right(BufReader::new(fs::File::open(p)?)),
        })
    }

    /// Read the entire contents of the input arg into a bytes vector.
    ///
    /// If the input arg is the `Stdin` variant, the entire contents of stdin
    /// are read.  Otherwise, if the input arg is a `Path` variant, the
    /// contents of the given path are read.
    ///
    /// # Errors
    ///
    /// Has the same error conditions as [`std::io::Read::read_to_end`] and
    /// [`std::fs::read`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use patharg::InputArg;
    /// use std::env::args_os;
    /// use std::io;
    ///
    /// fn main() -> io::Result<()> {
    ///     let infile = args_os().nth(1)
    ///                           .map(InputArg::from_arg)
    ///                           .unwrap_or_default();
    ///     let input = infile.read()?;
    ///     println!("Read {} bytes from input", input.len());
    ///     Ok(())
    /// }
    /// ```
    pub fn read(&self) -> io::Result<Vec<u8>> {
        match self {
            InputArg::Stdin => {
                let mut vec = Vec::new();
                io::stdin().lock().read_to_end(&mut vec)?;
                Ok(vec)
            }
            InputArg::Path(p) => fs::read(p),
        }
    }

    /// Read the entire contents of the input arg into a string.
    ///
    /// If the input arg is the `Stdin` variant, the entire contents of stdin
    /// are read.  Otherwise, if the input arg is a `Path` variant, the
    /// contents of the given path are read.
    ///
    /// # Errors
    ///
    /// Has the same error conditions as [`std::io::read_to_string`] and
    /// [`std::fs::read_to_string`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use patharg::InputArg;
    /// use std::env::args_os;
    /// use std::io;
    ///
    /// fn main() -> io::Result<()> {
    ///     let infile = args_os().nth(1)
    ///                           .map(InputArg::from_arg)
    ///                           .unwrap_or_default();
    ///     let input = infile.read_to_string()?;
    ///     println!("Read {} characters from input", input.len());
    ///     Ok(())
    /// }
    /// ```
    pub fn read_to_string(&self) -> io::Result<String> {
        match self {
            InputArg::Stdin => io::read_to_string(io::stdin().lock()),
            InputArg::Path(p) => fs::read_to_string(p),
        }
    }

    /// Return an iterator over the lines of the input arg.
    ///
    /// If the input arg is the `Stdin` variant, this locks stdin and returns
    /// an iterator over its lines; the lock is released once the iterator is
    /// dropped.  Otherwise, if the input arg is a `Path` variant, the given
    /// path is opened for reading, and an iterator over its lines is returned.
    ///
    /// The returned iterator yields instances of `std::io::Result<String>`,
    /// where each individual item has the same error conditions as
    /// [`std::io::BufRead::read_line()`].
    ///
    /// # Errors
    ///
    /// Has the same error conditions as [`InputArg::open()`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use patharg::InputArg;
    /// use std::env::args_os;
    /// use std::io;
    ///
    /// fn main() -> io::Result<()> {
    ///     let infile = args_os().nth(1)
    ///                           .map(InputArg::from_arg)
    ///                           .unwrap_or_default();
    ///     for (i, r) in infile.lines()?.enumerate() {
    ///         let line = r?;
    ///         println!("Line {} is {} characters long.", i + 1, line.len());
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub fn lines(&self) -> io::Result<Lines> {
        Ok(self.open()?.lines())
    }
}

#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
impl InputArg {
    /// Asynchronously open the input arg for reading.
    ///
    /// If the input arg is the `Stdin` variant, this returns a reference to
    /// stdin.  Otherwise, if the path arg is a `Path` variant, the given path
    /// is opened for reading.
    ///
    /// The returned reader implements [`tokio::io::AsyncRead`].
    ///
    /// # Errors
    ///
    /// Has the same error conditions as [`tokio::fs::File::open`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use patharg::InputArg;
    /// use std::env::args_os;
    /// use tokio::io::AsyncReadExt;
    ///
    /// #[tokio::main]
    /// async fn main() -> std::io::Result<()> {
    ///     let infile = args_os().nth(1)
    ///                           .map(InputArg::from_arg)
    ///                           .unwrap_or_default();
    ///     let mut f = infile.async_open().await?;
    ///     let mut buffer = [0; 16];
    ///     let n = f.read(&mut buffer).await?;
    ///     println!("First {} bytes: {:?}", n, &buffer[..n]);
    ///     Ok(())
    /// }
    /// ```
    pub async fn async_open(&self) -> io::Result<AsyncInputArgReader> {
        Ok(match self {
            InputArg::Stdin => AsyncEither::Left(tokio::io::stdin()),
            InputArg::Path(p) => AsyncEither::Right(tokio::fs::File::open(p).await?),
        })
    }

    /// Asynchronously read the entire contents of the input arg into a bytes
    /// vector.
    ///
    /// If the input arg is the `Stdin` variant, the entire contents of stdin
    /// are read.  Otherwise, if the input arg is a `Path` variant, the
    /// contents of the given path are read.
    ///
    /// # Errors
    ///
    /// Has the same error conditions as
    /// [`tokio::io::AsyncReadExt::read_to_end`] and [`tokio::fs::read`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use patharg::InputArg;
    /// use std::env::args_os;
    ///
    /// #[tokio::main]
    /// async fn main() -> std::io::Result<()> {
    ///     let infile = args_os().nth(1)
    ///                           .map(InputArg::from_arg)
    ///                           .unwrap_or_default();
    ///     let input = infile.async_read().await?;
    ///     println!("Read {} bytes from input", input.len());
    ///     Ok(())
    /// }
    /// ```
    pub async fn async_read(&self) -> io::Result<Vec<u8>> {
        match self {
            InputArg::Stdin => {
                let mut vec = Vec::new();
                tokio::io::stdin().read_to_end(&mut vec).await?;
                Ok(vec)
            }
            InputArg::Path(p) => tokio::fs::read(p).await,
        }
    }

    /// Asynchronously read the entire contents of the input arg into a string.
    ///
    /// If the input arg is the `Stdin` variant, the entire contents of stdin
    /// are read.  Otherwise, if the input arg is a `Path` variant, the
    /// contents of the given path are read.
    ///
    /// # Errors
    ///
    /// Has the same error conditions as
    /// [`tokio::io::AsyncReadExt::read_to_string`] and
    /// [`tokio::fs::read_to_string`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use patharg::InputArg;
    /// use std::env::args_os;
    ///
    /// #[tokio::main]
    /// async fn main() -> std::io::Result<()> {
    ///     let infile = args_os().nth(1)
    ///                           .map(InputArg::from_arg)
    ///                           .unwrap_or_default();
    ///     let input = infile.async_read_to_string().await?;
    ///     println!("Read {} characters from input", input.len());
    ///     Ok(())
    /// }
    /// ```
    pub async fn async_read_to_string(&self) -> io::Result<String> {
        match self {
            InputArg::Stdin => {
                let mut s = String::new();
                tokio::io::stdin().read_to_string(&mut s).await?;
                Ok(s)
            }
            InputArg::Path(p) => tokio::fs::read_to_string(p).await,
        }
    }

    /// Return a stream over the lines of the input arg.
    ///
    /// If the input arg is the `Stdin` variant, this returns a stream over the
    /// lines of stdin.  Otherwise, if the input arg is a `Path` variant, the
    /// given path is opened for reading, and a stream over its lines is
    /// returned.
    ///
    /// The returned stream yields instances of `std::io::Result<String>`,
    /// where each individual item has the same error conditions as
    /// [`tokio::io::AsyncBufReadExt::read_line()`].
    ///
    /// # Errors
    ///
    /// Has the same error conditions as [`InputArg::async_open()`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use patharg::InputArg;
    /// use std::env::args_os;
    /// use tokio_stream::StreamExt;
    ///
    /// #[tokio::main]
    /// async fn main() -> std::io::Result<()> {
    ///     let infile = args_os().nth(1)
    ///                           .map(InputArg::from_arg)
    ///                           .unwrap_or_default();
    ///     let mut i = 1;
    ///     let mut stream = infile.async_lines().await?;
    ///     while let Some(r) = stream.next().await {
    ///         let line = r?;
    ///         println!("Line {} is {} characters long.", i, line.len());
    ///         i += 1;
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn async_lines(&self) -> io::Result<AsyncLines> {
        Ok(LinesStream::new(
            tokio::io::BufReader::new(self.async_open().await?).lines(),
        ))
    }
}

impl fmt::Display for InputArg {
    /// Displays [`InputArg::Stdin`] as `-` (a single hyphen/dash) or as
    /// `<stdin>` if the `{:#}` format is used.  Always displays
    /// [`InputArg::Path`] using [`std::path::Path::display()`].
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // IMPORTANT: The default Display of Stdin has to round-trip back
            // to Stdin so that InputArg will work properly when used with
            // clap's `default_value_t`.
            InputArg::Stdin => {
                if f.alternate() {
                    write!(f, "<stdin>")
                } else {
                    write!(f, "-")
                }
            }
            InputArg::Path(p) => write!(f, "{}", p.display()),
        }
    }
}

impl<S: Into<PathBuf>> From<S> for InputArg {
    /// Convert a string to a [`InputArg`] using [`InputArg::from_arg()`].
    fn from(s: S) -> InputArg {
        InputArg::from_arg(s)
    }
}

impl From<InputArg> for OsString {
    /// Converts an input arg back to an `OsString`: `InputArg::Stdin` becomes
    /// `"-"`, and `InputArg::Path(p)` becomes `p.into()`.
    fn from(arg: InputArg) -> OsString {
        match arg {
            InputArg::Stdin => OsString::from("-"),
            InputArg::Path(p) => p.into(),
        }
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl Serialize for InputArg {
    /// Serializes [`InputArg::Stdin`] as `"-"` (a string containing a single
    /// hyphen/dash).  Serializes [`InputArg::Path`] as the inner [`PathBuf`];
    /// this will fail if the path is not valid UTF-8.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            InputArg::Stdin => "-".serialize(serializer),
            InputArg::Path(p) => p.serialize(serializer),
        }
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<'de> Deserialize<'de> for InputArg {
    /// Deserializes a string and converts it to an `InputArg` with
    /// [`InputArg::from_arg()`].
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        PathBuf::deserialize(deserializer).map(InputArg::from_arg)
    }
}

/// An output path that can refer to either standard output or a file system
/// path
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OutputArg {
    /// Refers to standard output.
    ///
    /// This is the variant returned by `OutputArg::default()`.
    #[default]
    Stdout,

    /// Refers to a file system path (stored in `.0`)
    Path(PathBuf),
}

impl OutputArg {
    /// Construct a `OutputArg` from a string, usually one taken from
    /// command-line arguments.  If the string equals `"-"` (i.e., it contains
    /// only a single hyphen/dash), [`OutputArg::Stdout`] is returned;
    /// otherwise, an [`OutputArg::Path`] is returned.
    ///
    /// # Example
    ///
    /// ```
    /// use patharg::OutputArg;
    /// use std::path::PathBuf;
    ///
    /// let p1 = OutputArg::from_arg("-");
    /// assert_eq!(p1, OutputArg::Stdout);
    ///
    /// let p2 = OutputArg::from_arg("./-");
    /// assert_eq!(p2, OutputArg::Path(PathBuf::from("./-")));
    /// ```
    pub fn from_arg<S: Into<PathBuf>>(arg: S) -> OutputArg {
        let arg = arg.into();
        if arg == Path::new("-") {
            OutputArg::Stdout
        } else {
            OutputArg::Path(arg)
        }
    }

    /// Returns true if the output arg is the `Stdout` variant of `OutputArg`.
    ///
    /// # Example
    ///
    /// ```
    /// use patharg::OutputArg;
    ///
    /// let p1 = OutputArg::from_arg("-");
    /// assert!(p1.is_stdout());
    ///
    /// let p2 = OutputArg::from_arg("file.txt");
    /// assert!(!p2.is_stdout());
    /// ```
    pub fn is_stdout(&self) -> bool {
        self == &OutputArg::Stdout
    }

    /// Returns true if the output arg is the `Path` variant of `OutputArg`.
    ///
    /// # Example
    ///
    /// ```
    /// use patharg::OutputArg;
    ///
    /// let p1 = OutputArg::from_arg("-");
    /// assert!(!p1.is_path());
    ///
    /// let p2 = OutputArg::from_arg("file.txt");
    /// assert!(p2.is_path());
    /// ```
    pub fn is_path(&self) -> bool {
        matches!(self, OutputArg::Path(_))
    }

    /// Retrieve a reference to the inner [`PathBuf`].  If the output arg is
    /// the `Stdout` variant, this returns `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use patharg::OutputArg;
    /// use std::path::PathBuf;
    ///
    /// let p1 = OutputArg::from_arg("-");
    /// assert_eq!(p1.path_ref(), None);
    ///
    /// let p2 = OutputArg::from_arg("file.txt");
    /// assert_eq!(p2.path_ref(), Some(&PathBuf::from("file.txt")));
    /// ```
    pub fn path_ref(&self) -> Option<&PathBuf> {
        match self {
            OutputArg::Stdout => None,
            OutputArg::Path(p) => Some(p),
        }
    }

    /// Retrieve a mutable reference to the inner [`PathBuf`].  If the output
    /// arg is the `Stdout` variant, this returns `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use patharg::OutputArg;
    /// use std::path::PathBuf;
    ///
    /// let mut p1 = OutputArg::from_arg("-");
    /// assert_eq!(p1.path_mut(), None);
    ///
    /// let mut p2 = OutputArg::from_arg("file.txt");
    /// assert_eq!(p2.path_mut(), Some(&mut PathBuf::from("file.txt")));
    /// ```
    pub fn path_mut(&mut self) -> Option<&mut PathBuf> {
        match self {
            OutputArg::Stdout => None,
            OutputArg::Path(p) => Some(p),
        }
    }

    /// Consume the output arg and return the inner [`PathBuf`].  If the output
    /// arg is the `Stdout` variant, this returns `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use patharg::OutputArg;
    /// use std::path::PathBuf;
    ///
    /// let p1 = OutputArg::from_arg("-");
    /// assert_eq!(p1.into_path(), None);
    ///
    /// let p2 = OutputArg::from_arg("file.txt");
    /// assert_eq!(p2.into_path(), Some(PathBuf::from("file.txt")));
    /// ```
    pub fn into_path(self) -> Option<PathBuf> {
        match self {
            OutputArg::Stdout => None,
            OutputArg::Path(p) => Some(p),
        }
    }

    /// Open the output arg for writing.
    ///
    /// If the output arg is the `Stdout` variant, this returns a locked
    /// reference to stdout.  Otherwise, if the output arg is a `Path` variant,
    /// the given path is opened for writing; if the path does not exist, it is
    /// created.
    ///
    /// The returned writer implements [`std::io::Write`].
    ///
    /// # Errors
    ///
    /// Has the same error conditions as [`std::fs::File::create`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use patharg::OutputArg;
    /// use std::env::args_os;
    /// use std::io::{self, Write};
    ///
    /// fn main() -> io::Result<()> {
    ///     let outfile = args_os().nth(1)
    ///                            .map(OutputArg::from_arg)
    ///                            .unwrap_or_default();
    ///     let mut f = outfile.create()?;
    ///     // The "{}" is replaced by either the output filepath or a hyphen.
    ///     write!(&mut f, "I am writing to {}.", outfile)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn create(&self) -> io::Result<OutputArgWriter> {
        Ok(match self {
            OutputArg::Stdout => Either::Left(io::stdout().lock()),
            OutputArg::Path(p) => Either::Right(fs::File::create(p)?),
        })
    }

    /// Write a slice as the entire contents of the output arg.
    ///
    /// If the output arg is the `Stdout` variant, the given data is written to
    /// stdout.  Otherwise, if the output arg is a `Path` variant, the contents
    /// of the given path are replaced with the given data; if the path does
    /// not exist, it is created first.
    ///
    /// # Errors
    ///
    /// Has the same error conditions as [`std::io::Write::write_all`] and
    /// [`std::fs::write`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use patharg::OutputArg;
    /// use std::env::args_os;
    /// use std::io;
    ///
    /// fn main() -> io::Result<()> {
    ///     let outfile = args_os().nth(1)
    ///                            .map(OutputArg::from_arg)
    ///                            .unwrap_or_default();
    ///     outfile.write("This is the output arg's new content.\n")?;
    ///     Ok(())
    /// }
    /// ```
    pub fn write<C: AsRef<[u8]>>(&self, contents: C) -> io::Result<()> {
        match self {
            OutputArg::Stdout => io::stdout().lock().write_all(contents.as_ref()),
            OutputArg::Path(p) => fs::write(p, contents),
        }
    }
}

#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
impl OutputArg {
    /// Asynchronously open the output arg for writing.
    ///
    /// If the output arg is the `Stdout` variant, this returns a reference to
    /// stdout.  Otherwise, if the output arg is a `Path` variant, the given
    /// path is opened for writing; if the path does not exist, it is created.
    ///
    /// The returned writer implements [`tokio::io::AsyncWrite`].
    ///
    /// # Errors
    ///
    /// Has the same error conditions as [`tokio::fs::File::create`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use patharg::OutputArg;
    /// use std::env::args_os;
    /// use tokio::io::AsyncWriteExt;
    ///
    /// #[tokio::main]
    /// async fn main() -> std::io::Result<()> {
    ///     let outfile = args_os().nth(1)
    ///                            .map(OutputArg::from_arg)
    ///                            .unwrap_or_default();
    ///     let mut f = outfile.async_create().await?;
    ///     // The "{}" is replaced by either the output filepath or a hyphen.
    ///     let msg = format!("I am writing to {}.\n", outfile);
    ///     f.write_all(msg.as_ref()).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn async_create(&self) -> io::Result<AsyncOutputArgWriter> {
        Ok(match self {
            OutputArg::Stdout => AsyncEither::Left(tokio::io::stdout()),
            OutputArg::Path(p) => AsyncEither::Right(tokio::fs::File::create(p).await?),
        })
    }

    /// Asynchronously write a slice as the entire contents of the output arg.
    ///
    /// If the output arg is the `Stdout` variant, the given data is written to
    /// stdout.  Otherwise, if the output arg is a `Path` variant, the contents
    /// of the given path are replaced with the given data; if the path does
    /// not exist, it is created first.
    ///
    /// # Errors
    ///
    /// Has the same error conditions as
    /// [`tokio::io::AsyncWriteExt::write_all`] and [`tokio::fs::write`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use patharg::OutputArg;
    /// use std::env::args_os;
    ///
    /// #[tokio::main]
    /// async fn main() -> std::io::Result<()> {
    ///     let outfile = args_os().nth(1)
    ///                            .map(OutputArg::from_arg)
    ///                            .unwrap_or_default();
    ///     outfile.async_write("This is the output arg's new content.\n").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn async_write<C: AsRef<[u8]>>(&self, contents: C) -> io::Result<()> {
        match self {
            OutputArg::Stdout => {
                let mut stdout = tokio::io::stdout();
                stdout.write_all(contents.as_ref()).await?;
                stdout.flush().await
            }
            OutputArg::Path(p) => tokio::fs::write(p, contents).await,
        }
    }
}

impl fmt::Display for OutputArg {
    /// Displays [`OutputArg::Stdout`] as `-` (a single hyphen/dash) or as
    /// `<stdout>` if the `{:#}` format is used.  Always displays
    /// [`OutputArg::Path`] using [`std::path::Path::display()`].
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // IMPORTANT: The default Display of Stdout has to round-trip back
            // to Stdout so that OutputArg will work properly when used with
            // clap's `default_value_t`.
            OutputArg::Stdout => {
                if f.alternate() {
                    write!(f, "<stdout>")
                } else {
                    write!(f, "-")
                }
            }
            OutputArg::Path(p) => write!(f, "{}", p.display()),
        }
    }
}

impl<S: Into<PathBuf>> From<S> for OutputArg {
    /// Convert a string to a [`OutputArg`] using [`OutputArg::from_arg()`].
    fn from(s: S) -> OutputArg {
        OutputArg::from_arg(s)
    }
}

impl From<OutputArg> for OsString {
    /// Converts an output arg back to an `OsString`: `OutputArg::Stdout`
    /// becomes `"-"`, and `OutputArg::Path(p)` becomes `p.into()`.
    fn from(arg: OutputArg) -> OsString {
        match arg {
            OutputArg::Stdout => OsString::from("-"),
            OutputArg::Path(p) => p.into(),
        }
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl Serialize for OutputArg {
    /// Serializes [`OutputArg::Stdout`] as `"-"` (a string containing a single
    /// hyphen/dash).  Serializes [`OutputArg::Path`] as the inner [`PathBuf`];
    /// this will fail if the path is not valid UTF-8.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            OutputArg::Stdout => "-".serialize(serializer),
            OutputArg::Path(p) => p.serialize(serializer),
        }
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<'de> Deserialize<'de> for OutputArg {
    /// Deserializes a string and converts it to an `OutputArg` with
    /// [`OutputArg::from_arg()`].
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        PathBuf::deserialize(deserializer).map(OutputArg::from_arg)
    }
}

/// The type of the readers returned by [`InputArg::open()`].
///
/// This type implements [`std::io::BufRead`].
pub type InputArgReader = Either<StdinLock<'static>, BufReader<fs::File>>;

/// The type of the writers returned by [`OutputArg::create()`].
///
/// This type implements [`std::io::Write`].
pub type OutputArgWriter = Either<StdoutLock<'static>, fs::File>;

/// The type of the iterators returned by [`InputArg::lines()`].
///
/// This iterator yields instances of `std::io::Result<String>`.
pub type Lines = io::Lines<InputArgReader>;

cfg_if! {
    if #[cfg(feature = "tokio")] {
       /// The type of the asynchronous readers returned by
       /// [`InputArg::async_open()`].
       ///
       /// This type implements [`tokio::io::AsyncRead`].
       #[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
       pub type AsyncInputArgReader = AsyncEither<tokio::io::Stdin, tokio::fs::File>;

       /// The type of the asynchronous writers returned by
       /// [`OutputArg::async_create()`].
       ///
       /// This type implements [`tokio::io::AsyncWrite`].
       #[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
       pub type AsyncOutputArgWriter = AsyncEither<tokio::io::Stdout, tokio::fs::File>;

       /// The type of the streams returned by [`InputArg::async_lines()`].
       ///
       /// This stream yields instances of `std::io::Result<String>`.
       #[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
       pub type AsyncLines = LinesStream<tokio::io::BufReader<AsyncInputArgReader>>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    mod inputarg {
        use super::*;

        #[test]
        fn test_assert_stdin_from_osstring() {
            let s = OsString::from("-");
            let p = InputArg::from(s);
            assert!(p.is_stdin());
            assert!(!p.is_path());
        }

        #[test]
        fn test_assert_path_from_osstring() {
            let s = OsString::from("./-");
            let p = InputArg::from(s);
            assert!(!p.is_stdin());
            assert!(p.is_path());
        }

        #[test]
        fn test_assert_stdin_from_osstr() {
            let s = OsStr::new("-");
            let p = InputArg::from(s);
            assert!(p.is_stdin());
            assert!(!p.is_path());
        }

        #[test]
        fn test_assert_path_from_osstr() {
            let s = OsStr::new("./-");
            let p = InputArg::from(s);
            assert!(!p.is_stdin());
            assert!(p.is_path());
        }

        #[test]
        fn test_assert_stdin_from_pathbuf() {
            let s = PathBuf::from("-");
            let p = InputArg::from(s);
            assert!(p.is_stdin());
            assert!(!p.is_path());
        }

        #[test]
        fn test_assert_path_from_pathbuf() {
            let s = PathBuf::from("./-");
            let p = InputArg::from(s);
            assert!(!p.is_stdin());
            assert!(p.is_path());
        }

        #[test]
        fn test_assert_stdin_from_path() {
            let s = Path::new("-");
            let p = InputArg::from(s);
            assert!(p.is_stdin());
            assert!(!p.is_path());
        }

        #[test]
        fn test_assert_path_from_path() {
            let s = Path::new("./-");
            let p = InputArg::from(s);
            assert!(!p.is_stdin());
            assert!(p.is_path());
        }

        #[test]
        fn test_assert_stdin_from_string() {
            let s = String::from("-");
            let p = InputArg::from(s);
            assert!(p.is_stdin());
            assert!(!p.is_path());
        }

        #[test]
        fn test_assert_path_from_string() {
            let s = String::from("./-");
            let p = InputArg::from(s);
            assert!(!p.is_stdin());
            assert!(p.is_path());
        }

        #[test]
        fn test_assert_stdin_from_str() {
            let p = InputArg::from("-");
            assert!(p.is_stdin());
            assert!(!p.is_path());
        }

        #[test]
        fn test_assert_path_from_str() {
            let p = InputArg::from("./-");
            assert!(!p.is_stdin());
            assert!(p.is_path());
        }

        #[test]
        fn test_default() {
            assert_eq!(InputArg::default(), InputArg::Stdin);
        }

        #[test]
        fn test_stdin_path_ref() {
            let p = InputArg::Stdin;
            assert_eq!(p.path_ref(), None);
        }

        #[test]
        fn test_path_path_ref() {
            let p = InputArg::Path(PathBuf::from("-"));
            assert_eq!(p.path_ref(), Some(&PathBuf::from("-")));
        }

        #[test]
        fn test_stdin_path_mut() {
            let mut p = InputArg::Stdin;
            assert_eq!(p.path_mut(), None);
        }

        #[test]
        fn test_path_path_mut() {
            let mut p = InputArg::Path(PathBuf::from("-"));
            assert_eq!(p.path_mut(), Some(&mut PathBuf::from("-")));
        }

        #[test]
        fn test_stdin_into_path() {
            let p = InputArg::Stdin;
            assert_eq!(p.into_path(), None);
        }

        #[test]
        fn test_path_into_path() {
            let p = InputArg::Path(PathBuf::from("-"));
            assert_eq!(p.into_path(), Some(PathBuf::from("-")));
        }

        #[test]
        fn test_display_stdin() {
            let p = InputArg::Stdin;
            assert_eq!(p.to_string(), "-");
        }

        #[test]
        fn test_display_alternate_stdin() {
            let p = InputArg::Stdin;
            assert_eq!(format!("{:#}", p), "<stdin>");
        }

        #[test]
        fn test_display_path() {
            let p = InputArg::from_arg("./-");
            assert_eq!(p.to_string(), "./-");
        }

        #[test]
        fn test_stdin_into_osstring() {
            let p = InputArg::Stdin;
            assert_eq!(OsString::from(p), OsString::from("-"));
        }

        #[test]
        fn test_path_into_osstring() {
            let p = InputArg::Path(PathBuf::from("./-"));
            assert_eq!(OsString::from(p), OsString::from("./-"));
        }

        #[cfg(feature = "serde")]
        mod serding {
            use super::*;

            #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
            struct Input {
                path: InputArg,
            }

            #[test]
            fn test_stdin_to_json() {
                let val = Input {
                    path: InputArg::Stdin,
                };
                assert_eq!(serde_json::to_string(&val).unwrap(), r#"{"path":"-"}"#);
            }

            #[test]
            fn test_path_to_json() {
                let val = Input {
                    path: InputArg::Path(PathBuf::from("foo.txt")),
                };
                assert_eq!(
                    serde_json::to_string(&val).unwrap(),
                    r#"{"path":"foo.txt"}"#
                );
            }

            #[test]
            fn test_stdin_from_json() {
                let s = r#"{"path": "-"}"#;
                assert_eq!(
                    serde_json::from_str::<Input>(s).unwrap(),
                    Input {
                        path: InputArg::Stdin
                    }
                );
            }

            #[test]
            fn test_path_from_json() {
                let s = r#"{"path": "./-"}"#;
                assert_eq!(
                    serde_json::from_str::<Input>(s).unwrap(),
                    Input {
                        path: InputArg::Path(PathBuf::from("./-"))
                    }
                );
            }
        }
    }

    mod outputarg {
        use super::*;

        #[test]
        fn test_assert_stdout_from_osstring() {
            let s = OsString::from("-");
            let p = OutputArg::from(s);
            assert!(p.is_stdout());
            assert!(!p.is_path());
        }

        #[test]
        fn test_assert_path_from_osstring() {
            let s = OsString::from("./-");
            let p = OutputArg::from(s);
            assert!(!p.is_stdout());
            assert!(p.is_path());
        }

        #[test]
        fn test_assert_stdout_from_osstr() {
            let s = OsStr::new("-");
            let p = OutputArg::from(s);
            assert!(p.is_stdout());
            assert!(!p.is_path());
        }

        #[test]
        fn test_assert_path_from_osstr() {
            let s = OsStr::new("./-");
            let p = OutputArg::from(s);
            assert!(!p.is_stdout());
            assert!(p.is_path());
        }

        #[test]
        fn test_assert_stdout_from_pathbuf() {
            let s = PathBuf::from("-");
            let p = OutputArg::from(s);
            assert!(p.is_stdout());
            assert!(!p.is_path());
        }

        #[test]
        fn test_assert_path_from_pathbuf() {
            let s = PathBuf::from("./-");
            let p = OutputArg::from(s);
            assert!(!p.is_stdout());
            assert!(p.is_path());
        }

        #[test]
        fn test_assert_stdout_from_path() {
            let s = Path::new("-");
            let p = OutputArg::from(s);
            assert!(p.is_stdout());
            assert!(!p.is_path());
        }

        #[test]
        fn test_assert_path_from_path() {
            let s = Path::new("./-");
            let p = OutputArg::from(s);
            assert!(!p.is_stdout());
            assert!(p.is_path());
        }

        #[test]
        fn test_assert_stdout_from_string() {
            let s = String::from("-");
            let p = OutputArg::from(s);
            assert!(p.is_stdout());
            assert!(!p.is_path());
        }

        #[test]
        fn test_assert_path_from_string() {
            let s = String::from("./-");
            let p = OutputArg::from(s);
            assert!(!p.is_stdout());
            assert!(p.is_path());
        }

        #[test]
        fn test_assert_stdout_from_str() {
            let p = OutputArg::from("-");
            assert!(p.is_stdout());
            assert!(!p.is_path());
        }

        #[test]
        fn test_assert_path_from_str() {
            let p = OutputArg::from("./-");
            assert!(!p.is_stdout());
            assert!(p.is_path());
        }

        #[test]
        fn test_default() {
            assert_eq!(OutputArg::default(), OutputArg::Stdout);
        }

        #[test]
        fn test_stdout_path_ref() {
            let p = OutputArg::Stdout;
            assert_eq!(p.path_ref(), None);
        }

        #[test]
        fn test_path_path_ref() {
            let p = OutputArg::Path(PathBuf::from("-"));
            assert_eq!(p.path_ref(), Some(&PathBuf::from("-")));
        }

        #[test]
        fn test_stdout_path_mut() {
            let mut p = OutputArg::Stdout;
            assert_eq!(p.path_mut(), None);
        }

        #[test]
        fn test_path_path_mut() {
            let mut p = OutputArg::Path(PathBuf::from("-"));
            assert_eq!(p.path_mut(), Some(&mut PathBuf::from("-")));
        }

        #[test]
        fn test_stdout_into_path() {
            let p = OutputArg::Stdout;
            assert_eq!(p.into_path(), None);
        }

        #[test]
        fn test_path_into_path() {
            let p = OutputArg::Path(PathBuf::from("-"));
            assert_eq!(p.into_path(), Some(PathBuf::from("-")));
        }

        #[test]
        fn test_display_stdout() {
            let p = OutputArg::Stdout;
            assert_eq!(p.to_string(), "-");
        }

        #[test]
        fn test_display_alternate_stdout() {
            let p = OutputArg::Stdout;
            assert_eq!(format!("{:#}", p), "<stdout>");
        }

        #[test]
        fn test_display_path() {
            let p = OutputArg::from_arg("./-");
            assert_eq!(p.to_string(), "./-");
        }

        #[test]
        fn test_stdout_into_osstring() {
            let p = OutputArg::Stdout;
            assert_eq!(OsString::from(p), OsString::from("-"));
        }

        #[test]
        fn test_path_into_osstring() {
            let p = OutputArg::Path(PathBuf::from("./-"));
            assert_eq!(OsString::from(p), OsString::from("./-"));
        }

        #[cfg(feature = "serde")]
        mod serding {
            use super::*;

            #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
            struct Output {
                path: OutputArg,
            }

            #[test]
            fn test_stdout_to_json() {
                let val = Output {
                    path: OutputArg::Stdout,
                };
                assert_eq!(serde_json::to_string(&val).unwrap(), r#"{"path":"-"}"#);
            }

            #[test]
            fn test_path_to_json() {
                let val = Output {
                    path: OutputArg::Path(PathBuf::from("foo.txt")),
                };
                assert_eq!(
                    serde_json::to_string(&val).unwrap(),
                    r#"{"path":"foo.txt"}"#
                );
            }

            #[test]
            fn test_stdout_from_json() {
                let s = r#"{"path": "-"}"#;
                assert_eq!(
                    serde_json::from_str::<Output>(s).unwrap(),
                    Output {
                        path: OutputArg::Stdout
                    }
                );
            }

            #[test]
            fn test_path_from_json() {
                let s = r#"{"path": "./-"}"#;
                assert_eq!(
                    serde_json::from_str::<Output>(s).unwrap(),
                    Output {
                        path: OutputArg::Path(PathBuf::from("./-"))
                    }
                );
            }
        }
    }
}
