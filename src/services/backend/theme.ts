import { ThemeConfig } from '@/types/theme'
import { invoke } from '@tauri-apps/api/core'

export async function loadThemeConfig(): Promise<ThemeConfig> {
  return await invoke<ThemeConfig>('load_theme_config')
}

export async function setThemeConfig(config: ThemeConfig): Promise<void> {
  return await invoke<void>('set_theme_config', { config })
}

export async function getWallpaperDataUrl(): Promise<string> {
  return await invoke<string>('get_wallpaper')
}
