<script setup lang="ts">
import { ref } from 'vue'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'

const emit = defineEmits<{
  add: [text: string]
}>()

const text = ref('')
const input = ref<{ $el: HTMLInputElement } | null>(null)

const submit = () => {
  const value = text.value
  if (!value.trim()) return
  emit('add', value)
  text.value = ''
  focus()
}

const focus = () => {
  input.value?.$el?.focus()
}

defineExpose({ focus })
</script>

<template>
  <form class="flex items-center gap-2" @submit.prevent="submit">
    <Input
      ref="input"
      v-model="text"
      class="h-10"
      placeholder="Add a new todo…"
      autocomplete="off"
      aria-label="New todo"
    />
    <Button type="submit" class="h-10 px-4" :disabled="!text.trim()">
      Add
    </Button>
  </form>
</template>
