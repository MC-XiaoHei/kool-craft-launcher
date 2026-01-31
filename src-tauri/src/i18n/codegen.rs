use anyhow::Result;
use tap::Pipe;

pub fn generate_i18n_keys_def() -> Result<String> {
    i18n_parser::get_translate_keys()?
        .into_iter()
        .map(|key| format!("\"{key}\""))
        .collect::<Vec<_>>()
        .join(" | ")
        .pipe(|content| format!("export type I18nKeys = {content};"))
        .pipe(Ok)
}
