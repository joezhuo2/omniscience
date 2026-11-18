import { FeedbackButton } from '@crystalflux/feedback-button';
import { useEffect, useId, useState, type MouseEvent } from 'react';
import { invoke } from '../../lib/ipc';
import {
  RANGES,
  areaPath,
  eventIndexes,
  frame,
  lastIndex,
  linePath,
  rangeFor,
  seriesStats,
  shapeCharts,
  zoomFrame,
  type Frame,
  type HistoryEvent,
  type HistorySample,
  type ShapedChart,
} from '../../lib/history';
import { useSettingsStore } from '../../settings';
import { Card } from './Card';
import { Segmented } from '../Segmented';
import { useToastStore } from '../../toasts';

const REFRESH_MS = 60_000;
const MIN_DRAG = 5;
const EVENT_COLORS: Record<HistoryEvent['kind'], string> = {
  power_source: 'var(--color-island-muted)',
  throttle: 'var(--color-island-warn)',
  alert: 'var(--color-island-crit)',
};
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
  events,
  hover,
  drag,
  onHover,
  onDragStart,
}: {
  chart: ShapedChart;
  frame: Frame;
  events: HistoryEvent[];
  hover: number | null;
  drag: { a: number; b: number } | null;
  onHover: (index: number | null) => void;
  onDragStart: (index: number) => void;
}) {
  const descId = useId();
  const marks = eventIndexes(events, f);
  const hoverLabels =
    hover == null
      ? ''
      : marks
          .filter((m) => m.index === hover)
          .map((m) => m.event.label)
          .join(', ');
  const stats = chart.series.flatMap((s) => {
    const st = seriesStats(s.values);
    return st
      ? [
          `${chart.series.length > 1 ? `${s.label} ` : ''}min ${chart.format(st.min)} avg ${chart.format(st.avg)} max ${chart.format(st.max)}`,
        ]
      : [];
  });
  const width = Math.max(f.count - 1, 1);
  const at = hover ?? lastIndex(chart);
  const long = f.count * f.step >= 24 * 3_600_000;
  const indexAt = (event: MouseEvent<SVGSVGElement>) => {
    const box = event.currentTarget.getBoundingClientRect();
    const ratio = box.width > 0 ? (event.clientX - box.left) / box.width : 0;
    return Math.min(f.count - 1, Math.max(0, Math.round(ratio * width)));
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
        aria-describedby={descId}
        viewBox={`0 0 ${width} 100`}
        preserveAspectRatio="none"
        className="mt-1 block h-14 w-full overflow-visible"
        onMouseMove={(event) => onHover(indexAt(event))}
        onMouseDown={(event) => {
          event.preventDefault();
          onDragStart(indexAt(event));
        }}
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
        {marks.map(({ index, event }) => (
          <line
            key={`${event.ts_ms}-${event.kind}-${event.label}`}
            data-testid="history-event"
            x1={index}
            x2={index}
            y1={0}
            y2={100}
            stroke={EVENT_COLORS[event.kind]}
            strokeDasharray="3 3"
            vectorEffect="non-scaling-stroke"
          />
        ))}
        {drag && (
          <rect
            data-testid="history-selection"
            x={Math.min(drag.a, drag.b)}
            y={0}
            width={Math.abs(drag.b - drag.a)}
            height={100}
            fill="var(--color-island-muted)"
            fillOpacity={0.25}
          />
        )}
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
      <span id={descId} className="sr-only">
        {marks.length === 1 ? '1 event shown' : `${marks.length} events shown`}
      </span>
      <div className="relative flex justify-between gap-2 text-[9px] whitespace-nowrap text-island-muted tabular-nums">
        <span className={hover != null ? 'invisible' : ''}>{clock(f.from, long)}</span>
        <span className={hover != null ? 'invisible' : ''}>
          {clock(f.from + f.count * f.step, long)}
        </span>
        {hover != null && (
          <span className="absolute inset-x-0 text-center text-island-text">
            {clock(f.from + hover * f.step, long)}
            {hoverLabels && ` � ${hoverLabels}`}
          </span>
        )}
      </div>
      {stats.length > 0 && (
        <div className="truncate text-[9px] text-island-muted tabular-nums">
          {stats.join(' � ')}
        </div>
      )}
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
  const [data, setData] = useState<{
    now: number;
    samples: HistorySample[];
    events: HistoryEvent[];
  } | null>(null);
  const [hover, setHover] = useState<number | null>(null);
  const [zoom, setZoom] = useState<{ from: number; to: number } | null>(null);
  const [drag, setDrag] = useState<{ a: number; b: number } | null>(null);
  const push = useToastStore((s) => s.push);
  useEffect(() => {
    let alive = true;
    const fetch = () => {
      const now = Date.now();
      const sinceMs = frame(range.hours, now).from;
      Promise.all([
        invoke<HistorySample[] | null>('get_telemetry_history', { sinceMs }),
        invoke<HistoryEvent[] | null>('get_history_events', { sinceMs }).catch(() => null),
      ])
        .then(
          ([samples, events]) =>
            alive && setData({ now, samples: samples ?? [], events: events ?? [] }),
        )
        .catch(() => alive && setData({ now, samples: [], events: [] }));
    };
    fetch();
    const timer = setInterval(fetch, REFRESH_MS);
    return () => {
      alive = false;
      clearInterval(timer);
    };
  }, [range.hours]);
  const f = zoom ? zoomFrame(zoom.from, zoom.to) : frame(range.hours, data?.now ?? 0);
  useEffect(() => {
    if (!drag) return;
    const finish = () => {
      setDrag(null);
      const a = Math.min(drag.a, drag.b);
      const b = Math.max(drag.a, drag.b);
      if (b - a >= MIN_DRAG) setZoom({ from: f.from + a * f.step, to: f.from + (b + 1) * f.step });
    };
    window.addEventListener('mouseup', finish);
    return () => window.removeEventListener('mouseup', finish);
  }, [drag, f.from, f.step]);
  const exportAs = (format: 'csv' | 'json') => async () => {
    try {
      const path = await invoke<string>('export_history', {
        format,
        sinceMs: f.from,
        untilMs: f.from + f.count * f.step,
      });
      push(`Saved to ${path}`);
    } catch (e) {
      push(String(e));
      throw e;
    }
  };
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
      onChange={(hours) => {
        setChosen(hours);
        setZoom(null);
      }}
    />
  );
  const tools = (
    <div className="mb-2 flex flex-wrap items-center gap-2">
      {zoom && (
        <button type="button" className="config-button" onClick={() => setZoom(null)}>
          Reset zoom
        </button>
      )}
      <FeedbackButton
        className="config-button"
        aria-label="Export history as CSV"
        disabled={!data}
        action={exportAs('csv')}
      >
        CSV
      </FeedbackButton>
      <FeedbackButton
        className="config-button"
        aria-label="Export history as JSON"
        disabled={!data}
        action={exportAs('json')}
      >
        JSON
      </FeedbackButton>
    </div>
  );
  return (
    <Card
      section="history"
      title="History"
      aside={selector}
      summary={<span className="text-[11px] text-island-muted">{range.label}</span>}
    >
      {tools}
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
              events={data.events}
              hover={hover}
              drag={drag}
              onHover={(i) => {
                setHover(i);
                if (i != null) setDrag((d) => d && { ...d, b: i });
              }}
              onDragStart={(i) => setDrag({ a: i, b: i })}
            />
          ))}
        </div>
      )}
    </Card>
  );
}
