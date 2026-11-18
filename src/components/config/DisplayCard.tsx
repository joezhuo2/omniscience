import { SmoothRange } from '@crystalflux/smooth-range';
import { GlassSwitch } from '@crystalflux/switch-glass';
import { useEffect, useRef, useState } from 'react';
import { useToastStore } from '../../toasts';
import { Card } from '../dashboard/Card';
import { useNativeControl, type Undo } from './useNativeControl';
import { Tip } from '../Tip';

export type DisplayToggles = {
  hdr: boolean | null;
  hdr_supported: boolean;
  night_light: boolean | null;
  brightness: number | null;
  error: string | null;
};

export type DisplayStatus = {
  device: string | null;
  current: { width: number; height: number; hz: number } | null;
  resolutions: [number, number][];
  rates: number[];
  auto: boolean;
  supported: boolean;
  error: string | null;
};

export function DisplayCard({ active = true }: { active?: boolean }) {
  const { status, busy, error, apply } = useNativeControl<DisplayStatus>(
    'get_display_mode',
    active,
  );
  const toggles = useNativeControl<DisplayToggles>('get_display_toggles', active);
  const push = useToastStore((st) => st.push);
  const toggleError = toggles.error || toggles.status?.error;
  useEffect(() => {
    if (toggleError) push(toggleError);
  }, [toggleError, push]);
  const [draft, setDraft] = useState<number | null>(null);
  const brightness = toggles.status?.brightness ?? null;
  const commit = async () => {
    if (draft === null) return;
    await toggles.apply(
      'set_brightness',
      { percent: draft },
      brightness !== null
        ? { label: 'Brightness', command: 'set_brightness', args: { percent: brightness } }
        : undefined,
    );
    setDraft(null);
  };
  const latestCommit = useRef(commit);
  useEffect(() => {
    latestCommit.current = commit;
  });
  const toggle = (label: string, command: string, value: boolean) => (
    <GlassSwitch
      className="text-xs"
      checked={value}
      disabled={toggles.busy}
      label={label}
      onChange={(on) =>
        void toggles.apply(command, { on }, { label, command, args: { on: value } })
      }
    />
  );
  const disabled = !status?.supported || busy;
  const current = status?.current;
  const undoRefresh: Undo | undefined =
    status?.auto || current
      ? {
          label: 'Refresh rate',
          command: 'set_display_refresh',
          args: { hz: status?.auto ? null : current?.hz },
        }
      : undefined;
  return (
    <Card section="display" title="Display">
      {status?.device && <p className="mb-2 text-xs text-island-muted">{status.device}</p>}
      <p className="mb-1 text-[11px] text-island-muted">Refresh rate</p>
      <div className="flex flex-wrap gap-1" role="group" aria-label="Refresh rate">
        <Tip content="Lowest rate of at least 60 Hz on battery, highest when plugged in">
          <button
            type="button"
            className="config-button"
            aria-pressed={!!status?.auto}
            disabled={disabled}
            onClick={() => void apply('set_display_refresh', { hz: null }, undoRefresh)}
          >
            Auto
          </button>
        </Tip>
        {status?.rates.map((hz) => (
          <button
            type="button"
            key={hz}
            className="config-button"
            aria-pressed={!status.auto && current?.hz === hz}
            disabled={disabled}
            onClick={() => void apply('set_display_refresh', { hz }, undoRefresh)}
          >
            {hz} Hz
          </button>
        ))}
      </div>
      <p className="mb-1 mt-2 text-[11px] text-island-muted">Resolution</p>
      <div className="flex flex-wrap gap-1" role="group" aria-label="Resolution">
        {status?.resolutions.map(([width, height]) => (
          <button
            type="button"
            key={`${width}x${height}`}
            className="config-button"
            aria-pressed={current?.width === width && current.height === height}
            disabled={disabled}
            onClick={() =>
              void apply(
                'set_display_resolution',
                { width, height },
                current
                  ? {
                      label: 'Resolution',
                      command: 'set_display_resolution',
                      args: { width: current.width, height: current.height },
                    }
                  : undefined,
              )
            }
          >
            {width}×{height}
          </button>
        ))}
      </div>
      {(toggles.status?.hdr_supported ||
        toggles.status?.night_light != null ||
        brightness !== null) && (
        <div className="mt-2 flex flex-col gap-2">
          {toggles.status?.hdr_supported && toggle('HDR', 'set_hdr', toggles.status.hdr === true)}
          {toggles.status?.night_light != null &&
            toggle('Night light', 'set_night_light', toggles.status.night_light)}
          {brightness !== null && (
            <label className="block text-xs">
              <span className="flex justify-between">
                <span>Brightness</span>
                <span>{draft ?? brightness}%</span>
              </span>
              <SmoothRange
                className="w-full"
                min={0}
                max={100}
                step={1}
                value={draft ?? brightness}
                disabled={toggles.busy}
                aria-label="Brightness"
                onValue={setDraft}
                onPointerUp={() => setTimeout(() => void latestCommit.current(), 350)}
                onKeyUp={() => void commit()}
              />
            </label>
          )}
        </div>
      )}
      <p className="mt-2 text-xs" role="status" aria-live="polite">
        {busy
          ? 'Applying display mode…'
          : !status
            ? 'Reading display modes…'
            : current
              ? `Current: ${current.width}×${current.height} at ${current.hz} Hz.`
              : 'Display modes are unavailable.'}
      </p>
      {(error || status?.error) && (
        <p role="alert" className="mt-2 text-xs text-amber-400 break-words">
          {error || status?.error}
        </p>
      )}
    </Card>
  );
}
