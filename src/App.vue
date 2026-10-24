<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import AppHeader from './components/AppHeader.vue'
import EmptyState from './components/EmptyState.vue'
import FilterTabs from './components/FilterTabs.vue'
import TodoInput from './components/TodoInput.vue'
import TodoList from './components/TodoList.vue'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { createTodo, deleteTodo, listTodos, updateTodo } from './api'
import { createLoadGate, emptyState, filterTodos, type Filter } from './todoList'
import type { Todo } from './types'

const todos = reactive<Todo[]>([])
const error = ref<string | null>(null)
const filter = ref<Filter>('all')
const inputRef = ref<{ focus: () => void } | null>(null)
const gate = createLoadGate()

const loadTodos = async () => {
  const token = gate.beginLoad()
  try {
    const items = await listTodos()
    if (!gate.canApply(token)) return
    error.value = null
    todos.splice(0, todos.length, ...items)
  } catch (e) {
    if (!gate.canApply(token)) return
    error.value = (e as Error).message
  }
}

const addTodo = async (text: string) => {
  const trimmed = text.trim()
  if (!trimmed) return
  gate.beginMutation()
  try {
    const created = await createTodo({ text: trimmed })
    todos.push(created)
    error.value = null
  } catch (e) {
    error.value = (e as Error).message
  } finally {
    gate.endMutation()
  }
}

const removeTodo = async (id: string) => {
  gate.beginMutation()
  try {
    await deleteTodo(id)
    const idx = todos.findIndex((todo) => todo.id === id)
    if (idx !== -1) todos.splice(idx, 1)
    error.value = null
  } catch (e) {
    error.value = (e as Error).message
  } finally {
    gate.endMutation()
  }
}

const toggleTodo = async (id: string) => {
  const todo = todos.find((item) => item.id === id)
  if (!todo) return
  gate.beginMutation()
  try {
    const updated = await updateTodo(id, { completed: !todo.completed })
    Object.assign(todo, updated)
    error.value = null
  } catch (e) {
    error.value = (e as Error).message
  } finally {
    gate.endMutation()
  }
}

const completedCount = computed(() => todos.filter((todo) => todo.completed).length)
const totalCount = computed(() => todos.length)
const counts = computed(() => ({
  all: totalCount.value,
  active: totalCount.value - completedCount.value,
  completed: completedCount.value,
}))
const visibleTodos = computed(() => filterTodos(todos, filter.value))
const empty = computed(() => emptyState(filter.value))

const focusInput = () => {
  inputRef.value?.focus()
}

loadTodos()
</script>

<template>
  <div class="bg-muted/40 text-foreground min-h-svh">
    <AppHeader :completed-count="completedCount" :total-count="totalCount" />
    <main class="mx-auto flex w-full max-w-2xl flex-col gap-6 px-4 py-8">
      <p
        v-if="error"
        role="alert"
        class="border-destructive/30 bg-destructive/10 text-destructive rounded-lg border px-3 py-2 text-sm"
      >
        {{ error }}
      </p>
      <Card class="shadow-sm">
        <CardHeader>
          <CardTitle class="text-lg tracking-tight">Your list</CardTitle>
          <CardDescription>Add a task, then check it off when it is done.</CardDescription>
        </CardHeader>
        <CardContent class="flex flex-col gap-4">
          <TodoInput ref="inputRef" @add="addTodo" />
          <FilterTabs v-model="filter" :counts="counts" />
          <TodoList
            v-if="visibleTodos.length > 0"
            :todos="visibleTodos"
            @toggle="toggleTodo"
            @remove="removeTodo"
          />
          <EmptyState
            v-else
            :title="empty.title"
            :description="empty.description"
            :action-label="empty.actionLabel"
            @action="focusInput"
          />
        </CardContent>
      </Card>
    </main>
  </div>
</template>
