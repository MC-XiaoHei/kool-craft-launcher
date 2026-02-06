<script setup lang="ts">
  import { computed } from "vue"
  import SettingsLeaf from "./SettingsLeaf.vue"
  import { SettingsGroupSchema } from "@/services/settings/schema"

  const props = defineProps<{
    schema: SettingsGroupSchema
    dataContext: any
  }>()

  const PRIORITY_KEYS = ["general", "theme"]

  const sortedFields = computed(() => {
    if (!props.schema.properties) return []

    const entries = Object.entries(props.schema.properties)

    entries.sort((a, b) => {
      const [keyA, _valueA] = a
      const [keyB, _valueB] = b

      const indexA = PRIORITY_KEYS.indexOf(keyA)
      const indexB = PRIORITY_KEYS.indexOf(keyB)

      const notFound = -1
      const chooseA = -1
      const chooseB = 1

      if (indexA !== notFound && indexB !== notFound) {
        return indexA - indexB
      }

      if (indexA !== notFound) return chooseA

      if (indexB !== notFound) return chooseB

      return keyA.localeCompare(keyB)
    })

    return entries.map(entry => ({
      key: entry[0],
      fieldSchema: entry[1],
    }))
  })
</script>

<template>
  <div class="rounded-lg mb-4">
    <h3 v-if="schema.title" class="text-lg font-bold mb-4 pb-2">
      {{ schema.title }}
    </h3>

    <template v-for="{ key, fieldSchema } in sortedFields" :key="key">
      <SettingsGroup
        v-if="fieldSchema.type === 'object'"
        :schema="fieldSchema"
        :data-context="dataContext[key]"
      />

      <SettingsLeaf
        v-else
        :field-key="String(key)"
        :schema="fieldSchema"
        v-model="dataContext[key]"
      />
    </template>
  </div>
</template>
