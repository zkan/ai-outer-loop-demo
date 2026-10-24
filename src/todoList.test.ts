import { describe, expect, it } from 'vitest'
import type { Todo } from './types'
import { createLoadGate, emptyState, filterTodos } from './todoList'

function todo(partial: Partial<Todo> & Pick<Todo, 'id' | 'text' | 'completed'>): Todo {
  return {
    createdAt: '2026-10-24T00:00:00.000Z',
    updatedAt: '2026-10-24T00:00:00.000Z',
    ...partial,
  }
}

const items = [
  todo({ id: '1', text: 'Buy milk', completed: false }),
  todo({ id: '2', text: 'Walk', completed: true }),
]

describe('filterTodos', () => {
  it('returns every todo, only active, or only completed', () => {
    expect(filterTodos(items, 'all').map((item) => item.id)).toEqual(['1', '2'])
    expect(filterTodos(items, 'active').map((item) => item.id)).toEqual(['1'])
    expect(filterTodos(items, 'completed').map((item) => item.id)).toEqual(['2'])
  })
})

describe('emptyState', () => {
  it('points at the field above when the whole list is empty', () => {
    const copy = emptyState('all')
    expect(copy.description).toContain('above')
    expect(copy.description).not.toContain('below')
    expect(copy.actionLabel).toBe('Add one')
  })

  it('uses filter-specific copy without an add action', () => {
    expect(emptyState('active').actionLabel).toBeUndefined()
    expect(emptyState('completed').title).toMatch(/completed/i)
  })
})

describe('createLoadGate', () => {
  it('drops a list response that arrives after an add has started', () => {
    const gate = createLoadGate()
    const token = gate.beginLoad()
    gate.beginMutation()
    expect(gate.canApply(token)).toBe(false)
    gate.endMutation()
    expect(gate.canApply(token)).toBe(false)
  })

  it('drops a list response that started while an add was in flight', () => {
    const gate = createLoadGate()
    gate.beginMutation()
    const token = gate.beginLoad()
    gate.endMutation()
    expect(gate.canApply(token)).toBe(false)
  })

  it('applies a list response when nothing else is in flight', () => {
    const gate = createLoadGate()
    const token = gate.beginLoad()
    expect(gate.canApply(token)).toBe(true)
  })
})
