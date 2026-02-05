<script setup lang="ts">
  import { toRef } from "vue"
  import { Input } from "@/components/ui/input"
  import { useSchemaValidation } from "@/composables/useSchemaValidation"
  import type { JSONSchema7 } from "json-schema"

  const props = defineProps<{
    modelValue: any
    schema: JSONSchema7
    placeholder?: string
  }>()

  const emit = defineEmits(["update:modelValue"])

  const valueRef = toRef(props, "modelValue")
  const { errorMsg } = useSchemaValidation(props.schema, valueRef)

  const onInput = (e: Event) => {
    const val = (e.target as HTMLInputElement).value

    if (props.schema.type === "integer" || props.schema.type === "number") {
      const num = parseFloat(val)
      emit("update:modelValue", val === "" ? undefined : isNaN(num) ? val : num)
    } else {
      emit("update:modelValue", val)
    }
  }
</script>

<template>
  <div class="w-full relative">
    <Input
      :model-value="modelValue"
      :type="schema.type === 'integer' || schema.type === 'number' ? 'number' : 'text'"
      :placeholder="placeholder"
      @input="onInput"
      :class="{ 'border-red-500 focus-visible:ring-red-500': !!errorMsg }"
    />

    <transition
      enter-active-class="transition-all duration-200 ease-out"
      enter-from-class="opacity-0 -translate-y-1"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition-all duration-150 ease-in"
      leave-from-class="opacity-100 translate-y-0"
      leave-to-class="opacity-0 -translate-y-1"
    >
      <div v-if="errorMsg" class="absolute left-0 -bottom-5 text-[10px] text-red-500 leading-none">
        {{ errorMsg }}
      </div>
    </transition>
  </div>
</template>
