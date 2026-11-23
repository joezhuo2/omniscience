import { mockIPC } from '@tauri-apps/api/mocks';
import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useSettingsStore, type Settings } from '../../settings';
import { useTelemetryStore } from '../../store';
import { sampleDetail, sampleSettings, sampleSnapshot } from '../../test/fixtures';
import { useToastStore } from '../../toasts';
import type { IntegrationSettings } from '../../types/generated/IntegrationSettings';
import { SettingsView } from './SettingsView';

describe('settings headings', () => {
  beforeEach(() => {
    useSettingsStore.setState({ settings: sampleSettings, error: null });
    mockIPC(() => null);
  });

  it('groups the settings under headings in order', () => {
    render(<SettingsView onBack={() => {}} />);
    expect(screen.getAllByRole('heading', { level: 2 }).map((h) => h.textContent)).toEqual([
      'General',
      'Island and pill',
      'Thresholds and units',
      'Monitoring',
      'Alerts',
      'Integrations',
      'System config',
      'Data and support',
    ]);
  });

  it('gives each resettable heading its own reset button', () => {
    render(<SettingsView onBack={() => {}} />);
    for (const name of [
      'Island and pill',
      'Thresholds and units',
      'Monitoring',
      'Alerts',
      'Integrations',
    ]) {
      expect(screen.getByRole('button', { name: `Reset ${name}` })).toBeInTheDocument();
    }
    expect(screen.queryByRole('button', { name: 'Reset General' })).not.toBeInTheDocument();
  });
});

