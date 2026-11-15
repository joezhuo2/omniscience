import {
  cloneElement,
  isValidElement,
  useCallback,
  useEffect,
  useId,
  useLayoutEffect,
  useRef,
  useState,
  type ReactElement,
  type ReactNode,
} from 'react'
import { createPortal } from 'react-dom'
import { GlassSurface } from '@crystalflux/surface-glass'
import { place, type Side } from '@crystalflux/position'
import styles from './GlassTooltip.module.css'

const WARM_MS = 400
let warmUntil = 0

export type GlassTooltipProps = {
  content: ReactNode
  side?: Side
  delay?: number
  children: ReactNode
}

export function GlassTooltip({ content, side = 'top', delay = 500, children }: GlassTooltipProps) {
  const id = useId()
  const root = useRef<HTMLSpanElement>(null)
  const bubble = useRef<HTMLDivElement>(null)
  const timer = useRef<number>(undefined)
  const [open, setOpen] = useState(false)
  const [pos, setPos] = useState<{ x: number; y: number; side: Side }>()

  const hide = useCallback(() => {
    window.clearTimeout(timer.current)
    setOpen((was) => {
      if (was) warmUntil = Date.now() + WARM_MS
      return false
    })
  }, [])

  const show = () => {
    window.clearTimeout(timer.current)
    const wait = Date.now() < warmUntil ? 0 : delay
    if (wait === 0) setOpen(true)
    else timer.current = window.setTimeout(() => setOpen(true), wait)
  }

  useLayoutEffect(() => {
    if (!open || !root.current || !bubble.current) return setPos(undefined)
    const { offsetWidth: width, offsetHeight: height } = bubble.current
    setPos(place((root.current.firstElementChild ?? root.current).getBoundingClientRect(), { width, height }, { side, align: 'center' }))
  }, [open, side, content])

  useEffect(() => {
    if (!open) return
    const onKey = (event: KeyboardEvent) => event.key === 'Escape' && hide()
    document.addEventListener('keydown', onKey)
    window.addEventListener('scroll', hide, true)
    return () => {
      document.removeEventListener('keydown', onKey)
      window.removeEventListener('scroll', hide, true)
    }
  }, [open, hide])

  useEffect(() => () => window.clearTimeout(timer.current), [])

  const child = isValidElement(children)
    ? cloneElement(children as ReactElement<{ 'aria-describedby'?: string }>, {
        'aria-describedby': open ? id : undefined,
      })
    : children

  return (
    <>
      <span
        ref={root}
        className={styles.root}
        onPointerEnter={show}
        onPointerLeave={hide}
        onPointerDown={hide}
        onFocus={(event) => event.target.matches(':focus-visible') && show()}
        onBlur={hide}
      >
        {child}
      </span>
      {open &&
        createPortal(
          <div
            ref={bubble}
            className={styles.positioner}
            style={{ left: pos?.x ?? 0, top: pos?.y ?? 0, visibility: pos ? 'visible' : 'hidden' }}
          >
            <GlassSurface
              id={id}
              role="tooltip"
              radius={10}
              strength={20}
              bezel={10}
              className={styles.bubble}
              data-side={pos?.side ?? side}
            >
              {content}
            </GlassSurface>
          </div>,
          document.body,
        )}
    </>
  )
}
