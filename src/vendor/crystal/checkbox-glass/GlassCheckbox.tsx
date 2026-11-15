import type { ReactNode } from 'react'
import { GlassSurface } from '@crystalflux/surface-glass'
import styles from './GlassCheckbox.module.css'

export type GlassCheckboxProps = {
  checked: boolean
  onChange: (checked: boolean) => void
  disabled?: boolean
  label?: ReactNode
  className?: string
}

export function GlassCheckbox({ checked, onChange, disabled, label, className, ...rest }: GlassCheckboxProps) {
  return (
    <label className={[styles.root, className].filter(Boolean).join(' ')} {...rest}>
      <input
        type="checkbox"
        className={styles.input}
        checked={checked}
        disabled={disabled}
        onChange={(event) => onChange(event.target.checked)}
      />
      <GlassSurface radius={7} className={styles.box}>
        <svg viewBox="0 0 24 24" className={styles.mark} aria-hidden="true">
          <path d="M5 12.5l4.5 4.5L19 7.5" pathLength="1" />
        </svg>
      </GlassSurface>
      {label && <span>{label}</span>}
    </label>
  )
}
