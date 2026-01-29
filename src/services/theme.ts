import { Colord, colord, extend } from "colord"
import mixPlugin from "colord/plugins/mix"
import namesPlugin from "colord/plugins/names"
import harmoniesPlugin from "colord/plugins/harmonies"
import { watch } from "vue"
import { settings } from "@/services/settings/value"

extend([mixPlugin, namesPlugin, harmoniesPlugin])

export function initTheme() {
  updateTheme(settings.value.theme.primaryHex)
  watch(
    () => settings.value.theme.primaryHex,
    val => updateTheme(val),
  )
}

export function updateTheme(primaryHex: string) {
  const primary = colord(primaryHex)
  const neutralSat = getNeutralSaturation(primary)
  const chartColors = generateChartColors(primary)

  const lightVars = getLightVars(primary, neutralSat, chartColors)
  const darkVars = getDarkVars(primary, neutralSat, chartColors)

  injectCss(lightVars, darkVars)
}

type ThemeVars = Record<string, string>

function toCss(color: Colord): string {
  const { h, s, l, a } = color.toHsl()
  const css = `hsl(${h.toFixed(1)} ${s.toFixed(1)}% ${l.toFixed(1)}%`
  return a < 1 ? `${css} / ${a})` : `${css})`
}

function getTintedNeutral(color: Colord, saturation: number, lightness: number): string {
  const { h } = color.toHsl()
  return toCss(colord({ h, s: saturation, l: lightness }))
}

function getContrastText(background: Colord): string {
  return background.isDark() ? "#ffffff" : "#0f172a"
}

function getNeutralSaturation(primary: Colord): number {
  const { s } = primary.toHsl()
  return Math.min(s, 12)
}

function generateChartColors(primary: Colord): string[] {
  const charts = primary.harmonies("analogous").map(toCss)

  while (charts.length < 5) {
    const lastColor = colord(charts[charts.length - 1])
    charts.push(toCss(lastColor.rotate(40).lighten(0.1)))
  }

  return charts
}

function getLightVars(primary: Colord, neutralSat: number, charts: string[]): ThemeVars {
  const neutral = (l: number) => getTintedNeutral(primary, neutralSat, l)

  return {
    "--background": neutral(99),
    "--foreground": neutral(5),
    "--card": neutral(99),
    "--card-foreground": neutral(5),
    "--popover": neutral(99),
    "--popover-foreground": neutral(5),

    "--primary": toCss(primary),
    "--primary-foreground": getContrastText(primary),

    "--secondary": neutral(94),
    "--secondary-foreground": neutral(20),

    "--muted": neutral(94),
    "--muted-foreground": neutral(40),

    "--accent": neutral(94),
    "--accent-foreground": neutral(20),

    "--destructive": "hsl(0 84.2% 60.2%)",
    "--destructive-foreground": "hsl(0 0% 98%)",

    "--border": neutral(90),
    "--input": neutral(90),
    "--ring": toCss(primary.alpha(0.3)),

    "--sidebar": neutral(98),
    "--sidebar-foreground": neutral(5),
    "--sidebar-primary": toCss(primary),
    "--sidebar-primary-foreground": getContrastText(primary),
    "--sidebar-accent": neutral(94),
    "--sidebar-accent-foreground": neutral(20),
    "--sidebar-border": neutral(90),
    "--sidebar-ring": toCss(primary.alpha(0.3)),

    "--chart-1": charts[0],
    "--chart-2": charts[1],
    "--chart-3": charts[2],
    "--chart-4": charts[3],
    "--chart-5": charts[4],
  }
}

function getDarkVars(primary: Colord, neutralSat: number, charts: string[]): ThemeVars {
  const neutral = (l: number) => getTintedNeutral(primary, neutralSat, l)

  return {
    "--background": neutral(4),
    "--foreground": neutral(98),
    "--card": neutral(6),
    "--card-foreground": neutral(98),
    "--popover": neutral(6),
    "--popover-foreground": neutral(98),

    "--primary": toCss(primary),
    "--primary-foreground": getContrastText(primary),

    "--secondary": neutral(15),
    "--secondary-foreground": neutral(98),

    "--muted": neutral(15),
    "--muted-foreground": neutral(65),

    "--accent": neutral(15),
    "--accent-foreground": neutral(98),

    "--destructive": "hsl(0 62.8% 30.6%)",
    "--destructive-foreground": "hsl(0 0% 98%)",

    "--border": neutral(20),
    "--input": neutral(20),
    "--ring": toCss(primary.alpha(0.4)),

    "--sidebar": neutral(4),
    "--sidebar-foreground": neutral(96),
    "--sidebar-primary": toCss(primary),
    "--sidebar-primary-foreground": "#ffffff",
    "--sidebar-accent": neutral(15),
    "--sidebar-accent-foreground": neutral(98),
    "--sidebar-border": neutral(20),
    "--sidebar-ring": toCss(primary.alpha(0.4)),

    "--chart-1": toCss(colord(charts[0]).lighten(0.1)),
    "--chart-2": toCss(colord(charts[1]).lighten(0.1)),
    "--chart-3": toCss(colord(charts[2]).lighten(0.1)),
    "--chart-4": toCss(colord(charts[3]).lighten(0.1)),
    "--chart-5": toCss(colord(charts[4]).lighten(0.1)),
  }
}

function formatCssRules(selector: string, vars: ThemeVars): string {
  const rules = Object.entries(vars)
    .map(([key, value]) => `${key}: ${value};`)
    .join(" ")
  return `${selector} { ${rules} }`
}

function injectCss(lightVars: ThemeVars, darkVars: ThemeVars): void {
  const cssContent = [formatCssRules(":root", lightVars), formatCssRules(".dark", darkVars)].join(
    "\n",
  )

  const styleId = "dynamic-theme-styles"
  let styleTag = document.getElementById(styleId)

  if (!styleTag) {
    styleTag = document.createElement("style")
    styleTag.id = styleId
    document.head.appendChild(styleTag)
  }

  styleTag.textContent = cssContent
}
