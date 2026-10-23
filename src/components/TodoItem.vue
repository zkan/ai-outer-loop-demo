<script setup lang="ts">
import type { Todo } from '../types'

interface Props {
  todo: Todo
}

const { todo } = defineProps<Props>()

const emit = defineEmits<{
  toggle: [id: string]
  remove: [id: string]
}>()
</script>

<template>
  <li class="todo-item" :class="{ done: todo.completed }">
    <div class="checkbox-wrap">
      <input
        type="checkbox"
        :checked="todo.completed"
        @change="emit('toggle', todo.id)"
      />
      <span class="checkbox-mark" aria-hidden="true">✓</span>
    </div>
    <span class="todo-text">{{ todo.text }}</span>
    <button class="icon-btn" aria-label="Delete todo" @click="emit('remove', todo.id)">
      ×
    </button>
  </li>
</template>
