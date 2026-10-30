import { act, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';
import { useTelemetryStore } from '../store';
import { sampleSnapshot } from '../test/fixtures';
import { Pill } from './Pill';

describe('Pill', () => {
  beforeEach(() => useTelemetryStore.setState({ snapshot: null, lastTickAt: null }));

  it('renders six rings with no text', () => {
    render(<Pill />);
    expect(screen.getAllByRole('img')).toHaveLength(6);
    expect(screen.getByTestId('pill').textContent).toBe('');
  });

  it('is the drag handle, including the rings inside it', () => {
    render(<Pill />);
    expect(screen.getByTestId('pill')).toHaveAttribute('data-tauri-drag-region', 'deep');
  });

  it('shows every ring as missing before the first tick', () => {
    render(<Pill />);
    expect(screen.queryAllByTestId('ring-arc')).toHaveLength(0);
  });

  it('centers five live rings when the discrete GPU is unavailable', () => {
    render(<Pill />);
    act(() => useTelemetryStore.getState().setSnapshot(sampleSnapshot));
    // Five metrics plus the upload and CPU/iGPU temperature arcs.
    expect(screen.getAllByTestId('ring-arc')).toHaveLength(8);
    expect(screen.queryByRole('img', { name: 'dGPU' })).not.toBeInTheDocument();
    expect(screen.getAllByRole('img')).toHaveLength(5);
    expect(screen.getByTestId('pill')).toHaveClass('justify-center');
    act(() => useTelemetryStore.getState().setSnapshot({ ...sampleSnapshot, core: { ...sampleSnapshot.core, dgpu: { percent: 0, level: 'normal' } } }));
    expect(screen.getAllByRole('img')).toHaveLength(6);
    expect(screen.getByRole('img', { name: 'dGPU' })).toBeInTheDocument();
  });
});
