use crate::config::codegen::{generate_config_type_def, generate_config_watcher};
use crate::utils::codegen::unwrap_safe::{
    gen_command_invokers, gen_config_watcher, gen_event_functions, gen_types,
};
use specta::datatype::{FunctionResultVariant, PrimitiveType};
use specta::{DataType, TypeCollection};
use specta_typescript::{Typescript, datatype};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;

#[cfg(debug_assertions)]
pub fn do_codegen() {
    let mut types = specta::export();
    gen_types(&types);
    gen_config_watcher();
    gen_command_invokers(&mut types);
    gen_event_functions(&types);
}

// use .unwrap() in this mod is safe, because these functions only calls in debug env
#[cfg(debug_assertions)]
mod unwrap_safe {
    use super::*;
    use crate::ipc::command::generate_command_invokers;
    use crate::ipc::event::generate_event_functions;
    use specta::TypeCollection;
    use std::sync::OnceLock;

    pub(super) fn gen_types(types: &TypeCollection) {
        let path = "../src/bindings/types.ts";

        Typescript::default().export_to(path, types).unwrap();

        let mut file = OpenOptions::new().append(true).open(path).unwrap();

        writeln!(file).unwrap();
        writeln!(file, "{}", generate_config_type_def()).unwrap();
    }

    pub(super) fn gen_config_watcher() {
        let path = "../src/bindings/config.ts";
        let content = generate_config_watcher();
        fs::write(path, content).unwrap();
    }

    pub(super) fn gen_command_invokers(types: &mut TypeCollection) {
        let path = "../src/bindings/commands.ts";
        let content = generate_command_invokers(types);
        fs::write(path, content).unwrap();
    }

    pub(super) fn gen_event_functions(types: &TypeCollection) {
        let path = "../src/bindings/events.ts";
        let content = generate_event_functions(types);
        fs::write(path, content).unwrap();
    }
}

pub fn indent_all(text: impl Into<String>, space_of_num: usize) -> String {
    let space = " ".repeat(space_of_num);
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
            let config = Typescript::default();
            datatype(&config, &variant, types).unwrap_or_else(|_| "any".to_string())
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
