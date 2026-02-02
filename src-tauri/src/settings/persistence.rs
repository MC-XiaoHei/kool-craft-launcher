use crate::settings::traits::SettingsPersistence;
use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs::{File, copy, create_dir_all, read_to_string, remove_file, rename};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

pub struct FilePersistence {
    path: PathBuf,
}

impl FilePersistence {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn backup_path(&self) -> PathBuf {
        self.path.with_extension("backup")
    }

    async fn atomic_write(&self, content: String) -> Result<()> {
        let parent = self
            .path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));

        if !parent.exists() {
            create_dir_all(parent)
                .await
                .with_context(|| format!("Failed to create directory {parent:?}"))?;
        }

        let temp_name = format!(".{}.tmp", Uuid::new_v4());
        let temp_path = parent.join(temp_name);

        {
            let mut file = File::create(&temp_path)
                .await
                .with_context(|| format!("Failed to create temp file {temp_path:?}"))?;

            file.write_all(content.as_bytes())
                .await
                .with_context(|| "Failed to write content to temp file")?;

            file.flush()
                .await
                .with_context(|| "Failed to flush temp file")?;
        }

        if let Err(e) = rename(&temp_path, &self.path).await {
            let _ = remove_file(&temp_path).await;
            return Err(anyhow::anyhow!("Failed to rename settings file: {e}"));
        }

        Ok(())
    }
}

#[async_trait]
impl SettingsPersistence for FilePersistence {
    async fn load(&self) -> Result<Option<String>> {
        match read_to_string(&self.path).await {
            Ok(content) => Ok(Some(content)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(anyhow::anyhow!("Failed to read file: {e}")),
        }
    }

    async fn backup(&self) -> Result<()> {
        copy(&self.path, &self.backup_path()).await?;
        Ok(())
    }

    async fn save(&self, content: String) -> Result<()> {
        self.atomic_write(content).await
    }

    fn source_description(&self) -> String {
        format!("{:?}", self.path)
    }
}
