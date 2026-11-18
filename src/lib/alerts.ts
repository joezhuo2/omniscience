const pad = (n: number) => String(n).padStart(2, '0');

export function minutesToTime(min: number): string {
  const m = Math.min(1439, Math.max(0, Math.round(min)));
  return `${pad(Math.floor(m / 60))}:${pad(m % 60)}`;
}

export function timeToMinutes(value: string): number | null {
  const match = /^(\d{1,2}):(\d{2})$/.exec(value);
  if (!match) return null;
  const h = Number(match[1]);
  const m = Number(match[2]);
  return h < 24 && m < 60 ? h * 60 + m : null;
}

export function isSnoozed(untilMs: number, now: number): boolean {
  return untilMs > now;
}

export function formatSnoozeEnd(untilMs: number): string {
  const d = new Date(untilMs);
  return `${pad(d.getHours())}:${pad(d.getMinutes())}`;
}
