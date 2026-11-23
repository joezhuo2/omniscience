// Prints the updater manifest (latest.json) for release assets in a directory, in the shape
// tauri-action writes. Usage: node scripts/updater-json.mjs <dir> <version> <owner/repo> [notes]
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

/** Updater platform keys per installer, matching tauri-action. */
const PLATFORMS = [
  [/_x64_en-US\.msi$/, ['windows-x86_64', 'windows-x86_64-msi']],
  [/_x64-setup\.exe$/, ['windows-x86_64-nsis']],
  [
    /universal\.app\.tar\.gz$/,
    ['darwin-aarch64', 'darwin-x86_64', 'darwin-aarch64-app', 'darwin-x86_64-app'],
  ],
];

export function updaterJson(files, readSig, { version, repo, notes, now = new Date() }) {
  const platforms = {};
  for (const [pattern, keys] of PLATFORMS) {
    const file = files.find((f) => pattern.test(f));
    if (!file || !files.includes(`${file}.sig`)) continue;
    const entry = {
      signature: readSig(`${file}.sig`).trim(),
      url: `https://github.com/${repo}/releases/download/v${version}/${file}`,
    };
    for (const key of keys) platforms[key] = entry;
  }
  if (!platforms['windows-x86_64'] || !platforms['darwin-x86_64']) {
    throw new Error(`missing signed Windows or macOS updater bundle in: ${files.join(', ')}`);
  }
  return { version, notes, pub_date: now.toISOString(), platforms };
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [dir, version, repo, notes = ''] = process.argv.slice(2);
  const files = readdirSync(dir);
  const json = updaterJson(files, (f) => readFileSync(join(dir, f), 'utf8'), {
    version,
    repo,
    notes,
  });
  console.log(JSON.stringify(json, null, 2));
}
