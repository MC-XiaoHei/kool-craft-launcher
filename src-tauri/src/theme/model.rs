use anyhow::{Context, anyhow};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectMode {
    Auto,
    Mica,
    Vibrancy,
    Wallpaper,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeMode {
    Auto,
    Dark,
    Light,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ThemeConfig {
    pub effect: EffectMode,
    pub theme: ThemeMode,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            effect: EffectMode::Auto,
            theme: ThemeMode::Auto,
        }
    }
}

const CONFIG_DIR: &str = ".kcl";
const CONFIG_FILE: &str = "theme.json";

impl ThemeConfig {
    fn config_path() -> anyhow::Result<PathBuf> {
        dirs::home_dir()
            .map(|h| h.join(CONFIG_DIR).join(CONFIG_FILE))
            .ok_or_else(|| anyhow!("Unable to determine user home directory"))
    }

    pub(super) fn sanitize(&mut self) {
        #[cfg(target_os = "windows")]
        if self.effect == EffectMode::Vibrancy {
            self.effect = EffectMode::Auto;
            warn!("Vibrancy effect is not supported on Windows. Fallback to Auto.");
        }

        #[cfg(target_os = "macos")]
        if self.effect == EffectMode::Mica {
            self.effect = EffectMode::Auto;
            warn!("Mica effect is not supported on macOS. Fallback to Auto.");
        }
    }

    pub(super) fn load_from(path: &Path) -> anyhow::Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file at {:?}", path))?;

        let config =
            serde_json::from_str(&content).context("Failed to parse config JSON content")?;

        Ok(Some(config))
    }

    pub(super) fn save_to(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to ensure config directory exists")?;
        }

        let content =
            serde_json::to_string_pretty(self).context("Failed to serialize theme config")?;

        fs::write(path, content)
            .with_context(|| format!("Failed to write config to {:?}", path))?;

        Ok(())
    }

    pub fn load() -> Self {
        let path = match Self::config_path() {
            Ok(p) => p,
            Err(e) => {
                warn!("Could not determine config path: {:#}, using default.", e);
                return Self::default();
            }
        };

        match Self::load_from(&path) {
            Ok(Some(mut config)) => {
                config.sanitize();
                info!("Theme configuration loaded from {:?}", path);
                config
            }
            Ok(None) => {
                info!("No config file found at {:?}. Using default theme.", path);
                Self::default()
            }
            Err(e) => {
                warn!(
                    "Failed to load config from {:?}: {:#}, using default.",
                    path, e
                );
                Self::default()
            }
        }
    }

    pub fn save(&self) {
        match Self::config_path() {
            Ok(path) => {
                if let Err(e) = self.save_to(&path) {
                    error!("Failed to save theme config to {:?}: {:#}", path, e);
                } else {
                    info!("Theme configuration saved to {:?}.", path);
                }
            }
            Err(e) => {
                error!("Could not determine config path to save: {:#}", e);
            }
        }
    }
}
