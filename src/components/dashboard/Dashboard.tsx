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
  const hasBattery = useTelemetryStore((state) => state.snapshot?.detail?.battery !== null);
  return (
    <div data-testid="dashboard" className="flex flex-col gap-2 pt-1">
      <Header onSettings={onSettings} />
      <MetricStrip />
      <SectionLayout hidden={hasBattery ? [] : ['battery']}>
        {(id) => {
          const Component = SECTIONS[id];
          return <Component />;
        }}
      </SectionLayout>
      <Toasts />
    </div>
  );
}
