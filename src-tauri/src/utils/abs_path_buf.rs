use anyhow::Result;
use path_absolutize::Absolutize;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::ops::Deref;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AbsPathBuf {
    buf: PathBuf,
}

impl AbsPathBuf {
    pub fn new<S: AsRef<OsStr> + ?Sized>(s: &S) -> Result<Self> {
        Path::new(s).to_path_buf().try_into()
    }

    pub fn join<P: AsRef<Path>>(&self, path: P) -> Self {
        Self {
            buf: self.buf.join(path).to_path_buf(),
        }
    }

    pub fn push<P: AsRef<Path>>(&mut self, path: P) {
        self.buf.push(path);
    }
}

impl Deref for AbsPathBuf {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.buf
    }
}

impl AsRef<Path> for AbsPathBuf {
    fn as_ref(&self) -> &Path {
        self.buf.as_path()
    }
}

impl AsRef<OsStr> for AbsPathBuf {
    fn as_ref(&self) -> &OsStr {
        self.buf.as_os_str()
    }
}

impl From<AbsPathBuf> for PathBuf {
    fn from(abs: AbsPathBuf) -> Self {
        abs.buf
    }
}

impl TryFrom<PathBuf> for AbsPathBuf {
    type Error = anyhow::Error;
    fn try_from(path: PathBuf) -> Result<Self> {
        if path.is_absolute() {
            Ok(AbsPathBuf { buf: path })
        } else {
            Ok(AbsPathBuf {
                buf: path.absolutize()?.to_path_buf(),
            })
        }
    }
}
