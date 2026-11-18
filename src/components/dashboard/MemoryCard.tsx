import { formatBytes } from '../../lib/format';
import { ACCENTS } from '../../lib/levels';
import { useTelemetryStore } from '../../store';
import { Card } from './Card';
import { Skeleton } from './Skeleton';

const gb = (b: number) => `${(b / 1024 ** 3).toFixed(1)} GB`;

export function MemoryCard() {
  const memory = useTelemetryStore((s) => s.snapshot?.detail?.memory);
  const loaded = useTelemetryStore((s) => s.snapshot?.detail != null);
  const limit = memory?.commit_limit_bytes ?? 0;
  const pct = memory && limit > 0 ? Math.min(100, (memory.committed_bytes / limit) * 100) : 0;
  return (
    <Card
      title="Memory"
      section="memory"
      summary={
        memory && limit > 0 ? (
          <span className="text-[11px] text-island-text tabular-nums">
            {gb(memory.committed_bytes)} / {gb(limit)}
          </span>
        ) : null
      }
    >
      {!loaded ? (
        <Skeleton className="h-6" />
      ) : !memory || limit <= 0 ? (
        <p className="text-[11px] text-island-muted">No memory detail</p>
      ) : (
        <div className="space-y-1">
          <div className="flex items-baseline justify-between gap-2 text-[11px]">
            <span className="text-island-text">Committed</span>
            <span data-testid="memory-committed" className="text-island-muted tabular-nums">
              {gb(memory.committed_bytes)} / {gb(limit)}
            </span>
          </div>
          <div className="h-1 overflow-hidden rounded-full bg-island-track">
            <div
              className="h-full rounded-full transition-[width] duration-500"
              style={{ width: `${pct}%`, background: ACCENTS.ram }}
            />
          </div>
          <div className="flex gap-3 text-[10px] text-island-muted tabular-nums">
            {memory.cached_bytes != null && (
              <span data-testid="memory-cached">Cached {formatBytes(memory.cached_bytes)}</span>
            )}
            {memory.compressed_bytes != null && (
              <span data-testid="memory-compressed">
                Compressed {formatBytes(memory.compressed_bytes)}
              </span>
            )}
          </div>
        </div>
      )}
    </Card>
  );
}
