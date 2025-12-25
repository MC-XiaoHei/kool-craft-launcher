use crate::java_runtime::model::JavaRuntime;
use crate::resolver::VersionManifest;
use crate::resolver::model::{Arguments, AssetIndex, Downloads, JavaVersion, Library, Logging};
use os_info::Info;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::LazyLock;
use thiserror::Error;
use LaunchError::IncompleteVersionManifest;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LaunchRequest {
    pub minecraft_folder: PathBuf,
    pub manifest: LaunchVersionManifest,
    pub java_profile: JavaRuntime,
    pub custom_info: CustomInfo,
    pub os_info: Info,
}

impl LaunchRequest {
    pub fn get_rule_context(&self) -> RuleContext {
        RuleContext {
            os_info: self.os_info.clone(),
            user_features: self.get_user_features(),
        }
    }

    pub fn get_user_features(&self) -> HashMap<String, bool> {
        HashMap::from([
            ("is_demo_user".into(), false), // TODO
            ("has_quick_plays_support".into(), false), // never use quick play file
            (
                "has_custom_resolution".into(),
                self.custom_info.custom_resolution.is_some(),
            ),
            (
                "is_quick_play_singleplayer".into(),
                matches!(self.custom_info.quick_play, QuickPlayInfo::SinglePlayer(_)),
            ),
            (
                "is_quick_play_multiplayer".into(),
                matches!(self.custom_info.quick_play, QuickPlayInfo::MultiPlayer(_)),
            ),
            (
                "is_quick_play_realms".into(),
                matches!(self.custom_info.quick_play, QuickPlayInfo::Realms(_)),
            ),
        ])
    }

    pub fn get_arguments_context(&self) -> ArgumentsContext {
        ArgumentsContext::default() // TODO
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CustomInfo {
    pub quick_play: QuickPlayInfo,
    pub custom_jvm_args: Vec<String>,
    pub custom_game_args: Vec<String>,
    pub custom_resolution: Option<GameResolution>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GameResolution {
    pub width: u64,
    pub height: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum QuickPlayInfo {
    SinglePlayer(String),
    MultiPlayer(String),
    Realms(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct ArgumentsContext {
    pub user_type: String,
    pub user_properties: String,
    pub auth_player_name: String,
    pub version_name: String,
    pub game_directory: String,
    pub assets_root: String,
    pub assets_index_name: String,
    pub auth_uuid: String,
    pub auth_access_token: String,
    #[serde(rename = "clientid")]
    pub client_id: String,
    pub auth_xuid: String,
    pub version_type: String,
    pub resolution_width: u64,
    pub resolution_height: u64,
    #[serde(rename = "quickPlayPath")]
    pub quick_play_path: String,
    #[serde(rename = "quickPlaySingleplayer")]
    pub quick_play_single_player: String,
    #[serde(rename = "quickPlayMultiplayer")]
    pub quick_play_multi_player: String,
    #[serde(rename = "quickPlayRealms")]
    pub quick_play_realms: String,
    pub natives_directory: String,
    pub launcher_name: String,
    pub launcher_version: String,
    pub classpath: String,
}

impl ArgumentsContext {
    pub fn replace_args_placeholders(&self, args: Vec<String>) -> Vec<String> {
        let map: HashMap<String, String> = serde_json::to_value(self)
            .unwrap_or_default()
            .as_object()
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                    .collect()
            })
            .unwrap_or_default();

        args.into_iter()
            .map(|arg| Self::replace_placeholder(arg, &map))
            .collect()
    }

    fn replace_placeholder(arg: impl Into<String>, map: &HashMap<String, String>) -> String {
        static RE: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"\$\{([^}]+)}")
                .expect("Internal error: Failed to compile placeholder regex") // this should never happen...
        });

        RE.replace_all(&arg.into(), |caps: &Captures| {
            let key = caps.get(1).map(|m| m.as_str()).unwrap_or("");

            if let Some(value) = map.get(key) {
                value.clone()
            } else {
                String::new()
            }
        })
        .into_owned()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LaunchVersionManifest {
    pub arguments: ArgumentsInfo,
    pub asset_index: AssetIndex,
    pub assets: String,
    pub compliance_level: u8,
    pub downloads: Downloads,
    pub id: String,
    pub java_version: JavaVersion,
    pub libraries: Vec<Library>,
    pub logging: Logging,
    pub main_class: String,
    pub release_time: String,
    pub time: String,
    pub version_type: String,
    pub inherits_from: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ArgumentsInfo {
    Legacy(String),
    Modern(Arguments),
}

#[derive(Debug, Clone)]
pub struct RuleContext {
    pub os_info: Info,
    pub user_features: HashMap<String, bool>,
}

#[derive(Debug, Error)]
pub enum LaunchError {
    #[error("Incomplete version manifest")]
    IncompleteVersionManifest,
    #[error("Invalid java runtime")]
    InvalidJavaRuntime,
}

impl TryFrom<VersionManifest> for LaunchVersionManifest {
    type Error = LaunchError;

    fn try_from(manifest: VersionManifest) -> Result<Self, Self::Error> {
        let arguments = if let Some(args) = manifest.arguments {
            ArgumentsInfo::Modern(args)
        } else if let Some(legacy_args) = manifest.minecraft_arguments {
            ArgumentsInfo::Legacy(legacy_args)
        } else {
            return Err(IncompleteVersionManifest);
        };
        let asset_index = manifest.asset_index.ok_or(IncompleteVersionManifest)?;
        let assets = manifest.assets.ok_or(IncompleteVersionManifest)?;
        let compliance_level = manifest.compliance_level.ok_or(IncompleteVersionManifest)?;
        let downloads = manifest.downloads.ok_or(IncompleteVersionManifest)?;
        let logging = manifest.logging.ok_or(IncompleteVersionManifest)?;
        let java_version = manifest.java_version.ok_or(IncompleteVersionManifest)?;

        Ok(LaunchVersionManifest {
            arguments,
            asset_index,
            assets,
            compliance_level,
            downloads,
            id: manifest.id,
            java_version,
            libraries: manifest.libraries,
            logging,
            main_class: manifest.main_class,
            release_time: manifest.release_time,
            time: manifest.time,
            version_type: manifest.version_type,
            inherits_from: manifest.inherits_from,
        })
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::launcher::model::ArgumentsContext;
    use std::collections::HashMap;

    #[test]
    fn test_placeholder_replacement() {
        let value = "test_value".to_string();
        let launch_args = HashMap::from([("placeholder".to_string(), value.clone())]);

        let result = ArgumentsContext::replace_placeholder("${placeholder}", &launch_args);
        assert_eq!(
            result,
            value.clone(),
            "Placeholder should be replaced with the correct value"
        );

        let no_replace = "no_replacement_needed";
        let result_no_replace =
            ArgumentsContext::replace_placeholder(no_replace, &launch_args);
        assert_eq!(
            result_no_replace, no_replace,
            "String without placeholder should remain unchanged"
        );

        let result_unknown_key =
            ArgumentsContext::replace_placeholder("${unknown_placeholder}", &launch_args);
        assert_eq!(
            result_unknown_key, "",
            "Placeholder should be replaced with \"\" when key is unknown"
        );
    }
}
