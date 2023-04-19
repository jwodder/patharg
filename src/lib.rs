//! Most CLI commands that take file paths as arguments follow the convention
//! of treating a path of `-` (a single hyphen/dash) as referring to either
//! standard input or standard output (depending on whether the path is read
//! from or written to).  The `patharg` crate lets your programs follow this
//! convention too: it provides a `PathArg` type that wraps a command-line
//! argument, with methods for reading from or writing to either the given path
//! or — if the argument is just a hyphen — the appropriate standard stream.
//!
//! [`PathArg`] implements `From<OsString>` and `From<String>`, so you can use
//! it seamlessly with your favorite Rust source of command-line arguments, be
//! it [`clap`][], [`lexopt`][], plain old
//! [`std::env::args`]/[`std::env::args_os`], or whatever else is out there.
//! The source repository contains examples of two of these:
//!
//! - [`examples/flipcase.rs`][flipcase] shows how to use this crate with
//!   `clap`.
//! - [`examples/revchars.rs`][revchars] shows how to use this crate with
//!   `lexopt`.
//!
//! [`clap`]: https://crates.io/crates/clap
//! [`lexopt`]: https://crates.io/crates/lexopt
//! [flipcase]: https://github.com/jwodder/patharg/blob/master/examples/flipcase.rs
//! [revchars]: https://github.com/jwodder/patharg/blob/master/examples/revchars.rs
use either::Either;
use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::io::{self, BufRead, BufReader, Read, StdinLock, StdoutLock, Write};
use std::path::PathBuf;

/// A representation of a command-line argument that can refer to either a
/// standard stream (stdin or stdout) or a file system path.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PathArg {
    /// Refers to a standard stream (stdin or stdout).  Which stream is used
    /// depends on whether the `PathArg` is read from or written to.
    ///
    /// This is the variant returned by `PathArg::default()`.
    #[default]
    Std,

    /// Refers to a file system path (stored in `.0`)
    Path(PathBuf),
}

impl PathArg {
    /// Construct a `PathArg` from a string, usually one taken from
    /// command-line arguments.  If the string equals `"-"` (i.e., it contains
    /// only a single hyphen/dash), [`PathArg::Std`] is returned; otherwise, a
    /// [`PathArg::Path`] is returned.
    ///
    /// ```
    /// use patharg::PathArg;
    /// use std::path::PathBuf;
    ///
    /// let p1 = PathArg::from_arg("-");
    /// assert_eq!(p1, PathArg::Std);
    ///
    /// let p2 = PathArg::from_arg("./-");
    /// assert_eq!(p2, PathArg::Path(PathBuf::from("./-")));
    /// ```
    pub fn from_arg<S: AsRef<OsStr>>(arg: S) -> PathArg {
        let arg = arg.as_ref();
        if arg == "-" {
            PathArg::Std
        } else {
            PathArg::Path(arg.into())
        }
    }

    /// Returns true if the path arg is the `Std` variant of `PathArg`.
    pub fn is_std(&self) -> bool {
        self == &PathArg::Std
    }

    /// Returns true if the path arg is the `Path` variant of `PathArg`.
    pub fn is_path(&self) -> bool {
        matches!(self, PathArg::Path(_))
    }

    /// Retrieve a reference to the inner [`PathBuf`].  If the path arg is
    /// the `Std` variant, this returns `None`.
    pub fn path_ref(&self) -> Option<&PathBuf> {
        match self {
            PathArg::Std => None,
            PathArg::Path(p) => Some(p),
        }
    }

    /// Retrieve a mutable reference to the inner [`PathBuf`].  If the path arg
    /// is the `Std` variant, this returns `None`.
    pub fn path_mut(&mut self) -> Option<&mut PathBuf> {
        match self {
            PathArg::Std => None,
            PathArg::Path(p) => Some(p),
        }
    }

    /// Consume the path arg and return the inner [`PathBuf`].  If the path arg
    /// is the `Std` variant, this returns `None`.
    pub fn into_path(self) -> Option<PathBuf> {
        match self {
            PathArg::Std => None,
            PathArg::Path(p) => Some(p),
        }
    }

