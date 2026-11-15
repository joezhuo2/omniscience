import { useId, useLayoutEffect, useRef, useState, type KeyboardEvent } from 'react'
import { createPortal } from 'react-dom'
import { GlassSurface } from '@crystalflux/surface-glass'
import { place, useDismiss } from '@crystalflux/position'
import styles from './GlassSelect.module.css'

export type GlassSelectOption = string | { value: string; label: string }

export type GlassSelectProps = {
  options: GlassSelectOption[]
  value?: string
  onChange?: (value: string) => void
  placeholder?: string
  disabled?: boolean
  'aria-label'?: string
  className?: string
}

const normalize = (option: GlassSelectOption) =>
  typeof option === 'string' ? { value: option, label: option } : option

export function GlassSelect({
  options,
  value,
  onChange,
  placeholder = 'Select',
  disabled,
  className,
  'aria-label': ariaLabel,
}: GlassSelectProps) {
  const id = useId()
  const items = options.map(normalize)
  const selected = items.findIndex((item) => item.value === value)
  const trigger = useRef<HTMLSpanElement>(null)
  const list = useRef<HTMLDivElement>(null)
  const pill = useRef<HTMLSpanElement>(null)
  const [open, setOpen] = useState(false)
  const [active, setActive] = useState(0)
  const [pos, setPos] = useState<{ x: number; y: number; width: number }>()

  const openList = () => {
    setActive(Math.max(selected, 0))
    setOpen(true)
  }
  const close = () => setOpen(false)
  const choose = (index: number) => {
    onChange?.(items[index]!.value)
    close()
  }

  useDismiss([trigger, list], open, close)

  useLayoutEffect(() => {
    if (!open || !trigger.current || !list.current) return setPos(undefined)
    const anchor = trigger.current.getBoundingClientRect()
    const { offsetWidth: width, offsetHeight: height } = list.current
    const { x, y } = place(anchor, { width, height }, { side: 'bottom', align: 'start', gap: 6 })
    setPos({ x, y, width: anchor.width })
  }, [open, options.length])

  useLayoutEffect(() => {
    const el = list.current?.querySelectorAll<HTMLElement>('[role="option"]')[active]
    const glide = pill.current
    if (!open || !el || !glide) return
    el.scrollIntoView({ block: 'nearest' })
    glide.style.transform = `translateY(${el.offsetTop}px)`
    glide.style.height = `${el.offsetHeight}px`
    if (!glide.hasAttribute('data-glide')) requestAnimationFrame(() => glide.setAttribute('data-glide', ''))
  }, [open, active, pos])

  const onKeyDown = (event: KeyboardEvent) => {
    const last = items.length - 1
    if (!open) {
      if (['ArrowDown', 'ArrowUp', 'Enter', ' '].includes(event.key)) {
        event.preventDefault()
        openList()
      }
      return
    }
    const go = (next: number) => {
      event.preventDefault()
      setActive(next)
    }
    if (event.key === 'ArrowDown') go(Math.min(active + 1, last))
    else if (event.key === 'ArrowUp') go(Math.max(active - 1, 0))
    else if (event.key === 'Home') go(0)
    else if (event.key === 'End') go(last)
    else if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault()
      choose(active)
    } else if (event.key === 'Tab') close()
  }

  return (
    <>
      <span ref={trigger} className={styles.root}>
      <GlassSurface
        as="button"
        type="button"
        radius={12}
        role="combobox"
        aria-haspopup="listbox"
        aria-expanded={open}
        aria-controls={open ? `${id}-list` : undefined}
        aria-activedescendant={open ? `${id}-${active}` : undefined}
        aria-label={ariaLabel}
        disabled={disabled}
        className={[styles.trigger, className].filter(Boolean).join(' ')}
        onClick={() => (open ? close() : openList())}
        onKeyDown={onKeyDown}
      >
        <span className={selected < 0 ? styles.placeholder : undefined}>
          {selected < 0 ? placeholder : items[selected]!.label}
        </span>
        <svg className={styles.chevron} viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
          <path d="M6 9l6 6 6-6" />
        </svg>
      </GlassSurface>
      </span>
      {open &&
        createPortal(
          <div
            ref={list}
            className={styles.positioner}
            style={{ left: pos?.x ?? 0, top: pos?.y ?? 0, minWidth: pos?.width, visibility: pos ? 'visible' : 'hidden' }}
          >
            <GlassSurface
              id={`${id}-list`}
              role="listbox"
              radius={14}
              strength={24}
              bezel={14}
              className={styles.list}
              onPointerDown={(event) => event.preventDefault()}
            >
              <span ref={pill} className={styles.pill} aria-hidden="true" />
              {items.map((item, index) => (
                <div
                  key={item.value}
                  id={`${id}-${index}`}
                  role="option"
                  aria-selected={index === selected}
                  data-active={index === active || undefined}
                  className={styles.option}
                  onPointerMove={() => setActive(index)}
                  onClick={() => choose(index)}
                >
                  {item.label}
                </div>
              ))}
            </GlassSurface>
          </div>,
          document.body,
        )}
    </>
  )
}
