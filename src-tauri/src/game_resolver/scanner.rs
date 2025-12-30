use crate::constants::minecraft_dir::VERSIONS_DIR_NAME;
use crate::utils::abs_path_buf::AbsPathBuf;
use anyhow::Result;
use async_trait::async_trait;
use log::warn;
use tokio::fs;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionMetadata {
    pub id: String,
    pub json_path: AbsPathBuf,
    pub jar_path: AbsPathBuf,
}

#[async_trait]
pub trait VersionScanner: Send + Sync {
    async fn scan_versions(&self, minecraft_folder: AbsPathBuf) -> Result<Vec<VersionMetadata>>;
}

pub struct FileSystemScanner;

#[async_trait]
impl VersionScanner for FileSystemScanner {
    async fn scan_versions(&self, minecraft_folder: AbsPathBuf) -> Result<Vec<VersionMetadata>> {
        let versions_dir = minecraft_folder.join(VERSIONS_DIR_NAME);

        if !fs::try_exists(&versions_dir).await? {
            return Ok(vec![]);
        }

        let mut detected_versions = Vec::new();
        let mut entries = fs::read_dir(versions_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path: AbsPathBuf = entry.path().try_into()?;
            if path.is_dir()
                && let Some(metadata) = self.inspect_directory(path).await
            {
                detected_versions.push(metadata);
            }
        }

        Ok(detected_versions)
    }
}

impl FileSystemScanner {
    async fn inspect_directory(&self, dir: AbsPathBuf) -> Option<VersionMetadata> {
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
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_scan_invalid_versions() {
        let res = FileSystemScanner
            .inspect_directory(PathBuf::new().try_into().unwrap())
            .await;
        assert_eq!(res, None);
    }
}
