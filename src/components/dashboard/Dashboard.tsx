import { useId, useState } from 'react';
import { ConfigView } from '../config/ConfigView';
import { BatteryCard } from './BatteryCard';
import { CpuHeatmap } from './CpuHeatmap';
import { GpuPowerCard } from './GpuPowerCard';
import { useTelemetryStore } from '../../store';
import { SectionLayout } from './SectionLayout';
import { Header } from './Header';
import { MetricStrip } from './MetricStrip';
import { NetworkCard } from './NetworkCard';
import { ProcessList } from './ProcessList';
import { Toasts } from './Toasts';

/** Expanded view. Mounted on expand and unmounted on collapse. */
const SECTIONS = {
  'gpu-power': GpuPowerCard,
  battery: BatteryCard,
  'cpu-cores': CpuHeatmap,
  network: NetworkCard,
  processes: ProcessList,
};

export function Dashboard({ onSettings }: { onSettings?: () => void }) {
  const [tab, setTab] = useState<'info' | 'config'>('info');
  const [configOpened, setConfigOpened] = useState(false);
  const selectTab = (name: 'info' | 'config') => {
    if (name === 'config') setConfigOpened(true);
    setTab(name);
  };
  const id = useId();
  const hasBattery = useTelemetryStore((state) => state.snapshot?.detail?.battery !== null);
  return (
    <div data-testid="dashboard" className="flex flex-col gap-2 pt-1">
      <Header onSettings={onSettings} />
      <div
        role="tablist"
        aria-label="Dashboard"
        className="flex gap-1 rounded-lg bg-island-hover p-1"
      >
        {(['info', 'config'] as const).map((name, index) => (
          <button
            key={name}
            id={`${id}-${name}-tab`}
            role="tab"
            type="button"
            aria-selected={tab === name}
            aria-controls={`${id}-${name}-panel`}
            tabIndex={tab === name ? 0 : -1}
            onClick={() => selectTab(name)}
            onKeyDown={(event) => {
              const next =
                event.key === 'Home'
                  ? 'info'
                  : event.key === 'End'
                    ? 'config'
                    : event.key === 'ArrowLeft' || event.key === 'ArrowRight'
                      ? index === 0
                        ? 'config'
                        : 'info'
                      : null;
              if (next) {
                event.preventDefault();
                selectTab(next);
                document.getElementById(`${id}-${next}-tab`)?.focus();
              }
            }}
            className={`flex-1 rounded-md px-3 py-1.5 text-xs capitalize transition-colors hover:text-island-text focus-visible:outline-2 focus-visible:outline-island-muted ${tab === name ? 'bg-island-faint text-island-text' : 'text-island-muted'}`}
          >
            {name}
          </button>
        ))}
      </div>
      <div
        id={`${id}-info-panel`}
        role="tabpanel"
        aria-labelledby={`${id}-info-tab`}
        hidden={tab !== 'info'}
      >
        <div className="flex flex-col gap-2">
          <MetricStrip />
          <SectionLayout hidden={hasBattery ? [] : ['battery']}>
            {(id) => {
              const Component = SECTIONS[id];
              return <Component />;
            }}
          </SectionLayout>
        </div>
      </div>
      <div
        id={`${id}-config-panel`}
        role="tabpanel"
        aria-labelledby={`${id}-config-tab`}
        hidden={tab !== 'config'}
      >
        {configOpened && <ConfigView />}
      </div>
      <Toasts />
    </div>
  );
}
