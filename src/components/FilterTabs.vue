<script setup lang="ts">
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import type { Filter } from '@/todoList'

defineProps<{
  counts: { all: number; active: number; completed: number }
}>()

const filter = defineModel<Filter>({ required: true })
</script>

<template>
  <Tabs v-model="filter" class="w-full">
    <TabsList class="grid w-full grid-cols-3" aria-label="Filter todos">
      <TabsTrigger value="all">
        All
        <span class="text-muted-foreground tabular-nums">{{ counts.all }}</span>
      </TabsTrigger>
      <TabsTrigger value="active">
        Active
        <span class="text-muted-foreground tabular-nums">{{ counts.active }}</span>
      </TabsTrigger>
      <TabsTrigger value="completed">
        Completed
        <span class="text-muted-foreground tabular-nums">{{ counts.completed }}</span>
      </TabsTrigger>
    </TabsList>
  </Tabs>
</template>
