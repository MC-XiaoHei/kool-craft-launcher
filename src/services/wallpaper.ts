import { getWallpaperDataUrl } from "@/bindings/commands"
import { ref } from "vue"

const cacheKey = "wallpaper_cache"
export const wallpaperUrl = ref(getCachedWallpaper())

export async function loadWallpaper() {
  if (wallpaperUrl.value) return
  const base64 = await getWallpaperDataUrl()
  if (!base64) return
  cacheWallpaper(base64).then()
  wallpaperUrl.value = base64
}

export async function cacheWallpaper(base64: string) {
  localStorage.setItem(cacheKey, base64)
}

export function getCachedWallpaper(): string | null {
  const cached = localStorage.getItem(cacheKey)
  if (cached === "" || !cached) return null
  return cached
}
