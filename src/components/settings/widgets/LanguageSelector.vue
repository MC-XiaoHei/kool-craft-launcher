<script setup lang="ts">
  import { computed } from "vue"
  import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
  } from "@/components/ui/select"

  import type { Locales } from "@/bindings/types"
  import { availableLocales, t } from "@/services/i18n"

  const modelValue = defineModel<Locales | string>({
    get(val) {
      return val === undefined || val === null ? "" : String(val)
    },
    set(val) {
      return val
    },
  })

  const options = computed(() => {
    return availableLocales.map(key => ({
      label: t(key, "lang-name"),
      value: key,
    }))
  })
</script>

<template>
  <Select v-model="modelValue">
    <SelectTrigger class="w-full">
      <SelectValue />
    </SelectTrigger>

    <SelectContent>
      <SelectItem v-for="opt in options" :key="opt.value" :value="opt.value">
        {{ opt.label }}
      </SelectItem>
    </SelectContent>
  </Select>
</template>
