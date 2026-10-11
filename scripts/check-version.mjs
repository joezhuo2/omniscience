// Fails if the app version differs between package.json, Cargo.toml and tauri.conf.json.
// With a tag argument (e.g. `v0.1.0`), also fails if the tag does not match.
import { readFileSync } from 'node:fs';

const read = (path) => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');

const versions = {
  'package.json': JSON.parse(read('package.json')).version,
  'src-tauri/Cargo.toml': read('src-tauri/Cargo.toml').match(/^version\s*=\s*"([^"]+)"/m)?.[1],
  'src-tauri/tauri.conf.json': JSON.parse(read('src-tauri/tauri.conf.json')).version,
};

const tag = process.argv[2];
if (tag) versions[`tag ${tag}`] = tag.replace(/^v/, '');

const unique = new Set(Object.values(versions));
for (const [file, version] of Object.entries(versions)) console.log(`${file}: ${version}`);
if (unique.size !== 1 || unique.has(undefined)) {
  console.error('Version mismatch. Bump all three files together (see docs/releasing.md).');
  process.exit(1);
}
