use crate::settings::modules::theme::{ThemeSettings, ThemeEffect};
use crate::utils::os_info::{is_macos, is_windows};
use log::warn;
use os_info::Info;

impl ThemeSettings {
    pub fn sanitize(&mut self, os_info: &Info) {
        if is_windows(os_info) && self.effect == ThemeEffect::Vibrancy {
            self.effect = ThemeEffect::Auto;
            warn!("Vibrancy effect is not supported on Windows. Fallback to Auto.");
        }

        if is_macos(os_info) && self.effect == ThemeEffect::Mica {
            self.effect = ThemeEffect::Auto;
            warn!("Mica effect is not supported on macOS. Fallback to Auto.");
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::modules::theme::ThemeMode;
    use crate::utils::os_info::mock_info;
    use os_info::Type;

    #[test]
    fn test_macos_settings_should_be_sanitize() {
        let os_info = mock_info(Type::Windows, "10.0", "x86_64");

        let mut macos_settings_should_be_sanitize = ThemeSettings {
            effect: ThemeEffect::Vibrancy,
            mode: ThemeMode::Dark,
            test: Default::default(),
            test_group: Default::default(),
        };

        macos_settings_should_be_sanitize.sanitize(&os_info);

        assert_eq!(macos_settings_should_be_sanitize.effect, ThemeEffect::Auto);
        assert_eq!(macos_settings_should_be_sanitize.mode, ThemeMode::Dark);
    }

    #[test]
    fn test_windows_settings_should_be_sanitize() {
        let os_info = mock_info(Type::Macos, "14.0", "arm64");

        let mut windows_settings_should_be_sanitize = ThemeSettings {
            effect: ThemeEffect::Mica,
            mode: ThemeMode::Dark,
            test: Default::default(),
            test_group: Default::default(),
        };

        windows_settings_should_be_sanitize.sanitize(&os_info);

        assert_eq!(windows_settings_should_be_sanitize.effect, ThemeEffect::Auto);
        assert_eq!(windows_settings_should_be_sanitize.mode, ThemeMode::Dark);
    }

    #[test]
    fn test_settings_should_not_be_sanitize() {
        let os_info = mock_info(Type::Macos, "14.0", "arm64");

        let mut settings_should_not_be_sanitize = ThemeSettings {
            effect: ThemeEffect::Auto,
            mode: ThemeMode::Light,
            test: Default::default(),
            test_group: Default::default(),
        };

        settings_should_not_be_sanitize.sanitize(&os_info);

        assert_eq!(settings_should_not_be_sanitize.effect, ThemeEffect::Auto);
        assert_eq!(settings_should_not_be_sanitize.mode, ThemeMode::Light);
    }
}
