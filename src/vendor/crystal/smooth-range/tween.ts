export const easeOutQuart = (t: number) => 1 - Math.pow(1 - Math.min(Math.max(t, 0), 1), 4)

export const glideValue = (from: number, to: number, elapsed: number, duration: number) =>
  duration <= 0 ? to : from + (to - from) * easeOutQuart(elapsed / duration)

export const snap = (x: number, min: number, step: number, continuous: boolean) =>
  step > 0 && !continuous ? min + Math.round((x - min) / step) * step : x

export const percentOf = (value: number, min: number, max: number) =>
  max > min ? Math.min(Math.max(((value - min) / (max - min)) * 100, 0), 100) : 0
