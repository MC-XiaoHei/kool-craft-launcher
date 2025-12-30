#![cfg_attr(coverage_nightly, coverage(off))]
#![cfg(test)]

use crate::ui_theme::model::{EffectMode, ThemeConfig, ThemeMode};
use std::{env, fs};
use uuid::Uuid;

#[test]
fn test_default_values() {
    let config = ThemeConfig::default();
    assert_eq!(config.effect, EffectMode::Auto);
    assert_eq!(config.theme, ThemeMode::Auto);
}

#[test]
fn test_json_serialization() {
    let config = ThemeConfig {
        effect: EffectMode::Vibrancy,
        theme: ThemeMode::Dark,
    };

    let json = serde_json::to_string(&config).expect("Should serialize");
    let loaded: ThemeConfig = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(loaded.effect, EffectMode::Vibrancy);
    assert_eq!(loaded.theme, ThemeMode::Dark);
}

#[test]
fn test_sanitize() {
    let mut config_should_be_sanitize = ThemeConfig {
        #[cfg(target_os = "macos")]
        effect: EffectMode::Mica,
        #[cfg(target_os = "windows")]
        effect: EffectMode::Vibrancy,
        theme: ThemeMode::Dark,
    };

    config_should_be_sanitize.sanitize();

    assert_eq!(config_should_be_sanitize.effect, EffectMode::Auto);
    assert_eq!(config_should_be_sanitize.theme, ThemeMode::Dark);

    let mut config_should_not_be_sanitize = ThemeConfig {
        effect: EffectMode::Auto,
        theme: ThemeMode::Light,
    };

    config_should_not_be_sanitize.sanitize();

    assert_eq!(config_should_not_be_sanitize.effect, EffectMode::Auto);
    assert_eq!(config_should_not_be_sanitize.theme, ThemeMode::Light);
}

#[test]
fn test_file_io_lifecycle() {
    let temp_dir = env::temp_dir();
    let temp_file = temp_dir.join(format!("kcl_test_theme_{}.json", Uuid::new_v4()));

    let load_result = ThemeConfig::load_from(&temp_file);
    assert!(load_result.is_ok());
    assert!(load_result.unwrap().is_none());

    let config_to_save = ThemeConfig {
        effect: EffectMode::Vibrancy,
        theme: ThemeMode::Light,
    };
    let save_result = config_to_save.save_to(&temp_file);
    assert!(save_result.is_ok(), "Save failed: {:?}", save_result.err());
    assert!(temp_file.exists(), "File should be created");

    let loaded_result = ThemeConfig::load_from(&temp_file);
    assert!(
        loaded_result.is_ok(),
        "Load failed: {:?}",
        loaded_result.err()
    );

    let loaded_config = loaded_result.unwrap().expect("Should have config content");
    assert_eq!(loaded_config.effect, EffectMode::Vibrancy);
    assert_eq!(loaded_config.theme, ThemeMode::Light);

    let _ = fs::remove_file(temp_file);
}
