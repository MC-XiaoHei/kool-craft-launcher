use crate::detector::model::VersionManifest;
use crate::detector::scanner::VersionMetadata;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;

#[derive(Error, Debug)]
pub enum VersionLoadError {
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON Parse Error: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("Circular Dependency Detected: {0}")]
    CircularDependency(String),
}

pub struct VersionLoader;

impl VersionLoader {
    pub async fn load_and_resolve(
        &self,
        metadata: &VersionMetadata,
        root_dir: &Path
    ) -> Result<VersionManifest, VersionLoadError> {
        let mut chain = Vec::new();
        let mut current_id = Some(metadata.id.clone());
        let mut visited = Vec::new();

        while let Some(id) = current_id {
            if visited.contains(&id) {
                return Err(VersionLoadError::CircularDependency(id));
            }
            visited.push(id.clone());

            let json_path = self.resolve_json_path(root_dir, &id);

            let content = fs::read_to_string(json_path).await?;

            let manifest: VersionManifest = serde_json::from_str(&content)?;

            current_id = manifest.inherits_from.clone();
            chain.push(manifest);
        }

        chain.reverse();
        let mut resolved = chain.first().cloned().unwrap();

        for child in chain.iter().skip(1) {
            self.merge(&mut resolved, child);
        }

        Ok(resolved)
    }

    fn resolve_json_path(&self, root_dir: &Path, version_id: &str) -> PathBuf {
        root_dir.join("versions").join(version_id).join(format!("{}.json", version_id))
    }

