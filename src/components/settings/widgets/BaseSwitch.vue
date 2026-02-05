<script setup lang="ts">
  import { computed } from "vue"
  import { Switch } from "@/components/ui/switch"
  import type { JSONSchema7 } from "json-schema"

  const props = defineProps<{
    modelValue: any
    schema?: JSONSchema7
  }>()

  const emit = defineEmits(["update:modelValue"])

  const isChecked = computed({
    get: () => !!props.modelValue,
    set: val => emit("update:modelValue", val),
  })
</script>

<template>
  <div class="flex items-center h-9">
    <Switch v-model:checked="isChecked" />
  </div>
</template>
