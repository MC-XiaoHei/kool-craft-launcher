use anyhow::Result;
use macros::{config, config_type};

#[config(name = "theme", post_process = post_process)]
#[serde(rename_all = "camelCase")]
pub struct ThemeConfig {
    pub effect: EffectMode,
    pub theme: ThemeMode,
}

fn post_process(config: &mut ThemeConfig) -> Result<()> {
    let os_info = os_info::get();
    config.sanitize(&os_info);
    Ok(())
}

#[config_type]
#[derive(Default, PartialEq, Eq)]
pub enum EffectMode {
    #[default]
    Auto,
    Mica,
    Vibrancy,
    Wallpaper,
}

#[config_type]
#[derive(Default, PartialEq, Eq)]
pub enum ThemeMode {
    #[default]
    Auto,
    Dark,
    Light,
}
