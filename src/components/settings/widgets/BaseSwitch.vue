<script setup lang="ts">
  import { Switch } from "@/components/ui/switch"
  import type { JSONSchema7 } from "json-schema"

  defineProps<{
    schema?: JSONSchema7
  }>()

  const modelValue = defineModel<boolean>({ default: false })
</script>

<template>
  <div class="flex items-center h-9">
    <Switch v-model:checked="modelValue" />
  </div>
</template>
