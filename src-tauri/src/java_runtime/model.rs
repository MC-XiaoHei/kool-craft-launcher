use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct JavaRuntime {
    pub path: PathBuf,
    pub major_version: u8,
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
