import { formatBytes, formatPercent, formatRate, formatTemp } from '../../lib/format';
import { ACCENTS, CRITICAL, WARNING } from '../../lib/levels';
import { useTelemetryStore } from '../../store';
import type { DriveInfo, PhysicalDisk } from '../../types/telemetry';
import { Card } from './Card';
import { Skeleton } from './Skeleton';
import { Tip } from '../Tip';

function DriveRow({ drive }: { drive: DriveInfo }) {
  const used = Math.max(0, drive.total_bytes - drive.free_bytes);
  const pct = drive.total_bytes > 0 ? Math.min(100, (used / drive.total_bytes) * 100) : 0;
  return (
    <div data-testid="drive" className="space-y-1">
      <div className="flex items-baseline justify-between gap-2 text-[11px]">
        <span className="min-w-0 truncate text-island-text">
          {drive.name}
          {drive.label && <span className="pl-1.5 text-island-muted">{drive.label}</span>}
        </span>
        <span className="shrink-0 text-island-muted tabular-nums">
          {formatBytes(used)} / {formatBytes(drive.total_bytes)}
        </span>
      </div>
      <div className="h-1 overflow-hidden rounded-full bg-island-track">
        <div
          className="h-full rounded-full transition-[width] duration-500"
          style={{ width: `${pct}%`, background: ACCENTS.disk }}
        />
      </div>
      <div className="flex items-baseline justify-between gap-2 text-[10px] text-island-muted tabular-nums">
        <span>{formatBytes(drive.free_bytes)} free</span>
        {(drive.read_bps != null || drive.write_bps != null) && (
          <span>
            {drive.read_bps != null && `R ${formatRate(drive.read_bps)}`}
            {drive.read_bps != null && drive.write_bps != null && ' · '}
            {drive.write_bps != null && `W ${formatRate(drive.write_bps)}`}
          </span>
        )}
      </div>
    </div>
  );
}

function DiskRow({ disk }: { disk: PhysicalDisk }) {
  const health = disk.health;
  const healthy = health == null || health === 'Healthy';
  return (
    <div
      data-testid="disk"
      className="flex items-baseline justify-between gap-2 text-[10px] text-island-muted tabular-nums"
    >
      <Tip content={disk.name}>
        <span className="min-w-0 truncate text-island-text">{disk.name}</span>
      </Tip>
      <span className="shrink-0">
        {[
          disk.kind,
          disk.temp_c != null ? formatTemp(disk.temp_c) : null,
          disk.wear_pct != null ? `Wear ${formatPercent(disk.wear_pct)}` : null,
        ]
          .filter(Boolean)
          .join(' · ')}
        {disk.health && (
          <span
            data-testid="disk-health"
            style={healthy ? undefined : { color: health === 'Unhealthy' ? CRITICAL : WARNING }}
          >
            {' · '}
            {disk.health}
          </span>
        )}
      </span>
    </div>
  );
}

export function DrivesCard() {
  const detail = useTelemetryStore((s) => s.snapshot?.detail);
  const first = detail?.drives[0];
  return (
    <Card
      title="Drives"
      section="drives"
      summary={
        first ? (
          <span className="text-[11px] text-island-text tabular-nums">
            {formatBytes(first.free_bytes)} free
          </span>
        ) : null
      }
    >
      {!detail ? (
        <Skeleton className="h-6" />
      ) : (
        <div className="space-y-2">
          {detail.drives.length === 0 && detail.disks.length === 0 && (
            <p className="text-[11px] text-island-muted">No drives found</p>
          )}
          {detail.drives.map((d) => (
            <DriveRow key={d.name} drive={d} />
          ))}
          {detail.disks.length > 0 && (
            <div className="space-y-0.5">
              {detail.disks.map((d) => (
                <DiskRow key={d.name} disk={d} />
              ))}
            </div>
          )}
        </div>
      )}
    </Card>
  );
}
