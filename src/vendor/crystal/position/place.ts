export type Side = 'top' | 'bottom' | 'left' | 'right'
export type Align = 'start' | 'center' | 'end'

export type PlaceOptions = {
  side?: Side
  align?: Align
  gap?: number
  margin?: number
  viewport?: { width: number; height: number }
}

const opposite: Record<Side, Side> = { top: 'bottom', bottom: 'top', left: 'right', right: 'left' }

const clamp = (value: number, min: number, max: number) => Math.max(min, Math.min(value, max))

export function place(
  anchor: DOMRect,
  popup: { width: number; height: number },
  { side = 'bottom', align = 'start', gap = 8, margin = 8, viewport }: PlaceOptions = {},
) {
  const vw = viewport?.width ?? window.innerWidth
  const vh = viewport?.height ?? window.innerHeight
  const vertical = (s: Side) => s === 'top' || s === 'bottom'

  const room = (s: Side) => {
    if (s === 'top') return anchor.top - gap - margin - popup.height
    if (s === 'bottom') return vh - anchor.bottom - gap - margin - popup.height
    if (s === 'left') return anchor.left - gap - margin - popup.width
    return vw - anchor.right - gap - margin - popup.width
  }

  const used = room(side) < 0 && room(opposite[side]) > room(side) ? opposite[side] : side

  const cross = (start: number, size: number, own: number) =>
    align === 'start' ? start : align === 'end' ? start + size - own : start + (size - own) / 2

  let x: number
  let y: number
  if (vertical(used)) {
    x = cross(anchor.left, anchor.width, popup.width)
    y = used === 'top' ? anchor.top - gap - popup.height : anchor.bottom + gap
  } else {
    y = cross(anchor.top, anchor.height, popup.height)
    x = used === 'left' ? anchor.left - gap - popup.width : anchor.right + gap
  }

  return {
    x: clamp(x, margin, Math.max(margin, vw - margin - popup.width)),
    y: clamp(y, margin, Math.max(margin, vh - margin - popup.height)),
    side: used,
  }
}
