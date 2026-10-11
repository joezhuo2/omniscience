import { describe, expect, it } from 'vitest';
import { isMounted, isOpen, nextPhase, type IslandEvent, type Phase } from './island';

function run(start: Phase, ...events: IslandEvent[]): Phase {
  return events.reduce(nextPhase, start);
}

describe('island state machine', () => {
  it('expands through the window resize', () => {
    expect(run('collapsed', 'expand')).toBe('expanding');
    expect(run('collapsed', 'expand', 'resized')).toBe('expanded');
  });

  it('collapses only after the close animation finishes', () => {
    expect(run('expanded', 'collapse')).toBe('collapsing');
    expect(run('expanded', 'collapse', 'closed')).toBe('collapsed');
  });

  it('reverses a collapse in progress without another resize', () => {
    expect(run('expanded', 'collapse', 'expand')).toBe('expanded');
  });

  it('handles leaving while the window is still growing', () => {
    expect(run('collapsed', 'expand', 'collapse')).toBe('collapsing');
    // A late resize completion is ignored.
    expect(run('collapsed', 'expand', 'collapse', 'resized')).toBe('collapsing');
  });

  it('ignores events that do not apply', () => {
    expect(run('collapsed', 'collapse', 'closed', 'resized')).toBe('collapsed');
    expect(run('expanded', 'expand', 'resized')).toBe('expanded');
  });

  it('mounts the dashboard until fully collapsed and opens it only when expanded', () => {
    expect(isMounted('collapsed')).toBe(false);
    expect(isMounted('collapsing')).toBe(true);
    expect(isOpen('expanding')).toBe(false);
    expect(isOpen('expanded')).toBe(true);
  });
});
