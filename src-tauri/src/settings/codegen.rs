#![cfg_attr(coverage_nightly, coverage(off))]

use crate::utils::codegen::indent_all;
use indoc::formatdoc;
use macros::inventory;

#[inventory]
pub struct SettingsGroupInfo {
    pub name: &'static str,
    pub key: &'static str,
}

impl SettingsGroupInfo {
    fn to_type_element(&self) -> String {
        let name = self.name;
        let key = self.key;
        format!("{key}: {name}")
    }
}

pub fn generate_settings_type_def() -> String {
    let elements = inventory::iter::<SettingsGroupInfo>
        .into_iter()
        .map(SettingsGroupInfo::to_type_element)
        .collect::<Vec<_>>()
        .join("; ");
    format!("export type SettingsModule = {{ {elements} }}")
}