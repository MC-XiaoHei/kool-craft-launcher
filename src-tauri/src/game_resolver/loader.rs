use crate::constants::minecraft_dir::VERSIONS_DIR_NAME;
use crate::game_resolver::models::VersionData::{Broken, Normal};
use crate::game_resolver::models::{VersionData, VersionManifest};
use crate::game_resolver::scanner::VersionMetadata;
use crate::utils::abs_path_buf::AbsPathBuf;
use anyhow::Result;
use async_trait::async_trait;
use log::warn;
use tokio::fs;

#[async_trait]
pub trait VersionLoader: Send + Sync {
    async fn load_and_resolve(
        &self,
        minecraft_folder: AbsPathBuf,
        metadata: VersionMetadata,
    ) -> VersionData;
}

pub struct FileSystemVersionLoader;

impl FileSystemVersionLoader {
    fn resolve_json_path(
        &self,
        minecraft_folder: AbsPathBuf,
        version_id: impl Into<String>,
    ) -> AbsPathBuf {
        let version_id = version_id.into();
        minecraft_folder
            .join(VERSIONS_DIR_NAME)
            .join(version_id.clone())
            .join(format!("{version_id}.json"))
    }
}

#[async_trait]
impl VersionLoader for FileSystemVersionLoader {
    async fn load_and_resolve(
        &self,
        minecraft_folder: AbsPathBuf,
        metadata: VersionMetadata,
    ) -> VersionData {
        let id = metadata.id.clone();
        let resolved = self.load_manifest(minecraft_folder, id.clone()).await;

        match resolved {
            Err(e) => {
                warn!("Failed to resolve minecraft version {id}: {e:?}");
                Broken(id)
            }
            Ok(data) => Normal(data),
        }
    }
}

impl FileSystemVersionLoader {
    async fn load_manifest(
        &self,
        root_dir: AbsPathBuf,
        version_id: impl Into<String>,
    ) -> Result<VersionManifest> {
        let json_path = self.resolve_json_path(root_dir, version_id);
        let content = fs::read_to_string(&json_path).await?;
        let manifest: VersionManifest = serde_json::from_str(&content)?;
        Ok(manifest)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_load_error_io_not_found() {
        let temp_dir = tempdir().unwrap();
        let root_path = temp_dir.path().to_path_buf().try_into().unwrap();
        let loader = FileSystemVersionLoader;

        let result = loader.load_manifest(root_path, "non-existent").await;

        assert!(
            matches!(result, Err(_)),
            "Should return error when file is missing, but got: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_load_error_json_parse() {
        let temp_dir = tempdir().unwrap();
        let root_path: AbsPathBuf = temp_dir.path().to_path_buf().try_into().unwrap();
        let loader = FileSystemVersionLoader;

        let version_dir = root_path.join(VERSIONS_DIR_NAME).join("bad_json");
        fs::create_dir_all(&version_dir).await.unwrap();

        let json_path = version_dir.join("bad_json.json");

        let mut file = File::create(&json_path).await.unwrap();
        file.write_all(b"{ \"id\": \"bad_json\", \"incomplete\": ")
            .await
            .unwrap();
        file.flush().await.unwrap();

        let result = loader.load_manifest(root_path, "bad_json").await;

        assert!(
            matches!(result, Err(_)),
            "Should return error on invalid JSON, but got: {:?}",
            result
        );
    }
}
