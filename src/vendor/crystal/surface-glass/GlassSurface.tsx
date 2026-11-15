import type { ComponentPropsWithoutRef, ElementType, ReactNode } from 'react'
import styles from './GlassSurface.module.css'

type OwnProps = {
  radius?: number
  mask?: string
  strength?: number
  bezel?: number
  spotlight?: boolean | string
  children?: ReactNode
}

export type GlassSurfaceProps<T extends ElementType = 'div'> = OwnProps & { as?: T } & Omit<
    ComponentPropsWithoutRef<T>,
    keyof OwnProps | 'as'
  >

// omniscience: plain surface, no refraction maps or backdrop blur (too costly in the software-rendered overlay)
export function GlassSurface<T extends ElementType = 'div'>({
  as,
  radius = 12,
  mask: _mask,
  strength: _strength,
  bezel: _bezel,
  spotlight: _spotlight,
  className,
  style,
  ...rest
}: GlassSurfaceProps<T>) {
  const Tag: ElementType = as ?? 'div'
  return (
    <Tag
      {...rest}
      className={[styles.glass, className].filter(Boolean).join(' ')}
      style={{ borderRadius: radius, ...style }}
    />
  )
}
