import { SENSORS, sensorStates, type SensorKey } from '../../lib/sensors';
import { useSettingsStore } from '../../settings';
import { useTelemetryStore } from '../../store';
import { useStaleness } from '../../hooks/useStaleness';

export function SensorDetails() {
  const core = useTelemetryStore((s) => s.snapshot?.core);
  const enabled = useSettingsStore((s) => s.settings?.sensor_details ?? false);
  const { stale, ageSeconds } = useStaleness();
  if (!core) return null;
  const states = sensorStates(core);
  const rows = (Object.keys(SENSORS) as SensorKey[]).flatMap((key) =>
    states[key] === 'absent' || !(enabled || stale || states[key] === 'missing')
      ? []
      : [{ key, info: SENSORS[key], missing: states[key] === 'missing' }],
  );
  if (rows.length === 0) return null;
  return (
    <ul data-testid="sensor-details" className="space-y-1 text-[10px] leading-snug">
      {rows.map(({ key, info, missing }) => (
        <li
          key={key}
          data-problem={missing || stale ? true : undefined}
          className={missing || stale ? 'text-island-warn' : 'text-island-muted'}
        >
          <span className="font-medium">{info.label}</span>
          {`: ${info.source}${info.proxy ? ' (proxy measurement)' : ''}. `}
          {stale ? `No update for ${ageSeconds}s. ` : `Updated ${ageSeconds}s ago. `}
          {missing && `Missing: ${info.missingReason}.`}
        </li>
      ))}
    </ul>
  );
}
