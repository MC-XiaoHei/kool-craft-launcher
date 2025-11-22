const cacheKey = 'wallpaper_cache'

export async function cacheWallpaper(base64: string) {
    localStorage.setItem(cacheKey, base64)
}

export function getCachedWallpaper(): string | null {
    const cached = localStorage.getItem(cacheKey)
    if (cached === '' || !cached) return null
    return cached
}