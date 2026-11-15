import { Segmented } from '../Segmented';
import { useId, useState } from 'react';
import { ConfigView } from '../config/ConfigView';
import { BatteryCard } from './BatteryCard';
import { CpuHeatmap } from './CpuHeatmap';
import { FanCard } from './FanCard';
import { GpuPowerCard } from './GpuPowerCard';
import { HistoryCard } from './HistoryCard';
import { isEmbedded } from '../../lib/ipc';
import { useSettingsStore } from '../../settings';
import type { SectionId } from '../../lib/sections';
import { useTelemetryStore } from '../../store';
import { SectionLayout } from './SectionLayout';
import { Header } from './Header';
import { MetricStrip } from './MetricStrip';
import { NetworkCard } from './NetworkCard';
import { ProcessList } from './ProcessList';
import { SensorDetails } from './SensorDetails';
import { Toasts } from './Toasts';

/** Expanded view. Mounted on expand and unmounted on collapse. */
const SECTIONS = {
  'gpu-power': GpuPowerCard,
  battery: BatteryCard,
  fans: FanCard,
  'cpu-cores': CpuHeatmap,
  network: NetworkCard,
  processes: ProcessList,
  history: HistoryCard,
};

export function Dashboard({
  onSettings,
  onStyle,
  pinned,
  onTogglePin,
  showSystemUpdates = 0,
}: {
  onSettings?: () => void;
  onStyle?: () => void;
  pinned?: boolean;
  onTogglePin?: () => void;
  showSystemUpdates?: number;
}) {
  const embedded = isEmbedded();
  const configEnabled = useSettingsStore((s) => !embedded && (s.settings?.config_enabled ?? false));
  const [chosenTab, setTab] = useState<'info' | 'config'>(
    showSystemUpdates > 0 ? 'config' : 'info',
  );
  const tab = configEnabled ? chosenTab : 'info';
  const [configOpened, setConfigOpened] = useState(showSystemUpdates > 0);
  const selectTab = (name: 'info' | 'config') => {
    if (name === 'config') setConfigOpened(true);
    setTab(name);
  };
  const id = useId();
  const hasBattery = useTelemetryStore((state) => state.snapshot?.detail?.battery !== null);
  const hasFans = useTelemetryStore((state) => state.snapshot?.detail?.fans != null);
  const hiddenSections: SectionId[] = [];
  if (!hasBattery) hiddenSections.push('battery');
  if (!hasFans) hiddenSections.push('fans');
  return (
    <div data-testid="dashboard" className="flex flex-col gap-2 pt-1">
      <Header onSettings={onSettings} onStyle={onStyle} pinned={pinned} onTogglePin={onTogglePin} />
      {configEnabled && (
        <div className="card-surface rounded-[10px]">
          <Segmented
            role="tab"
            label="Dashboard"
            className="dashboard-tabs"
            items={[
              { value: 'info', label: 'info' },
              { value: 'config', label: 'config' },
            ]}
            value={tab}
            onChange={selectTab}
          />
        </div>
      )}
      <div id={`${id}-info-panel`} role="tabpanel" aria-label="info" hidden={tab !== 'info'}>
        <div className="flex flex-col gap-2">
          <div className="card-surface rounded-xl px-1 py-2">
            <MetricStrip />
          </div>
          <SensorDetails />
          <div className={embedded ? 'embed-grid' : 'contents'}>
            <SectionLayout hidden={hiddenSections} grid={embedded}>
              {(id) => {
                const Component = SECTIONS[id];
                return <Component />;
              }}
            </SectionLayout>
          </div>
        </div>
      </div>
      {configEnabled && (
        <div
          id={`${id}-config-panel`}
          role="tabpanel"
          aria-label="config"
          hidden={tab !== 'config'}
        >
          {configOpened && <ConfigView key={showSystemUpdates} active={tab === 'config'} />}
        </div>
      )}
      <Toasts />
    </div>
  );
}
