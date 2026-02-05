<script setup lang="ts">
  import { computed } from "vue"
  import { SettingsGroupSchema } from "@/services/settings/schema"
  import { Label } from "@/components/ui/label"
  import { resolveSettingsComponent } from "@/services/settings/registry"

  const props = defineProps<{
    fieldKey: string
    schema: SettingsGroupSchema
    modelValue: any
  }>()

  const emit = defineEmits(["update:modelValue"])

  const internalValue = computed({
    get: () => props.modelValue,
    set: val => emit("update:modelValue", val),
  })

  const targetWidget = computed(() => resolveSettingsComponent(props.schema))

  const targetProps = computed(() => {
    const baseProps = {
      id: props.fieldKey,
      schema: props.schema,
    }

    const customArgs = props.schema.args || {}

    return {
      ...baseProps,
      ...customArgs,
    }
  })
</script>

<template>
  <div
    class="flex flex-row items-center justify-between rounded-lg border p-4 shadow-sm mb-3 bg-card transition-colors"
  >
    <div class="space-y-0.5 flex flex-col pr-6 shrink">
      <Label :for="fieldKey" class="text-base font-medium">
        {{ schema.title || fieldKey }}
      </Label>
      <span v-if="schema.description" class="text-sm text-muted-foreground">
        {{ schema.description }}
      </span>
    </div>

    <div class="shrink-0 min-w-50 flex justify-end">
      <component
        :is="targetWidget"
        v-model="internalValue"
        v-bind="targetProps"
        class="w-full max-w-60"
      />
    </div>
  </div>
</template>
