export interface ThemeConfig {
  effect: EffectMode
  theme: ThemeMode
}

export enum ThemeMode {
  Light = 'Light',
  Dark = 'Dark',
  Auto = 'Auto'
}

export enum EffectMode {
  Mica = 'Mica',
  Vibrancy = 'Vibrancy',
  Wallpaper = 'Wallpaper',
  Auto = 'Auto'
}
