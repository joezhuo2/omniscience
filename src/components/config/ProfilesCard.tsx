import { FeedbackButton } from '@crystalflux/feedback-button';
import { ChevronDown, ChevronUp, Trash2 } from 'lucide-react';
import { useEffect, useState, type ReactNode } from 'react';
import { invoke } from '../../lib/ipc';
import { useSettingsStore } from '../../settings';
import { useToastStore } from '../../toasts';
import type { Profile } from '../../types/generated/Profile';
import { Segmented } from '../Segmented';
import { Card } from '../dashboard/Card';
import { HotkeyRecorder } from '../settings/SettingsView';

const MAX_PROFILES = 12;
const NONE = '';

type ApplyResult = { applied: string[]; errors: string[] };

const POWER = [
  { value: NONE, label: "Don't change" },
  { value: 'efficiency', label: 'Efficiency' },
  { value: 'balanced', label: 'Balanced' },
  { value: 'performance', label: 'Performance' },
];
const BOOST = [
  { value: NONE, label: "Don't change" },
  { value: '0', label: 'Off' },
  { value: '1', label: 'On' },
  { value: '2', label: 'Aggressive' },
];
const FAN = [
  { value: NONE, label: "Don't change" },
  { value: 'silent', label: 'Silent' },
  { value: 'performance', label: 'Performance' },
  { value: 'turbo', label: 'Turbo' },
];

function Field({
  label,
  value,
  onCommit,
  className = 'w-16',
  placeholder,
}: {
  label: string;
  value: string;
  onCommit: (v: string) => void;
  className?: string;
  placeholder?: string;
}) {
  const [draft, setDraft] = useState(value);
  const commit = () => {
    if (draft.trim() !== value) onCommit(draft.trim());
  };
  return (
    <input
      aria-label={label}
      value={draft}
      placeholder={placeholder}
      spellCheck={false}
      onChange={(e) => setDraft(e.target.value)}
      onBlur={commit}
      onKeyDown={(e) => e.key === 'Enter' && commit()}
      className={`config-input ${className}`}
    />
  );
}

const asNumber = (v: string) => (v === '' ? null : Number(v));
const validNumber = (v: string) => v === '' || Number.isFinite(Number(v));

