import { computed, watchEffect } from "vue"
import { config } from "@/services/config"
import { usePreferredDark } from "@vueuse/core"

const systemDark = usePreferredDark()

export const isDark = computed(() => {
  if (config.value.theme.mode === "Auto") return systemDark.value
  return config.value.theme.mode === "Dark"
})

export function listenToSystemThemeChanges() {
  watchEffect(() => {
    const root = document.documentElement
    if (isDark.value) root.classList.add("dark")
    else root.classList.remove("dark")
  })
}
