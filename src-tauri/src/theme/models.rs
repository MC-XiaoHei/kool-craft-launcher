use crate::settings::modules::theme::{ThemeEffect, ThemeSettings};
use crate::utils::os_info::{is_macos, is_windows};
use log::warn;
use os_info::Info;
use std::cmp::PartialEq;

impl ThemeEffect {
    pub fn sanitize(&mut self, os_info: &Info) {
        if is_windows(os_info) && *self == ThemeEffect::Vibrancy {
            *self = ThemeEffect::Auto;
            warn!("Vibrancy effect is not supported on Windows. Fallback to Auto.");
        }

        if is_macos(os_info) && *self == ThemeEffect::Mica {
            *self = ThemeEffect::Auto;
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

        let mut macos_settings_should_be_sanitize = ThemeEffect::Vibrancy;

        macos_settings_should_be_sanitize.sanitize(&os_info);

        assert_eq!(macos_settings_should_be_sanitize, ThemeEffect::Auto);
    }

    #[test]
    fn test_windows_settings_should_be_sanitize() {
        let os_info = mock_info(Type::Macos, "14.0", "arm64");

        let mut windows_settings_should_be_sanitize = ThemeEffect::Mica;

        windows_settings_should_be_sanitize.sanitize(&os_info);

        assert_eq!(windows_settings_should_be_sanitize, ThemeEffect::Auto);
    }

    #[test]
    fn test_settings_should_not_be_sanitize() {
        let os_info = mock_info(Type::Macos, "14.0", "arm64");

        let mut settings_should_not_be_sanitize = ThemeEffect::Auto;

        settings_should_not_be_sanitize.sanitize(&os_info);

        assert_eq!(settings_should_not_be_sanitize, ThemeEffect::Auto);
    }
}
