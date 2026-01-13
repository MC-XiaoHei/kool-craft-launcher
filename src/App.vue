<script lang="ts" setup>
  import { onMounted, ref } from "vue"
  import { getCurrentWindow } from "@tauri-apps/api/window"
  import { loadWallpaper } from "@/services/wallpaper"
  import { ThemeEffect } from "@/bindings/types"
  import { config } from "@/services/config"
  import AppBackground from "@/components/app/AppBackground.vue"
  import WindowControls from "@/components/app/WindowControls.vue"
  import AppSidebar from "@/components/app/AppSidebar.vue"
  import { listenToSystemThemeChanges } from "@/services/dark"
  import { initTheme } from "@/services/theme"

  const appWindow = getCurrentWindow()
  const overlayOpacity = ref(1)

  onMounted(async () => {
    initTheme()
    listenToSystemThemeChanges()
    await applyEffect(config.value.theme.effect, true)
    await appWindow.show()
    await appWindow.setFocus()
  })

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
</script>

<template>
  <div
    class="relative w-screen h-screen overflow-hidden text-[#202020] transition-colors duration-200 font-sans select-none dark:text-white"
  >
    <AppBackground :overlay-opacity="overlayOpacity" />
    <WindowControls />
    <div class="relative z-10 p-2 pl-0 size-full flex" data-tauri-drag-region>
      <AppSidebar />

      <div
        class="bg-background rounded-lg size-full shadow-sm overflow-hidden"
        data-tauri-drag-region
      >
        <RouterView data-tauri-drag-region />
      </div>
    </div>
  </div>
</template>

<style>
  html,
  body {
    background-color: transparent !important;
  }
</style>
