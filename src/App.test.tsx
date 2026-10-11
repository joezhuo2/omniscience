import { mockIPC } from '@tauri-apps/api/mocks';
import { act, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import App from './App';
import { COLLAPSE_DELAY_MS, COLLAPSE_FALLBACK_MS, EXPAND_DELAY_MS } from './lib/island';

// jsdom has no ResizeObserver.
class NoopResizeObserver {
  observe() {}
  disconnect() {}
  unobserve() {}
}

describe('App', () => {
  let calls: { cmd: string; args: unknown }[];

  beforeEach(() => {
    vi.useFakeTimers();
    vi.stubGlobal('ResizeObserver', NoopResizeObserver);
    calls = [];
    mockIPC((cmd, args) => {
      if (cmd === 'set_expanded') calls.push({ cmd, args });
      return null;
    });
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });

  const island = () => screen.getByTestId('island');
  const flush = async (ms: number) => {
    await act(async () => {
      await vi.advanceTimersByTimeAsync(ms);
    });
  };

  it('renders the collapsed pill', () => {
    render(<App />);
    expect(screen.getByTestId('pill')).toBeInTheDocument();
    expect(island()).toHaveAttribute('data-phase', 'collapsed');
  });

  it('does not expand on a brief pass over the pill', async () => {
    render(<App />);
    fireEvent.pointerEnter(island());
    await flush(EXPAND_DELAY_MS - 50);
    fireEvent.pointerLeave(island());
    await flush(EXPAND_DELAY_MS);
    expect(island()).toHaveAttribute('data-phase', 'collapsed');
    expect(calls).toHaveLength(0);
  });

  it('expands after hovering, snapping the window first', async () => {
    render(<App />);
    fireEvent.pointerEnter(island());
    await flush(EXPAND_DELAY_MS + 10);
    expect(calls[0]).toEqual({ cmd: 'set_expanded', args: { expanded: true } });
    expect(island()).toHaveAttribute('data-phase', 'expanded');
    expect(screen.getByTestId('dashboard')).toBeInTheDocument();
  });

  it('collapses after the cursor leaves and shrinks the window last', async () => {
    render(<App />);
    fireEvent.pointerEnter(island());
    await flush(EXPAND_DELAY_MS + 10);
    fireEvent.pointerLeave(island());
    await flush(COLLAPSE_DELAY_MS + 10);
    expect(island()).toHaveAttribute('data-phase', 'collapsing');
    expect(screen.queryByTestId('dashboard')).toBeNull();
    await flush(COLLAPSE_FALLBACK_MS + 10);
    expect(island()).toHaveAttribute('data-phase', 'collapsed');
    expect(calls.at(-1)).toEqual({ cmd: 'set_expanded', args: { expanded: false } });
  });

  it('stays open when the cursor comes back within the grace period', async () => {
    render(<App />);
    fireEvent.pointerEnter(island());
    await flush(EXPAND_DELAY_MS + 10);
    fireEvent.pointerLeave(island());
    await flush(COLLAPSE_DELAY_MS - 100);
    fireEvent.pointerEnter(island());
    await flush(COLLAPSE_DELAY_MS + 10);
    expect(island()).toHaveAttribute('data-phase', 'expanded');
  });
});
