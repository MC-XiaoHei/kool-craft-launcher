import { ref } from "vue"
import type { JSONSchema7 } from "json-schema"
import { getSettingsSchemas as invokeGetSettingsSchemas } from "@/bindings/commands"
import $RefParser from "@apidevtools/json-schema-ref-parser"
import { warn } from "@tauri-apps/plugin-log"

export const settingsSchema = ref(await getSettingsSchemas())

export type SettingsSchema = Record<string, SettingsGroupSchema>
export type SettingsGroupSchema = JSONSchema7 & {
  component?: string;
  properties?: { [key: string]: SettingsGroupSchema };
};

async function getSettingsSchemas(): Promise<SettingsSchema> {
  const rawData = await invokeGetSettingsSchemas()
  const processedMap: Record<string, SettingsGroupSchema> = {}

  for (const [key, schemaContent] of Object.entries(rawData)) {
    if (schemaContent) {
      try {
        processedMap[key] = (await $RefParser.dereference(schemaContent)) as SettingsGroupSchema
      } catch (err) {
        await warn(`Fail to parse settings group [${key}]: ${err}]`)
      }
    }
  }

  return processedMap
}
