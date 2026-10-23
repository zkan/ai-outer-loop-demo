<script setup lang="ts">
import { ref } from 'vue'

const emit = defineEmits<{
  add: [text: string]
}>()

const text = ref('')
const input = ref<HTMLInputElement | null>(null)

const submit = () => {
  emit('add', text.value)
  text.value = ''
  input.value?.focus()
}

const onKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Enter') submit()
}
</script>

<template>
  <div class="todo-input">
    <input
      ref="input"
      v-model="text"
      class="input"
      placeholder="Add a new todo…"
      autocomplete="off"
      @keydown="onKeydown"
    />
    <button class="btn btn-primary" :disabled="!text.trim()" @click="submit">
      Add
    </button>
  </div>
</template>
