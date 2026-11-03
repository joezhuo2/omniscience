import { Undo2 } from 'lucide-react';
import { useUndoStore } from '../../undo';

export function UndoButton() {
  const last = useUndoStore((s) => s.entries[s.entries.length - 1]);
  const reverting = useUndoStore((s) => s.reverting);
  const undo = useUndoStore((s) => s.undo);
  return (
    <button
      type="button"
      className="config-button flex items-center gap-1 text-xs"
      disabled={!last || reverting}
      onClick={() => void undo()}
    >
      <Undo2 size={12} aria-hidden />
      {last ? `Undo: ${last.label}` : 'Undo'}
    </button>
  );
}
