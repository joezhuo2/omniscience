// @vitest-environment node
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

// The window is transparent; any page background shows up as boxes around the rounded pill.
describe('index.css', () => {
  const css = readFileSync(new URL('./index.css', import.meta.url), 'utf8');

  it('keeps html, body and #root transparent', () => {
    expect(css).toMatch(/html,\s*body,\s*#root\s*\{[^}]*background:\s*transparent/);
  });

  it('lets the panel scroll inside the window height cap', () => {
    expect(css).toMatch(/\.panel\s*\{[^}]*max-height:/);
    expect(css).toMatch(/\.panel-scroll\s*\{[^}]*min-height:\s*0[^}]*overflow-y:\s*auto/);
  });

  it('disables bounce at the page and panel boundaries', () => {
    expect(css).toMatch(/html,\s*body,\s*#root\s*\{[^}]*overscroll-behavior:\s*none/);
    expect(css).toMatch(/\.panel-scroll\s*\{[^}]*overscroll-behavior:\s*none/);
  });

  it('imports tailwind', () => {
    expect(css).toMatch(/@import ['"]tailwindcss['"]/);
  });
});
