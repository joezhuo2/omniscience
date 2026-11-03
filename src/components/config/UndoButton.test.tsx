import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { mockIPC } from '@tauri-apps/api/mocks';
import { beforeEach, expect, it } from 'vitest';
import { useToastStore } from '../../toasts';
import { useUndoStore } from '../../undo';
import { GpuModeCard, type GpuStatus } from './GpuModeCard';
import { UndoButton } from './UndoButton';

const base: GpuStatus = {
  selected: null,
  current: 'med',
  supported: true,
  mux_supported: true,
  restart_required: false,
  error: null,
};

beforeEach(() => {
  useUndoStore.setState({ entries: [], reverting: false, revision: 0 });
  useToastStore.setState({ toasts: [] });
});

function mockGpu(failUndo = false) {
  let current = base.current;
  const calls: unknown[] = [];
  mockIPC((cmd, args) => {
    if (cmd === 'set_gpu_mode') {
      calls.push(args);
      if (failUndo && calls.length > 1) throw new Error('Firmware refused GPU mode');
      current = (args as { mode: GpuStatus['current'] }).mode;
    }
    return { ...base, current };
  });
  return calls;
}

const renderBoth = () =>
  render(
    <>
      <UndoButton />
      <GpuModeCard />
    </>,
  );

it('is disabled with no history', () => {
  mockGpu();
  renderBoth();
  expect(screen.getByRole('button', { name: 'Undo' })).toBeDisabled();
});

it('names the last change and re-applies the previous value through the same command', async () => {
  const calls = mockGpu();
  renderBoth();
  await waitFor(() => expect(screen.getByRole('button', { name: 'Min' })).toBeEnabled());
  fireEvent.click(screen.getByRole('button', { name: 'Min' }));
  const undo = await screen.findByRole('button', { name: 'Undo: GPU mode' });
  await waitFor(() => expect(undo).toBeEnabled());
  fireEvent.click(undo);
  await waitFor(() =>
    expect(useToastStore.getState().toasts.map((t) => t.text)).toEqual(['Undid GPU mode']),
  );
  expect(calls).toEqual([{ mode: 'min' }, { mode: 'med' }]);
  expect(screen.getByRole('button', { name: 'Undo' })).toBeDisabled();
  await waitFor(() =>
    expect(screen.getByRole('button', { name: 'Med' })).toHaveAttribute('aria-pressed', 'true'),
  );
});

it('reports a failed undo with a toast', async () => {
  mockGpu(true);
  renderBoth();
  await waitFor(() => expect(screen.getByRole('button', { name: 'Min' })).toBeEnabled());
  fireEvent.click(screen.getByRole('button', { name: 'Min' }));
  const undo = await screen.findByRole('button', { name: 'Undo: GPU mode' });
  await waitFor(() => expect(undo).toBeEnabled());
  fireEvent.click(undo);
  await waitFor(() =>
    expect(useToastStore.getState().toasts[0]?.text).toBe(
      'Could not undo GPU mode: Error: Firmware refused GPU mode',
    ),
  );
  expect(screen.getByRole('button', { name: 'Undo' })).toBeDisabled();
});

it('does not record a change that fails or selects the current value', async () => {
  mockIPC((cmd) => {
    if (cmd === 'set_gpu_mode') throw new Error('nope');
    return base;
  });
  renderBoth();
  await waitFor(() => expect(screen.getByRole('button', { name: 'Min' })).toBeEnabled());
  fireEvent.click(screen.getByRole('button', { name: 'Min' }));
  await screen.findByRole('alert');
  expect(useUndoStore.getState().entries).toEqual([]);
});

it('skips recording when the new value equals the previous one', async () => {
  mockGpu();
  renderBoth();
  await waitFor(() => expect(screen.getByRole('button', { name: 'Med' })).toBeEnabled());
  fireEvent.click(screen.getByRole('button', { name: 'Med' }));
  await waitFor(() => expect(screen.getByRole('button', { name: 'Med' })).toBeEnabled());
  expect(useUndoStore.getState().entries).toEqual([]);
});
