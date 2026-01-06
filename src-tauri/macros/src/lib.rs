mod config;
mod config_type;
mod inventory;

use crate::config_type::config_type_impl;
use crate::inventory::inventory_impl;
use config::config_impl;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn config(args: TokenStream, input: TokenStream) -> TokenStream {
    config_impl(args, input)
}

#[proc_macro_attribute]
pub fn config_type(args: TokenStream, input: TokenStream) -> TokenStream {
    config_type_impl(args, input)
}

#[proc_macro_attribute]
pub fn inventory(args: TokenStream, input: TokenStream) -> TokenStream {
    inventory_impl(args, input)
}
