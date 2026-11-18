import type { Settings } from '../settings';
import type { Overrides } from './appearance';

export interface GlassOptions {
  on: boolean;
  opacity: number;
}

export interface SettingsDocument {
  app: 'omniscience';
  version: 1;
  settings: Settings;
  appearance: Overrides;
  glass: GlassOptions;
}

export function buildSettingsDocument(
  settings: Settings,
  appearance: Overrides,
  glass: GlassOptions,
): SettingsDocument {
  return { app: 'omniscience', version: 1, settings, appearance, glass };
}

const isObject = (v: unknown): v is Record<string, unknown> =>
  typeof v === 'object' && v !== null && !Array.isArray(v);

const kind = (v: unknown) => (Array.isArray(v) ? 'array' : v === null ? 'null' : typeof v);

export function parseSettingsDocument(text: string, template: Settings): SettingsDocument {
  let doc: unknown;
  try {
    doc = JSON.parse(text);
  } catch {
    throw new Error('Not a valid JSON file.');
  }
  if (!isObject(doc) || doc.app !== 'omniscience' || doc.version !== 1) {
    throw new Error('Not an Omniscience settings file.');
  }
  const { settings, appearance, glass } = doc;
  if (!isObject(settings)) throw new Error('The settings are missing.');
  const picked: Record<string, unknown> = {};
  for (const [key, sample] of Object.entries(template)) {
    if (kind(settings[key]) !== kind(sample)) throw new Error(`The setting "${key}" is invalid.`);
    picked[key] = settings[key];
  }
  if (!isObject(appearance) || !Object.values(appearance).every((v) => typeof v === 'string')) {
    throw new Error('The appearance is invalid.');
  }
  if (
    !isObject(glass) ||
    typeof glass.on !== 'boolean' ||
    typeof glass.opacity !== 'number' ||
    !Number.isFinite(glass.opacity)
  ) {
    throw new Error('The glass options are invalid.');
  }
  return {
    app: 'omniscience',
    version: 1,
    settings: picked as unknown as Settings,
    appearance: appearance as Overrides,
    glass: { on: glass.on, opacity: Math.min(95, Math.max(20, glass.opacity)) },
  };
}

export function importedSettings(doc: SettingsDocument, current: Settings): Settings {
  return {
    ...doc.settings,
    launch_at_startup: current.launch_at_startup,
    alerts: { ...doc.settings.alerts, snooze_until_ms: current.alerts.snooze_until_ms },
  };
}
