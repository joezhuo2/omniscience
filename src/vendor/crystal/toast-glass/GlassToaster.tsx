import { useEffect, useMemo, useSyncExternalStore, type CSSProperties } from 'react'
import { createPortal } from 'react-dom'
import { GlassSurface } from '@crystalflux/surface-glass'
import styles from './GlassToaster.module.css'
import { dismiss, getServerSnapshot, getSnapshot, subscribe, type ToastItem } from './store'
import { createTimer } from './timer'

export type GlassToasterProps = {
  position?: 'bottom-right' | 'bottom-center' | 'top-right' | 'top-center'
  max?: number
}

const RADIUS = 22

function Toast({ item }: { item: ToastItem }) {
  const timer = useMemo(() => createTimer(item.duration, () => dismiss(item.id)), [item.id, item.duration])

  useEffect(() => {
    timer.resume()
    return () => timer.pause()
  }, [timer])

  return (
    <GlassSurface
      radius={RADIUS}
      className={styles.toast}
      data-closing={item.closing || undefined}
      style={{ '--duration': `${item.duration}ms` } as CSSProperties}
      onPointerEnter={timer.pause}
      onPointerLeave={timer.resume}
      onFocus={timer.pause}
      onBlur={timer.resume}
    >
      <svg className={styles.fuse} aria-hidden="true">
        <rect pathLength={100} style={{ rx: RADIUS - 1 }} />
      </svg>
      <span className={styles.message}>{item.message}</span>
      {item.action && (
        <button
          type="button"
          className={styles.action}
          onClick={() => {
            item.action!.onClick()
            dismiss(item.id)
          }}
        >
          {item.action.label}
        </button>
      )}
    </GlassSurface>
  )
}

export function GlassToaster({ position = 'bottom-right', max = 3 }: GlassToasterProps) {
  const items = useSyncExternalStore(subscribe, getSnapshot, getServerSnapshot)
  const mounted = useSyncExternalStore(
    () => () => {},
    () => true,
    () => false,
  )
  if (!mounted) return null

  return createPortal(
    <div className={styles.region} data-position={position} aria-live="polite" aria-relevant="additions text">
      {items.slice(0, max).map((item) => (
        <Toast key={item.id} item={item} />
      ))}
    </div>,
    document.body,
  )
}
