import { BatterySaverCard } from './BatterySaverCard';
import { CpuBoostCard } from './CpuBoostCard';
import { DriverUpdatesCard } from './DriverUpdatesCard';
import { DisplayCard } from './DisplayCard';
import { GpuModeCard } from './GpuModeCard';
import { PowerLimitsCard } from './PowerLimitsCard';
import { PowerModeCard } from './PowerModeCard';

export function ConfigView({ active = true }: { active?: boolean }) {
  return (
    <div className="flex flex-col gap-2">
      <DriverUpdatesCard active={active} />
      <GpuModeCard active={active} />
      <PowerLimitsCard active={active} />
      <CpuBoostCard active={active} />
      <PowerModeCard active={active} />
      <BatterySaverCard active={active} />
      <DisplayCard active={active} />
    </div>
  );
}
