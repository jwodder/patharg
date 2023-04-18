use std::ffi::{OsStr, OsString};
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
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

impl From<OsString> for PathArg {
    fn from(s: OsString) -> PathArg {
        PathArg::from_arg(s)
    }
}