    fn merge(&self, base: &mut VersionManifest, child: &VersionManifest) {
        base.id = child.id.clone();
        base.version_type = child.version_type.clone();
        base.main_class = child.main_class.clone();

        if let Some(child_args) = &child.arguments {
            match &mut base.arguments {
                Some(base_args) => {
                    base_args.game.extend(child_args.game.clone());
                    base_args.jvm.extend(child_args.jvm.clone());
                }
                None => base.arguments = Some(child_args.clone()),
            }
        }

        if let Some(ma) = &child.minecraft_arguments {
            base.minecraft_arguments = Some(ma.clone());
        }

        base.libraries.extend(child.libraries.clone());

        if child.asset_index.is_some() {
            base.asset_index = child.asset_index.clone();
        }

        if child.java_version.is_some() {
            base.java_version = child.java_version.clone();
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use tempfile::tempdir;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;
    use super::*;
    use crate::detector::model::{ArgumentValue, Arguments, AssetIndex, JavaVersion};

    struct ManifestBuilder {
        manifest: VersionManifest,
    }

    impl ManifestBuilder {
        fn new(id: &str) -> Self {
            Self {
                manifest: VersionManifest {
                    id: id.to_string(),
                    inherits_from: None,
                    time: "2023-01-01T00:00:00Z".to_string(),
                    release_time: "2023-01-01T00:00:00Z".to_string(),
                    version_type: "release".to_string(),
                    main_class: "net.minecraft.client.main.Main".to_string(),
                    minecraft_arguments: None,
                    arguments: None,
                    libraries: vec![],
                    asset_index: None,
                    downloads: None,
                    java_version: None,
                }
            }
        }

        fn with_args(mut self, game: Vec<&str>, jvm: Vec<&str>) -> Self {
            let game_args = game.into_iter().map(|s| ArgumentValue::Simple(s.to_string())).collect();
            let jvm_args = jvm.into_iter().map(|s| ArgumentValue::Simple(s.to_string())).collect();
            self.manifest.arguments = Some(Arguments { game: game_args, jvm: jvm_args });
            self
        }

        fn with_legacy_args(mut self, args: &str) -> Self {
            self.manifest.minecraft_arguments = Some(args.to_string());
            self
        }

        fn with_asset_index(mut self, id: &str) -> Self {
            self.manifest.asset_index = Some(AssetIndex {
                id: id.to_string(),
                sha1: "dummy_sha1".to_string(),
                size: 0,
                url: "http://dummy".to_string(),
                total_size: 0,
            });
            self
        }

        fn with_java(mut self, major: u32) -> Self {
            self.manifest.java_version = Some(JavaVersion {
                component: "java-runtime-alpha".to_string(),
                major_version: major,
            });
            self
        }

        fn build(self) -> VersionManifest {
            self.manifest
        }
    }

    #[test]
    fn test_merge_branch_arguments_append() {
        let loader = VersionLoader;

        let mut base = ManifestBuilder::new("base")
            .with_args(vec!["--parentGame"], vec!["-DparentJvm"])
            .build();

        let child = ManifestBuilder::new("child")
            .with_args(vec!["--childGame"], vec!["-DchildJvm"])
            .build();

        loader.merge(&mut base, &child);

        let args = base.arguments.unwrap();

        let game_vals: Vec<String> = args.game.iter().filter_map(|a| match a {
            ArgumentValue::Simple(s) => Some(s.clone()), _ => None
        }).collect();

        assert_eq!(game_vals, vec!["--parentGame", "--childGame"], "Game args should be appended");
    }

    #[test]
    fn test_merge_branch_arguments_create_if_missing() {
        let loader = VersionLoader;
        let mut base = ManifestBuilder::new("base").build();
        let child = ManifestBuilder::new("child")
            .with_args(vec!["--child"], vec![])
            .build();

        loader.merge(&mut base, &child);

        assert!(base.arguments.is_some(), "Should create arguments object if base is None");
    }

    #[test]
    fn test_merge_branch_legacy_arguments_override() {
        let loader = VersionLoader;
        let mut base = ManifestBuilder::new("base")
            .with_legacy_args("old args")
            .build();
        let child = ManifestBuilder::new("child")
            .with_legacy_args("new args")
            .build();

        loader.merge(&mut base, &child);

        assert_eq!(base.minecraft_arguments, Some("new args".to_string()), "Legacy args should be overridden");
    }

    #[test]
    fn test_merge_branch_legacy_arguments_preserve() {
        let loader = VersionLoader;
        let mut base = ManifestBuilder::new("base")
            .with_legacy_args("old args")
            .build();
        let child = ManifestBuilder::new("child").build();

        loader.merge(&mut base, &child);

        assert_eq!(base.minecraft_arguments, Some("old args".to_string()), "Should preserve base args if child is None");
    }

    #[test]
    fn test_merge_branch_asset_index_override() {
        let loader = VersionLoader;
        let mut base = ManifestBuilder::new("base")
            .with_asset_index("1.12")
            .build();
        let child = ManifestBuilder::new("child")
            .with_asset_index("1.12-modded")
            .build();

        loader.merge(&mut base, &child);

        assert_eq!(base.asset_index.unwrap().id, "1.12-modded");
    }

    #[test]
    fn test_merge_branch_asset_index_ignore_if_none() {
        let loader = VersionLoader;
        let mut base = ManifestBuilder::new("base")
            .with_asset_index("1.12")
            .build();
        let child = ManifestBuilder::new("child").build();

        loader.merge(&mut base, &child);

        assert_eq!(base.asset_index.unwrap().id, "1.12");
    }

    #[test]
    fn test_merge_branch_java_version_override() {
        let loader = VersionLoader;
        let mut base = ManifestBuilder::new("base")
            .with_java(8)
            .build();
        let child = ManifestBuilder::new("child")
            .with_java(17)
            .build();

        loader.merge(&mut base, &child);

        assert_eq!(base.java_version.unwrap().major_version, 17);
    }

    #[tokio::test]
    async fn test_load_error_io_not_found() {
        let temp_dir = tempdir().unwrap();
        let root_path = temp_dir.path();
        let loader = VersionLoader;

        let meta = VersionMetadata {
            id: "non-existent".to_string(),
            json_path: root_path.join("versions/non-existent/non-existent.json"),
            jar_path: root_path.join("versions/non-existent/non-existent.jar"),
        };

        let result = loader.load_and_resolve(&meta, root_path).await;

        assert!(
            matches!(result, Err(VersionLoadError::Io(_))),
            "Should return IO error when file is missing, but got: {:?}", result
        );
    }

    #[tokio::test]
    async fn test_load_error_json_parse() {
        let temp_dir = tempdir().unwrap();
        let root_path = temp_dir.path();
        let loader = VersionLoader;

        let version_dir = root_path.join("versions").join("bad_json");
        fs::create_dir_all(&version_dir).await.unwrap();

        let json_path = version_dir.join("bad_json.json");

        let mut file = File::create(&json_path).await.unwrap();
        file.write_all(b"{ \"id\": \"bad_json\", \"incomplete\": ").await.unwrap();
        file.flush().await.unwrap();

        let meta = VersionMetadata {
            id: "bad_json".to_string(),
            json_path,
            jar_path: version_dir.join("bad_json.jar"),
        };

        let result = loader.load_and_resolve(&meta, root_path).await;

        assert!(
            matches!(result, Err(VersionLoadError::Parse(_))),
            "Should return Parse error on invalid JSON, but got: {:?}", result
        );
    }
}