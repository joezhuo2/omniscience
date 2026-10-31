import { DriverUpdatesCard } from './DriverUpdatesCard';

export function ConfigView({ active = true }: { active?: boolean }) {
  return (
    <div className="flex flex-col gap-2">
      <DriverUpdatesCard active={active} />
    </div>
  );
}
