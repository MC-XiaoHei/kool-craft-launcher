use crate::utils::codegen::{get_import_line, get_import_list, resolve_ts_type};
use heck::ToLowerCamelCase;
use indoc::formatdoc;
use macros::inventory;
use specta::datatype::FunctionResultVariant;
use specta::{TypeCollection, datatype::Function};
use std::collections::HashMap;
use tauri::{Runtime, Wry, ipc::Invoke};

#[inventory]
pub struct CommandInfo {
    pub name: &'static str,
    pub function_info: fn(&mut TypeCollection) -> Vec<Function>,
    pub tauri_handler: fn(Invoke<Wry>) -> bool,
}

pub fn generate_command_invokers(types: &mut TypeCollection) -> String {
    let function_defs = get_functions(types)
        .into_iter()
        .map(|func| to_invoke_function(func, types))
        .collect::<Vec<_>>()
        .join("\n");
    let type_import_line = get_import_line(types);

    formatdoc! { r#"
        import {{ invoke }} from "@tauri-apps/api/core"
        {type_import_line}

        export type JsonValue = any

        {function_defs}
    "# }
}

pub fn command_handler() -> impl Fn(Invoke<Wry>) -> bool + Send + Sync + 'static {
    let handler_map: HashMap<&'static str, fn(Invoke<Wry>) -> bool> =
        inventory::iter::<CommandInfo>
            .into_iter()
            .map(|cmd| (cmd.name, cmd.tauri_handler))
            .collect();

    move |invoke| {
        let cmd_name = invoke.message.command();
        if let Some(handler) = handler_map.get(cmd_name) {
            handler(invoke)
        } else {
            false
        }
    }
}

fn to_invoke_function(function: Function, types: &TypeCollection) -> String {
    let rust_cmd_name = function.name().to_string();
    let ts_func_name = rust_cmd_name.to_lower_camel_case();

    let (args_def_list, args_usage_list): (Vec<String>, Vec<String>) = function
        .args()
        .map(|(rust_arg, typ)| {
            let ts_arg = rust_arg.to_lower_camel_case();
            let ts_type = resolve_ts_type(typ, types);

            let def = format!("{}: {}", ts_arg, ts_type);

            let usage = if rust_arg == &ts_arg {
                ts_arg
            } else {
                format!("{}: {}", rust_arg, ts_arg)
            };

            (def, usage)
        })
        .unzip();
    let args_def = args_def_list.join(", ");

    let invoke_params = if args_usage_list.is_empty() {
        String::new()
    } else {
        format!(", {{ {} }}", args_usage_list.join(", "))
    };

    let mut return_type = function
        .result()
        .map(|variant| match variant {
            FunctionResultVariant::Value(data_type) => data_type,
            FunctionResultVariant::Result(data_type, _) => data_type,
        })
        .map(|data_type| resolve_ts_type(data_type, types))
        .unwrap_or_else(|| "void".to_string());
    if return_type == "null" {
        return_type = "void".into();
    }

    formatdoc! { r#"
        export async function {ts_func_name}({args_def}): Promise<{return_type}> {{
            return await invoke("{rust_cmd_name}"{invoke_params});
        }}
    "# }
}

fn get_functions(types: &mut TypeCollection) -> Vec<Function> {
    inventory::iter::<CommandInfo>
        .into_iter()
        .flat_map(|cmd| (cmd.function_info)(types))
        .collect()
}
