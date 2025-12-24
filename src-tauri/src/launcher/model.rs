use crate::java_runtime::model::JavaRuntime;
use crate::launcher::model::LaunchError::IncompleteVersionManifest;
use crate::resolver::VersionManifest;
use crate::resolver::model::{Arguments, AssetIndex, Downloads, JavaVersion, Library, Logging};
use os_info::Info;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LaunchError {
    #[error("Version manifest is incomplete and cannot be launched.")]
    IncompleteVersionManifest,
    #[error("No suitable Java Runtime Environment found.")]
    NoSuitableJava,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LaunchRequest {
    pub minecraft_folder: PathBuf,
    pub manifest: LaunchVersionManifest,
    pub java_profile: JavaRuntime,
    pub os_info: Info,
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
