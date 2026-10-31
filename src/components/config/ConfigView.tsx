import { CpuBoostCard } from './CpuBoostCard';
import { DriverUpdatesCard } from './DriverUpdatesCard';
import { GpuModeCard } from './GpuModeCard';
import { PowerLimitsCard } from './PowerLimitsCard';

export function ConfigView({ active = true }: { active?: boolean }) {
  return (
    <div className="flex flex-col gap-2">
      <DriverUpdatesCard active={active} />
      <GpuModeCard active={active} />
      <PowerLimitsCard active={active} />
      <CpuBoostCard active={active} />
    </div>
  );
}
