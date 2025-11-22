use anyhow::{Context, Result, anyhow};
use log::{error, info, warn};
use std::fs;
use std::path::PathBuf;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum EffectMode {
    Auto,
    Mica,
    Vibrancy,
    Wallpaper,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum ThemeMode {
    Auto,
    Dark,
    Light,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
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
    fn config_path() -> Result<PathBuf> {
        dirs::home_dir()
            .map(|h| h.join(CONFIG_DIR).join(CONFIG_FILE))
            .ok_or_else(|| anyhow!("Unable to determine user home directory"))
    }

    fn sanitize(&mut self) {
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

    fn try_load() -> Result<Option<Self>> {
        let path = Self::config_path()?;

        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file at {:?}", path))?;

        let config =
            serde_json::from_str(&content).context("Failed to parse config JSON content")?;

        Ok(Some(config))
    }

    fn try_save(&self) -> Result<()> {
        let path = Self::config_path()?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to ensure config directory exists")?;
        }

        let content =
            serde_json::to_string_pretty(self).context("Failed to serialize theme config")?;

        fs::write(&path, content)
            .with_context(|| format!("Failed to write config to {:?}", path))?;

        Ok(())
    }

    pub fn load() -> Self {
        match Self::try_load() {
            Ok(Some(mut config)) => {
                config.sanitize();
                info!("Theme configuration loaded.");
                config
            }
            Ok(None) => {
                info!("No config file found. Using default theme.");
                Self::default()
            }
            Err(e) => {
                warn!("Failed to load config: {:#}, using default.", e);
                Self::default()
            }
        }
    }

    pub fn save(&self) {
        if let Err(e) = self.try_save() {
            error!("Failed to save theme config: {:#}", e);
        } else {
            info!("Theme configuration saved.");
        }
    }
}
