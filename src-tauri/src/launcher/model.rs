use crate::auth::model::PlayerProfile;
use crate::auth::model::UserType::Demo;
use crate::constants::launcher::{LAUNCHER_NAME, LAUNCHER_VERSION, SHORT_LAUNCHER_NAME};
use crate::constants::minecraft_behavior::DEFAULT_VERSION_INDEPENDENT;
use crate::constants::minecraft_dir::{ASSETS_DIR_NAME, VERSIONS_DIR_NAME};
use crate::java_runtime::model::JavaRuntime;
use crate::resolver::VersionManifest;
use crate::resolver::model::{
    Arguments, AssetIndex, Downloads, JavaVersion, Library, Logging, MinecraftFolderInfo,
};
use crate::utils::abs_path_buf::AbsPathBuf;
use LaunchError::IncompleteVersionManifest;
use anyhow::Result;
use os_info::Info;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;
use tap::Pipe;
use thiserror::Error;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LaunchRequest {
    pub minecraft_folder_info: MinecraftFolderInfo,
    pub manifest: LaunchVersionManifest,
    pub player_profile: PlayerProfile,
    pub java_profile: JavaRuntime,
    pub custom_info: CustomInfo,
}

impl LaunchRequest {
    pub fn new(
        minecraft_folder_info: MinecraftFolderInfo,
        manifest: LaunchVersionManifest,
        java_profile: JavaRuntime,
        custom_info: CustomInfo,
        player_profile: PlayerProfile,
    ) -> Result<Self> {
        Ok(LaunchRequest {
            minecraft_folder_info,
            manifest,
            java_profile,
            custom_info,
            player_profile,
        })
    }

    pub fn get_classpath_str(&self) -> Result<String> {
        let res = self
            .manifest
            .libraries
            .iter()
            .filter_map(|l| l.to_classpath_entry(self.get_natives_dir(), self.get_rule_context()))
            .pipe(std::env::join_paths)?;
        Ok(res.to_string_lossy().to_string())
    }

    pub fn get_natives_dir(&self) -> AbsPathBuf {
        self.minecraft_folder_info
            .path
            .join(VERSIONS_DIR_NAME)
            .join(self.manifest.id.clone())
            .join(VERSIONS_DIR_NAME)
    }

    pub fn get_natives_dir_str(&self) -> Result<String> {
        Ok(self.get_natives_dir().to_string_lossy().to_string())
    }

    pub fn get_game_dir(&self) -> AbsPathBuf {
        if self
            .minecraft_folder_info
            .settings
            .is_version_independent
            .unwrap_or(DEFAULT_VERSION_INDEPENDENT)
        {
            self.minecraft_folder_info
                .path
                .join(VERSIONS_DIR_NAME)
                .join(self.manifest.id.clone())
        } else {
            self.minecraft_folder_info.path.clone()
        }
    }

    pub fn get_game_dir_str(&self) -> String {
        self.get_game_dir().to_string_lossy().to_string()
    }

    pub fn get_assets_dir(&self) -> AbsPathBuf {
        self.minecraft_folder_info.path.join(ASSETS_DIR_NAME)
    }

    pub fn get_assets_dir_str(&self) -> String {
        self.get_assets_dir().to_string_lossy().to_string()
    }

    pub fn get_rule_context(&self) -> RuleContext {
        RuleContext {
            os_info: os_info::get(),
            user_features: self.get_user_features(),
        }
    }

    pub fn get_user_features(&self) -> HashMap<String, bool> {
        HashMap::from([
            ("has_quick_plays_support".into(), false), // never use quick play file
            ("is_demo_user".into(), self.player_profile.user_type == Demo),
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

    pub fn get_arguments_context(&self) -> Result<ArgumentsContext> {
        let custom = &self.custom_info;

        let ctx = ArgumentsContext {
            user_type: self.player_profile.user_type.to_string(),
            user_properties: "{}".into(),
            auth_player_name: self.player_profile.name.clone(),
            auth_access_token: self.player_profile.access_token.clone(),
            auth_uuid: self.player_profile.uuid.simple().to_string(),
            auth_xuid: self.player_profile.xuid.clone(),
            version_name: self.manifest.id.clone(),
            version_type: SHORT_LAUNCHER_NAME.into(),
            game_directory: self.get_game_dir_str(),
            natives_directory: self.get_natives_dir_str()?,
            assets_index_name: self.manifest.asset_index.id.clone(),
            assets_root: self.get_assets_dir_str(),
            game_assets: self.get_assets_dir_str(),
            resolution_width: custom.custom_resolution.clone().unwrap_or_default().width,
            resolution_height: custom.custom_resolution.clone().unwrap_or_default().height,
            quick_play_single_player: custom.quick_play.get_single_player().unwrap_or_default(),
            quick_play_multi_player: custom.quick_play.get_multi_player().unwrap_or_default(),
            quick_play_realms: custom.quick_play.get_realms().unwrap_or_default(),
            launcher_name: LAUNCHER_NAME.into(),
            launcher_version: LAUNCHER_VERSION.into(),
            client_id: "".into(), // TODO
            classpath: self.get_classpath_str()?,
        };

        Ok(ctx)
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, Eq, PartialEq)]
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

impl QuickPlayInfo {
    pub fn get_single_player(&self) -> Option<String> {
        if let QuickPlayInfo::SinglePlayer(s) = self {
            Some(s.clone())
        } else {
            None
        }
    }

    pub fn get_multi_player(&self) -> Option<String> {
        if let QuickPlayInfo::MultiPlayer(s) = self {
            Some(s.clone())
        } else {
            None
        }
    }

    pub fn get_realms(&self) -> Option<String> {
        if let QuickPlayInfo::Realms(s) = self {
            Some(s.clone())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ArgumentsContext {
    // yeah, I know this is VERY complex, but I don't have any better idea...
    pub user_type: String,
    pub user_properties: String,
    pub auth_player_name: String,
    pub auth_access_token: String,
    pub auth_uuid: String,
    pub auth_xuid: String,
    pub version_name: String,
    pub version_type: String,
    pub game_directory: String,
    pub natives_directory: String,
    pub assets_root: String,
    pub game_assets: String,
    pub assets_index_name: String,
    pub resolution_width: u64,
    pub resolution_height: u64,
    #[serde(rename = "quickPlaySingleplayer")]
    pub quick_play_single_player: String,
    #[serde(rename = "quickPlayMultiplayer")]
    pub quick_play_multi_player: String,
    #[serde(rename = "quickPlayRealms")]
    pub quick_play_realms: String,
    pub launcher_name: String,
    pub launcher_version: String,
    #[serde(rename = "clientid")]
    pub client_id: String,
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
    #[error("Invalid version '{0}'")]
    InvalidVersionName(String),
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
        let result_no_replace = ArgumentsContext::replace_placeholder(no_replace, &launch_args);
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
