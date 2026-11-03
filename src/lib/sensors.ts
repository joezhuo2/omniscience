import { CPU_TEMP_SOURCE } from './metrics';
import type { CoreMetrics } from '../types/telemetry';

export type SensorKey =
  'cpu' | 'cpu_temp' | 'ram' | 'disk' | 'net' | 'igpu' | 'igpu_temp' | 'dgpu' | 'dgpu_temp';

export interface SensorInfo {
  label: string;
  source: string;
  proxy: boolean;
  missingReason: string;
}

export const SENSORS: Record<SensorKey, SensorInfo> = {
  cpu: {
    label: 'CPU usage',
    source: 'Operating system processor counters, averaged over all cores',
    proxy: false,
    missingReason: 'The operating system did not report processor load',
  },
  cpu_temp: {
    label: 'CPU temperature',
    source: CPU_TEMP_SOURCE,
    proxy: true,
    missingReason: 'This machine exposes no ACPI thermal zone, or reading it needs elevated access',
  },
  ram: {
    label: 'Memory',
    source: 'Operating system memory counters',
    proxy: false,
    missingReason: 'The operating system did not report memory use',
  },
  disk: {
    label: 'Disk activity',
    source: 'Operating system disk activity counters',
    proxy: false,
    missingReason: 'Disk performance counters are unavailable or disabled',
  },
  net: {
    label: 'Network',
    source: 'Byte counters of the network adapters, compared between ticks',
    proxy: false,
    missingReason: 'No network adapter reported counters',
  },
  igpu: {
    label: 'Integrated GPU usage',
    source: 'GPU engine utilisation counters of the integrated adapter',
    proxy: false,
    missingReason: 'No integrated GPU was found or its counters are unavailable',
  },
  igpu_temp: {
    label: 'Integrated GPU temperature',
    source: 'Integrated GPU driver sensor',
    proxy: false,
    missingReason: 'The driver does not expose a temperature for this GPU',
  },
  dgpu: {
    label: 'Discrete GPU usage',
    source: 'GPU engine utilisation counters of the discrete adapter',
    proxy: false,
    missingReason: 'No discrete GPU was found, or it is powered down',
  },
  dgpu_temp: {
    label: 'Discrete GPU temperature',
    source: 'Discrete GPU driver sensor',
    proxy: false,
    missingReason: 'The driver does not expose a temperature, or the GPU is powered down',
  },
};

export type SensorState = 'ok' | 'missing' | 'absent';

export function sensorStates(core: CoreMetrics): Record<SensorKey, SensorState> {
  const temp = (present: boolean, value: number | null): SensorState =>
    value != null ? 'ok' : present ? 'missing' : 'absent';
  return {
    cpu: 'ok',
    cpu_temp: temp(true, core.cpu_temp_c),
    ram: 'ok',
    disk: core.disk ? 'ok' : 'missing',
    net: 'ok',
    igpu: core.igpu ? 'ok' : 'absent',
    igpu_temp: temp(core.igpu != null, core.igpu_temp_c),
    dgpu: core.dgpu ? 'ok' : 'absent',
    dgpu_temp: temp(core.dgpu != null, core.dgpu_temp_c),
  };
}
