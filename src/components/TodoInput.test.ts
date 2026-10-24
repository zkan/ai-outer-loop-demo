import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import TodoInput from './TodoInput.vue'

describe('TodoInput', () => {
  it('focuses the text field when focus() is called', async () => {
    const wrapper = mount(TodoInput, { attachTo: document.body })
    ;(wrapper.vm as unknown as { focus: () => void }).focus()
    await wrapper.vm.$nextTick()
    expect(document.activeElement).toBe(wrapper.get('input').element)
    wrapper.unmount()
  })

  it('emits the typed text and clears the field', async () => {
    const wrapper = mount(TodoInput)
    await wrapper.get('input').setValue('Buy milk')
    await wrapper.get('form').trigger('submit')
    expect(wrapper.emitted('add')?.[0]).toEqual(['Buy milk'])
    expect((wrapper.get('input').element as HTMLInputElement).value).toBe('')
  })
})
