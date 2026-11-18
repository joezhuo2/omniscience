import { Pencil, PictureInPicture2, Pin, Settings } from 'lucide-react';
import { StaleDot } from './StaleDot';

/** Dashboard header. */
export function Header({
  onSettings,
  onStyle,
  pinned = false,
  onTogglePin,
  onDetach,
}: {
  onSettings?: () => void;
  onStyle?: () => void;
  pinned?: boolean;
  onTogglePin?: () => void;
  onDetach?: () => void;
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
      {onDetach && (
        <button
          aria-label="Open in a window"
          className="rounded-md p-1 text-island-muted hover:bg-island-track/50 hover:text-island-text"
          onClick={onDetach}
        >
          <PictureInPicture2 size={14} aria-hidden />
        </button>
      )}
      {onStyle && (
        <button
          aria-label="Appearance"
          className="rounded-md p-1 text-island-muted hover:bg-island-track/50 hover:text-island-text"
          onClick={onStyle}
        >
          <Pencil size={14} aria-hidden />
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
