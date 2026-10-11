import { describe, expect, it } from 'vitest';
import { acceleratorFromEvent, displayAccelerator, type KeyLike } from './hotkey';

const press = (code: string, key: string, mods: Partial<KeyLike> = {}): KeyLike => ({
  code,
  key,
  ctrlKey: false,
  metaKey: false,
  altKey: false,
  shiftKey: false,
  ...mods,
});

describe('acceleratorFromEvent', () => {
  it('builds an accelerator from modifiers and a letter', () => {
    expect(acceleratorFromEvent(press('KeyO', 'O', { ctrlKey: true, shiftKey: true }))).toBe(
      'CommandOrControl+Shift+O',
    );
    expect(acceleratorFromEvent(press('KeyM', 'µ', { metaKey: true, altKey: true }))).toBe(
      'CommandOrControl+Alt+M',
    );
  });

  it('supports digits, function keys and space', () => {
    expect(acceleratorFromEvent(press('Digit5', '%', { altKey: true, shiftKey: true }))).toBe(
      'Alt+Shift+5',
    );
    expect(acceleratorFromEvent(press('F9', 'F9', { ctrlKey: true }))).toBe('CommandOrControl+F9');
    expect(acceleratorFromEvent(press('Space', ' ', { altKey: true }))).toBe('Alt+Space');
  });

  it('waits while only modifiers are held', () => {
    expect(acceleratorFromEvent(press('ShiftLeft', 'Shift', { shiftKey: true }))).toBeNull();
  });

  it('rejects bare keys and unsupported keys', () => {
    expect(acceleratorFromEvent(press('KeyO', 'o'))).toBeNull();
    expect(acceleratorFromEvent(press('BracketLeft', '[', { ctrlKey: true }))).toBeNull();
  });
});

describe('displayAccelerator', () => {
  it('shows the platform modifier', () => {
    expect(displayAccelerator('CommandOrControl+Shift+O', false)).toBe('Ctrl+Shift+O');
    expect(displayAccelerator('CommandOrControl+Shift+O', true)).toBe('⌘+Shift+O');
  });
});
