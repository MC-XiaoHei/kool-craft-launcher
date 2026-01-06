import { invoke } from "@tauri-apps/api/core"

export async function getWallpaperDataUrl(): Promise<string> {
  return await invoke<string>("get_wallpaper")
}

export async function refreshWindowTheme(): Promise<void> {
  return await invoke("refresh_window_theme")
}
