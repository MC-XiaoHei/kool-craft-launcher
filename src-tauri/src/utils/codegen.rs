#![cfg_attr(coverage_nightly, coverage(off))]

use crate::i18n::codegen::generate_i18n_keys_def;
use crate::ipc::command::generate_command_invokers;
use crate::ipc::event::generate_event_functions;
use crate::settings::codegen::{generate_settings_type_def, generate_settings_watcher};
use anyhow::{Context, Result};
use specta::datatype::{FunctionResultVariant, PrimitiveType};
use specta::{DataType, TypeCollection};
use specta_typescript::{Typescript, datatype};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::OnceLock;

pub fn do_codegen() -> Result<()> {
    clear_bindings_dir()?;

    let mut types = specta::export();
    gen_types(&types)?;
    gen_settings_watcher()?;
    gen_command_invokers(&mut types)?;
    gen_event_functions(&types)?;

    Ok(())
}

fn clear_bindings_dir() -> Result<()> {
    const PATH: &str = "../src/bindings";
    if fs::exists(PATH)? {
        if fs::metadata(PATH)?.is_dir() {
            fs::remove_dir_all(PATH)?;
        } else {
            fs::remove_file(PATH)?;
        }
    }
    fs::create_dir(PATH)?;
    Ok(())
}

fn gen_types(types: &TypeCollection) -> Result<()> {
    const PATH: &str = "../src/bindings/types.ts";

    Typescript::default().export_to(PATH, types)?;

    let mut file = OpenOptions::new().append(true).open(PATH)?;

    writeln!(file, "{}\n", generate_settings_type_def())?;
    writeln!(file, "{}\n", generate_i18n_keys_def()?)?;

    Ok(())
}

fn gen_settings_watcher() -> Result<()> {
    const PATH: &str = "../src/bindings/settings.ts";
    let content = generate_settings_watcher();
    fs::write(PATH, content).context("Failed to write settings to file")
}

fn gen_command_invokers(types: &mut TypeCollection) -> Result<()> {
    const PATH: &str = "../src/bindings/commands.ts";
    let content = generate_command_invokers(types);
    fs::write(PATH, content).context("Failed to write commands to file")
}

fn gen_event_functions(types: &TypeCollection) -> Result<()> {
    const PATH: &str = "../src/bindings/events.ts";
    let content = generate_event_functions(types);
    fs::write(PATH, content).context("Failed to write events to file")
}

pub fn indent_all(text: impl Into<String>, num_of_space: usize) -> String {
    let space = " ".repeat(num_of_space);
    text.into()
        .lines()
        .map(|line| format!("{}{}", space, line))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn resolve_ts_type(data_type: &DataType, types: &TypeCollection) -> String {
    let mut raw = match data_type {
        DataType::Reference(reference) if reference.name() == "Result" => {
            if let Some((_, data_type)) = reference.generics().first() {
                resolve_ts_type(data_type, types)
            } else {
                "void".to_string()
            }
        }

        // TODO detect channel type here
        DataType::Reference(reference) => {
            if reference.generics().is_empty() {
                reference.name().to_string()
            } else {
                let generics: Vec<String> = reference
                    .generics()
                    .iter()
                    .map(|(_, generic)| resolve_ts_type(generic, types))
                    .collect();
                format!("{}<{}>", reference.name(), generics.join(", "))
            }
        }

        _ => {
            let variant = FunctionResultVariant::Value(data_type.clone());
            let settings = Typescript::default();
            datatype(&settings, &variant, types).unwrap_or_else(|_| "any".to_string())
        }
    };

    if raw == "JsonValue" {
        raw = "any".to_string();
    }

    raw
}

pub fn get_import_list(types: &TypeCollection) -> Vec<String> {
    let mut imports = types
        .into_iter()
        .filter_map(|(_, data_type)| {
            let name = data_type.name().to_string();

            match name.as_str() {
                "Result" => None,
                // TODO detect channel type here
                _ => Some(name.clone()),
            }
        })
        .collect::<Vec<_>>();

    imports.sort();
    imports.dedup();

    imports
}

pub fn get_import_line(types: &TypeCollection) -> String {
    let type_import_list = get_import_list(types).join(", ");
    if type_import_list.is_empty() {
        String::new()
    } else {
        format!(r#"import {{ {type_import_list} }} from "@/bindings/types""#)
    }
}
