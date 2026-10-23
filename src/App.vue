<script setup lang="ts">
import { reactive, computed, ref } from 'vue'
import AppHeader from './components/AppHeader.vue'
import TodoInput from './components/TodoInput.vue'
import TodoList from './components/TodoList.vue'
import EmptyState from './components/EmptyState.vue'
import { createTodo, deleteTodo, listTodos, updateTodo } from './api'
import type { Todo } from './types'

const todos = reactive<Todo[]>([])
const error = ref<string | null>(null)

const loadTodos = async () => {
  try {
    error.value = null
    todos.splice(0, todos.length, ...await listTodos())
  } catch (e) {
    error.value = (e as Error).message
  }
}

const addTodo = async (text: string) => {
  const trimmed = text.trim()
  if (!trimmed) return
  try {
    const created = await createTodo({ text: trimmed })
    todos.push(created)
  } catch (e) {
    error.value = (e as Error).message
  }
}

const removeTodo = async (id: string) => {
  try {
    await deleteTodo(id)
    const idx = todos.findIndex((t: Todo) => t.id === id)
    if (idx !== -1) todos.splice(idx, 1)
  } catch (e) {
    error.value = (e as Error).message
  }
}

const toggleTodo = async (id: string) => {
  const todo = todos.find((t: Todo) => t.id === id)
  if (!todo) return
  try {
    const updated = await updateTodo(id, { completed: !todo.completed })
    Object.assign(todo, updated)
  } catch (e) {
    error.value = (e as Error).message
  }
}

const completedCount = computed(() => todos.filter((t: Todo) => t.completed).length)
const totalCount = computed(() => todos.length)

const inputRef = ref<HTMLInputElement | null>(null)

const focusInput = () => {
  inputRef.value?.focus()
}

// Seed the shell from the backend on first render.
loadTodos()
</script>

<template>
  <div class="app">
    <AppHeader :completed-count="completedCount" :total-count="totalCount" />
    <main class="container">
      <section class="main">
        <div class="error-banner" v-if="error">
          <span class="error-icon" aria-hidden="true">!</span>
          <span class="error-text">{{ error }}</span>
        </div>
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
