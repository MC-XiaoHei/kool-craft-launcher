mod config;
mod config_type;

use config::config_impl;
use proc_macro::TokenStream;
use crate::config_type::config_type_impl;

#[proc_macro_attribute]
pub fn config(args: TokenStream, input: TokenStream) -> TokenStream {
    config_impl(args, input)
}

#[proc_macro_attribute]
pub fn config_type(args: TokenStream, input: TokenStream) -> TokenStream {
    config_type_impl(args, input)
}