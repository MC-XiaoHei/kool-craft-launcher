import { FluentBundle, FluentResource } from "@fluent/bundle"
import { createFluentVue } from "fluent-vue"

const modules = import.meta.glob("../../locales/*/*.ftl", {
  eager: true,
  query: "?raw",
})

const bundles: FluentBundle[] = []

function registerLocales() {
  for (const path in modules) {
    const locale = parseLocale(path)
    if (!locale) continue
    const ftlContent = readFtlContent(path)
    registerLocale(locale, ftlContent)
  }
}

function parseLocale(path: string): string | null {
  const localeMatch = path.match(/locales\/([^/]+)\/.*\.ftl$/)
  return localeMatch ? localeMatch[1] : null
}

function readFtlContent(path: string) {
  return (modules[path] as any).default
}

function registerLocale(locale: string, content: string) {
  const resource = new FluentResource(content)
  const bundle = new FluentBundle(locale)
  const errors = bundle.addResource(resource)
  if (errors.length) {
    console.warn(`Error parsing i18n file for ${locale}: ${errors.toString()}`)
  } else {
    console.info(`Loaded locale ${locale}`)
    bundles.push(bundle)
  }
}

registerLocales()

export const i18n = createFluentVue({
  bundles: bundles,
})
