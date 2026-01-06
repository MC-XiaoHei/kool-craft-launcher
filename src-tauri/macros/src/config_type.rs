use proc_macro::TokenStream;
use quote::quote_spanned;
use syn::spanned::Spanned;
use syn::{Item, parse_macro_input};

pub fn config_type_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_item = parse_macro_input!(input as Item);
    let span = input_item.span();

    let expanded = quote_spanned! { span =>
        #[derive(
            std::fmt::Debug,
            std::clone::Clone,
            serde::Serialize,
            serde::Deserialize,
            schemars::JsonSchema,
            specta::Type
        )]
        #input_item
    };

    TokenStream::from(expanded)
}
