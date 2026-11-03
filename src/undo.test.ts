import { beforeEach, expect, it, vi } from 'vitest';
import { useToastStore } from './toasts';
import { useUndoStore } from './undo';

beforeEach(() => {
  useUndoStore.setState({ entries: [], reverting: false, revision: 0 });
  useToastStore.setState({ toasts: [] });
});

it('reverts the most recent change first', async () => {
  const order: string[] = [];
  const { record } = useUndoStore.getState();
  record('first', async () => void order.push('first'));
  record('second', async () => void order.push('second'));
  await useUndoStore.getState().undo();
  expect(order).toEqual(['second']);
  expect(useUndoStore.getState().entries.map((e) => e.label)).toEqual(['first']);
  expect(useUndoStore.getState().revision).toBe(1);
  expect(useToastStore.getState().toasts.map((t) => t.text)).toEqual(['Undid second']);
});

it('keeps only the 10 most recent entries', () => {
  for (let i = 0; i < 12; i++) useUndoStore.getState().record(`change ${i}`, async () => {});
  const labels = useUndoStore.getState().entries.map((e) => e.label);
  expect(labels).toHaveLength(10);
  expect(labels[0]).toBe('change 2');
  expect(labels[9]).toBe('change 11');
});

it('drops a failed entry and reports the error without bumping the revision', async () => {
  useUndoStore.getState().record('GPU mode', () => Promise.reject('Firmware refused'));
  await useUndoStore.getState().undo();
  const state = useUndoStore.getState();
  expect(state.entries).toEqual([]);
  expect(state.reverting).toBe(false);
  expect(state.revision).toBe(0);
  expect(useToastStore.getState().toasts[0]?.text).toBe(
    'Could not undo GPU mode: Firmware refused',
  );
});

it('does nothing when empty', async () => {
  await useUndoStore.getState().undo();
  expect(useToastStore.getState().toasts).toEqual([]);
});

it('ignores undo calls while a revert is running', async () => {
  let finish: () => void = () => {};
  const earlier = vi.fn(async () => {});
  const slow = vi.fn(() => new Promise<void>((resolve) => (finish = resolve)));
  useUndoStore.getState().record('earlier', earlier);
  useUndoStore.getState().record('slow', slow);
  const first = useUndoStore.getState().undo();
  await useUndoStore.getState().undo();
  expect(useUndoStore.getState().entries.map((e) => e.label)).toEqual(['earlier']);
  finish();
  await first;
  expect(slow).toHaveBeenCalledTimes(1);
  expect(earlier).not.toHaveBeenCalled();
});
