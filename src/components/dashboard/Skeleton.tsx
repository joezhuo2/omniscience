import { GlassSkeleton } from '@crystalflux/skeleton-glass';

export function Skeleton({ className = '' }: { className?: string }) {
  return (
    <div data-testid="skeleton" aria-hidden className={className}>
      <GlassSkeleton height="100%" />
    </div>
  );
}
