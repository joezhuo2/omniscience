import { mockIPC } from '@tauri-apps/api/mocks';
import { act, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { useTelemetryStore } from '../../store';
import { sampleDetail, sampleSnapshot } from '../../test/fixtures';
import type { CustomCard } from '../../types/generated/CustomCard';
import { CustomCards } from './CustomCards';

const card = (over: Partial<CustomCard> = {}): CustomCard => ({
  id: 'lab',
  title: 'Lab',
  rows: [],
  refresh_s: null,
  error: null,
  ...over,
});

describe('custom cards', () => {
  let cards: CustomCard[];
  let data: () => unknown;
  let calls: string[];

  beforeEach(() => {
    vi.useFakeTimers();
    cards = [];
    calls = [];
    data = () => null;
    useTelemetryStore.setState({ snapshot: null, lastTickAt: null });
    localStorage.clear();
    mockIPC((cmd, args) => {
      calls.push(cmd === 'get_custom_card_data' ? `${cmd}:${(args as { id: string }).id}` : cmd);
      if (cmd === 'get_custom_cards') return cards;
      if (cmd === 'get_custom_card_data') return data();
      return null;
    });
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  const flush = () => act(() => vi.advanceTimersByTimeAsync(0));

  it('fills placeholders from the latest telemetry', async () => {
    act(() =>
      useTelemetryStore.getState().setSnapshot({ ...sampleSnapshot, detail: sampleDetail }),
    );
    cards = [
      card({
        rows: [
          { label: 'CPU', value: '{core.cpu.percent|1}%' },
          { label: 'LAN', value: '{detail.net_adapters.0.ipv4}' },
          { label: 'Gone', value: '{core.nope}' },
        ],
      }),
    ];
    render(<CustomCards />);
    await flush();
    const article = screen.getByRole('article', { name: 'Lab' });
    expect(article).toHaveTextContent('CPU23.0%');
    expect(article).toHaveTextContent('LAN192.168.1.20');
    expect(article).toHaveTextContent('Gone--');
  });

  it('shows the error of a broken card instead of rows', async () => {
    cards = [card({ rows: [{ label: 'A', value: 'b' }], error: 'bad JSON in lab.json' })];
    render(<CustomCards />);
    await flush();
    expect(screen.getByText('bad JSON in lab.json')).toBeInTheDocument();
    expect(screen.queryByText('A')).not.toBeInTheDocument();
  });

  it('fetches the source on mount and on its interval', async () => {
    let n = 0;
    data = () => ({ count: ++n });
    cards = [card({ refresh_s: 5, rows: [{ label: 'Count', value: '{data.count}' }] })];
    render(<CustomCards />);
    await flush();
    expect(screen.getByRole('article', { name: 'Lab' })).toHaveTextContent('Count1');
    await act(() => vi.advanceTimersByTimeAsync(5000));
    expect(screen.getByRole('article', { name: 'Lab' })).toHaveTextContent('Count2');
    expect(calls.filter((c) => c === 'get_custom_card_data:lab')).toHaveLength(2);
  });

  it('does not fetch a card without refresh_s and shows -- for its data', async () => {
    cards = [card({ rows: [{ label: 'Count', value: '{data.count}' }] })];
    render(<CustomCards />);
    await flush();
    await act(() => vi.advanceTimersByTimeAsync(60_000));
    expect(calls.some((c) => c.startsWith('get_custom_card_data'))).toBe(false);
    expect(screen.getByRole('article', { name: 'Lab' })).toHaveTextContent('Count--');
  });

  it('keeps the last data and shows a muted error when a fetch fails', async () => {
    let fail = false;
    data = () => {
      if (fail) throw 'timed out';
      return { count: 7 };
    };
    cards = [card({ refresh_s: 5, rows: [{ label: 'Count', value: '{data.count}' }] })];
    render(<CustomCards />);
    await flush();
    fail = true;
    await act(() => vi.advanceTimersByTimeAsync(5000));
    const article = screen.getByRole('article', { name: 'Lab' });
    expect(article).toHaveTextContent('Count7');
    expect(screen.getByRole('status')).toHaveTextContent('timed out');
    fail = false;
    await act(() => vi.advanceTimersByTimeAsync(5000));
    expect(screen.queryByRole('status')).not.toBeInTheDocument();
  });

  it('reloads the card list every 30 seconds', async () => {
    render(<CustomCards />);
    await flush();
    expect(screen.queryByRole('article')).not.toBeInTheDocument();
    cards = [card({ rows: [{ label: 'A', value: 'b' }] })];
    await act(() => vi.advanceTimersByTimeAsync(30_000));
    expect(screen.getByRole('article', { name: 'Lab' })).toBeInTheDocument();
    expect(calls.filter((c) => c === 'get_custom_cards')).toHaveLength(2);
  });
});
