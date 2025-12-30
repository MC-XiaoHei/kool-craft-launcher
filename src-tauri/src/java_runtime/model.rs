use crate::game_launcher::model::LaunchError::InvalidJavaRuntime;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct JavaInstance {
    pub path: PathBuf,
    pub version: String,
    pub major_version: u32,
    pub arch: JavaArch,
    pub vendor_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JavaArch {
    X86,
    X64,
    Arm64,
    Unknown(String),
}

impl JavaInstance {
    pub fn get_java_executable_path(&self) -> PathBuf {
        self.path.clone().join("bin").join("java")
    }

    pub fn get_java_executable_path_str(&self) -> Result<String> {
        let str = self
            .get_java_executable_path()
            .to_str()
            .ok_or(InvalidJavaRuntime)?
            .to_string();
        Ok(str)
    }
}

impl Ord for JavaInstance {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major_version.cmp(&other.major_version)
    }
}

impl PartialOrd for JavaInstance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
