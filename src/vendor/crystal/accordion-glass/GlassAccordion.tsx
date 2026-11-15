import { createContext, useContext, useId, useState, type ReactNode } from 'react'
import { GlassSurface, type GlassSurfaceProps } from '@crystalflux/surface-glass'
import styles from './GlassAccordion.module.css'

type Ctx = { isOpen: (id: string) => boolean; toggle: (id: string) => void }
const Context = createContext<Ctx | null>(null)

export type GlassAccordionProps = Omit<GlassSurfaceProps<'div'>, 'as'> & { multiple?: boolean }

export function GlassAccordion({ multiple, radius = 18, className, ...rest }: GlassAccordionProps) {
  const [open, setOpen] = useState<string[]>([])
  const ctx: Ctx = {
    isOpen: (id) => open.includes(id),
    toggle: (id) =>
      setOpen((prev) => (prev.includes(id) ? prev.filter((x) => x !== id) : multiple ? [...prev, id] : [id])),
  }
  return (
    <Context.Provider value={ctx}>
      <GlassSurface radius={radius} className={[styles.root, className].filter(Boolean).join(' ')} {...rest} />
    </Context.Provider>
  )
}

export type GlassAccordionItemProps = {
  title: ReactNode
  children?: ReactNode
  className?: string
  open?: boolean
  onOpenChange?: (open: boolean) => void
  aside?: ReactNode
}

export function GlassAccordionItem({ title, children, className, open: controlled, onOpenChange, aside }: GlassAccordionItemProps) {
  const ctx = useContext(Context)
  const id = useId()
  if (!ctx) throw new Error('GlassAccordionItem must be used inside GlassAccordion')
  const open = controlled ?? ctx.isOpen(id)
  return (
    <div className={[styles.item, className].filter(Boolean).join(' ')} data-open={open}>
      <h3 className={styles.heading}>
        <button
          type="button"
          id={`${id}-t`}
          className={styles.trigger}
          aria-expanded={open}
          aria-controls={`${id}-p`}
          onClick={() => (onOpenChange ? onOpenChange(!open) : ctx.toggle(id))}
        >
          <span>{title}</span>
          <svg viewBox="0 0 24 24" className={styles.chevron} aria-hidden="true">
            <path d="M6 9l6 6 6-6" />
          </svg>
        </button>
        {aside}
      </h3>
      <div id={`${id}-p`} role="region" aria-labelledby={`${id}-t`} className={styles.panel} inert={!open} aria-hidden={!open}>
        <div className={styles.inner}>
          <div className={styles.content}>{children}</div>
        </div>
      </div>
    </div>
  )
}
