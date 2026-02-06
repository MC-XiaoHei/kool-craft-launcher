<script setup lang="ts">
  import { computed } from "vue"
  import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
  } from "@/components/ui/select"
  import type { JSONSchema7 } from "json-schema"

  const props = defineProps<{
    schema: JSONSchema7
    placeholder?: string
  }>()

  const modelValue = defineModel<any>({
    get(val) {
      return val === undefined || val === null ? "" : String(val)
    },
    set(val) {
      return val
    },
  })

  const options = computed(() => {
    return (props.schema.enum || []).map(val => ({
      label: String(val),
      value: String(val),
    }))
  })
</script>

<template>
  <Select v-model="modelValue">
    <SelectTrigger class="w-full">
      <SelectValue :placeholder="placeholder || String(schema.default || 'Select...')" />
    </SelectTrigger>
    <SelectContent>
      <SelectItem v-for="opt in options" :key="opt.value" :value="opt.value">
        {{ opt.label }}
      </SelectItem>
    </SelectContent>
  </Select>
</template>
