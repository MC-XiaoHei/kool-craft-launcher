import { watchConfigStore } from "@/bindings/config"
import { ConfigModule } from "@/bindings/types"
import { invoke } from "@tauri-apps/api/core"
import { ref } from "vue"

export const config = ref<ConfigModule>(await getConfig())

async function getConfig(): Promise<ConfigModule> {
  let raw = await invoke<any>("get_config_values_json")
  const entries = Object.entries(raw).map(([key, string]) => [key, JSON.parse(string as string)])
  return Object.fromEntries(entries) as ConfigModule
}

export async function setConfig<T>(key: string, value: T): Promise<void> {
  let json = JSON.stringify(value)
  await invoke("set_config", {
    key: key,
    value: json,
  })
}

watchConfigStore()
