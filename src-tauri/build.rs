use anyhow::{Context, Result};
use i18n_parser::{get_translate_keys, resolve_all_ftl_files};
use indoc::formatdoc;
use std::path::Path;
use std::path::PathBuf;
use std::{env, fs};
use tauri_build::Attributes;

fn main() -> Result<()> {
    generate_translate_keys_def()?;
    build()?;
    Ok(())
}

fn generate_translate_keys_def() -> Result<()> {
    add_ftl_files_to_rerun_track_list()?;
    let message_ids = get_translate_keys()?;
    let rust_code = generate_rust_code(message_ids);
    write_to_output_dir("generated_i18n_keys.rs", rust_code)?;
    Ok(())
}

fn add_ftl_files_to_rerun_track_list() -> Result<()> {
    resolve_all_ftl_files()?
        .into_iter()
        .for_each(add_to_rerun_track_list);

    Ok(())
}

fn generate_rust_code(keys: Vec<String>) -> String {
    let constants = keys
        .into_iter()
        .map(format_as_rust_constant)
        .collect::<Vec<_>>()
        .join("\n");

    formatdoc! { r#"
        // AUTOMATICALLY GENERATED. DO NOT EDIT.

        #[allow(dead_code)]
        pub mod i18n_keys {{
        {constants}
        }}
    "# }
}

fn format_as_rust_constant(key: String) -> String {
    let const_name = key.to_uppercase().replace("-", "_");
    format!(
        "    pub const {}: &str = \"{}\";",
        const_name, key
    )
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
