import { Settings } from 'lucide-react';
import { StaleDot } from './StaleDot';

/** Dashboard header. */
export function Header({ onSettings }: { onSettings?: () => void }) {
  return (
    <header className="flex h-9 items-center gap-2 px-1">
      <span className="flex-1 text-xs font-semibold tracking-wide text-island-text">
        Omniscience
      </span>
      <StaleDot />
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
