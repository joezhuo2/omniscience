import { create } from 'zustand';

const KEY = 'omniscience.glass';
const OPACITY_KEY = 'omniscience.glassOpacity';
export const DEFAULT_GLASS_OPACITY = 60;

function read(key: string): string | null {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}

function write(key: string, value: string) {
  try {
    localStorage.setItem(key, value);
  } catch {
    /* the choice still applies for this session */
  }
}

function apply(on: boolean, opacity: number) {
  const root = document.documentElement;
  root.dataset.glass = on ? 'on' : 'off';
  root.style.setProperty('--glass-alpha', `${opacity}%`);
}

export const useGlass = create<{
  on: boolean;
  opacity: number;
  setOn: (on: boolean) => void;
  setOpacity: (opacity: number) => void;
}>((set, get) => ({
  on: read(KEY) !== 'off',
  opacity: Number(read(OPACITY_KEY)) || DEFAULT_GLASS_OPACITY,
  setOn: (on) => {
    write(KEY, on ? 'on' : 'off');
    apply(on, get().opacity);
    set({ on });
  },
  setOpacity: (opacity) => {
    write(OPACITY_KEY, String(opacity));
    apply(get().on, opacity);
    set({ opacity });
  },
}));

apply(useGlass.getState().on, useGlass.getState().opacity);