    /// Open the path arg for reading.
    ///
    /// If the path arg is the `Std` variant, this returns a locked reference
    /// to stdin.  Otherwise, if the path arg is a `Path` variant, the given
    /// path is opened for reading.
    ///
    /// The returned reader implements [`std::io::BufRead`].
    ///
    /// # Errors
    ///
    /// Has the same error conditions as [`std::fs::File::open`].
    pub fn open(&self) -> io::Result<PathReader> {
        Ok(match self {
            PathArg::Std => Either::Left(io::stdin().lock()),
            PathArg::Path(p) => Either::Right(BufReader::new(fs::File::open(p)?)),
        })
    }

    /// Open the path arg for writing.
    ///
    /// If the path arg is the `Std` variant, this returns a locked reference
    /// to stdout.  Otherwise, if the path arg is a `Path` variant, the given
    /// path is opened for writing; if the path does not exist, it is created.
    ///
    /// The returned writer implements [`std::io::Write`].
    ///
    /// # Errors
    ///
    /// Has the same error conditions as [`std::fs::File::create`].
    pub fn create(&self) -> io::Result<PathWriter> {
        Ok(match self {
            PathArg::Std => Either::Left(io::stdout().lock()),
            PathArg::Path(p) => Either::Right(fs::File::create(p)?),
        })
    }

    /// Write a slice as the entire contents of the path arg.
    ///
    /// If the path arg is the `Std` variant, the given data is written to
    /// stdout.  Otherwise, if the path arg is a `Path` variant, the contents
    /// of the given path are replaced with the given data, if the path does
    /// not exist, it is created first.
    ///
    /// # Errors
    ///
    /// Has the same error conditions as [`std::io::Write::write_all`] and
    /// [`std::fs::write`].
    pub fn write<C: AsRef<[u8]>>(&self, contents: C) -> io::Result<()> {
        match self {
            PathArg::Std => io::stdout().lock().write_all(contents.as_ref()),
            PathArg::Path(p) => fs::write(p, contents),
        }
    }

    /// Read the entire contents of the path arg into a bytes vector.
    ///
    /// If the path arg is the `Std` variant, the entire contents of stdin are
    /// read.  Otherwise, if the path arg is a `Path` variant, the contents of
    /// the given path are read.
    ///
    /// # Errors
    ///
    /// Has the same error conditions as [`std::io::Read::read_to_end`] and
    /// [`std::fs::read`].
    pub fn read(&self) -> io::Result<Vec<u8>> {
        match self {
            PathArg::Std => {
                let mut vec = Vec::new();
                io::stdin().lock().read_to_end(&mut vec)?;
                Ok(vec)
            }
            PathArg::Path(p) => fs::read(p),
        }
    }

    /// Read the entire contents of the path arg into a string.
    ///
    /// If the path arg is the `Std` variant, the entire contents of stdin are
    /// read.  Otherwise, if the path arg is a `Path` variant, the contents of
    /// the given path are read.
    ///
    /// # Errors
    ///
    /// Has the same error conditions as [`std::io::read_to_string`] and
    /// [`std::fs::read_to_string`].
    pub fn read_to_string(&self) -> io::Result<String> {
        match self {
            PathArg::Std => io::read_to_string(io::stdin().lock()),
            PathArg::Path(p) => fs::read_to_string(p),
        }
    }

    /// Return an iterator over the lines of the path arg.
    ///
    /// If the path arg is the `Std` variant, this locks stdin and returns an
    /// iterator over its lines; the lock is released once the iterator is
    /// dropped.  Otherwise, if the path arg is a `Path` variant, the given
    /// path is opened for reading, and its lines are iterated over.
    ///
    /// The returned iterator yields instances of `std::io::Result<String>`,
    /// where each individual item has the same error conditions as
    /// [`std::io::BufRead::read_line()`].
    ///
    /// # Errors
    ///
    /// Has the same error conditions as [`PathArg::open()`].
    pub fn lines(&self) -> io::Result<Lines> {
        Ok(self.open()?.lines())
    }
}

impl fmt::Display for PathArg {
    /// Displays [`PathArg::Std`] as `-` (a single hyphen/dash) and displays
    /// [`PathArg::Path`] using [`std::path::Path::display()`].
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PathArg::Std => write!(f, "-"),
            PathArg::Path(p) => write!(f, "{}", p.display()),
        }
    }
}

