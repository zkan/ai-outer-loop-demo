<script setup lang="ts">
import TodoItem from './TodoItem.vue'
import type { Todo } from '../types'

interface Props {
  todos: Todo[]
}

const { todos } = defineProps<Props>()
const emit = defineEmits<{
  toggle: [id: string]
  remove: [id: string]
}>()
</script>

<template>
  <ul class="todo-list">
    <TodoItem
      v-for="todo in todos"
      :todo="todo"
      :key="todo.id"
      @toggle="emit('toggle', todo.id)"
      @remove="emit('remove', todo.id)"
    />
  </ul>
</template>
