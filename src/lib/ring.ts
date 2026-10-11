/** Stroke dash values that draw `percent` (clamped to 0-100) of a circle of radius `r`. */
export function ringDash(percent: number, r: number) {
  const circumference = 2 * Math.PI * r;
  const p = Math.min(100, Math.max(0, percent));
  return { circumference, offset: circumference * (1 - p / 100) };
}
