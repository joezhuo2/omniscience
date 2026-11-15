import { useEffect, useRef, type ComponentProps } from 'react'
import styles from './SmoothRange.module.css'
import { glideValue, percentOf, snap } from './tween'

export type SmoothRangeProps = Omit<ComponentProps<'input'>, 'type' | 'onChange'> & {
  onValue: (v: number) => void
  continuous?: boolean
}

const glideMs = 300
const holdMs = 250

const paint = (el: HTMLInputElement) =>
  el.style.setProperty('--sr-pct', `${percentOf(Number(el.value), Number(el.min || 0), Number(el.max || 100))}%`)

export function SmoothRange({
  onValue,
  continuous = false,
  className,
  onPointerDown,
  onPointerUp,
  onPointerCancel,
  ...props
}: SmoothRangeProps) {
  const ref = useRef<HTMLInputElement>(null)
  const down = useRef(0)
  const from = useRef(0)
  const target = useRef<number | null>(null)
  const raf = useRef(0)
  const gliding = useRef(false)

  const shown = useRef(Number(props.value))
  const tween = useRef(0)

  useEffect(() => {
    if (ref.current) paint(ref.current)
  })

  // omniscience: glide the thumb when the value changes from outside a drag
  useEffect(() => {
    const el = ref.current
    const to = Number(props.value)
    const from = shown.current
    shown.current = to
    if (!el || from === to || down.current || gliding.current || reduced()) return
    cancelAnimationFrame(tween.current)
    const step = el.step
    el.step = 'any'
    const start = performance.now()
    const tick = (now: number) => {
      const t = Math.min(1, (now - start) / glideMs)
      el.value = String(from + (to - from) * (1 - (1 - t) ** 3))
      paint(el)
      if (t < 1) tween.current = requestAnimationFrame(tick)
      else el.step = step
    }
    el.value = String(from)
    tween.current = requestAnimationFrame(tick)
  }, [props.value])

  useEffect(() => () => cancelAnimationFrame(tween.current), [])

  useEffect(() => () => cancelAnimationFrame(raf.current), [])

  const held = () => performance.now() - down.current > holdMs
  const reduced = () => window.matchMedia('(prefers-reduced-motion: reduce)').matches

  const glideTo = (el: HTMLInputElement, to: number) => {
    gliding.current = true
    const min = Number(el.min || 0)
    const step = el.step === 'any' ? 0 : Number(el.step || 1)
    const start = performance.now()
    const begin = from.current
    let last = begin
    const tick = (now: number) => {
      const t = now - start
      const v = snap(glideValue(begin, to, t, glideMs), min, step, continuous)
      if (v !== last) {
        last = v
        el.value = String(v)
        paint(el)
        onValue(v)
      }
      if (t < glideMs) raf.current = requestAnimationFrame(tick)
      else gliding.current = false
    }
    raf.current = requestAnimationFrame(tick)
  }

  return (
    <input
      {...props}
      ref={ref}
      type="range"
      className={[styles.range, className].filter(Boolean).join(' ')}
      onPointerDown={(e) => {
        cancelAnimationFrame(raf.current)
        down.current = performance.now()
        from.current = Number(e.currentTarget.value)
        target.current = null
        onPointerDown?.(e)
      }}
      onChange={(e) => {
        const el = e.currentTarget
        const v = Number(el.value)
        if (!down.current || held() || reduced()) {
          target.current = null
          paint(el)
          return onValue(v)
        }
        target.current = v
        el.value = String(from.current)
      }}
      onPointerUp={(e) => {
        const to = target.current
        if (to !== null && to !== from.current) {
          if (held()) onValue(to)
          else glideTo(e.currentTarget, to)
        }
        down.current = 0
        target.current = null
        onPointerUp?.(e)
      }}
      onPointerCancel={(e) => {
        down.current = 0
        target.current = null
        onPointerCancel?.(e)
      }}
    />
  )
}
