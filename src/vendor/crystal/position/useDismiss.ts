import { useEffect, useRef, type RefObject } from 'react'

const stack: symbol[] = []

export function useDismiss(refs: RefObject<Element | null>[], open: boolean, onClose: () => void) {
  const close = useRef(onClose)
  close.current = onClose
  useEffect(() => {
    if (!open) return
    const layer = Symbol()
    stack.push(layer)
    const top = () => stack[stack.length - 1] === layer

    const onPointerDown = (event: PointerEvent) => {
      if (!top()) return
      const target = event.target as Node
      if (refs.some((ref) => ref.current?.contains(target))) return
      close.current()
    }
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key !== 'Escape' || !top()) return
      event.stopPropagation()
      close.current()
    }

    document.addEventListener('pointerdown', onPointerDown, true)
    document.addEventListener('keydown', onKeyDown, true)
    return () => {
      stack.splice(stack.indexOf(layer), 1)
      document.removeEventListener('pointerdown', onPointerDown, true)
      document.removeEventListener('keydown', onKeyDown, true)
    }
  }, [open])
}
