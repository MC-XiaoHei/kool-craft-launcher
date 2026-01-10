import { pauseWatchConfigStore, resumeWatchConfigStore, watchConfigStore } from "@/bindings/config"
import { ConfigModule, ConfigUpdateEvent } from "@/bindings/types"
import { nextTick, ref } from "vue"
import { listen } from "@tauri-apps/api/event"
import { getConfigValuesJson, setConfig as invokeSetConfig } from "@/bindings/commands"

export const config = ref<ConfigModule>(await getConfig())

async function getConfig(): Promise<ConfigModule> {
  const raw = await getConfigValuesJson()
  const entries = Object.entries(raw).map(([key, value]) => [key, JSON.parse(<string>value)])
  return Object.fromEntries(entries) as ConfigModule
}

export async function setConfig<T>(key: string, value: T): Promise<void> {
  const json = JSON.stringify(value)
  await invokeSetConfig(key, json)
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
