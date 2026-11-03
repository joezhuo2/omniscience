import { create } from 'zustand';
import { useToastStore } from './toasts';

export interface UndoEntry {
  id: number;
  label: string;
  revert: () => Promise<void>;
}

const MAX_ENTRIES = 10;
let nextId = 1;

interface UndoState {
  entries: UndoEntry[];
  reverting: boolean;
  revision: number;
  record: (label: string, revert: () => Promise<void>) => void;
  undo: () => Promise<void>;
}

export const useUndoStore = create<UndoState>((set, get) => ({
  entries: [],
  reverting: false,
  revision: 0,
  record: (label, revert) =>
    set({ entries: [...get().entries, { id: nextId++, label, revert }].slice(-MAX_ENTRIES) }),
  undo: async () => {
    const { entries, reverting } = get();
    const entry = entries[entries.length - 1];
    if (!entry || reverting) return;
    set({ entries: entries.slice(0, -1), reverting: true });
    try {
      await entry.revert();
      useToastStore.getState().push(`Undid ${entry.label}`);
      set({ revision: get().revision + 1 });
    } catch (e) {
      useToastStore.getState().push(`Could not undo ${entry.label}: ${String(e)}`);
    } finally {
      set({ reverting: false });
    }
  },
}));
