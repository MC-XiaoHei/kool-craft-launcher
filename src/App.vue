<script lang="ts" setup>
  import { onMounted, ref } from "vue"
  import { getCurrentWindow } from "@tauri-apps/api/window"
  import { useRouter } from "vue-router"
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

  const router = useRouter()
  const transitionName = ref("")

  router.afterEach((_, from) => {
    if (!from.name) {
      transitionName.value = ""
      return
    }
    const toDepth = history.state?.position || 0
    const fromDepth = history.state?.back ? history.state.position - 1 : history.state.position + 1
    transitionName.value = toDepth < fromDepth ? "zoom-out" : "zoom-in"
  })

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
        class="relative bg-background rounded-lg size-full shadow-sm overflow-hidden perspective-distant backface-hidden"
        data-tauri-drag-region
      >
        <RouterView v-slot="{ Component }">
          <Transition :name="transitionName">
            <component
              data-tauri-drag-region
              :is="Component"
              class="absolute top-0 left-0 w-full h-full backface-hidden"
            />
          </Transition>
        </RouterView>
      </div>
    </div>
  </div>
</template>

<!--suppress CssUnusedSymbol -->
<style>
  html,
  body {
    background-color: transparent !important;
  }

  .zoom-in-enter-active,
  .zoom-in-leave-active,
  .zoom-out-enter-active,
  .zoom-out-leave-active {
    transition: all 0.4s cubic-bezier(0.25, 1, 0.5, 1);
    position: absolute;
    width: 100%;
    height: 100%;
    transform-origin: center center;
  }

  .zoom-in-enter-from {
    opacity: 0;
    transform: scale(0.92);
    z-index: 10;
  }
  .zoom-in-enter-to {
    opacity: 1;
    transform: scale(1);
    z-index: 10;
  }

  .zoom-in-leave-from {
    opacity: 1;
    transform: scale(1);
    z-index: 1;
  }
  .zoom-in-leave-to {
    opacity: 0;
    transform: scale(0.92);
    z-index: 1;
  }

  .zoom-out-leave-from {
    opacity: 1;
    transform: scale(1);
    z-index: 10;
  }
  .zoom-out-leave-to {
    opacity: 0;
    transform: scale(0.92);
    z-index: 10;
  }

  .zoom-out-enter-from {
    opacity: 0;
    transform: scale(0.92);
    z-index: 1;
  }
  .zoom-out-enter-to {
    opacity: 1;
    transform: scale(1);
    z-index: 1;
  }
</style>
