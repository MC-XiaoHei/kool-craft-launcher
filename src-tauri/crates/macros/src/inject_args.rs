use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemStruct, parse_quote};

pub fn inject_args_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(input as ItemStruct);
    let struct_name = &item_struct.ident;

    let mut generated_functions = Vec::new();

    if let syn::Fields::Named(fields) = &mut item_struct.fields {
        for field in fields.named.iter_mut() {
            let mut args_tokens = None;
            let mut attr_index_to_remove = None;

            for (i, attr) in field.attrs.iter().enumerate() {
                if attr.path().is_ident("component") {
                    match attr.parse_args::<proc_macro2::TokenStream>() {
                        Ok(tokens) => {
                            args_tokens = Some(tokens);
                            attr_index_to_remove = Some(i);
                        }
                        Err(e) => {
                            return e.to_compile_error().into();
                        }
                    }
                    break;
                }
            }

            if let Some(tokens) = args_tokens {
                if let Some(i) = attr_index_to_remove {
                    field.attrs.remove(i);
                }

                let field_name = field.ident.as_ref().unwrap();
                let field_type = &field.ty;

                let fn_name = format_ident!("__schema_gen_{}_{}", struct_name, field_name);
                let fn_name_str = fn_name.to_string();

                field.attrs.push(parse_quote!(
                    #[schemars(schema_with = #fn_name_str)]
                ));

                let generated_func = quote! {
                    #[allow(non_snake_case)]
                    fn #fn_name(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
                        let mut schema = <#field_type as schemars::JsonSchema>::json_schema(generator);
                        <#field_type>::inject_args(&mut schema, #tokens);
                        schema
                    }
                };

                generated_functions.push(generated_func);
            }
        }
    }

    let output = quote! {
        #item_struct
        #(#generated_functions)*
    };

    output.into()
}
