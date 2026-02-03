use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, ItemStruct, Field, Ident, Result, Error};

pub fn inject_args_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(input as ItemStruct);
    let mut generated_functions = Vec::new();

    if let Err(e) = process_fields(&item_struct.ident, &mut item_struct.fields, &mut generated_functions) {
        return e.to_compile_error().into();
    }

    let output = quote! {
        #item_struct
        #(#generated_functions)*
    };

    output.into()
}

fn process_fields(
    struct_name: &Ident,
    fields: &mut syn::Fields,
    generated_functions: &mut Vec<proc_macro2::TokenStream>,
) -> Result<()> {
    if let syn::Fields::Named(fields_named) = fields {
        for field in fields_named.named.iter_mut() {
            if let Some(fn_tokens) = process_single_field(struct_name, field)? {
                generated_functions.push(fn_tokens);
            }
        }
    }
    Ok(())
}

fn process_single_field(
    struct_name: &Ident,
    field: &mut Field,
) -> Result<Option<proc_macro2::TokenStream>> {
    let args_tokens = match extract_component_args(field)? {
        Some(tokens) => tokens,
        None => return Ok(None),
    };

    let field_name = field.ident.as_ref()
        .ok_or_else(|| Error::new_spanned(&*field, "Field must have a name"))?
        .clone();

    let fn_name = format_ident!("__schema_gen_{}_{}", struct_name, field_name);

    inject_schemars_attr(field, &fn_name);

    let generated_func = generate_schema_fn(&fn_name, &field.ty, args_tokens);

    Ok(Some(generated_func))
}

fn extract_component_args(field: &mut Field) -> Result<Option<proc_macro2::TokenStream>> {
    let mut attr_index_to_remove = None;
    let mut args_tokens = None;

    for (i, attr) in field.attrs.iter().enumerate() {
        if attr.path().is_ident("component") {
            args_tokens = Some(attr.parse_args::<proc_macro2::TokenStream>()?);
            attr_index_to_remove = Some(i);
            break;
        }
    }

    if let Some(i) = attr_index_to_remove {
        field.attrs.remove(i);
        Ok(args_tokens)
    } else {
        Ok(None)
    }
}

fn inject_schemars_attr(field: &mut Field, fn_name: &Ident) {
    let fn_name_str = fn_name.to_string();
    field.attrs.push(parse_quote!(
        #[schemars(schema_with = #fn_name_str)]
    ));
}

fn generate_schema_fn(
    fn_name: &Ident,
    field_type: &syn::Type,
    args_tokens: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {
        #[allow(non_snake_case)]
        fn #fn_name(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
            let mut schema = <#field_type as schemars::JsonSchema>::json_schema(generator);
            let args_data = <#field_type>::inject_args(#args_tokens);

            let mut json_val = serde_json::json!(args_data);
            if !json_val.is_object() {
                json_val = serde_json::json!({
                    "value": json_val
                });
            }
            schema.insert("args".to_string(), json_val);
            schema
        }
    }
}
