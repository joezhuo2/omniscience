import { Pill } from './components/Pill';
import { useTelemetry } from './hooks/useTelemetry';

export default function App() {
  useTelemetry();
  return (
    <div className="flex h-full w-full items-start justify-center">
      <div className="glass w-full overflow-hidden rounded-[21px]">
        <Pill />
      </div>
    </div>
  );
}
