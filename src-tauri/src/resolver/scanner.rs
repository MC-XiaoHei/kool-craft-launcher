use async_trait::async_trait;
use std::io;
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Clone)]
pub struct VersionMetadata {
    pub id: String,
    pub json_path: PathBuf,
    pub jar_path: PathBuf,
}

#[async_trait]
pub trait VersionScanner: Send + Sync {
    async fn scan_versions(&self, root_dir: &Path) -> Result<Vec<VersionMetadata>, io::Error>;
}

pub struct FileSystemScanner;

#[async_trait]
impl VersionScanner for FileSystemScanner {
    async fn scan_versions(&self, root_dir: &Path) -> Result<Vec<VersionMetadata>, io::Error> {
        let versions_dir = root_dir.join("versions");

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
        let dir_name = dir.file_name()?.to_str()?;
        let json_path = dir.join(format!("{}.json", dir_name));
        let jar_path = dir.join(format!("{}.jar", dir_name));

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
