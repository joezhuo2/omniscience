import { ArrowDownUp, Cpu, Gpu, HardDrive, MemoryStick, Microchip } from 'lucide-react';
import type { MetricKey } from '../lib/levels';

const ICONS = {
  cpu: Cpu,
  ram: MemoryStick,
  igpu: Microchip,
  dgpu: Gpu,
  disk: HardDrive,
  net: ArrowDownUp,
} as const;

export function RingIcon({ metric, size }: { metric: MetricKey; size: number }) {
  const Icon = ICONS[metric];
  return <Icon size={size} strokeWidth={2} aria-hidden />;
}
