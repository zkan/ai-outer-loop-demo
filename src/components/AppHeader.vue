<script setup lang="ts">
import { ref } from 'vue'
import { ListTodoIcon, MoonIcon, SunIcon } from '@lucide/vue'
import { Button } from '@/components/ui/button'
import { applyTheme } from '@/theme'
import Stats from './Stats.vue'

defineProps<{
  completedCount: number
  totalCount: number
}>()

const dark = ref(document.documentElement.classList.contains('dark'))

const toggleTheme = () => {
  dark.value = !dark.value
  applyTheme(dark.value, true)
}
</script>

<template>
  <header class="bg-background/80 sticky top-0 z-10 border-b backdrop-blur">
    <div class="mx-auto flex h-16 max-w-2xl items-center justify-between gap-4 px-4">
      <div class="flex min-w-0 items-center gap-3">
        <div class="bg-primary text-primary-foreground flex size-9 shrink-0 items-center justify-center rounded-lg">
          <ListTodoIcon class="size-4" />
        </div>
        <div class="min-w-0">
          <h1 class="truncate text-base font-semibold tracking-tight">Outer Loop</h1>
          <p class="text-muted-foreground text-sm leading-none">Todos</p>
        </div>
      </div>
      <div class="flex items-center gap-2">
        <Stats :completed="completedCount" :total="totalCount" />
        <Button
          variant="outline"
          size="icon"
          :aria-label="dark ? 'Switch to light mode' : 'Switch to dark mode'"
          @click="toggleTheme"
        >
          <SunIcon v-if="dark" />
          <MoonIcon v-else />
        </Button>
      </div>
    </div>
  </header>
</template>
