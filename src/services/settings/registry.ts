import type { Component } from "vue"
import BaseInput from "@/components/settings/widgets/BaseInput.vue"
import BaseSwitch from "@/components/settings/widgets/BaseSwitch.vue"
import BaseSelect from "@/components/settings/widgets/BaseSelect.vue"
import ColorPicker from "@/components/settings/widgets/ColorPicker.vue"
import LanguageSelector from "@/components/settings/widgets/LanguageSelector.vue"

const defaultTypeMap: Record<string, Component> = {
  string: BaseInput,
  integer: BaseInput,
  number: BaseInput,
  boolean: BaseSwitch,
}

const customComponentRegistry: Record<string, Component> = {
  "color": ColorPicker,
  "language": LanguageSelector,
}

export function resolveSettingsComponent(schema: any): Component {
  if (!schema) return BaseInput

  if (schema.component && customComponentRegistry[schema.component]) {
    return customComponentRegistry[schema.component]
  }

  if (schema.enum) {
    return BaseSelect
  }

  if (schema.type && defaultTypeMap[schema.type]) {
    return defaultTypeMap[schema.type]
  }

  return BaseInput
}
