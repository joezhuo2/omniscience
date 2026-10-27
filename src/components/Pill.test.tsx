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

  it('draws arcs from the snapshot and leaves a missing dGPU dashed', () => {
    render(<Pill />);
    act(() => useTelemetryStore.getState().setSnapshot(sampleSnapshot));
    // Six utilization arcs plus CPU/iGPU temperature arcs; dGPU remains missing.
    expect(screen.getAllByTestId('ring-arc')).toHaveLength(8);
    expect(screen.getByRole('img', { name: 'dGPU' })).toHaveClass('opacity-40');
  });
});
