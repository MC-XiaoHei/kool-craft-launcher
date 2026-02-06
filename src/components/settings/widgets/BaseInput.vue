<script setup lang="ts">
  import { Input } from "@/components/ui/input"
  import { Field, FieldError } from "@/components/ui/field"
  import { useSchemaValidation } from "@/composables/useSchemaValidation"
  import type { JSONSchema7 } from "json-schema"

  const props = defineProps<{
    schema: JSONSchema7
    placeholder?: string
  }>()

  const modelValue = defineModel<any>({
    set(val) {
      if (props.schema.type === "integer" || props.schema.type === "number") {
        if (val === "" || val === null || val === undefined) return undefined
        const num = parseFloat(val as string)
        return isNaN(num) ? val : num
      }
      return val
    },
  })

  const { errorMsg } = useSchemaValidation(props.schema, modelValue)
</script>

<template>
  <Field class="w-full" :data-invalid="!!errorMsg">
    <Input
      v-model="modelValue"
      :type="schema.type === 'integer' || schema.type === 'number' ? 'number' : 'text'"
      :placeholder="placeholder"
      :aria-invalid="!!errorMsg"
    />

    <FieldError v-if="errorMsg">
      {{ errorMsg }}
    </FieldError>
  </Field>
</template>
