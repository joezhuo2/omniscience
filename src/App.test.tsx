import { emit } from '@tauri-apps/api/event';
import { mockIPC } from '@tauri-apps/api/mocks';
import { act, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import App from './App';
import { HOVER_EVENT } from './hooks/useIsland';
import { COLLAPSE_FALLBACK_MS, EXPAND_DELAY_MS } from './lib/island';
import { useUpdaterStore } from './updater';

// jsdom has no ResizeObserver.
class NoopResizeObserver {
  observe() {}
  disconnect() {}
  unobserve() {}
}

describe('App', () => {
  let expandCalls: unknown[];

  beforeEach(() => {
    vi.useFakeTimers();
    vi.stubGlobal('ResizeObserver', NoopResizeObserver);
    useUpdaterStore.setState({ banner: null });
    expandCalls = [];
    mockIPC(
      (cmd, args) => {
        if (cmd === 'set_expanded') expandCalls.push(args);
        if (cmd === 'get_settings') return { auto_check_updates: false };
        return null;
      },
      { shouldMockEvents: true },
    );
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
  const hover = async (inside: boolean) => {
    await act(async () => {
      await emit(HOVER_EVENT, inside);
    });
  };

  it('renders the collapsed pill with no panel', async () => {
    render(<App />);
    await flush(0);
    expect(screen.getByTestId('pill')).toBeInTheDocument();
    expect(island()).toHaveAttribute('data-phase', 'collapsed');
    expect(island()).not.toHaveClass('expanded');
    expect(screen.queryByTestId('dashboard')).toBeNull();
  });

  it('does not open on a brief pass over the pill', async () => {
    render(<App />);
    await flush(0);
    await hover(true);
    await flush(EXPAND_DELAY_MS - 50);
    await hover(false);
    await flush(EXPAND_DELAY_MS * 2);
    expect(island()).toHaveAttribute('data-phase', 'collapsed');
    expect(expandCalls).toHaveLength(0);
  });

  it('opens after hovering, snapping the window first', async () => {
    render(<App />);
    await flush(0);
    await hover(true);
    await flush(EXPAND_DELAY_MS + 10);
    expect(expandCalls[0]).toEqual({ expanded: true });
    expect(island()).toHaveClass('expanded');
    expect(screen.getByTestId('dashboard')).toBeInTheDocument();
  });

  it('starts closing the moment the cursor leaves and shrinks the window last', async () => {
    render(<App />);
    await flush(0);
    await hover(true);
    await flush(EXPAND_DELAY_MS + 10);
    await hover(false);
    expect(island()).toHaveAttribute('data-phase', 'collapsing');
    expect(island()).not.toHaveClass('expanded');
    // Content stays mounted (fading out) until the shape finishes shrinking.
    expect(screen.getByTestId('dashboard')).toBeInTheDocument();
    await flush(COLLAPSE_FALLBACK_MS + 10);
    expect(island()).toHaveAttribute('data-phase', 'collapsed');
    expect(screen.queryByTestId('dashboard')).toBeNull();
    expect(expandCalls.at(-1)).toEqual({ expanded: false });
  });

  it('reverses immediately when the cursor comes back mid-collapse', async () => {
    render(<App />);
    await flush(0);
    await hover(true);
    await flush(EXPAND_DELAY_MS + 10);
    await hover(false);
    await flush(100);
    await hover(true);
    expect(island()).toHaveAttribute('data-phase', 'expanded');
  });

  it('opens on its own for an update prompt', async () => {
    render(<App />);
    await flush(0);
    act(() => useUpdaterStore.setState({ banner: { kind: 'available', version: '1.2.0' } }));
    await flush(10);
    expect(island()).toHaveAttribute('data-phase', 'expanded');
    expect(screen.getByTestId('update-banner')).toHaveTextContent('v1.2.0 available');
    // Moving the cursor across and away does not close it before the user answers.
    await hover(true);
    await hover(false);
    await flush(COLLAPSE_FALLBACK_MS * 2);
    expect(island()).toHaveAttribute('data-phase', 'expanded');
    // Answering "No" closes it.
    act(() => useUpdaterStore.getState().dismiss());
    await flush(COLLAPSE_FALLBACK_MS + 10);
    expect(island()).toHaveAttribute('data-phase', 'collapsed');
  });
});
