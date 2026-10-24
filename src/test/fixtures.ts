import type { DetailMetrics, Snapshot } from '../types/telemetry';

const metric = (percent: number, level: 'normal' | 'warning' | 'critical' = 'normal') => ({
  percent,
  level,
});

const GB = 1024 ** 3;

export const sampleDetail: DetailMetrics = {
  cores: [
    { id: 0, usage_pct: 12, freq_mhz: 3200, watts: 0.5, level: 'normal' },
    { id: 1, usage_pct: 95, freq_mhz: 5100, watts: 2.1, level: 'critical' },
  ],
  temps: [{ label: 'ACPI THRM', celsius: 48 }],
  gpus: [
    {
      name: 'AMD Radeon(TM) 890M Graphics',
      kind: 'integrated',
      usage_pct: 4,
      mem_used_bytes: 0.6 * GB,
      mem_total_bytes: null,
      temp_c: 45,
      fan_rpm: null,
      throttling: false,
    },
    {
      name: 'NVIDIA GeForce RTX 5060 Laptop GPU',
      kind: 'discrete',
      usage_pct: 12,
      mem_used_bytes: 5.2 * GB,
      mem_total_bytes: 8 * GB,
      temp_c: null,
      fan_rpm: 2100,
      throttling: true,
    },
  ],
  power: { apu: 8.2, cpu: 2.8, gpu: 3.2, npu: null },
  cpu_limit_pct: 0,
  ping_ms: 14,
  net_adapters: [
    { name: 'Wi-Fi', down_bps: 1258291, up_bps: 81920, ipv4: '192.168.1.20' },
    { name: 'vEthernet (WSL)', down_bps: 0, up_bps: 0, ipv4: '172.20.0.1' },
  ],
  processes: [
    { key: 'chrome', name: 'chrome.exe', pids: [10, 11], cpu_pct: 6, mem_bytes: 2 * GB },
    { key: 'code', name: 'Code.exe', pids: [20], cpu_pct: 3, mem_bytes: 3 * GB },
  ],
};

export const sampleSnapshot: Snapshot = {
  timestamp_ms: 0,
  core: {
    cpu: metric(23),
    cpu_mhz: 3249,
    ram: { used_bytes: 12.4 * GB, total_bytes: 31.1 * GB, percent: 40, level: 'normal' },
    disk: metric(3),
    net: { down_bps: 1258291, up_bps: 81920, down_pct: 40, up_pct: 5, level: 'normal' },
    igpu: metric(4),
    dgpu: null,
    package_watts: 8.2,
  },
  detail: null,
};
