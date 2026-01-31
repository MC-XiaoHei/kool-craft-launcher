use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Item, parse_macro_input};

pub fn settings_type_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_item = parse_macro_input!(input as Item);
    let span = input_item.span();

    let is_struct = matches!(input_item, Item::Struct(_));

    let serde_attribute = if is_struct {
        quote! {
            #[serde(rename_all = "camelCase")]
            #[serde(default)]
        }
    } else {
        quote! {}
    };

    let expanded = quote_spanned! { span =>
        #[derive(
            Default,
            std::fmt::Debug,
            std::clone::Clone,
            serde::Serialize,
            serde::Deserialize,
            schemars::JsonSchema,
            specta::Type
        )]
        #serde_attribute
        #input_item
    };

    TokenStream::from(expanded)
}
