<script setup lang="ts">
  import SettingsLeaf from "./SettingsLeaf.vue"
  import { SettingsGroupSchema } from "@/services/settings/schema"

  defineProps<{
    schema: SettingsGroupSchema
    dataContext: any
  }>()
</script>

<template>
  <div class="schema-group border p-4 rounded-lg mb-4">
    <h3 v-if="schema.title" class="text-lg font-bold mb-4 border-b pb-2">
      {{ schema.title }}
    </h3>

    <template v-for="(fieldSchema, key) in schema.properties" :key="key">
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
