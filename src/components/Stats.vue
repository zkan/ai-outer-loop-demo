<script setup lang="ts">
interface Props {
  completed: number
  total: number
}
const props = defineProps<Props>()
</script>

<template>
  <span class="stats">{{ props.completed }}/{{ props.total }}</span>
</template>
