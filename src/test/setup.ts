import '@testing-library/jest-dom/vitest';
import { clearMocks, mockIPC } from '@tauri-apps/api/mocks';
import { cleanup } from '@testing-library/react';
import { dismiss, getSnapshot } from '../vendor/crystal/toast-glass/store';
import { afterEach, beforeEach } from 'vitest';

// Components talk to the Rust backend through Tauri IPC; give every test a no-op backend.
// Node-environment tests (e.g. CSS file checks) have no window.
const hasDom = typeof window !== 'undefined';

beforeEach(() => {
  if (hasDom) {
    localStorage.clear();
    mockIPC(() => null);
  }
});

afterEach(() => {
  if (!hasDom) return;
  cleanup();
  clearMocks();
  getSnapshot().forEach((t) => dismiss(t.id));
});

if (hasDom && !window.matchMedia) {
  window.matchMedia = (query: string) =>
    ({
      matches: query.includes('reduce'),
      media: query,
      onchange: null,
      addEventListener: () => {},
      removeEventListener: () => {},
      addListener: () => {},
      removeListener: () => {},
      dispatchEvent: () => false,
    }) as MediaQueryList;
}

if (hasDom && !window.ResizeObserver) {
  window.ResizeObserver = class {
    observe() {}
    unobserve() {}
    disconnect() {}
  };
}
