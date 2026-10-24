import type { Todo } from './types'

export type Filter = 'all' | 'active' | 'completed'

export function filterTodos(todos: readonly Todo[], filter: Filter): Todo[] {
  if (filter === 'active') return todos.filter((todo) => !todo.completed)
  if (filter === 'completed') return todos.filter((todo) => todo.completed)
  return [...todos]
}

export function emptyState(filter: Filter): {
  title: string
  description: string
  actionLabel?: string
} {
  if (filter === 'active') {
    return {
      title: 'Nothing in progress',
      description: 'Active todos will show up here.',
    }
  }
  if (filter === 'completed') {
    return {
      title: 'Nothing completed yet',
      description: 'Finished todos will show up here.',
    }
  }
  return {
    title: 'No todos yet',
    description: 'Add your first one in the field above.',
    actionLabel: 'Add one',
  }
}

export interface LoadToken {
  generation: number
  mutationsAtStart: number
}

/**
 * Decides whether a list response may replace local state.
 * A response is stale when a mutation started after the request, or when the
 * request began while a mutation was already in flight.
 */
export function createLoadGate() {
  let generation = 0
  let inFlightMutations = 0

  return {
    beginLoad(): LoadToken {
      return { generation, mutationsAtStart: inFlightMutations }
    },
    beginMutation() {
      generation += 1
      inFlightMutations += 1
    },
    endMutation() {
      inFlightMutations = Math.max(0, inFlightMutations - 1)
    },
    canApply(token: LoadToken) {
      return (
        token.generation === generation &&
        token.mutationsAtStart === 0 &&
        inFlightMutations === 0
      )
    },
  }
}
