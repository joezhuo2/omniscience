import { useLayoutEffect, useRef, useState, type KeyboardEvent, type ReactNode } from 'react';
import { Tip } from './Tip';

type Item<T> = { value: T; label: ReactNode; disabled?: boolean; tip?: ReactNode };

export function Segmented<T extends string | number>({
  items,
  value,
  onChange,
  label,
  role = 'radio',
  className = '',
}: {
  items: Item<T>[];
  value: T;
  onChange: (value: T) => void;
  label: string;
  role?: 'radio' | 'tab';
  className?: string;
}) {
  const track = useRef<HTMLDivElement>(null);
  const buttons = useRef<(HTMLButtonElement | null)[]>([]);
  const index = items.findIndex((item) => item.value === value);
  const last = useRef(index);
  const [thumb, setThumb] = useState<{ left: number; right: number; dir: string }>();

  useLayoutEffect(() => {
    const host = track.current;
    if (!host) return;
    const dir = index > last.current ? 'fwd' : index < last.current ? 'back' : '';
    last.current = index;
    const measure = () => {
      const el = buttons.current[index];
      if (!el || !host.offsetWidth) return;
      const box = el.getBoundingClientRect();
      const outer = host.getBoundingClientRect();
      setThumb({ left: box.left - outer.left, right: outer.right - box.right, dir });
    };
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(host);
    return () => observer.disconnect();
  }, [index, items.length]);

  const onKeyDown = (event: KeyboardEvent) => {
    const enabled = items.flatMap((item, i) => (item.disabled ? [] : [i]));
    const at = enabled.indexOf(index);
    const n = enabled.length;
    const next =
      event.key === 'Home'
        ? enabled[0]
        : event.key === 'End'
          ? enabled[n - 1]
          : event.key === 'ArrowRight'
            ? enabled[(at + 1) % n]
            : event.key === 'ArrowLeft'
              ? enabled[(at - 1 + n) % n]
              : undefined;
    if (next === undefined) return;
    event.preventDefault();
    onChange(items[next]!.value);
    buttons.current[next]?.focus();
  };

  return (
    <div
      ref={track}
      onKeyDown={onKeyDown}
      role={role === 'radio' ? 'radiogroup' : 'tablist'}
      aria-label={label}
      className={`segmented relative flex rounded-md bg-island-track p-0.5 text-[10px] ${className}`}
    >
      {thumb && (
        <span
          aria-hidden
          className="segmented-thumb"
          data-dir={thumb.dir}
          style={{ left: thumb.left, right: thumb.right }}
        />
      )}
      {items.map((item, i) => {
        const active = i === index;
        return (
          <Tip key={String(item.value)} content={item.tip}>
            <button
              ref={(el) => {
                buttons.current[i] = el;
              }}
              type="button"
              role={role}
              {...(role === 'radio' ? { 'aria-checked': active } : { 'aria-selected': active })}
              tabIndex={active ? 0 : -1}
              disabled={item.disabled}
              onClick={() => onChange(item.value)}
              className={`relative rounded px-1.5 py-0.5 tabular-nums transition-colors disabled:opacity-35 ${
                active ? 'text-island-text' : 'text-island-muted hover:text-island-text'
              }`}
            >
              {item.label}
            </button>
          </Tip>
        );
      })}
    </div>
  );
}
