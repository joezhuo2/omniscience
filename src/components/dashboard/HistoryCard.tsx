import { useEffect, useState, type MouseEvent } from 'react';
import { invoke } from '../../lib/ipc';
import {
  RANGES,
  areaPath,
  frame,
  lastIndex,
  linePath,
  rangeFor,
  shapeCharts,
  type Frame,
  type HistorySample,
  type ShapedChart,
} from '../../lib/history';
import { useSettingsStore } from '../../settings';
import { Card } from './Card';
import { Segmented } from '../Segmented';

const REFRESH_MS = 60_000;
const COLORS = ['var(--color-chart-1)', 'var(--color-chart-2)', 'var(--color-chart-3)'];

function clock(ts: number, long: boolean): string {
  return new Date(ts).toLocaleString(undefined, {
    ...(long ? { month: 'short', day: 'numeric' } : {}),
    hour: '2-digit',
    minute: '2-digit',
  });
}

function HistoryChart({
  chart,
  frame: f,
  hover,
  onHover,
}: {
  chart: ShapedChart;
  frame: Frame;
  hover: number | null;
  onHover: (index: number | null) => void;
}) {
  const width = Math.max(f.count - 1, 1);
  const at = hover ?? lastIndex(chart);
  const long = f.count * f.step >= 24 * 3_600_000;
  const move = (event: MouseEvent<SVGSVGElement>) => {
    const box = event.currentTarget.getBoundingClientRect();
    const ratio = box.width > 0 ? (event.clientX - box.left) / box.width : 0;
    onHover(Math.min(f.count - 1, Math.max(0, Math.round(ratio * width))));
  };
  return (
    <figure className="m-0 min-w-0" data-testid="history-chart">
      <figcaption className="flex items-baseline justify-between gap-2 text-[10px]">
        <span className="text-island-muted">{chart.title}</span>
        <span className="flex min-w-0 gap-2 truncate text-island-text tabular-nums">
          {chart.series.map((s, i) => {
            const v = at == null ? null : s.values[at];
            return (
              <span key={s.label} className="flex items-center gap-1">
                <span
                  aria-hidden
                  className="inline-block h-1.5 w-1.5 rounded-full"
                  style={{ background: COLORS[i] }}
                />
                {chart.series.length > 1 && <span className="text-island-muted">{s.label}</span>}
                {v == null ? '--' : chart.format(v)}
              </span>
            );
          })}
        </span>
      </figcaption>
      <svg
        role="img"
        aria-label={`${chart.title} history`}
        viewBox={`0 0 ${width} 100`}
        preserveAspectRatio="none"
        className="mt-1 block h-14 w-full overflow-visible"
        onMouseMove={move}
        onMouseLeave={() => onHover(null)}
      >
        <line
          x1={0}
          x2={width}
          y1={100}
          y2={100}
          stroke="var(--color-island-faint)"
          vectorEffect="non-scaling-stroke"
        />
        {chart.series.map((s, i) => (
          <g key={s.label}>
            {i === 0 && (
              <path d={areaPath(s.values, chart.max)} fill={COLORS[i]} fillOpacity={0.15} />
            )}
            <path
              d={linePath(s.values, chart.max)}
              fill="none"
              stroke={COLORS[i]}
              strokeWidth={1.5}
              strokeLinejoin="round"
              vectorEffect="non-scaling-stroke"
            />
          </g>
        ))}
        {hover != null && (
          <line
            x1={hover}
            x2={hover}
            y1={0}
            y2={100}
            stroke="var(--color-island-muted)"
            vectorEffect="non-scaling-stroke"
          />
        )}
      </svg>
      <div className="relative flex justify-between gap-2 text-[9px] whitespace-nowrap text-island-muted tabular-nums">
        <span className={hover != null ? 'invisible' : ''}>{clock(f.from, long)}</span>
        <span className={hover != null ? 'invisible' : ''}>
          {clock(f.from + f.count * f.step, long)}
        </span>
        {hover != null && (
          <span className="absolute inset-x-0 text-center text-island-text">
            {clock(f.from + hover * f.step, long)}
          </span>
        )}
      </div>
    </figure>
  );
}

export function HistoryCard() {
  const retention = useSettingsStore((s) => s.settings?.history_hours ?? 24);
  const loaded = useSettingsStore((s) => s.settings != null);
  const load = useSettingsStore((s) => s.load);
  useEffect(() => {
    if (!loaded) void load();
  }, [loaded, load]);
  const [chosen, setChosen] = useState(24);
  const range = rangeFor(chosen, retention);
  const [data, setData] = useState<{ now: number; samples: HistorySample[] } | null>(null);
  const [hover, setHover] = useState<number | null>(null);
  useEffect(() => {
    let alive = true;
    const fetch = () => {
      const now = Date.now();
      invoke<HistorySample[] | null>('get_telemetry_history', {
        sinceMs: frame(range.hours, now).from,
      })
        .then((samples) => alive && setData({ now, samples: samples ?? [] }))
        .catch(() => alive && setData({ now, samples: [] }));
    };
    fetch();
    const timer = setInterval(fetch, REFRESH_MS);
    return () => {
      alive = false;
      clearInterval(timer);
    };
  }, [range.hours]);
  const f = frame(range.hours, data?.now ?? 0);
  const charts = data ? shapeCharts(data.samples, f) : [];
  const selector = (
    <Segmented
      label="History range"
      items={RANGES.map((r) => ({
        value: r.hours,
        label: r.label,
        disabled: r.hours > retention,
        tip: r.hours > retention ? 'Longer than the history kept (see Settings)' : undefined,
      }))}
      value={range.hours}
      onChange={setChosen}
    />
  );
  return (
    <Card
      section="history"
      title="History"
      aside={selector}
      summary={<span className="text-[11px] text-island-muted">{range.label}</span>}
    >
      {!data ? (
        <div aria-busy className="h-14" />
      ) : charts.length === 0 ? (
        <p className="text-[11px] text-island-muted">
          Collecting history. The first point appears after a minute.
        </p>
      ) : (
        <div className="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-3">
          {charts.map((chart) => (
            <HistoryChart
              key={`${range.label}-${chart.title}`}
              chart={chart}
              frame={f}
              hover={hover}
              onHover={setHover}
            />
          ))}
        </div>
      )}
    </Card>
  );
}
