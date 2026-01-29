<script setup lang="ts">
  import { wallpaperUrl } from "@/services/wallpaper"
  import { computed } from "vue"
  import { settings } from "@/services/settings/value"

  const opacity = computed(() => {
    const isWallpaperMode = settings.value.theme.effect == "Wallpaper"
    return isWallpaperMode ? 1 : 0
  })
</script>

<template>
  <div
    class="absolute inset-0 -z-10 pointer-events-none overflow-hidden bg-gray-500"
    :style="{ opacity: opacity }"
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
</template>

<style scoped>
  .bg-noise {
    background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 200 200' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noiseFilter'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.8' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noiseFilter)'/%3E%3C/svg%3E");
  }
</style>
