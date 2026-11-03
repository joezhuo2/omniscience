import { Pin, Settings } from 'lucide-react';
import { StaleDot } from './StaleDot';

/** Dashboard header. */
export function Header({
  onSettings,
  pinned = false,
  onTogglePin,
}: {
  onSettings?: () => void;
  pinned?: boolean;
  onTogglePin?: () => void;
}) {
  return (
    <header className="flex h-9 items-center gap-2 px-1">
      <span className="flex-1 text-xs font-semibold tracking-wide text-island-text">
        Omniscience
      </span>
      <StaleDot />
      {onTogglePin && (
        <button
          aria-label={pinned ? 'Unpin dashboard' : 'Pin dashboard open'}
          aria-pressed={pinned}
          className={`rounded-md p-1 hover:bg-island-track/50 hover:text-island-text ${pinned ? 'text-island-text' : 'text-island-muted'}`}
          onClick={onTogglePin}
        >
          <Pin size={14} aria-hidden fill={pinned ? 'currentColor' : 'none'} />
        </button>
      )}
      {onSettings && (
        <button
          aria-label="Settings"
          className="rounded-md p-1 text-island-muted hover:bg-island-track/50 hover:text-island-text"
          onClick={onSettings}
        >
          <Settings size={14} aria-hidden />
        </button>
      )}
    </header>
  );
}
