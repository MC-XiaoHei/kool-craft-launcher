import type { I18nKeys } from "@/bindings/types"

declare module "vue" {
  interface ComponentCustomProperties {
    $t(key: I18nKeys, values?: Record<string, unknown>): string
  }
}

export {}
