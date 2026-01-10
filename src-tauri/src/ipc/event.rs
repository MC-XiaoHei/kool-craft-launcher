use crate::ipc::command::CommandInfo;
use crate::utils::codegen::{get_import_line, get_import_list};
use indoc::formatdoc;
use macros::inventory;
use specta::TypeCollection;
use tap::Conv;

#[inventory]
#[derive(Clone)]
pub struct EventInfo {
    pub name: &'static str,
}

pub fn generate_event_functions(types: &TypeCollection) -> String {
    let event_function_defs = get_event_infos()
        .iter()
        .map(|info| info.to_functions())
        .collect::<Vec<_>>()
        .join("\n");
    let type_import_line = get_import_line(types);

    formatdoc! { r#"
        import {{ emit, emitTo, listen, once, EventTarget, UnlistenFn }} from "@tauri-apps/api/event"
        {type_import_line}

        {event_function_defs}
    "# }
}

fn get_event_infos() -> Vec<EventInfo> {
    inventory::iter::<EventInfo>.into_iter().cloned().collect()
}

impl EventInfo {
    pub fn to_functions(&self) -> String {
        [self.to_listen_functions(), self.to_emit_functions()].join("\n")
    }

    fn to_listen_functions(&self) -> String {
        let name = self.name;
        formatdoc! { r#"
            export async function listen{name}(
              func: (event: {name}) => Promise<void>,
            ): Promise<UnlistenFn> {{
              return await listen<{name}>("{name}", async event => {{
                await func(event.payload)
              }})
            }}

            export async function listen{name}Once(
              func: (event: {name}) => Promise<void>,
            ): Promise<void> {{
              await once<{name}>("{name}", async event => {{
                await func(event.payload)
              }})
            }}
        "# }
    }

    fn to_emit_functions(&self) -> String {
        let name = self.name;
        formatdoc! { r#"
            export async function emit{name}(payload: {name}): Promise<void> {{
              await emit<{name}>("{name}", payload)
            }}

            export async function emit{name}To(
              to: EventTarget | string,
              payload: {name},
            ): Promise<void> {{
              await emitTo<{name}>(to, "{name}", payload)
            }}
        "# }
    }
}
