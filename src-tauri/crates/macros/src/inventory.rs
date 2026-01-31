use proc_macro::TokenStream;
use quote::quote_spanned;
use syn::spanned::Spanned;
use syn::{ItemStruct, parse_macro_input};

pub fn inventory_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_struct = parse_macro_input!(input as ItemStruct);
    let span = input_struct.span();
    let struct_name = &input_struct.ident;

    let expanded = quote_spanned! { span=>
        inventory::collect!(#struct_name);

        #input_struct
    };

    TokenStream::from(expanded)
}
