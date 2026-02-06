use crate::i18n::locales::{
    Locales, get_system_locale, get_system_locale_or_default, refresh_lang,
};
use crate::settings::modules::theme::{ThemeEffect, ThemeMode, ThemeSettings};
use crate::theme::effect::apply_effect;
use crate::utils::global_app_handle::get_global_app_handle;
use anyhow::Result;
use log::info;
use macros::settings;
use crate::settings::components::Language;

#[settings(name = "general", post_process = post_process, update_handler = on_update)]
pub struct GeneralSettings {
    pub lang: Language,
}

fn post_process(settings: &mut GeneralSettings) -> Result<()> {
    if settings.lang.is_none() {
        settings.lang = Language::some(get_system_locale_or_default());
    }
    Ok(())
}

fn on_update(neo: &GeneralSettings, _old: GeneralSettings) -> Result<()> {
    refresh_lang(neo.clone());
    Ok(())
}
