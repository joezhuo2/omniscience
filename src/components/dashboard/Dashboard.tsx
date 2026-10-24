import { BatteryCard } from './BatteryCard';
import { CpuHeatmap } from './CpuHeatmap';
import { GpuPowerCard } from './GpuPowerCard';
import { Header } from './Header';
import { MetricStrip } from './MetricStrip';
import { NetworkCard } from './NetworkCard';
import { ProcessList } from './ProcessList';
import { Toasts } from './Toasts';

/** Expanded view. Mounted on expand and unmounted on collapse. */
export function Dashboard({ onSettings }: { onSettings?: () => void }) {
  return (
    <div data-testid="dashboard" className="flex flex-col gap-2 pt-1">
      <Header onSettings={onSettings} />
      <MetricStrip />
      <GpuPowerCard />
      <BatteryCard />
      <CpuHeatmap />
      <NetworkCard />
      <ProcessList />
      <Toasts />
    </div>
  );
}
