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
//!
//! See [`examples/flipcase.rs`][flipcase] in the source repository for an
//! example of how to use this crate with `clap`.
//!
//! [`clap`]: https://crates.io/crates/clap
//! [`lexopt`]: https://crates.io/crates/lexopt
//! [flipcase]: https://github.com/jwodder/patharg/blob/master/examples/flipcase.rs
use either::Either;
use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::io::{self, BufRead, BufReader, Read as _, StdinLock, StdoutLock, Write as _};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PathArg {
    #[default]
    Std,
    Path(PathBuf),
}

impl PathArg {
    pub fn from_arg<S: AsRef<OsStr>>(arg: S) -> PathArg {
        let arg = arg.as_ref();
        if arg == "-" {
            PathArg::Std
        } else {
            PathArg::Path(arg.into())
        }
    }

    pub fn is_std(&self) -> bool {
        self == &PathArg::Std
    }

    pub fn is_path(&self) -> bool {
        matches!(self, PathArg::Path(_))
    }

    pub fn path_ref(&self) -> Option<&Path> {
        match self {
            PathArg::Std => None,
            PathArg::Path(p) => Some(p),
        }
    }

    // Requires Rust 1.68+ due to `impl DerefMut for PathBuf` not being
    // introduced until that version
    #[cfg(feature = "path_buf_deref_mut")]
    pub fn path_mut(&mut self) -> Option<&mut Path> {
        match self {
            PathArg::Std => None,
            PathArg::Path(p) => Some(p),
        }
    }

    pub fn open(&self) -> io::Result<Read> {
        Ok(match self {
            PathArg::Std => Either::Left(io::stdin().lock()),
            PathArg::Path(p) => Either::Right(BufReader::new(fs::File::open(p)?)),
        })
    }

    pub fn create(&self) -> io::Result<Write> {
        Ok(match self {
            PathArg::Std => Either::Left(io::stdout().lock()),
            PathArg::Path(p) => Either::Right(fs::File::create(p)?),
        })
    }

    pub fn write<C: AsRef<[u8]>>(&self, contents: C) -> io::Result<()> {
        match self {
            PathArg::Std => io::stdout().lock().write_all(contents.as_ref()),
            PathArg::Path(p) => fs::write(p, contents),
        }
    }

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

    pub fn read_to_string(&self) -> io::Result<String> {
        match self {
            PathArg::Std => io::read_to_string(io::stdin().lock()),
            PathArg::Path(p) => fs::read_to_string(p),
        }
    }

    pub fn lines(&self) -> io::Result<Lines> {
        Ok(self.open()?.lines())
    }
}

impl fmt::Display for PathArg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PathArg::Std => write!(f, "-"),
            PathArg::Path(p) => write!(f, "{}", p.display()),
        }
    }
}

impl<S: AsRef<OsStr>> From<S> for PathArg {
    fn from(s: S) -> PathArg {
        PathArg::from_arg(s)
    }
}

pub type Read = Either<StdinLock<'static>, BufReader<fs::File>>;
pub type Write = Either<StdoutLock<'static>, fs::File>;
pub type Lines = io::Lines<Read>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

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
        assert_eq!(p.path_ref(), Some(Path::new("-")));
    }

    #[test]
    #[cfg(feature = "path_buf_deref_mut")]
    fn test_none_path_mut() {
        let mut p = PathArg::Std;
        assert_eq!(p.path_ref(), None);
    }

    #[test]
    #[cfg(feature = "path_buf_deref_mut")]
    fn test_some_path_mut() {
        let mut p = PathArg::Path(PathBuf::from("-"));
        assert_eq!(p.path_mut(), Some(Path::new("-")));
    }
}
