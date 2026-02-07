import { FluentBundle, FluentResource } from "@fluent/bundle"
import { createFluentVue } from "fluent-vue"
import { settings } from "@/services/settings/value"
import { watch } from "vue"
import { I18nKeys, Locales } from "@/bindings/types"
import { pascalCase } from "change-case"

const modules = import.meta.glob("../../locales/*/*.ftl", {
  eager: true,
  query: "?raw",
})

const bundles: Record<Locales, FluentBundle> = Object.entries(modules).reduce(
  (acc, [path, file]) => {
    const locale = getLocaleFromPath(path)
    if (!locale) return acc

    const content = (file as any).default
    const bundle = getLocaleBundle(locale, content)
    if (!bundle) return acc

    const key: Locales = pascalCase(locale) as Locales
    acc[key] = bundle

    return acc
  },
  {} as Record<Locales, FluentBundle>,
)

function getLocaleFromPath(path: string): string | null {
  const localeMatch = path.match(/locales\/([^/]+)\/.*\.ftl$/)
  return localeMatch ? localeMatch[1] : null
}

function getLocaleBundle(locale: string, content: string): FluentBundle | null {
  const resource = new FluentResource(content)
  const bundle = new FluentBundle(locale)
  const errors = bundle.addResource(resource)
  if (errors.length) {
    console.warn(`Error parsing i18n file for ${locale}: ${errors.toString()}`)
    return null
  } else {
    console.info(`Loaded locale ${locale}`)
    return bundle
  }
}

function getSettingsLangBundle() {
  const key = settings.value.general.lang
  if (!key) return getDefaultLangBundle()
  return bundles[key]
}

function getDefaultLangBundle() {
  return bundles["EnUs"]
}

watch(
  () => settings.value.general.lang,
  () => {
    i18n.bundles = [getSettingsLangBundle(), getDefaultLangBundle()]
  },
)

export const availableLocales = Object.keys(bundles) as Locales[]

export function t(locale: Locales, key: I18nKeys, args?: Record<string, any>): string {
  const bundle = bundles[locale]

  if (!bundle) {
    console.warn(`No bundle found for locale: ${locale}`)
    return key
  }

  const message = bundle.getMessage(key)

  if (message && message.value) {
    return bundle.formatPattern(message.value, args)
  }

  return key
}

export const i18n = createFluentVue({
  bundles: [getSettingsLangBundle(), getDefaultLangBundle()],
})
