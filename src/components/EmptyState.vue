<script setup lang="ts">
interface Props {
  message?: string
  actionLabel?: string
  onAction?: () => void
}
const props = defineProps<Props>()
</script>

<template>
  <div class="empty-state">
    <p class="empty-icon" aria-hidden="true">▢</p>
    <p class="empty-message">{{ message }}</p>
    <button
      v-if="actionLabel && onAction"
      class="btn btn-primary empty-cta"
      @click="props.onAction"
    >
      {{ actionLabel }}
    </button>
  </div>
</template>
