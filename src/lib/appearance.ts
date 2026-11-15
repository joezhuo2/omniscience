export type Overrides = Record<string, string>;

const KEY = 'omniscience.appearance';

export const COLOR_TOKENS: { name: string; label: string }[] = [
  { name: '--color-island-bg', label: 'Background' },
  { name: '--color-island-text', label: 'Text' },
  { name: '--color-island-muted', label: 'Muted text' },
  { name: '--color-island-faint', label: 'Faint' },
  { name: '--color-island-track', label: 'Track' },
  { name: '--color-island-hover', label: 'Card' },
  { name: '--color-island-warn', label: 'Warning' },
  { name: '--color-island-crit', label: 'Critical' },
  { name: '--color-island-ok', label: 'OK' },
  { name: '--color-chart-1', label: 'Chart 1' },
  { name: '--color-chart-2', label: 'Chart 2' },
  { name: '--color-chart-3', label: 'Chart 3' },
  { name: '--color-allow-bg', label: 'Allow button' },
  { name: '--color-allow-text', label: 'Allow text' },
  { name: '--color-deny-bg', label: 'Deny button' },
  { name: '--color-deny-text', label: 'Deny text' },
];

export const FONT_VAR = '--font-sans';
export const SIZE_VAR = 'font-size';

export const FONT_PRESETS: { label: string; value: string | null }[] = [
  { label: 'System', value: null },
  { label: 'Serif', value: "ui-serif, Georgia, 'Times New Roman', serif" },
  { label: 'Rounded', value: "ui-rounded, 'SF Pro Rounded', 'Segoe UI', system-ui, sans-serif" },
  { label: 'Mono', value: null },
];

let defaults: Overrides | null = null;

function readDefaults(): Overrides {
  if (!defaults) {
    const cs = getComputedStyle(document.documentElement);
    defaults = {};
    for (const name of [...COLOR_TOKENS.map((t) => t.name), FONT_VAR, '--font-mono']) {
      defaults[name] = cs.getPropertyValue(name).trim();
    }
  }
  return defaults;
}

export function defaultFor(name: string): string {
  return readDefaults()[name] ?? '';
}

export function fontValue(label: string): string | null {
  if (label === 'Mono') return defaultFor('--font-mono') || null;
  return FONT_PRESETS.find((p) => p.label === label)?.value ?? null;
}

export function fontLabel(value: string | undefined): string {
  if (!value) return 'System';
  return FONT_PRESETS.find((p) => fontValue(p.label) === value)?.label ?? 'System';
}

export function toHex(color: string): string {
  const m = /^#([0-9a-f]{3})$/i.exec(color);
  if (m?.[1]) return '#' + [...m[1]].map((c) => c + c).join('');
  return /^#[0-9a-f]{6}$/i.test(color) ? color.toLowerCase() : '#000000';
}

export function loadAppearance(): Overrides {
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(KEY) ?? '{}');
    if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) return {};
    return Object.fromEntries(
      Object.entries(parsed).filter(([, v]) => typeof v === 'string'),
    ) as Overrides;
  } catch {
    return {};
  }
}

export function saveAppearance(overrides: Overrides): void {
  try {
    if (Object.keys(overrides).length === 0) localStorage.removeItem(KEY);
    else localStorage.setItem(KEY, JSON.stringify(overrides));
  } catch {
    return;
  }
}

export function applyAppearance(overrides: Overrides): void {
  readDefaults();
  const style = document.documentElement.style;
  for (const name of [...COLOR_TOKENS.map((t) => t.name), FONT_VAR, SIZE_VAR]) {
    const value = overrides[name];
    if (value) style.setProperty(name, value);
    else style.removeProperty(name);
  }
}
