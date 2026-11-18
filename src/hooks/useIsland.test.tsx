import { emit } from '@tauri-apps/api/event';
import { mockIPC } from '@tauri-apps/api/mocks';
import { act, renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { useSettingsStore, type Settings } from '../settings';
import { HOVER_DODGE_EVENT, HOVER_EVENT, useIsland } from './useIsland';

const withDelays = (open: number, close: number) =>
  useSettingsStore.setState({
    settings: { hover_open_delay_ms: open, hover_close_delay_ms: close } as Settings,
  });

describe('useIsland hover delays', () => {
  const flush = (ms: number) =>
    act(async () => {
      await vi.advanceTimersByTimeAsync(ms);
    });
  const hover = (inside: boolean) =>
    act(async () => {
      await emit(HOVER_EVENT, inside);
    });
  const setup = async () => {
    const hook = renderHook(() => useIsland(false, false, () => {}));
    await flush(0);
    return hook;
  };

  beforeEach(() => {
    vi.useFakeTimers();
    mockIPC(() => null, { shouldMockEvents: true });
  });

  afterEach(() => {
    vi.useRealTimers();
    useSettingsStore.setState({ settings: null });
  });

  it('waits the open delay before expanding and cancels when the cursor leaves', async () => {
    withDelays(300, 0);
    const { result } = await setup();
    await hover(true);
    await flush(299);
    expect(result.current.phase).toBe('collapsed');
    await hover(false);
    await flush(500);
    expect(result.current.phase).toBe('collapsed');
    await hover(true);
    await flush(300);
    expect(result.current.phase).not.toBe('collapsed');
  });

  it('waits the close delay and re-entering cancels it', async () => {
    withDelays(0, 400);
    const { result } = await setup();
    await hover(true);
    await flush(0);
    expect(result.current.phase).toBe('expanded');
    await hover(false);
    await flush(399);
    expect(result.current.phase).toBe('expanded');
    await hover(true);
    await flush(1000);
    expect(result.current.phase).toBe('expanded');
    await hover(false);
    await flush(400);
    expect(result.current.phase).toBe('collapsing');
  });

  it('opens at once with no settings loaded', async () => {
    const { result } = await setup();
    await hover(true);
    await flush(0);
    expect(result.current.phase).toBe('expanded');
  });

  it('closes at once on Escape even with a close delay', async () => {
    withDelays(0, 400);
    const { result } = await setup();
    await hover(true);
    await flush(0);
    act(() => {
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
    });
    expect(result.current.phase).toBe('collapsing');
  });

  it('holds the open for at least a second while dodging browser tabs', async () => {
    withDelays(100, 0);
    const { result } = await setup();
    await act(async () => {
      await emit(HOVER_DODGE_EVENT, true);
    });
    expect(result.current.dodging).toBe(true);
    await hover(true);
    await flush(999);
    expect(result.current.phase).toBe('collapsed');
    await flush(1);
    expect(result.current.phase).not.toBe('collapsed');
  });
});
