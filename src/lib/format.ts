const UNITS = ['B', 'KB', 'MB', 'GB', 'TB'];

/** 1536 -> "1.5 KB". No decimals at or above 100 for compactness. */
export function formatBytes(bytes: number | null | undefined): string {
  if (bytes == null || !Number.isFinite(bytes) || bytes < 0) return '--';
  let v = bytes;
  let i = 0;
  while (v >= 1024 && i < UNITS.length - 1) {
    v /= 1024;
    i++;
  }
  const digits = i === 0 || v >= 100 ? 0 : 1;
  return `${v.toFixed(digits)} ${UNITS[i]}`;
}

/** Bytes per second -> "1.2 MB/s". */
export function formatRate(bps: number | null | undefined): string {
  const b = formatBytes(bps);
  return b === '--' ? b : `${b}/s`;
}

/** "12.4 / 31.1 GB": used over total, both in GB. */
export function formatUsedTotal(
  used: number | null | undefined,
  total: number | null | undefined,
): string {
  if (used == null || total == null || total <= 0) return '--';
  const gb = 1024 ** 3;
  return `${(used / gb).toFixed(1)} / ${(total / gb).toFixed(1)} GB`;
}

export function formatPercent(p: number | null | undefined): string {
  if (p == null || !Number.isFinite(p)) return '--';
  return `${Math.round(p)}%`;
}

export function formatWatts(w: number | null | undefined): string {
  if (w == null || !Number.isFinite(w)) return '-- W';
  return `${w < 10 ? w.toFixed(1) : Math.round(w)} W`;
}

export function formatTemp(c: number | null | undefined): string {
  if (c == null || !Number.isFinite(c)) return '--';
  return `${Math.round(c)}°C`;
}
