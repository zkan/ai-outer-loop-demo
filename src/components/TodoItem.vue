<script setup lang="ts">
import { Trash2Icon } from '@lucide/vue'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import type { Todo } from '@/types'

defineProps<{
  todo: Todo
}>()

const emit = defineEmits<{
  toggle: [id: string]
  remove: [id: string]
}>()
</script>

<template>
  <li class="bg-card flex items-center gap-3 rounded-lg border px-3 py-2.5">
    <Checkbox
      :model-value="todo.completed"
      :aria-label="todo.completed ? 'Mark as not done' : 'Mark as done'"
      @update:model-value="emit('toggle', todo.id)"
    />
    <span
      class="min-w-0 flex-1 text-sm leading-6"
      :class="todo.completed ? 'text-muted-foreground line-through' : 'text-foreground'"
    >
      {{ todo.text }}
    </span>
    <Button
      variant="ghost"
      size="icon"
      aria-label="Delete todo"
      @click="emit('remove', todo.id)"
    >
      <Trash2Icon />
    </Button>
  </li>
</template>
