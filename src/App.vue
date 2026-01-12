<script lang="ts" setup>
  import { computed, onMounted, ref } from "vue"
  import { getCurrentWindow } from "@tauri-apps/api/window"
  import { getWallpaperDataUrl, refreshWindowTheme } from "@/bindings/commands"
  import { cacheWallpaper, getCachedWallpaper } from "@/composables/wallpaper"
  import { ThemeEffect } from "@/bindings/types"
  import { config } from "@/services/config"
  import { Button } from "@/components/ui/button"
  import { PhMinus, PhX } from "@phosphor-icons/vue"

  const appWindow = getCurrentWindow()
  const wallpaperUrl = ref(getCachedWallpaper())
  const overlayOpacity = ref(1)
  const systemDarkMediaQuery = window.matchMedia("(prefers-color-scheme: dark)")
  const systemIsDark = ref(systemDarkMediaQuery.matches)

  const isDark = computed(() => {
    if (config.value.theme.mode === "Auto") return systemIsDark.value
    return config.value.theme.mode === "Dark"
  })

  onMounted(async () => {
    listenToSystemThemeChanges()
    await applyEffect(config.value.theme.effect, true)
    await appWindow.show()
    await appWindow.setFocus()
  })

  function listenToSystemThemeChanges() {
    systemDarkMediaQuery.addEventListener("change", e => {
      systemIsDark.value = e.matches
      if (config.value.theme.mode === "Auto") {
        refreshWindowTheme()
      }
    })
  }

  async function loadWallpaper() {
    if (wallpaperUrl.value) return
    const base64 = await getWallpaperDataUrl()
    if (!base64) return
    cacheWallpaper(base64).then()
    wallpaperUrl.value = base64
  }

  async function applyEffect(target: ThemeEffect, force = false) {
    if (!force && config.value.theme.effect === target) return
    if (target === "Wallpaper") {
      await loadWallpaper()
      overlayOpacity.value = 1
      await new Promise(r => setTimeout(r, 200))
      config.value.theme.effect = target
    } else {
      config.value.theme.effect = target
      overlayOpacity.value = 0
      loadWallpaper().then()
    }
  }

  async function minimizeWindow() {
    await appWindow.minimize()
  }

  async function closeWindow() {
    await appWindow.close()
  }
</script>

<template>
  <div
    class="relative w-screen h-screen overflow-hidden text-[#202020] transition-colors duration-200 font-sans select-none dark:text-white"
    :class="{ dark: isDark }"
  >
    <div
      class="absolute inset-0 -z-10 pointer-events-none overflow-hidden transition-opacity duration-200 bg-gray-500"
      :style="{ opacity: overlayOpacity }"
    >
      <div
        class="absolute inset-0 bg-cover bg-center transform scale-125 blur-[125px] saturate-[2.1] will-change-transform"
        :style="{ backgroundImage: `url(${wallpaperUrl})` }"
      ></div>

      <div class="absolute inset-0 opacity-[0.04] bg-noise"></div>

      <div
        class="absolute inset-0 transition-colors duration-300 bg-[#f3f3f3]/80 mix-blend-overlay dark:bg-[#202020]/85 dark:mix-blend-normal"
      ></div>
    </div>

    <div class="absolute right-1 z-100 flex items-center">
      <Button
        variant="ghost"
        class="size-10 no-drag hover:bg-transparent text-muted-foreground hover:text-foreground"
        @click="minimizeWindow"
      >
        <PhMinus class="h-5 w-5" weight="bold" />
      </Button>

      <Button
        variant="ghost"
        class="size-10 no-drag hover:bg-transparent text-muted-foreground hover:text-red-600 dark:hover:text-red-500"
        @click="closeWindow"
      >
        <PhX class="h-5 w-5" weight="bold" />
      </Button>
    </div>

    <main class="relative z-10 h-full flex items-center justify-center p-10" data-tauri-drag-region>
      <div
        class="w-100 p-8 rounded-xl backdrop-blur-md shadow-2xl transition-colors duration-200 bg-white/50 border border-white/30 dark:bg-black/40 dark:border-white/10 dark:shadow-black/30"
      >
        <h1 class="text-2xl font-semibold text-center mt-0 mb-2">Vibrancy template</h1>

        <div class="text-center text-sm opacity-80 space-y-1 mb-6">
          <p>
            当前效果: <strong>{{ config.theme.effect }}</strong>
          </p>
          <p>
            当前主题: <strong>{{ config.theme.mode }}</strong>
          </p>
        </div>

        <hr class="border-0 h-px bg-black/10 dark:bg-white/10 my-5" />

        <div class="mb-5">
          <label class="block text-xs font-bold uppercase tracking-widest opacity-70 mb-2"
            >渲染策略</label
          >
          <div class="flex gap-2">
            <button
              v-for="effect in ['Mica', 'Vibrancy', 'Wallpaper', 'Auto']"
              :key="effect"
              @click="applyEffect(effect as any)"
              class="flex-1 px-3 py-2 rounded-md text-sm border border-transparent transition-all duration-200 hover:bg-black/10 dark:hover:bg-white/20 bg-black/5 dark:bg-white/10"
              :class="{
                'bg-blue-500! text-white! font-medium shadow-lg shadow-blue-500/30':
                  config.theme.effect === effect,
              }"
            >
              {{ effect === "Wallpaper" ? "Img" : effect }}
            </button>
          </div>
        </div>

        <div>
          <label class="block text-xs font-bold uppercase tracking-widest opacity-70 mb-2"
            >颜色主题</label
          >
          <div class="flex gap-2">
            <button
              v-for="mode in ['Auto', 'Light', 'Dark']"
              :key="mode"
              @click="config.theme.mode = mode as any"
              class="flex-1 px-3 py-2 rounded-md text-sm border border-transparent transition-all duration-200 hover:bg-black/10 dark:hover:bg-white/20 bg-black/5 dark:bg-white/10"
              :class="{
                'bg-blue-500! text-white! font-medium shadow-lg shadow-blue-500/30':
                  config.theme.mode === mode,
              }"
            >
              {{ mode }}
            </button>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<style>
  html,
  body {
    background-color: transparent !important;
  }
</style>

<style scoped>
  .bg-noise {
    background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 200 200' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noiseFilter'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.8' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noiseFilter)'/%3E%3C/svg%3E");
  }
</style>
