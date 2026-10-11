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

  it('shows every ring as missing before the first tick', () => {
    render(<Pill />);
    expect(screen.queryAllByTestId('ring-arc')).toHaveLength(0);
  });

  it('draws arcs from the snapshot and leaves a missing dGPU dashed', () => {
    render(<Pill />);
    act(() => useTelemetryStore.getState().setSnapshot(sampleSnapshot));
    // cpu, ram, igpu, disk, net outer + inner = 6 arcs; dgpu missing.
    expect(screen.getAllByTestId('ring-arc')).toHaveLength(6);
    expect(screen.getByRole('img', { name: 'dGPU' })).toHaveClass('opacity-40');
  });
});
