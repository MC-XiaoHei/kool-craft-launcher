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
  import { AcceptableValue } from "reka-ui"

  const props = defineProps<{
    modelValue: any
    schema: JSONSchema7
    placeholder?: string
  }>()

  const emit = defineEmits(["update:modelValue"])

  const options = computed(() => {
    const enums = props.schema.enum || []
    return enums.map(val => ({
      label: String(val),
      value: String(val),
    }))
  })

  const onUpdate = (val: AcceptableValue) => {
    emit("update:modelValue", val)
  }
</script>

<template>
  <Select :model-value="String(modelValue)" @update:model-value="onUpdate">
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
