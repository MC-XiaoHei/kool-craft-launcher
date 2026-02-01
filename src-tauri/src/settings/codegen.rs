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

    fn to_watch_function(&self) -> String {
        let key = self.key;
        formatdoc! { r#"
            watch(
              () => settings.value.{key},
              async val => {{
                if (isWatchingSettingsStore()) {{
                  await setSettings("{key}", val).then().catch(console.error)
                }}
              }},
              {{ deep: true }},
            )
        "# }
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

pub fn generate_settings_watcher() -> String {
    let elements = inventory::iter::<SettingsGroupInfo>
        .into_iter()
        .map(SettingsGroupInfo::to_watch_function)
        .map(|x| indent_all(x, 2))
        .collect::<Vec<_>>()
        .join("\n");
    formatdoc! { r#"
        import {{ watch }} from "vue"
        import {{
            settings,
            setSettings,
            resumeWatchSettingsStore,
            isWatchingSettingsStore
        }} from "@/services/settings/value"

        export function watchSettingsStore() {{
        {elements}
          resumeWatchSettingsStore()
        }}
    "# }
}
