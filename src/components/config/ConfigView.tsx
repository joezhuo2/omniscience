import { DriverUpdatesCard } from './DriverUpdatesCard';
import { DisplayCard } from './DisplayCard';
import { GpuModeCard } from './GpuModeCard';

export function ConfigView({ active = true }: { active?: boolean }) {
  return (
    <div className="flex flex-col gap-2">
      <DriverUpdatesCard active={active} />
      <GpuModeCard active={active} />
      <DisplayCard active={active} />
    </div>
  );
}
