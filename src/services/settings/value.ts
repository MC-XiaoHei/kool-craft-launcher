import { ref, toRaw, watch } from "vue"
import { getSettingsValuesJson, setSettings as invokeSetSettings } from "@/bindings/commands"
import { listenSettingsUpdateEvent } from "@/bindings/events"
import { SettingsModule } from "@/bindings/types"

export const settings = ref<SettingsModule>(await getSettings())

async function setSettings<T>(key: string, value: T): Promise<void> {
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

  const safeKey = key as keyof typeof settings.value

  const currentLocalValue = settings.value[safeKey]
  const currentLocalJson = JSON.stringify(toRaw(currentLocalValue))

  if (currentLocalJson !== value) {
    settings.value[safeKey] = JSON.parse(value)
  }
})

for (const key of Object.keys(settings.value) as Array<keyof typeof settings.value>) {
  watch(
    () => settings.value[key],
    async newVal => {
      await setSettings(key, newVal).then().catch(console.error)
    },
    { deep: true },
  )
}
