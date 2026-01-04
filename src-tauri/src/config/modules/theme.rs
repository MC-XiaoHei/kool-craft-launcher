use anyhow::Result;
use macros::config;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[config(name = "theme", post_process = post_process)]
pub struct ThemeConfig {
    pub effect: EffectMode,
    pub theme: ThemeMode,
}

fn post_process(config: &mut ThemeConfig) -> Result<()> {
    let os_info = os_info::get();
    config.sanitize(&os_info);
    Ok(())
}

#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug, PartialEq, Eq, JsonSchema)]
pub enum EffectMode {
    #[default]
    Auto,
    Mica,
    Vibrancy,
    Wallpaper,
}

#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug, PartialEq, Eq, JsonSchema)]
pub enum ThemeMode {
    #[default]
    Auto,
    Dark,
    Light,
}
