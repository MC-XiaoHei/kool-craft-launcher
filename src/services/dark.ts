import { computed, watchEffect } from "vue"
import { settings } from "@/services/settings"
import { usePreferredDark } from "@vueuse/core"

const systemDark = usePreferredDark()

export const isDark = computed(() => {
  if (settings.value.theme.mode === "Auto") return systemDark.value
  return settings.value.theme.mode === "Dark"
})

export function listenToSystemThemeChanges() {
  watchEffect(() => {
    const root = document.documentElement
    if (isDark.value) root.classList.add("dark")
    else root.classList.remove("dark")
  })
}