describe('integrations settings', () => {
  let saved: Settings[];
  let calls: string[];
  let writeText: ReturnType<typeof vi.fn>;
  let current: IntegrationSettings;

  const show = (over: Partial<IntegrationSettings> = {}) => {
    current = { ...sampleSettings.integrations, ...over };
    useSettingsStore.setState({
      settings: { ...sampleSettings, integrations: current },
      error: null,
    });
    render(<SettingsView onBack={() => {}} />);
  };
  const lastSaved = () => saved.at(-1)?.integrations;

  beforeEach(() => {
    saved = [];
    calls = [];
    writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true });
    useToastStore.setState({ toasts: [] });
    useTelemetryStore.setState({ snapshot: null, lastTickAt: null });
    mockIPC((cmd, args) => {
      calls.push(cmd);
      if (cmd === 'set_settings') {
        const s = (args as { settings: Settings }).settings;
        saved.push(s);
        return s;
      }
      return null;
    });
  });

  it('shows only the local API controls while everything is off', () => {
    show();
    expect(screen.getByLabelText('Local API')).not.toBeChecked();
    expect(screen.getByLabelText('Port')).toHaveValue(8765);
    expect(screen.queryByLabelText('Allow other devices on this network')).toBeNull();
    expect(screen.queryByLabelText('MQTT host')).toBeNull();
  });

  it('turns the local API on without changing other fields', async () => {
    show();
    fireEvent.click(screen.getByLabelText('Local API'));
    await waitFor(() => expect(lastSaved()).toEqual({ ...current, api_enabled: true }));
  });

  it('changes the port', async () => {
    show();
    const input = screen.getByLabelText('Port');
    fireEvent.change(input, { target: { value: '9000' } });
    fireEvent.blur(input);
    await waitFor(() => expect(lastSaved()).toEqual({ ...current, api_port: 9000 }));
  });

  it('allows other devices on the network', async () => {
    show({ api_enabled: true });
    fireEvent.click(screen.getByLabelText('Allow other devices on this network'));
    await waitFor(() => expect(lastSaved()).toEqual({ ...current, api_lan: true }));
  });

  it('turns MQTT on', async () => {
    show();
    fireEvent.click(screen.getByLabelText('MQTT'));
    await waitFor(() => expect(lastSaved()).toEqual({ ...current, mqtt_enabled: true }));
  });

  it.each([
    ['MQTT host', 'mqtt_host', 'mqtt.example', 'mqtt.example'],
    ['MQTT port', 'mqtt_port', '8883', 8883],
    ['MQTT username', 'mqtt_username', 'joe', 'joe'],
    ['MQTT password', 'mqtt_password', ' p w ', ' p w '],
    ['MQTT topic', 'mqtt_topic', 'home/pc', 'home/pc'],
    ['MQTT publish interval', 'mqtt_interval_s', '30', 30],
  ])('commits %s with the whole object', async (label, key, typed, expected) => {
    show({ mqtt_enabled: true });
    const input = screen.getByLabelText(label);
    fireEvent.change(input, { target: { value: typed } });
    fireEvent.blur(input);
    await waitFor(() => expect(lastSaved()).toEqual({ ...current, [key]: expected }));
  });

  it('hides the password and toggles discovery', async () => {
    show({ mqtt_enabled: true });
    expect(screen.getByLabelText('MQTT password')).toHaveAttribute('type', 'password');
    expect(screen.getByText('Stored in the settings file on this PC.')).toBeInTheDocument();
    fireEvent.click(screen.getByLabelText('Home Assistant discovery'));
    await waitFor(() => expect(lastSaved()).toEqual({ ...current, mqtt_discovery: true }));
  });

  describe('network addresses', () => {
    const lan = { api_enabled: true, api_lan: true, api_token: 'tok123' };

    it('says so while the detail tier is not available', () => {
      show(lan);
      expect(screen.getByText(/once detailed readings are available/)).toBeInTheDocument();
    });

    it('lists an address per adapter with an IPv4 and copies one', async () => {
      act(() =>
        useTelemetryStore.getState().setSnapshot({
          ...sampleSnapshot,
          detail: {
            ...sampleDetail,
            net_adapters: [
              ...sampleDetail.net_adapters,
              { name: 'Down', down_bps: 0, up_bps: 0, ipv4: null },
            ],
          },
        }),
      );
      show(lan);
      expect(screen.getByText('http://192.168.1.20:8765/?token=tok123')).toBeInTheDocument();
      expect(screen.getByText('http://172.20.0.1:8765/?token=tok123')).toBeInTheDocument();
      expect(screen.getAllByRole('button', { name: /^Copy address/ })).toHaveLength(2);
      fireEvent.click(screen.getByRole('button', { name: 'Copy address 192.168.1.20' }));
      await waitFor(() =>
        expect(writeText).toHaveBeenCalledWith('http://192.168.1.20:8765/?token=tok123'),
      );
    });

    it('asks for a new token by clearing it', async () => {
      act(() =>
        useTelemetryStore.getState().setSnapshot({ ...sampleSnapshot, detail: sampleDetail }),
      );
      show(lan);
      fireEvent.click(screen.getByRole('button', { name: 'New token' }));
      await waitFor(() => expect(lastSaved()).toEqual({ ...current, api_token: '' }));
    });

    it('does not offer addresses when LAN access is off', () => {
      act(() =>
        useTelemetryStore.getState().setSnapshot({ ...sampleSnapshot, detail: sampleDetail }),
      );
      show({ api_enabled: true });
      expect(screen.queryByRole('button', { name: 'New token' })).toBeNull();
    });
  });

  it('opens the custom cards folder', async () => {
    show();
    fireEvent.click(screen.getByRole('button', { name: 'Open cards folder' }));
    await waitFor(() => expect(calls).toContain('open_custom_cards_folder'));
  });

  it('resets only the integrations group', async () => {
    show({ api_enabled: true, api_port: 9999 });
    mockIPC((cmd, args) => {
      if (cmd === 'get_default_settings') return sampleSettings;
      if (cmd === 'set_settings') {
        const s = (args as { settings: Settings }).settings;
        saved.push(s);
        return s;
      }
      return null;
    });
    fireEvent.click(screen.getByRole('button', { name: 'Reset Integrations' }));
    fireEvent.click(screen.getByRole('button', { name: 'Confirm reset Integrations' }));
    await waitFor(() => expect(lastSaved()).toEqual(sampleSettings.integrations));
  });
});
