import { useToastStore } from '../../toasts';

export function Toasts() {
  const toasts = useToastStore((s) => s.toasts);
  const dismiss = useToastStore((s) => s.dismiss);
  if (toasts.length === 0) return null;
  return (
    <div aria-live="polite" className="space-y-1">
      {toasts.map((t) => (
        <button
          key={t.id}
          role="status"
          className="block w-full rounded-[10px] border-l-[3px] border-island-crit bg-island-hover px-2.5 py-2 text-left text-[12px] text-island-text"
          onClick={() => dismiss(t.id)}
        >
          {t.text}
        </button>
      ))}
    </div>
  );
}
