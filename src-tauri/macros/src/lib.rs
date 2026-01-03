mod config;

use config::config_impl;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn config(args: TokenStream, input: TokenStream) -> TokenStream {
    config_impl(args, input)
}
