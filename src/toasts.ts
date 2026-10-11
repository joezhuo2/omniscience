import { create } from 'zustand';

export interface Toast {
  id: number;
  text: string;
}

const TOAST_MS = 4000;
let nextId = 1;

interface ToastState {
  toasts: Toast[];
  push: (text: string) => void;
  dismiss: (id: number) => void;
}

export const useToastStore = create<ToastState>((set, get) => ({
  toasts: [],
  push: (text) => {
    const id = nextId++;
    set({ toasts: [...get().toasts, { id, text }].slice(-3) });
    setTimeout(() => get().dismiss(id), TOAST_MS);
  },
  dismiss: (id) => set({ toasts: get().toasts.filter((t) => t.id !== id) }),
}));
