<script setup lang="ts">
import TodoItem from './TodoItem.vue'
import type { Todo } from '@/types'

defineProps<{
  todos: Todo[]
}>()

const emit = defineEmits<{
  toggle: [id: string]
  remove: [id: string]
}>()
</script>

<template>
  <ul class="flex flex-col gap-2">
    <TodoItem
      v-for="todo in todos"
      :key="todo.id"
      :todo="todo"
      @toggle="emit('toggle', todo.id)"
      @remove="emit('remove', todo.id)"
    />
  </ul>
</template>
