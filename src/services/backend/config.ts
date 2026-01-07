import { pauseWatchConfigStore, resumeWatchConfigStore, watchConfigStore } from "@/bindings/config"
import { ConfigModule, ConfigUpdateEvent } from "@/bindings/types"
import { invoke } from "@tauri-apps/api/core"
import { nextTick, ref } from "vue"
import { listen } from "@tauri-apps/api/event"

export const config = ref<ConfigModule>(await getConfig())

async function getConfig(): Promise<ConfigModule> {
  const raw = await invoke<any>("get_config_values_json")
  const entries = Object.entries(raw).map(([key, string]) => [key, JSON.parse(string as string)])
  return Object.fromEntries(entries) as ConfigModule
}

export async function setConfig<T>(key: string, value: T): Promise<void> {
  const json = JSON.stringify(value)
  await invoke("set_config", {
    key: key,
    value: json,
  })
}

await listen<ConfigUpdateEvent>("config_update_event", async event => {
  const payload = event.payload
  const { key, value } = payload
  const entry = JSON.parse(value)
  pauseWatchConfigStore()
  // @ts-ignore
  config.value[key] = entry
  await nextTick()
  resumeWatchConfigStore()
})

watchConfigStore()
