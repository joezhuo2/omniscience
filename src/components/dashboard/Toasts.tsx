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
          className="block w-full rounded-lg border border-red-400/30 bg-red-950/80 px-2.5 py-1.5 text-left text-[11px] text-red-100"
          onClick={() => dismiss(t.id)}
        >
          {t.text}
        </button>
      ))}
    </div>
  );
}
