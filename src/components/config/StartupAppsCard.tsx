import { GlassSwitch } from '@crystalflux/switch-glass';
import type { StartupApp } from '../../types/telemetry';
import { Card } from '../dashboard/Card';
import { Tip } from '../Tip';
import { useNativeControl } from './useNativeControl';

const SOURCES: Record<StartupApp['source'], string> = {
  user: 'User',
  machine: 'All users',
  folder: 'Startup folder',
};

export function StartupAppsCard({ active = true }: { active?: boolean }) {
  const { status, busy, error, apply } = useNativeControl<StartupApp[]>('get_startup_apps', active);
  return (
    <Card section="startup-apps" title="Startup apps">
      {status?.length === 0 && <p className="text-xs">No startup apps.</p>}
      {status && status.length > 0 && (
        <ul className="space-y-1.5">
          {status.map((app) => (
            <li key={app.id} className="flex items-center justify-between gap-3 text-xs">
              <Tip content={app.command}>
                <span className="min-w-0 flex-1 truncate">{app.name}</span>
              </Tip>
              <span className="shrink-0 text-[11px] text-island-muted">{SOURCES[app.source]}</span>
              <Tip content={app.can_toggle ? undefined : 'Needs administrator rights'}>
                <span>
                  <GlassSwitch
                    checked={app.enabled}
                    disabled={busy || !app.can_toggle}
                    label={<span className="sr-only">{`${app.name} runs at startup`}</span>}
                    onChange={(enabled) =>
                      void apply('set_startup_app_enabled', { id: app.id, enabled })
                    }
                  />
                </span>
              </Tip>
            </li>
          ))}
        </ul>
      )}
      <p className="mt-2 text-xs" role="status" aria-live="polite">
        {busy ? 'Applying startup change…' : !status && !error ? 'Reading startup apps…' : ''}
      </p>
      {error && (
        <p role="alert" className="mt-2 text-xs text-amber-400 break-words">
          {error}
        </p>
      )}
    </Card>
  );
}
