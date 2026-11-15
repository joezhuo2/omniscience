import type { ReactNode } from 'react'
import { GlassSurface } from '@crystalflux/surface-glass'
import styles from './GlassSwitch.module.css'

export type GlassSwitchProps = {
  checked: boolean
  onChange: (checked: boolean) => void
  disabled?: boolean
  label?: ReactNode
  className?: string
}

export function GlassSwitch({ checked, onChange, disabled, label, className, ...rest }: GlassSwitchProps) {
  return (
    <label className={[styles.root, className].filter(Boolean).join(' ')} {...rest}>
      <input
        type="checkbox"
        role="switch"
        className={styles.input}
        checked={checked}
        disabled={disabled}
        onChange={(event) => onChange(event.target.checked)}
      />
      <GlassSurface radius={999} className={styles.track}>
        <span className={styles.knob} />
      </GlassSurface>
      {label && <span>{label}</span>}
    </label>
  )
}
