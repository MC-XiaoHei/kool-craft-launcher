import { useFluent } from "fluent-vue"
import type { I18nKeys } from "@/bindings/types"

export function useI18n() {
  const { $t, format, ...rest } = useFluent()

  const t = (key: I18nKeys, args?: Record<string, string | number>) => {
    return $t(key, args)
  }

  return {
    ...rest,
    $t: t,
    t,
  }
}
