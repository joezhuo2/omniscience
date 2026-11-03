import { BatterySaverCard } from './BatterySaverCard';
import { CpuBoostCard } from './CpuBoostCard';
import { DriverUpdatesCard } from './DriverUpdatesCard';
import { DisplayCard } from './DisplayCard';
import { GpuModeCard } from './GpuModeCard';
import { PowerLimitsCard } from './PowerLimitsCard';
import { PowerModeCard } from './PowerModeCard';
import { SectionToggleProvider } from './SectionToggle';
import { UndoButton } from './UndoButton';

export function ConfigView({ active = true }: { active?: boolean }) {
  return (
    <div className="flex flex-col gap-2">
      <div className="flex justify-end">
        <UndoButton />
      </div>
      <SectionToggleProvider section="driver-updates" active={active}>
        {(on) => <DriverUpdatesCard active={on} />}
      </SectionToggleProvider>
      <SectionToggleProvider section="gpu-mode" active={active}>
        {(on) => <GpuModeCard active={on} />}
      </SectionToggleProvider>
      <SectionToggleProvider section="power-limits" active={active}>
        {(on) => <PowerLimitsCard active={on} />}
      </SectionToggleProvider>
      <SectionToggleProvider section="cpu-boost" active={active}>
        {(on) => <CpuBoostCard active={on} />}
      </SectionToggleProvider>
      <SectionToggleProvider section="power-mode" active={active}>
        {(on) => <PowerModeCard active={on} />}
      </SectionToggleProvider>
      <SectionToggleProvider section="battery-saver" active={active}>
        {(on) => <BatterySaverCard active={on} />}
      </SectionToggleProvider>
      <SectionToggleProvider section="display" active={active}>
        {(on) => <DisplayCard active={on} />}
      </SectionToggleProvider>
    </div>
  );
}
