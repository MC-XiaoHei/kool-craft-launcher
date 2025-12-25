use crate::launcher::model::LaunchError::InvalidJavaRuntime;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct JavaRuntime {
    pub path: PathBuf,
    pub major_version: u8,
}

impl JavaRuntime {
    pub fn get_java_executable_path(&self) -> PathBuf {
        self.path.clone().join("bin").join("java")
    }

    pub fn get_java_executable_path_str(&self) -> Result<String> {
        let str = self.get_java_executable_path()
            .to_str()
            .ok_or(InvalidJavaRuntime)?
            .to_string();
        Ok(str)
    }
}

impl Ord for JavaRuntime {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major_version.cmp(&other.major_version)
    }
}

impl PartialOrd for JavaRuntime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
