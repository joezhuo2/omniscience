import { relaunch } from '@tauri-apps/plugin-process';
import { check, type Update } from '@tauri-apps/plugin-updater';
import { create } from 'zustand';

export const CHECK_UPDATE_EVENT = 'omniscience://check-update';

export type UpdateBanner =
  | { kind: 'available'; version: string }
  | { kind: 'installing'; version: string }
  | { kind: 'up_to_date' }
  | { kind: 'error'; message: string };

/** How long transient banners ("up to date", errors) stay up. */
const TRANSIENT_MS = 4000;

/** The updater calls, injectable for tests. */
export interface UpdaterDeps {
  check: () => Promise<Update | null>;
  relaunch: () => Promise<void>;
}

const realDeps: UpdaterDeps = { check: () => check(), relaunch };

interface UpdaterState {
  banner: UpdateBanner | null;
  /** Checks GitHub for a newer release. `manual` also reports "up to date" and errors. */
  checkNow: (manual: boolean, deps?: UpdaterDeps) => Promise<void>;
  install: (deps?: UpdaterDeps) => Promise<void>;
  dismiss: () => void;
}

let pending: Update | null = null;
let transientTimer: ReturnType<typeof setTimeout> | undefined;

export const useUpdaterStore = create<UpdaterState>((set, get) => {
  const transient = (banner: UpdateBanner) => {
    set({ banner });
    clearTimeout(transientTimer);
    transientTimer = setTimeout(() => {
      if (get().banner === banner) set({ banner: null });
    }, TRANSIENT_MS);
  };
  return {
    banner: null,
    checkNow: async (manual, deps = realDeps) => {
      try {
        const update = await deps.check();
        if (update) {
          pending = update;
          set({ banner: { kind: 'available', version: update.version } });
        } else if (manual) {
          transient({ kind: 'up_to_date' });
        }
      } catch (e) {
        if (manual) transient({ kind: 'error', message: `Update check failed: ${String(e)}` });
      }
    },
    install: async (deps = realDeps) => {
      const update = pending;
      const banner = get().banner;
      if (!update || banner?.kind !== 'available') return;
      set({ banner: { kind: 'installing', version: banner.version } });
      try {
        await update.downloadAndInstall();
        await deps.relaunch();
      } catch (e) {
        transient({ kind: 'error', message: `Update failed: ${String(e)}` });
      }
    },
    dismiss: () => {
      pending = null;
      set({ banner: null });
    },
  };
});
