<script setup lang="ts">
  import { computed, ref, watchEffect } from "vue"
  import SettingsGroup from "@/components/settings/SettingsGroup.vue"
  import { settingsSchema } from "@/services/settings/schema"
  import { settings } from "@/services/settings/value"
  import { Button } from "@/components/ui/button"
  import { Separator } from "@/components/ui/separator"
  import {
    PhCoffee,
    PhGameController,
    PhGear,
    PhInfo,
    PhPalette,
    PhSliders,
    PhUser,
  } from "@phosphor-icons/vue"

  const iconMap: Record<string, any> = {
    general: PhSliders,
    java: PhCoffee,
    account: PhUser,
    about: PhInfo,
    theme: PhPalette,
    launcher: PhGameController,
    default: PhGear,
  }

  const tabs = computed(() => Object.keys(settingsSchema.value || {}))
  const activeTab = ref("")

  watchEffect(() => {
    if (!activeTab.value && tabs.value.length > 0) {
      activeTab.value = tabs.value[0]
    }
  })

  function getIcon(key: string) {
    const lowerKey = key.toLowerCase()
    for (const mapKey in iconMap) {
      if (lowerKey.includes(mapKey)) return iconMap[mapKey]
    }
    return iconMap.default
  }

  function getLabel(key: string) {
    const schema = settingsSchema.value[key]
    return schema?.title || key.charAt(0).toUpperCase() + key.slice(1)
  }
</script>

<template>
  <div
    class="flex flex-col size-full p-6 bg-background text-foreground transition-colors duration-300"
  >
    <div class="space-y-1 mb-6" data-tauri-drag-region>
      <h1 class="text-2xl font-bold tracking-tight px-2" data-tauri-drag-region>Settings</h1>
    </div>

    <div class="flex flex-1 overflow-hidden">
      <aside class="w-48 flex-none flex flex-col gap-1 pr-4">
        <template v-for="key in tabs" :key="key">
          <Button
            variant="ghost"
            class="justify-start gap-2 px-3 h-9 text-sm font-medium rounded-md transition-all duration-200"
            :class="
              activeTab === key
                ? 'bg-muted text-primary'
                : 'text-muted-foreground hover:bg-muted/50'
            "
            @click="activeTab = key"
          >
            <component :is="getIcon(key)" weight="fill" class="size-4" />
            <span>{{ getLabel(key) }}</span>
          </Button>
        </template>
      </aside>

      <Separator orientation="vertical" class="h-full mx-2" />

      <main class="flex-1 h-full overflow-y-auto pl-6 pr-2 pb-10 scrollbar-hide">
        <Transition
          enter-active-class="transition ease-out duration-200"
          enter-from-class="opacity-0 translate-y-1"
          enter-to-class="opacity-100 translate-y-0"
          leave-active-class="transition ease-in duration-150"
          leave-from-class="opacity-100 translate-y-0"
          leave-to-class="opacity-0 translate-y-1"
          mode="out-in"
        >
          <div v-if="activeTab && settingsSchema[activeTab]" :key="activeTab" class="max-w-3xl">
            <SettingsGroup
              :schema="settingsSchema[activeTab]"
              :data-context="(settings as any)[activeTab]"
            />
          </div>
        </Transition>
      </main>
    </div>
  </div>
</template>

<style scoped>
  .scrollbar-hide::-webkit-scrollbar {
    display: none;
  }
  .scrollbar-hide {
    scrollbar-width: none;
  }
</style>
