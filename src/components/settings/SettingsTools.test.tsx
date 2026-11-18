import { mockIPC } from '@tauri-apps/api/mocks';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { applyAppearance, loadAppearance, saveAppearance } from '../../lib/appearance';
import { setUnits } from '../../lib/format';
import { useGlass } from '../../lib/glass';
import { buildSettingsDocument } from '../../lib/settingsFile';
import { useSettingsStore, type Settings } from '../../settings';
import { useToastStore } from '../../toasts';
import { SettingsView } from './SettingsView';

const CPU_CUSTOM = /CPU\s*Custom/;
const rule = { enabled: false, threshold: 90, duration_s: 30 };
const none = { cpu: null, ram: null, igpu: null, dgpu: null, disk: null, net: null };

const makeSettings = (over: Partial<Settings> = {}): Settings => ({
  warn_threshold: 70,
  crit_threshold: 90,
  cpu_temp_warn_c: 80,
  cpu_temp_crit_c: 90,
  poll_interval_ms: 1000,
  hotkey: 'CommandOrControl+Shift+O',
  profiles: [],
  active_profile: '',
  launch_at_startup: false,
  auto_check_updates: true,
  hide_over_fullscreen: true,
  ping_target: '1.1.1.1',
  show_public_ip: false,
  config_enabled: false,
  sustained_alerts: true,
  sensor_details: false,
  history_hours: 24,
  hover_open_delay_ms: 0,
  hover_close_delay_ms: 0,
  pill_rings: ['cpu', 'ram', 'igpu', 'dgpu', 'disk', 'net'],
  pill_numbers: 'off',
  dodge_browser_tabs: false,
  energy_price_per_kwh: 0,
  energy_currency: '$',
  temp_unit: 'celsius',
  net_unit: 'bytes',
  metric_thresholds: none,
  alerts: {
    cpu: rule,
    memory: rule,
    disk: rule,
    igpu: rule,
    dgpu: rule,
    cpu_temp: { ...rule, threshold: 85 },
    gpu_temp: rule,
    disk_space: rule,
    network_down: rule,
    ping_spike: rule,
    battery_full: rule,
    battery_low: rule,
    quiet_hours: false,
    quiet_start_min: 1320,
    quiet_end_min: 420,
    snooze_until_ms: 0,
  },
  ...over,
});

Element.prototype.scrollIntoView = vi.fn();

