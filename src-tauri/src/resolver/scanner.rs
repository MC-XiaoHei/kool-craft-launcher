use async_trait::async_trait;
use log::warn;
use std::io;
use std::path::{Path, PathBuf};
use tokio::fs;
use crate::constants::minecraft_dir::VERSIONS_DIR_NAME;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionMetadata {
    pub id: String,
    pub json_path: PathBuf,
    pub jar_path: PathBuf,
}

#[async_trait]
pub trait VersionScanner: Send + Sync {
    async fn scan_versions(
        &self,
        minecraft_folder: PathBuf,
    ) -> Result<Vec<VersionMetadata>, io::Error>;
}

pub struct FileSystemScanner;

#[async_trait]
impl VersionScanner for FileSystemScanner {
    async fn scan_versions(
        &self,
        minecraft_folder: PathBuf,
    ) -> Result<Vec<VersionMetadata>, io::Error> {
        let versions_dir = minecraft_folder.join(VERSIONS_DIR_NAME);

        if !fs::try_exists(&versions_dir).await? {
            return Ok(vec![]);
        }

        let mut detected_versions = Vec::new();
        let mut entries = fs::read_dir(versions_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir()
                && let Some(metadata) = self.inspect_directory(&path).await
            {
                detected_versions.push(metadata);
            }
        }

        Ok(detected_versions)
    }
}

impl FileSystemScanner {
    async fn inspect_directory(&self, dir: &Path) -> Option<VersionMetadata> {
        let Some(dir_name) = dir.file_name().and_then(|n| n.to_str()) else {
            warn!("Invalid version dir: {:?}", dir);
            return None;
        };
        let json_path = dir.join(format!("{dir_name}.json"));
        let jar_path = dir.join(format!("{dir_name}.jar"));

        if fs::try_exists(&json_path).await.unwrap_or(false) && json_path.is_file() {
            Some(VersionMetadata {
                id: dir_name.to_string(),
                json_path,
                jar_path,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scan_invalid_versions() {
        let res = FileSystemScanner.inspect_directory(Path::new("..")).await;
        assert_eq!(res, None);
    }
}