export function ProfilesCard({ active = true }: { active?: boolean }) {
  const { settings, error, load, update } = useSettingsStore();
  const push = useToastStore((s) => s.push);
  const [open, setOpen] = useState<string | null>(null);
  useEffect(() => {
    if (active && !settings) void load();
  }, [active, settings, load]);
  const profiles = settings?.profiles ?? [];
  const save = (next: Profile[]) => void update({ profiles: next });
  const edit = (id: string, patch: Partial<Profile>) =>
    save(profiles.map((p) => (p.id === id ? { ...p, ...patch } : p)));
  const editNumber = (id: string, key: 'spl_w' | 'sppt_w' | 'refresh_hz', v: string) =>
    validNumber(v) && edit(id, { [key]: asNumber(v) });
  const add = () => {
    const ids = new Set(profiles.map((p) => p.id));
    let n = profiles.length + 1;
    while (ids.has(`profile-${n}`)) n++;
    const id = `profile-${n}`;
    save([
      ...profiles,
      {
        id,
        name: `Profile ${n}`,
        power_mode: null,
        spl_w: null,
        sppt_w: null,
        cpu_boost: null,
        refresh_hz: null,
        fan_mode: null,
        hotkey: '',
        apps: [],
      },
    ]);
    setOpen(id);
  };
  const applyProfile = async (p: Profile) => {
    try {
      const { applied, errors } = await invoke<ApplyResult>('apply_profile', { id: p.id });
      push(
        [`${p.name} applied${applied.length ? `: ${applied.join(', ')}` : ''}`, ...errors].join(
          '\n',
        ),
      );
    } catch (e) {
      push(`${p.name} not applied: ${String(e)}`);
      throw e;
    }
  };
  const row = (label: string, control: ReactNode) => (
    <div className="flex items-center justify-between gap-2 text-xs">
      <span>{label}</span>
      {control}
    </div>
  );
  const numberRow = (
    p: Profile,
    label: string,
    name: string,
    key: 'spl_w' | 'sppt_w' | 'refresh_hz',
  ) =>
    row(
      label,
      <Field
        key={String(p[key])}
        label={`${p.name} ${name}`}
        placeholder="—"
        value={p[key] === null ? '' : String(p[key])}
        onCommit={(v) => editNumber(p.id, key, v)}
      />,
    );
  return (
    <Card section="profiles" title="Profiles">
      <ul className="flex flex-col gap-2">
        {profiles.map((p) => {
          const expanded = open === p.id;
          return (
            <li key={p.id} className="flex flex-col gap-2">
              <div className="flex items-center gap-1">
                <button
                  type="button"
                  className="config-button"
                  aria-label={`Edit ${p.name}`}
                  aria-expanded={expanded}
                  onClick={() => setOpen(expanded ? null : p.id)}
                >
                  {expanded ? (
                    <ChevronUp size={12} aria-hidden />
                  ) : (
                    <ChevronDown size={12} aria-hidden />
                  )}
                </button>
                <span className="min-w-0 flex-1 truncate text-xs">{p.name}</span>
                {settings?.active_profile === p.id && (
                  <span className="text-[10px] uppercase text-island-muted">Active</span>
                )}
                <FeedbackButton
                  className="config-button"
                  aria-label={`Apply ${p.name}`}
                  action={() => applyProfile(p)}
                >
                  Apply
                </FeedbackButton>
                <button
                  type="button"
                  className="config-button"
                  aria-label={`Delete ${p.name}`}
                  onClick={() => save(profiles.filter((q) => q.id !== p.id))}
                >
                  <Trash2 size={12} aria-hidden />
                </button>
              </div>
              {expanded && (
                <div className="flex flex-col gap-2 pl-2">
                  {row(
                    'Name',
                    <Field
                      key={p.name}
                      label={`${p.name} name`}
                      className="w-32"
                      value={p.name}
                      onCommit={(name) => name && edit(p.id, { name })}
                    />,
                  )}
                  {row(
                    'Power mode',
                    <Segmented
                      label={`${p.name} power mode`}
                      items={POWER}
                      value={p.power_mode ?? NONE}
                      onChange={(v) =>
                        edit(p.id, { power_mode: (v || null) as Profile['power_mode'] })
                      }
                    />,
                  )}
                  {row(
                    'CPU boost',
                    <Segmented
                      label={`${p.name} CPU boost`}
                      items={BOOST}
                      value={p.cpu_boost === null ? NONE : String(p.cpu_boost)}
                      onChange={(v) => edit(p.id, { cpu_boost: v === NONE ? null : Number(v) })}
                    />,
                  )}
                  {numberRow(p, 'Sustained limit (SPL, W)', 'SPL watts', 'spl_w')}
                  {numberRow(p, 'Boost limit (SPPT, W)', 'SPPT watts', 'sppt_w')}
                  {numberRow(p, 'Refresh rate (Hz)', 'refresh rate Hz', 'refresh_hz')}
                  {row(
                    'Fan mode',
                    <Segmented
                      label={`${p.name} fan mode`}
                      items={FAN}
                      value={p.fan_mode ?? NONE}
                      onChange={(v) => edit(p.id, { fan_mode: (v || null) as Profile['fan_mode'] })}
                    />,
                  )}
                  {row(
                    'Hotkey',
                    <span className="flex items-center gap-1">
                      <HotkeyRecorder
                        label={`${p.name} hotkey`}
                        value={p.hotkey}
                        onChange={(hotkey) => edit(p.id, { hotkey })}
                      />
                      <button
                        type="button"
                        className="config-button"
                        aria-label={`Clear ${p.name} hotkey`}
                        disabled={!p.hotkey}
                        onClick={() => edit(p.id, { hotkey: '' })}
                      >
                        Clear
                      </button>
                    </span>,
                  )}
                  <div className="flex flex-col gap-1 text-xs">
                    <span>Switch to this profile while these apps run</span>
                    <Field
                      key={p.apps.join(',')}
                      label={`${p.name} apps`}
                      className="w-full"
                      placeholder="game.exe, editor.exe"
                      value={p.apps.join(', ')}
                      onCommit={(v) =>
                        edit(p.id, {
                          apps: v
                            .split(',')
                            .map((a) => a.trim())
                            .filter(Boolean),
                        })
                      }
                    />
                  </div>
                </div>
              )}
            </li>
          );
        })}
      </ul>
      <button
        type="button"
        className="config-button mt-2"
        disabled={!settings || profiles.length >= MAX_PROFILES}
        onClick={add}
      >
        Add profile
      </button>
      {error && (
        <p role="alert" className="mt-2 text-xs text-amber-400 break-words">
          {error}
        </p>
      )}
    </Card>
  );
}
