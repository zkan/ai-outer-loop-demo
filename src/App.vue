<script setup lang="ts">
import { reactive, computed, ref } from 'vue'
import AppHeader from './components/AppHeader.vue'
import TodoInput from './components/TodoInput.vue'
import TodoList from './components/TodoList.vue'
import EmptyState from './components/EmptyState.vue'

interface Todo {
  id: string
  text: string
  done: boolean
  createdAt: number
}

const todos = reactive<Todo[]>([])
let idCounter = 0

const addTodo = (text: string) => {
  const trimmed = text.trim()
  if (!trimmed) return
  todos.push({
    id: String(++idCounter),
    text: trimmed,
    done: false,
    createdAt: Date.now(),
  })
}

const removeTodo = (id: string) => {
  const idx = todos.findIndex((t) => t.id === id)
  if (idx !== -1) todos.splice(idx, 1)
}

const toggleTodo = (id: string) => {
  const todo = todos.find((t) => t.id === id)
  if (todo) todo.done = !todo.done
}

const completedCount = computed(() => todos.filter((t) => t.done).length)
const totalCount = computed(() => todos.length)

const inputRef = ref<HTMLInputElement | null>(null)

const focusInput = () => {
  inputRef.value?.focus()
}
</script>

<template>
  <div class="app">
    <AppHeader :completed-count="completedCount" :total-count="totalCount" />
    <main class="container">
      <section class="main">
        <TodoInput ref="inputRef" @add="addTodo" />
        <TodoList :todos="todos" @toggle="toggleTodo" @remove="removeTodo" />
        <EmptyState
          v-if="todos.length === 0"
          message="No todos yet — add your first one below."
          action-label="Add one"
          :on-action="focusInput"
        />
      </section>
    </main>
  </div>
</template>
