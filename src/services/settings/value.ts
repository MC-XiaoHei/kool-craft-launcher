import { watchSettingsStore } from "@/bindings/settings"
import { SettingsModule } from "@/bindings/types"
import { nextTick, ref } from "vue"
import {
  getSettingsValuesJson,
  setSettings as invokeSetSettings,
} from "@/bindings/commands"
import { listenSettingsUpdateEvent } from "@/bindings/events"

export const settings = ref<SettingsModule>(await getSettings())
let watchingSettingsStore = false

export function pauseWatchSettingsStore() {
  watchingSettingsStore = false
}

export function resumeWatchSettingsStore() {
  watchingSettingsStore = true
}

// noinspection JSUnusedGlobalSymbols
export function isWatchingSettingsStore(): boolean {
  return watchingSettingsStore
}

// noinspection JSUnusedGlobalSymbols
export async function setSettings<T>(key: string, value: T): Promise<void> {
  const json = JSON.stringify(value)
  await invokeSetSettings(key, json)
}

async function getSettings(): Promise<SettingsModule> {
  const raw = await getSettingsValuesJson()
  const entries = Object.entries(raw).map(([key, value]) => [key, JSON.parse(<string>value)])
  return Object.fromEntries(entries) as SettingsModule
}

await listenSettingsUpdateEvent(async event => {
  const { key, value } = event
  const entry = JSON.parse(value)
  pauseWatchSettingsStore()
  // @ts-ignore
  settings.value[key] = entry
  await nextTick()
  resumeWatchSettingsStore()
})

watchSettingsStore()
