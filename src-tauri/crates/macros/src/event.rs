use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemStruct, parse_macro_input};

pub fn event_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_struct = parse_macro_input!(input as ItemStruct);
    let struct_name = &input_struct.ident;
    let struct_name_str = struct_name.to_string();

    let expanded = quote! {
        #[derive(Debug, Clone, serde::Serialize, specta::Type)]
        #input_struct

        impl #struct_name {
            pub fn emit(&self) -> Result<()> {
                crate::utils::global_app_handle::get_global_app_handle().emit(#struct_name_str, self)?;
                Ok(())
            }

            pub fn emit_to<T>(&self, target: T) -> Result<()>
            where
                T: Into<EventTarget>,
            {
                crate::utils::global_app_handle::get_global_app_handle().emit_to(target, #struct_name_str, self)?;
                Ok(())
            }

            pub fn emit_filter<F>(&self, filter: F) -> Result<()>
            where
                F: Fn(&EventTarget) -> bool,
            {
                crate::utils::global_app_handle::get_global_app_handle().emit_filter(#struct_name_str, self, filter)?;
                Ok(())
            }
        }

        inventory::submit! {
            crate::ipc::event::EventInfo {
                name: #struct_name_str,
            }
        }
    };
    TokenStream::from(expanded)
}
