import { FeedbackButton } from '@crystalflux/feedback-button';
import { invoke } from '../../lib/ipc';
import type { Settings } from '../../settings';
import { useTelemetryStore } from '../../store';
import { useToastStore } from '../../toasts';
import type { IntegrationSettings } from '../../types/generated/IntegrationSettings';
import { Row, TextInput, ThresholdInput, Toggle } from './SettingsView';

const button =
  'rounded border border-island-faint px-2 py-0.5 text-[10px] text-island-text hover:border-island-muted';

export function IntegrationsSettings({
  settings,
  set,
}: {
  settings: Settings;
  set: (patch: Partial<Settings>) => void;
}) {
  const i = settings.integrations;
  const patch = (change: Partial<IntegrationSettings>) =>
    set({ integrations: { ...i, ...change } });
  const adapters = useTelemetryStore((s) => s.snapshot?.detail?.net_adapters);
  const push = useToastStore((s) => s.push);
  const copy = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
    } catch (e) {
      push(`Could not copy the address: ${String(e)}`);
      throw e;
    }
  };
  const openFolder = async () => {
    try {
      await invoke('open_custom_cards_folder');
    } catch (e) {
      push(`Could not open the cards folder: ${String(e)}`);
      throw e;
    }
  };
  const addresses = (adapters ?? []).flatMap((a) => (a.ipv4 ? [a.ipv4] : []));

  return (
    <>
      <Row
        label="Local API"
        hint="JSON at /api/status, Prometheus at /metrics, and omniscience status in a terminal. This PC only."
      >
        <span className="flex shrink-0 items-center gap-2">
          <ThresholdInput
            key={`api-port-${i.api_port}`}
            label="Port"
            value={i.api_port}
            min={1024}
            max={65535}
            unit=""
            onCommit={(api_port) => patch({ api_port })}
          />
          <Toggle
            label="Local API"
            checked={i.api_enabled}
            onChange={(api_enabled) => patch({ api_enabled })}
          />
        </span>
      </Row>
      {i.api_enabled && (
        <Row
          label="Allow other devices on this network"
          hint="Shows live readings on a phone or tablet. Anyone with the address below can read them."
        >
          <Toggle
            label="Allow other devices on this network"
            checked={i.api_lan}
            onChange={(api_lan) => patch({ api_lan })}
          />
        </Row>
      )}
      {i.api_enabled && i.api_lan && (
        <div className="space-y-1.5 py-1.5 text-[11px] text-island-text">
          {adapters === undefined ? (
            <p className="text-[10px] text-island-muted">
              Addresses appear once detailed readings are available.
            </p>
          ) : addresses.length === 0 ? (
            <p className="text-[10px] text-island-muted">No network adapter has an IPv4 address.</p>
          ) : i.api_token === '' ? (
            <p className="text-[10px] text-island-muted">Creating a token…</p>
          ) : (
            addresses.map((ip) => {
              const url = `http://${ip}:${i.api_port}/?token=${i.api_token}`;
              return (
                <div key={ip} className="flex items-center justify-between gap-2">
                  <span className="min-w-0 font-mono text-[10px] break-all">{url}</span>
                  <FeedbackButton
                    aria-label={`Copy address ${ip}`}
                    action={() => copy(url)}
                    done="Copied"
                    className={`${button} shrink-0`}
                  >
                    Copy
                  </FeedbackButton>
                </div>
              );
            })
          )}
          <div className="flex justify-end">
            <button className={button} onClick={() => patch({ api_token: '' })}>
              New token
            </button>
          </div>
        </div>
      )}
      <Row label="MQTT">
        <Toggle
          label="MQTT"
          checked={i.mqtt_enabled}
          onChange={(mqtt_enabled) => patch({ mqtt_enabled })}
        />
      </Row>
      {i.mqtt_enabled && (
        <>
          <Row label="Host">
            <TextInput
              key={`mqtt-host-${i.mqtt_host}`}
              label="MQTT host"
              value={i.mqtt_host}
              onCommit={(mqtt_host) => patch({ mqtt_host })}
            />
          </Row>
          <Row label="Port">
            <ThresholdInput
              key={`mqtt-port-${i.mqtt_port}`}
              label="MQTT port"
              value={i.mqtt_port}
              min={1}
              max={65535}
              unit=""
              onCommit={(mqtt_port) => patch({ mqtt_port })}
            />
          </Row>
          <Row label="Username">
            <TextInput
              key={`mqtt-user-${i.mqtt_username}`}
              label="MQTT username"
              value={i.mqtt_username}
              onCommit={(mqtt_username) => patch({ mqtt_username })}
            />
          </Row>
          <Row label="Password" hint="Stored in the settings file on this PC.">
            <TextInput
              key={`mqtt-pass-${i.mqtt_password}`}
              label="MQTT password"
              type="password"
              value={i.mqtt_password}
              onCommit={(mqtt_password) => patch({ mqtt_password })}
            />
          </Row>
          <Row label="Topic">
            <TextInput
              key={`mqtt-topic-${i.mqtt_topic}`}
              label="MQTT topic"
              value={i.mqtt_topic}
              onCommit={(mqtt_topic) => patch({ mqtt_topic })}
            />
          </Row>
          <Row label="Publish every">
            <ThresholdInput
              key={`mqtt-interval-${i.mqtt_interval_s}`}
              label="MQTT publish interval"
              value={i.mqtt_interval_s}
              min={2}
              max={3600}
              unit="s"
              onCommit={(mqtt_interval_s) => patch({ mqtt_interval_s })}
            />
          </Row>
          <Row label="Home Assistant discovery">
            <Toggle
              label="Home Assistant discovery"
              checked={i.mqtt_discovery}
              onChange={(mqtt_discovery) => patch({ mqtt_discovery })}
            />
          </Row>
        </>
      )}
      <Row label="Custom cards" hint="Add a JSON file per card. See docs/custom-cards.md.">
        <FeedbackButton action={openFolder} done="Opened" className={button}>
          Open cards folder
        </FeedbackButton>
      </Row>
    </>
  );
}
