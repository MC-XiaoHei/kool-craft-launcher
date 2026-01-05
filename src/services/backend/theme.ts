import { ThemeConfig } from '@/bindings/types'
import { invoke } from '@tauri-apps/api/core'

export async function getThemeConfig(): Promise<ThemeConfig> {
  return await invoke<ThemeConfig>('get_theme_config')
}

export async function setThemeConfig(config: ThemeConfig): Promise<void> {
  return await invoke<void>('set_theme_config', { config })
}

export async function getWallpaperDataUrl(): Promise<string> {
  return await invoke<string>('get_wallpaper')
}
