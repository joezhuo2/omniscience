import { useEffect, useId, useLayoutEffect, useRef, useState, type CSSProperties, type ReactNode } from 'react'
import { GlassSurface, type GlassSurfaceProps } from '@crystalflux/surface-glass'
import styles from './GlassIsland.module.css'

export type GlassIslandProps = Omit<GlassSurfaceProps<'div'>, 'as' | 'children'> & {
  pill: ReactNode
  expandOn?: 'hover' | 'click'
  open?: boolean
  onOpenChange?: (open: boolean) => void
  children?: ReactNode
}

const interactive = 'button, a, input, select, textarea, label, [role="button"], [tabindex]'

export function GlassIsland({
  pill,
  expandOn = 'click',
  open: controlled,
  onOpenChange,
  radius = 28,
  className,
  style,
  children,
  ...rest
}: GlassIslandProps) {
  const [inner, setInner] = useState(false)
  const open = controlled ?? inner
  const root = useRef<HTMLSpanElement>(null)
  const pillRef = useRef<HTMLButtonElement>(null)
  const [size, setSize] = useState({ w: 0, h: 0 })
  const panelId = useId()

  const set = (next: boolean) => {
    if (controlled === undefined) setInner(next)
    onOpenChange?.(next)
  }

  useLayoutEffect(() => {
    const el = pillRef.current!
    const measure = () => setSize({ w: el.offsetWidth, h: el.offsetHeight })
    measure()
    const observer = new ResizeObserver(measure)
    observer.observe(el)
    return () => observer.disconnect()
  }, [])

  useEffect(() => {
    if (!open) return
    const onKey = (event: KeyboardEvent) => {
      if (event.key !== 'Escape') return
      set(false)
      pillRef.current?.focus()
    }
    const onDown = (event: PointerEvent) => {
      if (!root.current?.contains(event.target as Node)) set(false)
    }
    document.addEventListener('keydown', onKey)
    document.addEventListener('pointerdown', onDown)
    return () => {
      document.removeEventListener('keydown', onKey)
      document.removeEventListener('pointerdown', onDown)
    }
  })

  return (
    <span ref={root} style={{ display: 'contents' }}>
      <GlassSurface
        radius={radius}
        className={[styles.island, className].filter(Boolean).join(' ')}
        data-open={open}
        style={{ '--pw': `${size.w}px`, '--ph': `${size.h}px`, ...style } as CSSProperties}
        onMouseLeave={expandOn === 'hover' ? () => set(false) : undefined}
        onClick={(event) => {
          if (open && expandOn === 'click' && !(event.target as Element).closest(interactive)) set(false)
        }}
        {...rest}
      >
        <span className={styles.outline} aria-hidden="true" />
        <button
          ref={pillRef}
          type="button"
          className={styles.pill}
          aria-expanded={open}
          aria-controls={panelId}
          onClick={() => set(!open)}
          onMouseEnter={expandOn === 'hover' ? () => set(true) : undefined}
        >
          {pill}
        </button>
        <div id={panelId} className={styles.panel} inert={!open}>
          {children}
        </div>
      </GlassSurface>
    </span>
  )
}
