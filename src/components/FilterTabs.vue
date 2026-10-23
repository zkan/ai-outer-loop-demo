<script setup lang="ts">
import { ref } from 'vue'

type Filter = 'all' | 'active' | 'completed'

const selected = ref<Filter>('all')

const tabs: { value: Filter; label: string }[] = [
  { value: 'all', label: 'All' },
  { value: 'active', label: 'Active' },
  { value: 'completed', label: 'Completed' },
]
</script>

<template>
  <div class="filter-tabs" role="tablist" aria-label="Filter todos">
    <button
      v-for="tab in tabs"
      :key="tab.value"
      type="button"
      class="filter-tab"
      :class="{ active: selected === tab.value }"
      :aria-pressed="selected === tab.value"
      @click="selected = tab.value"
    >
      {{ tab.label }}
    </button>
  </div>
</template>
