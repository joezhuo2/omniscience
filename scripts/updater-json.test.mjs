import { describe, expect, it } from 'vitest';
import { updaterJson } from './updater-json.mjs';

const assets = [
  'Omniscience_0.11.0_universal.dmg',
  'Omniscience_0.11.0_x64-setup.exe',
  'Omniscience_0.11.0_x64-setup.exe.sig',
  'Omniscience_0.11.0_x64_en-US.msi',
  'Omniscience_0.11.0_x64_en-US.msi.sig',
  'Omniscience_universal.app.tar.gz',
  'Omniscience_universal.app.tar.gz.sig',
];
const opts = { version: '0.11.0', repo: 'o/r', notes: 'n', now: new Date(0) };

describe('updaterJson', () => {
  it('matches the tauri-action manifest', () => {
    const json = updaterJson(assets, (f) => `sig:${f}\n`, opts);
    const base = 'https://github.com/o/r/releases/download/v0.11.0/';
    expect(json.version).toBe('0.11.0');
    expect(json.pub_date).toBe('1970-01-01T00:00:00.000Z');
    expect(Object.keys(json.platforms).sort()).toEqual([
      'darwin-aarch64',
      'darwin-aarch64-app',
      'darwin-x86_64',
      'darwin-x86_64-app',
      'windows-x86_64',
      'windows-x86_64-msi',
      'windows-x86_64-nsis',
    ]);
    expect(json.platforms['windows-x86_64']).toEqual({
      signature: 'sig:Omniscience_0.11.0_x64_en-US.msi.sig',
      url: `${base}Omniscience_0.11.0_x64_en-US.msi`,
    });
    expect(json.platforms['windows-x86_64-nsis'].url).toBe(
      `${base}Omniscience_0.11.0_x64-setup.exe`,
    );
    expect(json.platforms['darwin-aarch64'].url).toBe(`${base}Omniscience_universal.app.tar.gz`);
  });

  it('refuses a manifest without signed bundles for both platforms', () => {
    const unsigned = assets.filter((f) => !f.endsWith('.app.tar.gz.sig'));
    expect(() => updaterJson(unsigned, () => 's', opts)).toThrow(/missing/);
  });
});
