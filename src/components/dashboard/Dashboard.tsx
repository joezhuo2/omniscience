import { CpuHeatmap } from './CpuHeatmap';
import { GpuPowerCard } from './GpuPowerCard';
import { Header } from './Header';
import { MetricStrip } from './MetricStrip';
import { ProcessList } from './ProcessList';
import { Toasts } from './Toasts';

/** Expanded view. Mounted on expand and unmounted on collapse. */
export function Dashboard({ onSettings }: { onSettings?: () => void }) {
  return (
    <div data-testid="dashboard" className="flex flex-col gap-2 px-2.5 pt-1 pb-2.5">
      <Header onSettings={onSettings} />
      <MetricStrip />
      <GpuPowerCard />
      <CpuHeatmap />
      <ProcessList />
      <Toasts />
    </div>
  );
}
