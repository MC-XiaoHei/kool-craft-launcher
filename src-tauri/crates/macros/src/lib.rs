mod command;
mod settings;
mod settings_type;
mod event;
mod inventory;

use crate::command::command_impl;
use crate::settings_type::settings_type_impl;
use crate::event::event_impl;
use crate::inventory::inventory_impl;
use settings::settings_impl;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn command(args: TokenStream, input: TokenStream) -> TokenStream {
    command_impl(args, input)
}

#[proc_macro_attribute]
pub fn settings(args: TokenStream, input: TokenStream) -> TokenStream {
    settings_impl(args, input)
}

#[proc_macro_attribute]
pub fn settings_type(args: TokenStream, input: TokenStream) -> TokenStream {
    settings_type_impl(args, input)
}

#[proc_macro_attribute]
pub fn event(args: TokenStream, input: TokenStream) -> TokenStream {
    event_impl(args, input)
}

#[proc_macro_attribute]
pub fn inventory(args: TokenStream, input: TokenStream) -> TokenStream {
    inventory_impl(args, input)
}
