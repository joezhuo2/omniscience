import '@testing-library/jest-dom/vitest';
import { clearMocks, mockIPC } from '@tauri-apps/api/mocks';
import { cleanup } from '@testing-library/react';
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
});
