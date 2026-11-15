import { GlassSurface, type GlassSurfaceProps } from '@crystalflux/surface-glass'
import styles from './GlassSkeleton.module.css'

export type GlassSkeletonProps = Omit<GlassSurfaceProps<'div'>, 'as'> & {
  width?: number | string
  height?: number | string
  circle?: boolean
}

export function GlassSkeleton({ width = '100%', height = 16, circle, radius = 8, className, style, ...rest }: GlassSkeletonProps) {
  return (
    <GlassSurface
      radius={circle ? 999 : radius}
      aria-hidden="true"
      className={[styles.skeleton, className].filter(Boolean).join(' ')}
      style={{ width, height: circle ? width : height, ...style }}
      {...rest}
    />
  )
}
