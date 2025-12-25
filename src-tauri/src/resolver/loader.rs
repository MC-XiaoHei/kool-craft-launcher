use crate::resolver::model::VersionManifest;
use crate::resolver::scanner::VersionMetadata;
use async_trait::async_trait;
use std::path::PathBuf;
use tap::Pipe;
use thiserror::Error;
use tokio::fs;

#[derive(Error, Debug)]
pub enum VersionLoadError {
    #[error("I/O Error loading {path}: {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },
    #[error("JSON Parse Error in {path}: {source}")]
    Parse {
        path: String,
        source: serde_json::Error,
    },
    #[error("Circular Dependency Detected: {0}")]
    CircularDependency(String),
}

#[async_trait]
pub trait VersionLoader: Send + Sync {
    async fn load_and_resolve(
        &self,
        minecraft_folder: PathBuf,
        metadata: VersionMetadata,
    ) -> Result<VersionManifest, VersionLoadError>;
}

pub struct FileSystemVersionLoader;

impl FileSystemVersionLoader {
    fn resolve_json_path(
        &self,
        minecraft_folder: PathBuf,
        version_id: impl Into<String>,
    ) -> PathBuf {
        let version_id = version_id.into();
        minecraft_folder
            .join("versions")
            .join(version_id.clone())
            .join(format!("{version_id}.json"))
    }
}

#[async_trait]
impl VersionLoader for FileSystemVersionLoader {
    async fn load_and_resolve(
        &self,
        minecraft_folder: PathBuf,
        metadata: VersionMetadata,
    ) -> Result<VersionManifest, VersionLoadError> {
        let resolved = self
            .build_inheritance_chain(minecraft_folder, metadata.id)
            .await?
            .pipe(|chain| self.merge_chain(chain));

        Ok(resolved)
    }
}

impl FileSystemVersionLoader {
    async fn build_inheritance_chain(
        &self,
        root_dir: PathBuf,
        start_id: impl Into<String>,
    ) -> Result<Vec<VersionManifest>, VersionLoadError> {
        let mut chain = Vec::new();
        let mut current_id = Some(start_id.into());
        let mut visited = Vec::new();

        while let Some(id) = current_id {
            if visited.contains(&id) {
                return Err(VersionLoadError::CircularDependency(id));
            }
            visited.push(id.clone());

            let manifest = self.load_single_manifest(root_dir.clone(), &id).await?;

            current_id = manifest.inherits_from.clone();
            chain.push(manifest);
        }

        Ok(chain)
    }

    async fn load_single_manifest(
        &self,
        root_dir: PathBuf,
        version_id: impl Into<String>,
    ) -> Result<VersionManifest, VersionLoadError> {
        let json_path = self.resolve_json_path(root_dir.to_path_buf(), version_id);

        let content = fs::read_to_string(&json_path)
            .await
            .map_err(|e| VersionLoadError::Io {
                path: json_path.display().to_string(),
                source: e,
            })?;

        let manifest: VersionManifest =
            serde_json::from_str(&content).map_err(|e| VersionLoadError::Parse {
                path: json_path.display().to_string(),
                source: e,
            })?;

        Ok(manifest)
    }

    fn merge_chain(&self, mut chain: Vec<VersionManifest>) -> VersionManifest {
        chain.reverse();

        if chain.is_empty() {
            return VersionManifest::default();
        }

        let mut resolved = chain[0].clone();

        for child in chain.iter().skip(1) {
            resolved.merge_with(child);
        }

        resolved
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
        let root_path = temp_dir.path();
        let loader = FileSystemVersionLoader;

        let meta = VersionMetadata {
            id: "non-existent".into(),
            json_path: root_path.join("versions/non-existent/non-existent.json"),
            jar_path: root_path.join("versions/non-existent/non-existent.jar"),
        };

        let result = loader.load_and_resolve(root_path.to_path_buf(), meta).await;

        assert!(
            matches!(result, Err(VersionLoadError::Io { .. })),
            "Should return IO error when file is missing, but got: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_load_error_json_parse() {
        let temp_dir = tempdir().unwrap();
        let root_path = temp_dir.path();
        let loader = FileSystemVersionLoader;

        let version_dir = root_path.join("versions").join("bad_json");
        fs::create_dir_all(&version_dir).await.unwrap();

        let json_path = version_dir.join("bad_json.json");

        let mut file = File::create(&json_path).await.unwrap();
        file.write_all(b"{ \"id\": \"bad_json\", \"incomplete\": ")
            .await
            .unwrap();
        file.flush().await.unwrap();

        let meta = VersionMetadata {
            id: "bad_json".into(),
            json_path,
            jar_path: version_dir.join("bad_json.jar"),
        };

        let result = loader.load_and_resolve(root_path.to_path_buf(), meta).await;

        assert!(
            matches!(result, Err(VersionLoadError::Parse { .. })),
            "Should return Parse error on invalid JSON, but got: {:?}",
            result
        );
    }
}
