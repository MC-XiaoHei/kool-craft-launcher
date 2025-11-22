import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { loadThemeConfig, setThemeConfig } from '@/services/backend/theme'
import { ThemeMode, EffectMode } from '@/types/theme'

export const useThemeStore = defineStore('theme', () => {
    const theme = ref(ThemeMode.Auto)
    const effect = ref(EffectMode.Auto)

    async function initTheme() {
        const config = await loadThemeConfig()
        theme.value = config.theme
        effect.value = config.effect
    }

    async function refreshTheme() {
        await setThemeConfig({ theme: theme.value, effect: effect.value })
    }

    watch([theme, effect], async () => {
        await refreshTheme()
    })

    return { initTheme, refreshTheme    , theme, effect }
})
