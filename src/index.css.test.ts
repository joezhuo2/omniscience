// @vitest-environment node
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

// The window is transparent; any page background shows up as boxes around the rounded pill.
describe('index.css', () => {
  const css = readFileSync(new URL('./index.css', import.meta.url), 'utf8');

  it('keeps html, body and #root transparent', () => {
    expect(css).toMatch(/html,\s*body,\s*#root\s*\{[^}]*background:\s*transparent/);
  });

  it('imports tailwind', () => {
    expect(css).toContain('@import "tailwindcss"');
  });
});