impl<S: AsRef<OsStr>> From<S> for PathArg {
    /// Convert a string to a [`PathArg`] using [`PathArg::from_arg()`].
    fn from(s: S) -> PathArg {
        PathArg::from_arg(s)
    }
}

/// The type of the readers returned by [`PathArg::open()`].
///
/// This type implements [`std::io::BufRead`].
pub type PathReader = Either<StdinLock<'static>, BufReader<fs::File>>;

/// The type of the writers returned by [`PathArg::create()`].
///
/// This type implements [`std::io::Write`].
pub type PathWriter = Either<StdoutLock<'static>, fs::File>;

/// The type of the iterators returned by [`PathArg::lines()`].
///
/// This iterator yields instances of `std::io::Result<String>`.
pub type Lines = io::Lines<PathReader>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::path::Path;

    #[test]
    fn test_assert_std_from_osstring() {
        let s = OsString::from("-");
        let p = PathArg::from(s);
        assert!(p.is_std());
        assert!(!p.is_path());
    }

    #[test]
    fn test_assert_path_from_osstring() {
        let s = OsString::from("./-");
        let p = PathArg::from(s);
        assert!(!p.is_std());
        assert!(p.is_path());
    }

    #[test]
    fn test_assert_std_from_osstr() {
        let s = OsStr::new("-");
        let p = PathArg::from(s);
        assert!(p.is_std());
        assert!(!p.is_path());
    }

    #[test]
    fn test_assert_path_from_osstr() {
        let s = OsStr::new("./-");
        let p = PathArg::from(s);
        assert!(!p.is_std());
        assert!(p.is_path());
    }

    #[test]
    fn test_assert_std_from_pathbuf() {
        let s = PathBuf::from("-");
        let p = PathArg::from(s);
        assert!(p.is_std());
        assert!(!p.is_path());
    }

    #[test]
    fn test_assert_path_from_pathbuf() {
        let s = PathBuf::from("./-");
        let p = PathArg::from(s);
        assert!(!p.is_std());
        assert!(p.is_path());
    }

    #[test]
    fn test_assert_std_from_path() {
        let s = Path::new("-");
        let p = PathArg::from(s);
        assert!(p.is_std());
        assert!(!p.is_path());
    }

    #[test]
    fn test_assert_path_from_path() {
        let s = Path::new("./-");
        let p = PathArg::from(s);
        assert!(!p.is_std());
        assert!(p.is_path());
    }

    #[test]
    fn test_assert_std_from_string() {
        let s = String::from("-");
        let p = PathArg::from(s);
        assert!(p.is_std());
        assert!(!p.is_path());
    }

    #[test]
    fn test_assert_path_from_string() {
        let s = String::from("./-");
        let p = PathArg::from(s);
        assert!(!p.is_std());
        assert!(p.is_path());
    }

    #[test]
    fn test_assert_std_from_str() {
        let p = PathArg::from("-");
        assert!(p.is_std());
        assert!(!p.is_path());
    }

    #[test]
    fn test_assert_path_from_str() {
        let p = PathArg::from("./-");
        assert!(!p.is_std());
        assert!(p.is_path());
    }

    #[test]
    fn test_default() {
        assert_eq!(PathArg::default(), PathArg::Std);
    }

    #[test]
    fn test_none_path_ref() {
        let p = PathArg::Std;
        assert_eq!(p.path_ref(), None);
    }

    #[test]
    fn test_some_path_ref() {
        let p = PathArg::Path(PathBuf::from("-"));
        assert_eq!(p.path_ref(), Some(&PathBuf::from("-")));
    }

    #[test]
    fn test_none_path_mut() {
        let mut p = PathArg::Std;
        assert_eq!(p.path_mut(), None);
    }

    #[test]
    fn test_some_path_mut() {
        let mut p = PathArg::Path(PathBuf::from("-"));
        assert_eq!(p.path_mut(), Some(&mut PathBuf::from("-")));
    }

    #[test]
    fn test_none_into_path() {
        let p = PathArg::Std;
        assert_eq!(p.into_path(), None);
    }

    #[test]
    fn test_some_into_path() {
        let p = PathArg::Path(PathBuf::from("-"));
        assert_eq!(p.into_path(), Some(PathBuf::from("-")));
    }
}
