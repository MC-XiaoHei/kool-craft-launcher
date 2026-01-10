<script lang="ts" setup>
  import { computed, onMounted, ref } from "vue"
  import { getCurrentWindow } from "@tauri-apps/api/window"
  import { getWallpaperDataUrl, refreshWindowTheme } from "@/bindings/commands"
  import { cacheWallpaper, getCachedWallpaper } from "@/composables/wallpaper"
  import { ThemeEffect } from "@/bindings/types"
  import { config } from "@/services/backend/config"

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
      if (config.value.theme.mode === "Auto") refreshWindowTheme()
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
</script>

<template>
  <div :class="{ 'theme-dark': isDark }" class="app-shell">
    <div :style="{ opacity: overlayOpacity }" class="wallpaper-layer">
      <div :style="{ backgroundImage: `url(${wallpaperUrl})` }" class="wallpaper-base"></div>
      <div class="noise-overlay"></div>
      <div class="tint-overlay"></div>
    </div>

    <main class="content-area" data-tauri-drag-region>
      <div class="card">
        <h1>Vibrancy template</h1>
        <p class="subtitle">
          当前效果: <strong>{{ config.theme.effect }}</strong>
        </p>
        <p class="subtitle">
          当前主题: <strong>{{ config.theme.mode }}</strong>
        </p>

        <hr class="divider" />

        <div class="control-group">
          <label>渲染策略</label>
          <div class="btn-row">
            <button
              :class="{ active: config.theme.effect === 'Mica' }"
              @click="applyEffect('Mica')"
            >
              Win11 Mica
            </button>
            <button
              :class="{ active: config.theme.effect === 'Vibrancy' }"
              @click="applyEffect('Vibrancy')"
            >
              macOS Vibrancy
            </button>
            <button
              :class="{ active: config.theme.effect === 'Wallpaper' }"
              @click="applyEffect('Wallpaper')"
            >
              Wallpaper (通用)
            </button>
            <button
              :class="{ active: config.theme.effect === 'Auto' }"
              @click="applyEffect('Auto')"
            >
              Auto
            </button>
          </div>
        </div>

        <div class="control-group">
          <label>颜色主题</label>
          <div class="btn-row">
            <button
              :class="{ active: config.theme.mode === 'Auto' }"
              @click="config.theme.mode = 'Auto'"
            >
              Auto
            </button>
            <button
              :class="{ active: config.theme.mode === 'Light' }"
              @click="config.theme.mode = 'Light'"
            >
              Light
            </button>
            <button
              :class="{ active: config.theme.mode === 'Dark' }"
              @click="config.theme.mode = 'Dark'"
            >
              Dark
            </button>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<style>
  :root {
    font-family: Inter, system-ui, Avenir, Helvetica, Arial, sans-serif;
    line-height: 1.5;
    font-weight: 400;
  }

  html,
  body {
    margin: 0;
    padding: 0;
    width: 100%;
    height: 100%;
    overflow: hidden;
    background-color: transparent !important;
  }
</style>

<style scoped>
  .app-shell {
    position: relative;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    color: #202020;
    transition: color 0.2s;
  }

  .app-shell.theme-dark {
    color: #ffffff;
  }

  .wallpaper-layer {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    z-index: -1;
    pointer-events: none;
    overflow: hidden;
    transition: opacity 0.2s cubic-bezier(0.25, 0.46, 0.45, 0.94);
    background-color: #808080;
  }

  .wallpaper-base {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-size: cover;
    background-position: center;

    filter: blur(125px) saturate(210%);
    transform: scale(1.2);

    will-change: transform, filter;
  }

  .noise-overlay {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    opacity: 0.04;
    background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 200 200' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noiseFilter'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.8' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noiseFilter)'/%3E%3C/svg%3E");
  }

  .tint-overlay {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    transition: background-color 0.3s ease;
  }

  .app-shell .tint-overlay {
    background-color: rgba(243, 243, 243, 0.82);
    mix-blend-mode: overlay;
  }

  .app-shell.theme-dark .tint-overlay {
    background-color: rgba(32, 32, 32, 0.85);
    mix-blend-mode: normal;
  }

  .content-area {
    position: relative;
    z-index: 1;
    height: 100%;
    padding: 40px;
    box-sizing: border-box;
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .card {
    background: rgba(255, 255, 255, 0.5);
    padding: 30px;
    border-radius: 12px;
    width: 400px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.3);
  }

  .theme-dark .card {
    background: rgba(0, 0, 0, 0.4);
    border: 1px solid rgba(255, 255, 255, 0.08);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  }

  h1 {
    margin-top: 0;
    font-size: 1.5rem;
    font-weight: 600;
    text-align: center;
  }

  .subtitle {
    text-align: center;
    font-size: 0.9rem;
    opacity: 0.8;
    margin-bottom: 5px;
  }

  .divider {
    border: 0;
    height: 1px;
    background: rgba(0, 0, 0, 0.1);
    margin: 20px 0;
  }

  .theme-dark .divider {
    background: rgba(255, 255, 255, 0.1);
  }

  .control-group {
    margin-bottom: 20px;
  }

  .control-group label {
    display: block;
    font-size: 0.8rem;
    font-weight: 600;
    margin-bottom: 8px;
    text-transform: uppercase;
    letter-spacing: 1px;
    opacity: 0.7;
  }

  .btn-row {
    display: flex;
    gap: 8px;
  }

  button {
    flex: 1;
    padding: 8px 12px;
    border: 1px solid transparent;
    border-radius: 6px;
    background-color: rgba(0, 0, 0, 0.05);
    color: inherit;
    cursor: pointer;
    font-size: 0.9rem;
    transition: all 0.2s;
  }

  .theme-dark button {
    background-color: rgba(255, 255, 255, 0.1);
  }

  button:hover {
    background-color: rgba(0, 0, 0, 0.1);
  }

  .theme-dark button:hover {
    background-color: rgba(255, 255, 255, 0.2);
  }

  button.active {
    background-color: #007aff;
    color: white;
    font-weight: 500;
    box-shadow: 0 2px 8px rgba(0, 122, 255, 0.3);
  }
</style>
