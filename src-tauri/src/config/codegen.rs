#![cfg_attr(coverage_nightly, coverage(off))]

use indoc::formatdoc;
use macros::inventory;
use crate::utils::codegen::indent_all;

#[inventory]
pub struct ConfigGroupInfo {
    pub name: &'static str,
    pub key: &'static str,
}

impl ConfigGroupInfo {
    fn to_type_element(&self) -> String {
        let name = self.name;
        let key = self.key;
        format!("{key}: {name}")
    }

    fn to_watch_function(&self) -> String {
        let key = self.key;
        formatdoc! { r#"
            watch(
              () => config.value.{key},
              async val => {{ await setConfig("{key}", val).then().catch(error) }},
              {{ deep: true }},
            )
        "# }
    }
}

pub fn get_config_type_def() -> String {
    let elements = inventory::iter::<ConfigGroupInfo>
        .into_iter()
        .map(|x| x.to_type_element())
        .collect::<Vec<_>>()
        .join("; ");
    format!("export type ConfigModule = {{ {elements} }}")
}

pub fn get_config_watcher() -> String {
    let elements = inventory::iter::<ConfigGroupInfo>
        .into_iter()
        .map(|x| x.to_watch_function())
        .map(|x| indent_all(x, 2))
        .collect::<Vec<_>>()
        .join("\n");
    formatdoc! { r#"
        import {{ watch }} from "vue"
        import {{ error }} from "@tauri-apps/plugin-log"
        import {{ config }} from "@/services/backend/config"
        import {{ setConfig }} from "../services/backend/config"

        export function watchConfigStore() {{
        {elements}
        }}
    "# }
}
