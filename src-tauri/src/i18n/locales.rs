use crate::settings::modules::general::GeneralSettings;
use crate::settings::store::SettingsStore;
use crate::utils::global_app_handle::get_global_app_handle;
use anyhow::{Result, anyhow};
use arc_swap::ArcSwap;
use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::{LanguageIdentifier, Loader, static_loader};
use heck::ToKebabCase;
use i18n_parser::DEFAULT_LANG;
use log::info;
use std::borrow::Cow;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, OnceLock, RwLock};
use strum::IntoEnumIterator;
use sys_locale::get_locale;
use tap::Pipe;
use tauri::Manager;

static_loader! {
    static LOCALES = {
        locales: "../locales",
        fallback_language: "en-US",
    };
}

static GLOBAL_LANG: OnceLock<ArcSwap<LanguageIdentifier>> = OnceLock::new();

pub fn t(key: I18nKeys) -> String {
    LOCALES.lookup(&current_lang(), key.into())
}

pub fn t_args(key: I18nKeys, args: &HashMap<Cow<'static, str>, FluentValue>) -> String {
    LOCALES.lookup_with_args(&current_lang(), key.into(), args)
}

pub fn current_lang() -> LanguageIdentifier {
    global_lang_storage().load().as_ref().clone()
}

pub fn refresh_lang(settings: GeneralSettings) -> Result<()> {
    let lang_id = compute_lang_from_settings(settings);
    global_lang_storage().store(Arc::new(lang_id));
    Ok(())
}

fn global_lang_storage() -> &'static ArcSwap<LanguageIdentifier> {
    GLOBAL_LANG.get_or_init(|| ArcSwap::from_pointee(default_lang()))
}

fn compute_lang_from_settings(settings: GeneralSettings) -> LanguageIdentifier {
    let system_locale = get_system_locale().unwrap_or_default();
    let lang = settings.lang.unwrap_or_else(get_system_locale_or_default);
    lang.as_lang_id()
}

fn default_lang() -> LanguageIdentifier {
    DEFAULT_LANG
        .parse()
        .expect("Internal Error: Failed to parse default language") // this should never happen
}

pub fn get_system_locale_or_default() -> Locales {
    get_system_locale().unwrap_or_default()
}

pub fn get_system_locale() -> Option<Locales> {
    let normalized_sys_locale = get_locale()?.to_kebab_case();

    find_exact_match(&normalized_sys_locale)
        .or_else(|| find_match_by_prefix(&normalized_sys_locale))
}

fn find_exact_match(locale_str: &str) -> Option<Locales> {
    Locales::from_str(locale_str).ok()
}

fn find_match_by_prefix(sys_locale_str: &str) -> Option<Locales> {
    let sys_lang_code = extract_lang_code(sys_locale_str)?;

    Locales::iter().find(|enum_variant| {
        let enum_str: &str = enum_variant.into();
        let enum_normalized = enum_str.to_kebab_case();

        extract_lang_code(&enum_normalized) == Some(sys_lang_code)
    })
}

fn extract_lang_code(locale_str: &str) -> Option<&str> {
    locale_str.split('-').next()
}

include!(concat!(env!("OUT_DIR"), "/i18n_generated.rs"));

impl Locales {
    pub fn as_lang_id(&self) -> LanguageIdentifier {
        let lang_str: &str = self.into();
        lang_str
            .parse::<LanguageIdentifier>()
            .expect("Internal Error: Language code is not valid") // this should never happen. i18n/tests.rs#test_locales_all_valid
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use crate::i18n::locales::Locales;
    use fluent_templates::LanguageIdentifier;
    use strum::IntoEnumIterator;

    #[test]
    fn test_locales_all_valid() {
        Locales::iter().map(Into::into).all(|locale_code: &str| {
            locale_code
                .parse::<LanguageIdentifier>()
                .unwrap_or_else(|_| panic!("Invalid locale code {locale_code}"));
            true
        });
    }
}
