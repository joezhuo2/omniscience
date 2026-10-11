/** Keyboard event fields the recorder needs (a subset of KeyboardEvent, for testing). */
export interface KeyLike {
  key: string;
  code: string;
  ctrlKey: boolean;
  metaKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
}

const MODIFIER_KEYS = new Set(['Control', 'Meta', 'Alt', 'Shift', 'AltGraph', 'OS']);

/**
 * Turns a key press into a Tauri accelerator such as `CommandOrControl+Shift+O`. Returns null
 * until a non-modifier key is pressed with at least one modifier (bare keys would hijack typing).
 */
export function acceleratorFromEvent(e: KeyLike): string | null {
  if (MODIFIER_KEYS.has(e.key)) return null;
  const mods: string[] = [];
  if (e.ctrlKey || e.metaKey) mods.push('CommandOrControl');
  if (e.altKey) mods.push('Alt');
  if (e.shiftKey) mods.push('Shift');
  if (mods.length === 0) return null;
  let key: string;
  if (/^Key[A-Z]$/.test(e.code)) key = e.code.slice(3);
  else if (/^Digit[0-9]$/.test(e.code)) key = e.code.slice(5);
  else if (/^F([1-9]|1[0-9]|2[0-4])$/.test(e.code)) key = e.code;
  else if (e.code === 'Space') key = 'Space';
  else return null;
  return [...mods, key].join('+');
}

/** `CommandOrControl+Shift+O` -> `Ctrl+Shift+O` (or `⌘+Shift+O` on macOS) for display. */
export function displayAccelerator(accel: string, mac: boolean): string {
  return accel.replace('CommandOrControl', mac ? '⌘' : 'Ctrl');
}
