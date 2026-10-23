import type { Todo } from './types'

const BASE = '/api/todos'

/**
 * Shared fetch helper. On any non-2xx response the server replies
 * `{ "error": string }`; we surface that uniformly as a thrown Error.
 */
async function request<T>(path: string, init: RequestInit = {}) {
  const { body, ...fetchInit } = init
  const res = await fetch(`${BASE}${path}`, {
    ...fetchInit,
    headers: { 'Content-Type': 'application/json', ...(fetchInit.headers || {}) },
    body: body,
  })
  let json: unknown = null
  try {
    json = await res.json()
  } catch {
    json = null
  }
  if (!res.ok) {
    const error = (json as { error?: string } | null)?.error ?? `Request failed (${res.status})`
    throw new Error(error)
  }
  return json as T
}

export async function listTodos(): Promise<Todo[]> {
  return request('')
}

export async function createTodo(input: { text: string }): Promise<Todo> {
  return request('', { method: 'POST', body: JSON.stringify(input) })
}

export async function updateTodo(id: string, input: { completed?: boolean; text?: string }): Promise<Todo> {
  return request(`/${id}`, { method: 'PATCH', body: JSON.stringify(input) })
}

export async function deleteTodo(id: string): Promise<void> {
  const res = await fetch(`${BASE}/${id}`, { method: 'DELETE' })
  if (!res.ok) {
    const json = (await res.json().catch(() => null)) as { error?: string } | null
    throw new Error(json?.error ?? `Request failed (${res.status})`)
  }
}
