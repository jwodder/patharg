use std::ffi::OsStr;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
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

    pub fn read_to_string(&self) -> io::Result<String> {
        match self {
            PathArg::Std => io::read_to_string(io::stdin().lock()),
            PathArg::Path(p) => std::fs::read_to_string(p),
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

    pub fn path_mut_ref(&mut self) -> Option<&mut Path> {
        match self {
            PathArg::Std => None,
            PathArg::Path(p) => Some(p),
        }
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
}
