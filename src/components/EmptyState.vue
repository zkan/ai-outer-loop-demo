<script setup lang="ts">
import { ListTodoIcon } from '@lucide/vue'
import { Button } from '@/components/ui/button'
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from '@/components/ui/empty'

defineProps<{
  title: string
  description: string
  actionLabel?: string
}>()

const emit = defineEmits<{
  action: []
}>()
</script>

<template>
  <Empty class="border bg-muted/30 py-10">
    <EmptyHeader>
      <EmptyMedia variant="icon">
        <ListTodoIcon />
      </EmptyMedia>
      <EmptyTitle>{{ title }}</EmptyTitle>
      <EmptyDescription>{{ description }}</EmptyDescription>
    </EmptyHeader>
    <EmptyContent v-if="actionLabel">
      <Button variant="outline" @click="emit('action')">
        {{ actionLabel }}
      </Button>
    </EmptyContent>
  </Empty>
</template>
