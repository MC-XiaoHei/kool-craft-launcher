<script setup lang="ts">
  import { computed } from "vue"
  import { SettingsGroupSchema } from "@/services/settings/schema"
  import { Input } from "@/components/ui/input"
  import { Switch } from "@/components/ui/switch"
  import { Label } from "@/components/ui/label"
  import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
  } from "@/components/ui/select"

  const modelValueUpdateEvent = "update:modelValue"

  const props = defineProps<{
    fieldKey: string
    schema: SettingsGroupSchema
    modelValue: any
  }>()

  const emit = defineEmits([modelValueUpdateEvent])

  const internalValue = computed({
    get: () => props.modelValue,
    set: val => emit(modelValueUpdateEvent, val),
  })

  const booleanValue = computed({
    get: () => props.modelValue as boolean,
    set: val => emit(modelValueUpdateEvent, val),
  })

  const numberValue = computed({
    get: () => props.modelValue,
    set: val => {
      const num = parseFloat(val as string)
      emit(modelValueUpdateEvent, isNaN(num) ? val : num)
    },
  })

  const widgetType = computed(() => {
    if (props.schema.type === "boolean") return "switch"
    if (props.schema.enum) return "select"
    if (props.schema.type === "integer" || props.schema.type === "number") return "number"
    return "text"
  })
</script>

<template>
  <div
    class="flex flex-row items-center justify-between rounded-lg border p-4 shadow-sm mb-3 bg-card"
  >
    <div class="space-y-0.5 flex flex-col pr-6 shrink">
      <Label :for="fieldKey" class="text-base font-medium">
        {{ schema.title || fieldKey }}
      </Label>
      <span v-if="schema.description" class="text-sm text-muted-foreground">
        {{ schema.description }}
      </span>
    </div>

    <div class="shrink-0 min-w-30 flex justify-end">
      <Switch v-if="widgetType === 'switch'" :id="fieldKey" v-model:checked="booleanValue" />

      <Select v-else-if="widgetType === 'select'" v-model="internalValue">
        <SelectTrigger :id="fieldKey" class="w-45">
          <SelectValue :placeholder="String(schema.default ?? 'Select')" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem v-for="opt in schema.enum" :key="String(opt)" :value="String(opt)">
            {{ opt }}
          </SelectItem>
        </SelectContent>
      </Select>

      <Input
        v-else-if="widgetType === 'number'"
        :id="fieldKey"
        type="number"
        v-model="numberValue"
        class="w-45 text-right"
      />

      <Input
        v-else
        :id="fieldKey"
        type="text"
        v-model="internalValue"
        class="w-60"
        :placeholder="String(schema.default ?? '')"
      />
    </div>
  </div>
</template>
