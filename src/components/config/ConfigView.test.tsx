import { fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { ConfigView } from './ConfigView';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

beforeEach(() => {
  vi.mocked(invoke).mockReturnValue(new Promise(() => {}));
});

afterEach(() => {
  localStorage.clear();
  vi.clearAllMocks();
});

describe('ConfigView card toggles', () => {
  it('defaults every card on, hides the body when off and persists', () => {
    const { unmount } = render(<ConfigView active={false} />);
    const switches = screen.getAllByRole('switch');
    expect(switches).toHaveLength(7);
    switches.forEach((s) => expect(s).toHaveAttribute('aria-checked', 'true'));
    const cpu = screen.getByRole('switch', { name: 'CPU boost enabled' });
    expect(screen.getByRole('group', { name: 'CPU boost' })).toBeInTheDocument();
    fireEvent.click(cpu);
    expect(cpu).toHaveAttribute('aria-checked', 'false');
    expect(screen.queryByRole('group', { name: 'CPU boost' })).not.toBeInTheDocument();
    expect(localStorage.getItem('omniscience:section:cpu-boost:off')).toBe('true');
    unmount();
    render(<ConfigView active={false} />);
    expect(screen.getByRole('switch', { name: 'CPU boost enabled' })).toHaveAttribute(
      'aria-checked',
      'false',
    );
    expect(screen.getByRole('switch', { name: 'Display enabled' })).toHaveAttribute(
      'aria-checked',
      'true',
    );
  });

  it('does not read the device for a disabled card', async () => {
    vi.useFakeTimers();
    localStorage.setItem('omniscience:section:cpu-boost:off', 'true');
    render(<ConfigView />);
    await vi.advanceTimersByTimeAsync(10);
    expect(invoke).not.toHaveBeenCalledWith('get_cpu_boost');
    expect(invoke).toHaveBeenCalledWith('get_gpu_mode');
    vi.useRealTimers();
  });
});
