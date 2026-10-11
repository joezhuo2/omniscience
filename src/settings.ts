import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { create } from 'zustand';
import type { Settings } from './types/generated/Settings';

export type { Settings };

export const SETTINGS_EVENT = 'omniscience://settings';
export const OPEN_SETTINGS_EVENT = 'omniscience://open-settings';

interface SettingsState {
  settings: Settings | null;
  error: string | null;
  load: () => Promise<void>;
  /** Applies a change; the backend clamps it and the stored value is what it actually applied. */
  update: (patch: Partial<Settings>) => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  settings: null,
  error: null,
  load: async () => {
    try {
      set({ settings: await invoke<Settings>('get_settings'), error: null });
    } catch (e) {
      set({ error: String(e) });
    }
  },
  update: async (patch) => {
    const current = get().settings;
    if (!current) return;
    try {
      const applied = await invoke<Settings>('set_settings', {
        settings: { ...current, ...patch },
      });
      set({ settings: applied, error: null });
    } catch (e) {
      set({ error: String(e) });
    }
  },
}));

/** Keeps the store in sync with changes made from the tray menu. */
export function subscribeSettings(): () => void {
  const unlisten = listen<Settings>(SETTINGS_EVENT, (e) =>
    useSettingsStore.setState({ settings: e.payload }),
  );
  return () => {
    unlisten.then((f) => f()).catch(() => {});
  };
}
