use std::ffi::OsString;
use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum PathArg {
    #[default]
    Std,
    Path(PathBuf),
}

impl PathArg {
    pub fn read_to_string(&self) -> io::Result<String> {
        match self {
            PathArg::Std => {
                let stdin = io::stdin();
                io::read_to_string(stdin.lock())
            }
            PathArg::Path(p) => std::fs::read_to_string(p),
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
        if s == "-" {
            PathArg::Std
        } else {
            PathArg::Path(s.into())
        }
    }
}