describe('settings tools', () => {
  let current: Settings;
  let saved: Settings[];
  let calls: [string, unknown][];
  let setFailure: string | null;
  let writeText: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    current = makeSettings();
    saved = [];
    calls = [];
    setFailure = null;
    writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true });
    useToastStore.setState({ toasts: [] });
    useSettingsStore.setState({ settings: null, error: null });
    mockIPC((cmd, args) => {
      calls.push([cmd, args]);
      if (cmd === 'get_settings') return current;
      if (cmd === 'set_settings') {
        if (setFailure) throw setFailure;
        const s = (args as { settings: Settings }).settings;
        saved.push(s);
        return s;
      }
      if (cmd === 'get_default_settings') {
        return makeSettings({ warn_threshold: 75, hover_open_delay_ms: 100, temp_unit: 'celsius' });
      }
      if (cmd === 'export_settings_file') return 'C:\\Downloads\\omniscience-settings.json';
      if (cmd === 'get_system_summary') return 'CPU: test';
      return null;
    });
  });

  afterEach(() => {
    setUnits('celsius', 'bytes');
    saveAppearance({});
    applyAppearance({});
  });

  it('switches the units and stores the choice', async () => {
    render(<SettingsView onBack={() => {}} />);
    fireEvent.click(await screen.findByRole('radio', { name: '°F' }));
    await waitFor(() => expect(saved.at(-1)?.temp_unit).toBe('fahrenheit'));
    fireEvent.click(screen.getByRole('radio', { name: 'Bits' }));
    await waitFor(() => expect(saved.at(-1)?.net_unit).toBe('bits'));
  });

  it('shows and stores temperature limits in the chosen unit', async () => {
    current = makeSettings({ temp_unit: 'fahrenheit' });
    render(<SettingsView onBack={() => {}} />);
    const warn = await screen.findByLabelText('CPU temperature warning');
    expect(warn).toHaveValue(176);
    fireEvent.change(warn, { target: { value: '185' } });
    fireEvent.blur(warn);
    await waitFor(() => expect(saved.at(-1)?.cpu_temp_warn_c).toBe(85));
    const rule = screen.getByLabelText('CPU temperature threshold');
    expect(rule).toHaveValue(185);
    fireEvent.change(rule, { target: { value: '203' } });
    fireEvent.blur(rule);
    await waitFor(() => expect(saved.at(-1)?.alerts.cpu_temp.threshold).toBe(95));
  });

  it('turns a per-metric pair on from the global pair and off to null', async () => {
    render(<SettingsView onBack={() => {}} />);
    expect(screen.queryByLabelText('CPU warning threshold')).toBeNull();
    fireEvent.click(await screen.findByRole('checkbox', { name: CPU_CUSTOM }));
    await waitFor(() => expect(saved).toHaveLength(1));
    expect(saved[0]?.metric_thresholds).toEqual({ ...none, cpu: { warn: 70, crit: 90 } });
    const warn = await screen.findByLabelText('CPU warning threshold');
    fireEvent.change(warn, { target: { value: '60' } });
    fireEvent.blur(warn);
    await waitFor(() => expect(saved).toHaveLength(2));
    expect(saved[1]?.metric_thresholds.cpu).toEqual({ warn: 60, crit: 90 });
    const crit = screen.getByLabelText('CPU critical threshold');
    fireEvent.change(crit, { target: { value: '95' } });
    fireEvent.blur(crit);
    await waitFor(() => expect(saved).toHaveLength(3));
    expect(saved[2]?.metric_thresholds.cpu).toEqual({ warn: 60, crit: 95 });
    fireEvent.click(screen.getByRole('checkbox', { name: CPU_CUSTOM }));
    await waitFor(() => expect(saved).toHaveLength(4));
    expect(saved[3]?.metric_thresholds).toEqual(none);
  });

  it('resets one group only, after a second click', async () => {
    current = makeSettings({ warn_threshold: 50, hover_open_delay_ms: 300 });
    render(<SettingsView onBack={() => {}} />);
    const button = await screen.findByRole('button', { name: 'Reset' });
    fireEvent.click(button);
    expect(saved).toHaveLength(0);
    fireEvent.click(screen.getByRole('button', { name: 'Confirm reset' }));
    await waitFor(() => expect(saved).toHaveLength(1));
    expect(saved[0]?.warn_threshold).toBe(75);
    expect(saved[0]?.hover_open_delay_ms).toBe(300);
    expect(await screen.findByText('Thresholds reset to defaults.')).toBeVisible();
  });

  it('picks another group to reset', async () => {
    current = makeSettings({ hover_open_delay_ms: 300 });
    render(<SettingsView onBack={() => {}} />);
    fireEvent.click(await screen.findByRole('combobox', { name: 'Settings group' }));
    fireEvent.click(await screen.findByRole('option', { name: 'Island and pill' }));
    fireEvent.click(screen.getByRole('button', { name: 'Reset' }));
    fireEvent.click(screen.getByRole('button', { name: 'Confirm reset' }));
    await waitFor(() => expect(saved).toHaveLength(1));
    expect(saved[0]?.hover_open_delay_ms).toBe(100);
    expect(saved[0]?.warn_threshold).toBe(70);
  });

  it('copies the system summary', async () => {
    render(<SettingsView onBack={() => {}} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Copy system summary' }));
    await waitFor(() => expect(writeText).toHaveBeenCalledWith('CPU: test'));
    expect(await screen.findByText('System summary copied to the clipboard.')).toBeVisible();
  });

  it('exports the settings, appearance and glass options', async () => {
    saveAppearance({ 'font-size': '110%' });
    render(<SettingsView onBack={() => {}} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Export settings' }));
    expect(
      await screen.findByText('Saved to C:\\Downloads\\omniscience-settings.json'),
    ).toBeVisible();
    const call = calls.find(([cmd]) => cmd === 'export_settings_file');
    const doc = JSON.parse((call?.[1] as { contents: string }).contents);
    expect(doc).toEqual({
      app: 'omniscience',
      version: 1,
      settings: current,
      appearance: { 'font-size': '110%' },
      glass: { on: useGlass.getState().on, opacity: useGlass.getState().opacity },
    });
  });

  const importFile = async (text: string) => {
    const input = await screen.findByLabelText('Settings file to import');
    const file = new File([text], 'settings.json', { type: 'application/json' });
    Object.defineProperty(file, 'text', { value: () => Promise.resolve(text) });
    fireEvent.change(input, { target: { files: [file] } });
  };

  it('imports a document and keeps the machine-specific values', async () => {
    current = makeSettings({
      launch_at_startup: true,
      alerts: { ...makeSettings().alerts, snooze_until_ms: 42 },
    });
    const incoming = makeSettings({
      warn_threshold: 55,
      launch_at_startup: false,
      alerts: { ...makeSettings().alerts, snooze_until_ms: 0 },
    });
    render(<SettingsView onBack={() => {}} />);
    await screen.findByLabelText('Warning threshold');
    await importFile(
      JSON.stringify(
        buildSettingsDocument(incoming, { 'font-size': '105%' }, { on: false, opacity: 40 }),
      ),
    );
    await waitFor(() => expect(saved).toHaveLength(1));
    expect(saved[0]?.warn_threshold).toBe(55);
    expect(saved[0]?.launch_at_startup).toBe(true);
    expect(saved[0]?.alerts.snooze_until_ms).toBe(42);
    expect(await screen.findByText('Settings imported.')).toBeVisible();
    expect(loadAppearance()).toEqual({ 'font-size': '105%' });
    expect(useGlass.getState().on).toBe(false);
    expect(useGlass.getState().opacity).toBe(40);
    useGlass.getState().setOn(true);
  });

  it('applies nothing when the file has the wrong shape', async () => {
    render(<SettingsView onBack={() => {}} />);
    await screen.findByLabelText('Warning threshold');
    await importFile('{"app":"other"}');
    expect(await screen.findByText(/Could not import settings: Not an Omniscience/)).toBeVisible();
    expect(saved).toHaveLength(0);
    expect(loadAppearance()).toEqual({});
  });

  it('shows the store error when the backend refuses the import', async () => {
    render(<SettingsView onBack={() => {}} />);
    await screen.findByLabelText('Warning threshold');
    setFailure = 'hotkey is taken';
    await importFile(
      JSON.stringify(buildSettingsDocument(makeSettings(), {}, { on: true, opacity: 60 })),
    );
    expect(await screen.findByText('Settings were not applied: hotkey is taken')).toBeVisible();
  });
});
