use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemFn, FnArg, Type};
use syn::punctuated::Punctuated;
use syn::token::Comma;

pub fn command_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();
    let fn_sig = &input_fn.sig;

    let mut stub_sig = fn_sig.clone();

    let filtered_inputs: Punctuated<FnArg, Comma> = stub_sig.inputs
        .into_iter()
        .filter(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                !is_tauri_injection_type(&pat_type.ty)
            } else {
                true
            }
        })
        .collect();

    stub_sig.inputs = filtered_inputs;

    stub_sig.generics.params.clear();
    stub_sig.generics.where_clause = None;

    let wrapper_ident = format_ident!("__inventory_wrapper_{}", fn_name);

    let expanded = quote! {
        #[tauri::command]
        #input_fn

        #[allow(non_snake_case)]
        fn #wrapper_ident(invoke: tauri::ipc::Invoke<tauri::Wry>) -> bool {
            let handler: fn(tauri::ipc::Invoke<tauri::Wry>) -> bool = tauri::generate_handler![#fn_name];
            handler(invoke)
        }

        const _: () = {
            #[specta::specta]
            #[allow(dead_code, unused_variables, non_snake_case)]
            #stub_sig {
                unimplemented!("Specta stub function should never be called");
            }

            inventory::submit! {
                crate::ipc::command::CommandInfo {
                    name: #fn_name_str,
                    function_info: |types| {
                        specta::function::collect_functions![#fn_name](types)
                    },
                    tauri_handler: |invoke| {
                        #wrapper_ident(invoke)
                    },
                }
            }
        };
    };

    TokenStream::from(expanded)
}

fn is_tauri_injection_type(ty: &Type) -> bool {
    let inner_type = if let Type::Reference(type_ref) = ty {
        &*type_ref.elem
    } else {
        ty
    };

    if let Type::Path(type_path) = inner_type {
        if let Some(segment) = type_path.path.segments.last() {
            let name = segment.ident.to_string();
            return matches!(
                name.as_str(),
                "State" | "AppHandle" | "Window" | "WebviewWindow" | "Webview" | "App" | "Wry"
            );
        }
    }
    false
}
