<script setup lang="ts">
import Stats from './Stats.vue'

interface Props {
  completedCount: number
  totalCount: number
}
const props = defineProps<Props>()
</script>

<template>
  <header class="app-header">
    <div class="brand">
      <span class="brand-mark" aria-hidden="true">
        <svg viewBox="0 0 16 16" fill="currentColor">
          <circle cx="8" cy="8" r="7.5" />
        </svg>
      </span>
      <div class="brand-text">
        <h1>Outer Loop</h1>
        <p class="tagline">todos</p>
      </div>
    </div>
    <div class="header-meta">
      <Stats :completed="props.completedCount" :total="props.totalCount" />
      <span class="loop-badge">Loop <span class="count">3</span></span>
    </div>
  </header>
</template>
