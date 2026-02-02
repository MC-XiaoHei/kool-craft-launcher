use anyhow::{Context, Result};
use heck::ToPascalCase;
use i18n_parser::{get_supported_locales, get_translate_keys, resolve_all_ftl_files, DEFAULT_LANG};
use indoc::formatdoc;
use std::path::Path;
use std::path::PathBuf;
use std::{env, fs};
use tauri_build::Attributes;

fn main() -> Result<()> {
    generate_i18n_codes()?;
    build()?;
    Ok(())
}

fn generate_i18n_codes() -> Result<()> {
    add_ftl_files_to_rerun_track_list()?;
    let keys_def = generate_keys_def()?;
    let locales_def = generate_locales_def()?;
    let content = formatdoc! { r#"
        // AUTOMATICALLY GENERATED. DO NOT EDIT.
        {keys_def}
        {locales_def}
    "# };
    write_to_output_dir("i18n_generated.rs", content)?;
    Ok(())
}

fn add_ftl_files_to_rerun_track_list() -> Result<()> {
    resolve_all_ftl_files()?
        .into_iter()
        .for_each(add_to_rerun_track_list);

    Ok(())
}

fn generate_keys_def() -> Result<String> {
    let elements = get_translate_keys()?
        .into_iter()
        .map(format_i18n_key_as_enum_element)
        .collect::<Vec<_>>()
        .join("\n");

    Ok(formatdoc! { r#"
        #[allow(dead_code)]
        #[derive(strum::IntoStaticStr, strum::EnumIter, Copy, Clone, Debug, Eq, PartialEq)]
        #[strum(ascii_case_insensitive)]
        pub enum I18nKeys {{
        {elements}
        }}
    "# })
}

fn format_i18n_key_as_enum_element(key: String) -> String {
    let variant_name = key.to_pascal_case();
    format!("    #[strum(serialize = \"{key}\")] {variant_name},")
}

fn generate_locales_def() -> Result<String> {
    let elements = get_supported_locales()?
        .into_iter()
        .map(format_locale_as_enum_element)
        .collect::<Vec<_>>()
        .join("\n");

    Ok(formatdoc! { r#"
        use macros::settings_type;

        #[allow(dead_code)]
        #[derive(strum::IntoStaticStr, strum::EnumIter, strum::EnumString, Copy, Eq, PartialEq)]
        #[settings_type]
        pub enum Locales {{
        {elements}
        }}
    "# })
}

fn format_locale_as_enum_element(key: String) -> String {
    let variant_name = key.to_pascal_case();
    if is_default_lang(&key) {
        format!("    #[strum(serialize = \"{key}\")] #[default] {variant_name},")
    } else {
        format!("    #[strum(serialize = \"{key}\")] {variant_name},")
    }
}

fn is_default_lang(key: &str) -> bool {
    key == DEFAULT_LANG
}

fn write_to_output_dir(file_name: &str, content: String) -> Result<()> {
    let out_dir = env::var("OUT_DIR").context("OUT_DIR environment variable not set")?;
    let dest_path = Path::new(&out_dir).join(file_name);

    fs::write(&dest_path, content)
        .with_context(|| format!("Failed to write generated code to {:?}", dest_path))
}

// some codes from https://github.com/tauri-apps/tauri/issues/13419#issuecomment-3398457618
fn build() -> Result<()> {
    #[allow(unused_mut)]
    let mut attributes = Attributes::new();
    #[cfg(windows)]
    {
        attributes = remove_default_app_manifest(attributes);
        add_manifest()?;
    }
    turn_linker_warns_to_errs();
    tauri_build::try_build(attributes)?;
    Ok(())
}

#[cfg(windows)]
fn remove_default_app_manifest(attributes: Attributes) -> Attributes {
    attributes.windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest())
}

#[cfg(windows)]
fn add_manifest() -> Result<()> {
    let manifest = resolve_windows_manifest_file()?;
    add_to_rerun_track_list(manifest.clone());
    embed_manifest_file(manifest.clone());
    Ok(())
}

#[cfg(windows)]
fn resolve_windows_manifest_file() -> Result<PathBuf> {
    const WINDOWS_MANIFEST_FILE: &str = "windows-app-manifest.xml";

    Ok(current_dir()?.join(WINDOWS_MANIFEST_FILE))
}

fn add_to_rerun_track_list(file: PathBuf) {
    println!("cargo:rerun-if-changed={}", file.display());
}

fn embed_manifest_file(manifest: PathBuf) {
    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest.display());
}

fn current_dir() -> Result<PathBuf> {
    env::current_dir().context("Couldn't get current directory")
}

fn turn_linker_warns_to_errs() {
    println!("cargo:rustc-link-arg=/WX");
}
