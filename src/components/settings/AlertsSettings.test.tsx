import { mockIPC } from '@tauri-apps/api/mocks';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { useSettingsStore, type Settings } from '../../settings';
import type { AlertRule } from '../../types/generated/AlertRule';
import { SettingsView } from './SettingsView';

const rule = (threshold: number, duration_s = 30): AlertRule => ({
  enabled: false,
  threshold,
  duration_s,
});

const makeSettings = (): Settings => ({
  warn_threshold: 70,
  crit_threshold: 90,
  cpu_temp_warn_c: 80,
  cpu_temp_crit_c: 90,
  poll_interval_ms: 1000,
  hotkey: 'CommandOrControl+Shift+O',
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
  alerts: {
    cpu: rule(90),
    memory: rule(90),
    disk: rule(90),
    igpu: rule(90),
    dgpu: rule(90),
    cpu_temp: rule(90),
    gpu_temp: rule(85),
    disk_space: rule(10, 0),
    network_down: rule(0, 30),
    ping_spike: rule(200),
    battery_full: rule(90, 0),
    battery_low: rule(20, 0),
    quiet_hours: false,
    quiet_start_min: 1320,
    quiet_end_min: 420,
    snooze_until_ms: 0,
  },
});

describe('AlertsSettings', () => {
  let current: Settings;
  let saved: Settings[];

  beforeEach(() => {
    current = makeSettings();
    saved = [];
    useSettingsStore.setState({ settings: null, error: null });
    mockIPC((cmd, args) => {
      if (cmd === 'get_settings') return current;
      if (cmd === 'set_settings') {
        const s = (args as { settings: Settings }).settings;
        saved.push(s);
        return s;
      }
      return null;
    });
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('hides the rules when the master toggle is off', async () => {
    render(<SettingsView onBack={() => {}} />);
    expect(await screen.findByLabelText('Low battery threshold')).toBeInTheDocument();
    fireEvent.click(screen.getByLabelText('Alerts'));
    await waitFor(() => expect(saved.at(-1)?.sustained_alerts).toBe(false));
    await waitFor(() => expect(screen.queryByLabelText('Low battery threshold')).toBeNull());
    expect(screen.queryByText('Quiet hours')).toBeNull();
    expect(screen.getByLabelText('Alerts')).toBeInTheDocument();
  });

  it('enables a rule and sends the whole alerts object', async () => {
    render(<SettingsView onBack={() => {}} />);
    fireEvent.click(await screen.findByRole('checkbox', { name: 'Memory' }));
    await waitFor(() => expect(saved).toHaveLength(1));
    const expected = makeSettings().alerts;
    expected.memory.enabled = true;
    expect(saved[0]?.alerts).toEqual(expected);
  });

  it('changes a threshold and a duration', async () => {
    render(<SettingsView onBack={() => {}} />);
    const threshold = await screen.findByLabelText('CPU temperature threshold');
    fireEvent.change(threshold, { target: { value: '95' } });
    fireEvent.blur(threshold);
    await waitFor(() => expect(saved).toHaveLength(1));
    const expected = makeSettings().alerts;
    expected.cpu_temp.threshold = 95;
    expect(saved[0]?.alerts).toEqual(expected);

    const duration = screen.getByLabelText('Ping spike duration');
    fireEvent.change(duration, { target: { value: '60' } });
    fireEvent.blur(duration);
    await waitFor(() => expect(saved).toHaveLength(2));
    expect(saved[1]?.alerts.ping_spike.duration_s).toBe(60);
  });

  it('notes the missing ping target', async () => {
    current.ping_target = '';
    render(<SettingsView onBack={() => {}} />);
    expect(await screen.findAllByText(/Needs a ping target/)).toHaveLength(2);
  });

  it('round-trips quiet hours as minutes after midnight', async () => {
    render(<SettingsView onBack={() => {}} />);
    expect(screen.queryByLabelText('Quiet hours from')).toBeNull();
    fireEvent.click(await screen.findByLabelText('Quiet hours'));
    const from = await screen.findByLabelText('Quiet hours from');
    expect(from).toHaveValue('22:00');
    expect(screen.getByLabelText('Quiet hours to')).toHaveValue('07:00');
    fireEvent.change(from, { target: { value: '23:30' } });
    await waitFor(() => expect(saved.at(-1)?.alerts.quiet_start_min).toBe(1410));
    expect(saved.at(-1)?.alerts.quiet_hours).toBe(true);
    fireEvent.change(screen.getByLabelText('Quiet hours to'), { target: { value: '06:15' } });
    await waitFor(() => expect(saved.at(-1)?.alerts.quiet_end_min).toBe(375));
    await waitFor(() => expect(screen.getByLabelText('Quiet hours from')).toHaveValue('23:30'));
  });

  it('snoozes for an hour and resumes', async () => {
    render(<SettingsView onBack={() => {}} />);
    const before = Date.now();
    fireEvent.click(await screen.findByRole('button', { name: '1 h' }));
    await waitFor(() => expect(saved).toHaveLength(1));
    const until = saved[0]?.alerts.snooze_until_ms;
    expect(until).toBeGreaterThanOrEqual(before + 3_600_000);
    expect(until).toBeLessThanOrEqual(Date.now() + 3_600_000);
    expect(await screen.findByText(/^Snoozed until \d\d:\d\d$/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Resume' }));
    await waitFor(() => expect(saved.at(-1)?.alerts.snooze_until_ms).toBe(0));
    expect(await screen.findByRole('button', { name: '8 h' })).toBeInTheDocument();
  });
});
