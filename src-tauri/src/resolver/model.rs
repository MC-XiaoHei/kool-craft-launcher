use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VersionManifest {
    pub arguments: Option<Arguments>,
    pub minecraft_arguments: Option<String>,
    pub asset_index: Option<AssetIndex>,
    pub assets: Option<String>,
    pub compliance_level: Option<u8>,
    pub downloads: Option<Downloads>,
    pub id: String,
    pub java_version: Option<JavaVersion>,
    #[serde(default)]
    pub libraries: Vec<Library>,
    #[serde(default)]
    pub logging: Option<Logging>,
    pub main_class: String,
    pub release_time: String,
    pub time: String,
    #[serde(rename = "type")]
    pub version_type: String,
    #[serde(default)]
    pub inherits_from: Option<String>,
    // when this struct modified, remember to update VersionManifest::merge_with
}

impl VersionManifest {
    pub fn merge_with(&mut self, child: &VersionManifest) {
        self.id = child.id.clone();
        self.version_type = child.version_type.clone();
        self.main_class = child.main_class.clone();

        if let Some(child_args) = &child.arguments {
            match &mut self.arguments {
                Some(base_args) => {
                    base_args.game.extend(child_args.game.clone());
                    base_args.jvm.extend(child_args.jvm.clone());
                }
                None => self.arguments = Some(child_args.clone()),
            }
        }

        if let Some(ma) = &child.minecraft_arguments {
            self.minecraft_arguments = Some(ma.clone());
        }

        self.libraries.extend(child.libraries.clone());

        if child.asset_index.is_some() {
            self.asset_index = child.asset_index.clone();
        }

        if child.java_version.is_some() {
            self.java_version = child.java_version.clone();
        }

        if child.downloads.is_some() {
            self.downloads = child.downloads.clone();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Arguments {
    #[serde(default)]
    pub game: Vec<ArgumentValue>,
    #[serde(default)]
    pub jvm: Vec<ArgumentValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum ArgumentValue {
    Simple(String),
    Complex {
        rules: Vec<Rule>,
        value: ArgumentValueContent,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum ArgumentValueContent {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
    pub total_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    pub name: String,
    #[serde(default)]
    pub downloads: Option<LibraryDownloads>,
    #[serde(default)]
    pub natives: Option<HashMap<String, String>>,
    #[serde(default)]
    pub rules: Option<Vec<Rule>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LibraryDownloads {
    pub artifact: Option<DownloadFile>,
    #[serde(default)]
    pub classifiers: Option<HashMap<String, DownloadFile>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DownloadFile {
    pub path: Option<String>,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Downloads {
    pub client: DownloadFile,
    #[serde(default)]
    pub client_mappings: Option<DownloadFile>,
    #[serde(default)]
    pub server: Option<DownloadFile>,
    #[serde(default)]
    pub server_mappings: Option<DownloadFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    pub component: String,
    pub major_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub action: String,
    #[serde(default)]
    pub os: Option<HashMap<String, String>>,
    #[serde(default)]
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Logging {
    #[serde(default)]
    pub client: Option<LoggingConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LoggingConfig {
    pub argument: String,
    pub file: DownloadFile,
    #[serde(rename = "type")]
    pub file_type: String,
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::resolver::VersionManifest;
    use crate::resolver::model::{
        ArgumentValue, Arguments, AssetIndex, DownloadFile, Downloads, JavaVersion,
    };

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
                    assets: None,
                    compliance_level: None,
                    downloads: None,
                    java_version: None,
                    logging: None,
                },
            }
        }

        fn with_args(mut self, game: Vec<&str>, jvm: Vec<&str>) -> Self {
            let game_args = game
                .into_iter()
                .map(|s| ArgumentValue::Simple(s.to_string()))
                .collect();
            let jvm_args = jvm
                .into_iter()
                .map(|s| ArgumentValue::Simple(s.to_string()))
                .collect();
            self.manifest.arguments = Some(Arguments {
                game: game_args,
                jvm: jvm_args,
            });
            self
        }

        fn with_legacy_args(mut self, args: &str) -> Self {
            self.manifest.minecraft_arguments = Some(args.to_string());
            self
        }

        fn with_client_jar(mut self, client: DownloadFile) -> Self {
            self.manifest.downloads = Some(Downloads {
                client,
                client_mappings: None,
                server: None,
                server_mappings: None,
            });
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
        let mut base = ManifestBuilder::new("base")
            .with_args(vec!["--parentGame"], vec!["-DparentJvm"])
            .build();

        let child = ManifestBuilder::new("child")
            .with_args(vec!["--childGame"], vec!["-DchildJvm"])
            .build();

        base.merge_with(&child);

        let args = base.arguments.unwrap();

        let game_vals: Vec<String> = args
            .game
            .iter()
            .filter_map(|a| match a {
                ArgumentValue::Simple(s) => Some(s.clone()),
                _ => None,
            })
            .collect();

        assert_eq!(
            game_vals,
            vec!["--parentGame", "--childGame"],
            "Game args should be appended"
        );
    }

    #[test]
    fn test_merge_branch_arguments_create_if_missing() {
        let mut base = ManifestBuilder::new("base").build();
        let child = ManifestBuilder::new("child")
            .with_args(vec!["--child"], vec![])
            .build();

        base.merge_with(&child);

        assert!(
            base.arguments.is_some(),
            "Should create arguments object if base is None"
        );
    }

    #[test]
    fn test_merge_branch_legacy_arguments_override() {
        let mut base = ManifestBuilder::new("base")
            .with_legacy_args("old args")
            .build();
        let child = ManifestBuilder::new("child")
            .with_legacy_args("new args")
            .build();

        base.merge_with(&child);

        assert_eq!(
            base.minecraft_arguments,
            Some("new args".to_string()),
            "Legacy args should be overridden"
        );
    }

    #[test]
    fn test_merge_branch_legacy_arguments_preserve() {
        let mut base = ManifestBuilder::new("base")
            .with_legacy_args("old args")
            .build();
        let child = ManifestBuilder::new("child").build();

        base.merge_with(&child);

        assert_eq!(
            base.minecraft_arguments,
            Some("old args".to_string()),
            "Should preserve base args if child is None"
        );
    }

    #[test]
    fn test_merge_branch_asset_index_override() {
        let mut base = ManifestBuilder::new("base")
            .with_asset_index("1.12")
            .build();
        let child = ManifestBuilder::new("child")
            .with_asset_index("1.12-modded")
            .build();

        base.merge_with(&child);

        assert_eq!(base.asset_index.unwrap().id, "1.12-modded");
    }

    #[test]
    fn test_merge_branch_asset_index_ignore_if_none() {
        let mut base = ManifestBuilder::new("base")
            .with_asset_index("1.12")
            .build();
        let child = ManifestBuilder::new("child").build();

        base.merge_with(&child);

        assert_eq!(base.asset_index.unwrap().id, "1.12");
    }

    #[test]
    fn test_merge_branch_java_version_override() {
        let mut base = ManifestBuilder::new("base").with_java(8).build();
        let child = ManifestBuilder::new("child").with_java(17).build();

        base.merge_with(&child);

        assert_eq!(base.java_version.unwrap().major_version, 17);
    }

    #[test]
    fn test_merge_branch_downloads_override() {
        let download_file = DownloadFile {
            path: Some("path/to/client".to_string()),
            sha1: "dummy_sha1".to_string(),
            size: 12345,
            url: "http://dummy".to_string(),
        };

        let mut base = ManifestBuilder::new("base").build();
        let child = ManifestBuilder::new("child")
            .with_client_jar(download_file.clone())
            .build();

        base.merge_with(&child);

        let downloads = base.downloads.unwrap();
        assert_eq!(
            download_file, downloads.client,
            "Downloads should be overridden by child"
        );
    }
}
