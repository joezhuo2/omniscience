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

/** "15/31 GB": used over total, rounded to whole GB so it fits under a ring. */
export function formatUsedTotal(
  used: number | null | undefined,
  total: number | null | undefined,
): string {
  if (used == null || total == null || total <= 0) return '--';
  const gb = 1024 ** 3;
  return `${Math.round(used / gb)}/${Math.round(total / gb)} GB`;
}

/** Bytes per second rounded to a whole number in the best unit: "1 MB/s", "80 KB/s". */
export function formatRateRounded(bps: number | null | undefined): string {
  if (bps == null || !Number.isFinite(bps) || bps < 0) return '--';
  let v = bps;
  let i = 0;
  while (Math.round(v) >= 1024 && i < UNITS.length - 1) {
    v /= 1024;
    i++;
  }
  return `${Math.round(v)} ${UNITS[i]}/s`;
}

/** MHz -> "3.2 GHz". */
export function formatGhz(mhz: number | null | undefined): string {
  if (mhz == null || !Number.isFinite(mhz) || mhz <= 0) return '--';
  return `${(mhz / 1000).toFixed(1)} GHz`;
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

export function formatRpm(rpm: number | null | undefined): string {
  if (rpm == null || !Number.isFinite(rpm) || rpm <= 0) return '--';
  return `${Math.round(rpm)} rpm`;
}

/** Round trip in ms; sub-millisecond replies (ICMP reports whole ms) read "<1 ms". */
export function formatLatency(ms: number | null | undefined): string {
  if (ms == null || !Number.isFinite(ms)) return '--';
  return ms < 1 ? '<1 ms' : `${Math.round(ms)} ms`;
}

/** Seconds -> "2h 05m", or "45m" under an hour. */
export function formatDuration(secs: number | null | undefined): string {
  if (secs == null || !Number.isFinite(secs) || secs < 0) return '--';
  const minutes = Math.round(secs / 60);
  const h = Math.floor(minutes / 60);
  const m = minutes % 60;
  return h > 0 ? `${h}h ${String(m).padStart(2, '0')}m` : `${m}m`;
}
