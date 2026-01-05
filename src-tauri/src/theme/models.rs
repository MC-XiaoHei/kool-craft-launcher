use crate::config::modules::theme::{EffectMode, ThemeConfig};
use crate::utils::os_info::{is_macos, is_windows};
use log::warn;
use os_info::Info;

impl ThemeConfig {
    pub fn sanitize(&mut self, os_info: &Info) {
        if is_windows(os_info) && self.effect == EffectMode::Vibrancy {
            self.effect = EffectMode::Auto;
            warn!("Vibrancy effect is not supported on Windows. Fallback to Auto.");
        }

        if is_macos(os_info) && self.effect == EffectMode::Mica {
            self.effect = EffectMode::Auto;
            warn!("Mica effect is not supported on macOS. Fallback to Auto.");
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::modules::theme::ThemeMode;
    use crate::utils::os_info::mock_info;
    use os_info::Type;

    #[test]
    fn test_macos_config_should_be_sanitize() {
        let os_info = mock_info(Type::Windows, "10.0", "x86_64");

        let mut macos_config_should_be_sanitize = ThemeConfig {
            effect: EffectMode::Vibrancy,
            theme: ThemeMode::Dark,
        };

        macos_config_should_be_sanitize.sanitize(&os_info);

        assert_eq!(macos_config_should_be_sanitize.effect, EffectMode::Auto);
        assert_eq!(macos_config_should_be_sanitize.theme, ThemeMode::Dark);
    }

    #[test]
    fn test_windows_config_should_be_sanitize() {
        let os_info = mock_info(Type::Macos, "14.0", "arm64");

        let mut windows_config_should_be_sanitize = ThemeConfig {
            effect: EffectMode::Mica,
            theme: ThemeMode::Dark,
        };

        windows_config_should_be_sanitize.sanitize(&os_info);

        assert_eq!(windows_config_should_be_sanitize.effect, EffectMode::Auto);
        assert_eq!(windows_config_should_be_sanitize.theme, ThemeMode::Dark);
    }

    #[test]
    fn test_config_should_not_be_sanitize() {
        let os_info = mock_info(Type::Macos, "14.0", "arm64");

        let mut config_should_not_be_sanitize = ThemeConfig {
            effect: EffectMode::Auto,
            theme: ThemeMode::Light,
        };

        config_should_not_be_sanitize.sanitize(&os_info);

        assert_eq!(config_should_not_be_sanitize.effect, EffectMode::Auto);
        assert_eq!(config_should_not_be_sanitize.theme, ThemeMode::Light);
    }
}
