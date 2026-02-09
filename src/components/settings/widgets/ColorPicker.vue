<script setup lang="ts">
  import { SketchPicker } from "vue-color"
  import type { JSONSchema7 } from "json-schema"
  import { Popover, PopoverTrigger, PopoverContent } from "@/components/ui/popover"
  import { Button } from "@/components/ui/button"
  import { computed } from "vue"

  const modelValue = defineModel<any>()

  const props = defineProps<{
    schema: JSONSchema7
    alpha?: boolean
  }>()

  const hideAlpha = computed(() => !props.alpha)
</script>

<template>
  <div class="w-full relative">
    <Popover>
      <PopoverTrigger as-child>
        <Button variant="outline">Select Color</Button>
      </PopoverTrigger>
      <PopoverContent as-child>
        <SketchPicker :disableAlpha="hideAlpha" v-model="modelValue" />
      </PopoverContent>
    </Popover>
  </div>
</template>
