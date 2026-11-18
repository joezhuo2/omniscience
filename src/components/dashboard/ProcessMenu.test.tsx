import { mockIPC } from '@tauri-apps/api/mocks';
import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { CONFIRM_MS } from '../../lib/processes';
import { useTelemetryStore } from '../../store';
import { sampleDetail, sampleSnapshot } from '../../test/fixtures';
import { useToastStore } from '../../toasts';
import { Dashboard } from './Dashboard';

const details = {
  pid: 10,
  name: 'chrome.exe',
  path: 'C:\\chrome.exe',
  command_line: null,
  priority: 'normal',
  efficiency: false,
  suspended: false,
};

describe('process menu', () => {
  let calls: { cmd: string; args: unknown }[];
  let results: unknown;
  let suspended: boolean;

  beforeEach(() => {
    calls = [];
    suspended = false;
    results = [{ pid: 10, outcome: 'ok', message: null }];
    useToastStore.setState({ toasts: [] });
    mockIPC((cmd, args) => {
      calls.push({ cmd, args });
      if (cmd === 'process_details') return { ...details, suspended };
      return cmd === 'open_process_location' ? null : results;
    });
    act(() =>
      useTelemetryStore.getState().setSnapshot({ ...sampleSnapshot, detail: sampleDetail }),
    );
  });

  const sent = (cmd: string) => calls.filter((c) => c.cmd === cmd).map((c) => c.args);
  const moreChrome = () => screen.getByRole('button', { name: 'More actions for chrome.exe' });
  const opened = async () => {
    render(<Dashboard />);
    fireEvent.click(moreChrome());
    await waitFor(() => expect(screen.getByRole('menuitem', { name: 'Copy path' })).toBeEnabled());
  };

  it('opens from right-click and from the button, loading the main process details', async () => {
    render(<Dashboard />);
    fireEvent.contextMenu(screen.getByText('Code').closest('li')!.firstElementChild!);
    expect(screen.getByRole('menu', { name: 'Actions for Code.exe' })).toBeInTheDocument();
    expect(sent('process_details')).toEqual([{ pid: 20 }]);
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(screen.queryByRole('menu')).toBeNull();
    fireEvent.click(moreChrome());
    await waitFor(() => expect(screen.getByRole('menuitem', { name: 'Copy path' })).toBeEnabled());
    expect(sent('process_details')).toContainEqual({ pid: 10 });
    expect(screen.getByRole('menuitem', { name: 'Copy command line' })).toBeDisabled();
    expect(screen.getByRole('menuitemradio', { name: 'Normal' })).toHaveAttribute(
      'aria-checked',
      'true',
    );
    expect(screen.getByRole('menuitemcheckbox', { name: 'Efficiency mode' })).toHaveAttribute(
      'aria-checked',
      'false',
    );
  });

  it.each([
    ['High', 'menuitemradio', 'set_process_priority', { pids: [10, 11], priority: 'high' }],
    [
      'Efficiency mode',
      'menuitemcheckbox',
      'set_process_efficiency',
      { pids: [10, 11], enabled: true },
    ],
    ['Suspend', 'menuitem', 'suspend_processes', { pids: [10, 11] }],
    ['Open file location', 'menuitem', 'open_process_location', { pid: 10 }],
  ])('%s sends %s to the right command', async (name, role, cmd, args) => {
    await opened();
    fireEvent.click(screen.getByRole(role, { name }));
    expect(sent(cmd)).toEqual([args]);
  });

  it('offers Resume for a suspended process', async () => {
    suspended = true;
    await opened();
    await waitFor(() => expect(screen.getByRole('menuitem', { name: 'Resume' })).toBeEnabled());
    fireEvent.click(screen.getByRole('menuitem', { name: 'Resume' }));
    expect(sent('resume_processes')).toEqual([{ pids: [10, 11] }]);
  });

  it('copies the path and toasts', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true });
    await opened();
    fireEvent.click(screen.getByRole('menuitem', { name: 'Copy path' }));
    expect(writeText).toHaveBeenCalledWith('C:\\chrome.exe');
    await waitFor(() => expect(useToastStore.getState().toasts[0]?.text).toBe('Path copied'));
  });

  it('needs a second activation to end the process tree', async () => {
    await opened();
    fireEvent.click(screen.getByRole('menuitem', { name: 'End process tree' }));
    expect(sent('kill_process_tree')).toEqual([]);
    fireEvent.click(screen.getByRole('menuitem', { name: 'Confirm end process tree?' }));
    expect(sent('kill_process_tree')).toEqual([{ pids: [10, 11] }]);
  });

  it('disarms the tree confirmation after the confirm window', async () => {
    await opened();
    vi.useFakeTimers();
    try {
      fireEvent.click(screen.getByRole('menuitem', { name: 'End process tree' }));
      act(() => vi.advanceTimersByTime(CONFIRM_MS + 10));
      expect(screen.getByRole('menuitem', { name: 'End process tree' })).toBeInTheDocument();
    } finally {
      vi.useRealTimers();
    }
  });

  it('summarises failures in a toast', async () => {
    results = [
      { pid: 10, outcome: 'protected', message: null },
      { pid: 11, outcome: 'access_denied', message: 'Access is denied' },
    ];
    await opened();
    fireEvent.click(screen.getByRole('menuitem', { name: 'Suspend' }));
    await waitFor(() =>
      expect(useToastStore.getState().toasts[0]?.text).toBe(
        'Suspend: 1 protected, 1 access denied. Access is denied',
      ),
    );
  });

  it('moves with the arrow keys and returns focus to the button on Escape', async () => {
    await opened();
    const button = moreChrome();
    const items = Array.from(
      screen.getByRole('menu').querySelectorAll<HTMLButtonElement>('[role^="menuitem"]'),
    ).filter((el) => !el.disabled);
    expect(items[0]).toHaveFocus();
    fireEvent.keyDown(items[0]!, { key: 'ArrowDown' });
    expect(items[1]).toHaveFocus();
    fireEvent.keyDown(items[1]!, { key: 'ArrowUp' });
    fireEvent.keyDown(items[0]!, { key: 'ArrowUp' });
    expect(items[items.length - 1]).toHaveFocus();
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(screen.queryByRole('menu')).toBeNull();
    expect(button).toHaveFocus();
  });

  it('expands a group into members with the same menu on a single pid', async () => {
    render(<Dashboard />);
    expect(screen.queryByRole('button', { name: 'Show Code.exe processes' })).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Show chrome.exe processes' }));
    const group = screen.getByRole('group', { name: 'chrome.exe processes' });
    expect(group).toHaveTextContent('PID 10');
    expect(group).toHaveTextContent('1.5 GB');
    fireEvent.click(
      within(group).getByRole('button', { name: 'More actions for chrome.exe PID 11' }),
    );
    await waitFor(() => expect(screen.getByRole('menuitem', { name: 'Copy path' })).toBeEnabled());
    expect(screen.queryByRole('menuitem', { name: /Pin/ })).toBeNull();
    fireEvent.click(screen.getByRole('menuitem', { name: 'Suspend' }));
    expect(sent('suspend_processes')).toEqual([{ pids: [11] }]);
    fireEvent.click(screen.getByRole('button', { name: 'Hide chrome.exe processes' }));
    expect(screen.queryByRole('group', { name: 'chrome.exe processes' })).toBeNull();
  });

  it('pins a group to the top and remembers it', () => {
    const names = () =>
      Array.from(screen.getByTestId('process-rows').querySelectorAll('li')).map(
        (li) => li.textContent,
      );
    const { unmount } = render(<Dashboard />);
    fireEvent.click(screen.getByRole('tab', { name: 'ram' }));
    expect(names()[0]).toContain('Code');
    fireEvent.click(moreChrome());
    fireEvent.click(screen.getByRole('menuitem', { name: 'Pin' }));
    expect(names()[0]).toContain('chrome');
    expect(screen.getByLabelText('Pinned')).toBeInTheDocument();
    expect(JSON.parse(localStorage.getItem('omniscience:pinned-processes')!)).toEqual(['chrome']);
    unmount();
    render(<Dashboard />);
    fireEvent.click(screen.getByRole('tab', { name: 'ram' }));
    expect(names()[0]).toContain('chrome');
    fireEvent.click(moreChrome());
    expect(screen.getByRole('menuitem', { name: 'Unpin' })).toBeInTheDocument();
  });
});
