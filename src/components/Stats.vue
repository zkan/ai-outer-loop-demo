<script setup lang="ts">
import { Badge } from '@/components/ui/badge'

defineProps<{
  completed: number
  total: number
}>()
</script>

<template>
  <Badge variant="secondary" class="h-7 px-2.5 text-xs tabular-nums">
    {{ completed }}/{{ total }} done
  </Badge>
</template>
