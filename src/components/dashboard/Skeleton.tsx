/** Placeholder block shown while detail data loads (about one tick after expanding). */
export function Skeleton({ className = '' }: { className?: string }) {
  return (
    <div
      data-testid="skeleton"
      aria-hidden
      className={`animate-pulse rounded-md bg-slate-700/40 ${className}`}
    />
  );
}
