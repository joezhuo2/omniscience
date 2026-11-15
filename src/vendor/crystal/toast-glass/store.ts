import type { ReactNode } from 'react'

export type ToastOptions = {
  action?: { label: string; onClick: () => void }
  duration?: number
}

export type ToastItem = {
  id: number
  message: ReactNode
  action?: ToastOptions['action']
  duration: number
  closing: boolean
}

const EXIT_MS = 160
const none: ToastItem[] = []
let items = none
let nextId = 0
const listeners = new Set<() => void>()

const set = (next: ToastItem[]) => {
  items = next
  listeners.forEach((listener) => listener())
}

export const subscribe = (listener: () => void) => {
  listeners.add(listener)
  return () => {
    listeners.delete(listener)
  }
}

export const getSnapshot = () => items
export const getServerSnapshot = () => none

export function toast(message: ReactNode, { action, duration = 4000 }: ToastOptions = {}) {
  const id = ++nextId
  set([...items, { id, message, action, duration, closing: false }])
  return id
}

export function dismiss(id: number) {
  const item = items.find((entry) => entry.id === id)
  if (!item || item.closing) return
  set(items.map((entry) => (entry === item ? { ...entry, closing: true } : entry)))
  const reduce = typeof matchMedia === 'function' && matchMedia('(prefers-reduced-motion: reduce)').matches
  setTimeout(() => set(items.filter((entry) => entry.id !== id)), reduce ? 0 : EXIT_MS)
}
