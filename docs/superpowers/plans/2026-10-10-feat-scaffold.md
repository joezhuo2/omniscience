# feat/scaffold Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stand up the Tauri 2 + React 19 + TypeScript + Tailwind v4 project. The result is a frameless,
transparent, always-on-top placeholder pill at the top-center of the primary monitor, with lint, format,
test, and CI wired up.

**Architecture:** Vite serves a React app into a single Tauri webview window labeled `main`. Rust
positions the window top-center using the monitor work area (pure positioning math lives in
`window.rs` and is unit-tested), then shows it. No telemetry yet. That is `feat/telemetry`.

**Tech Stack:** Tauri 2.12, Rust 1.98 (edition 2024, MSRV 1.90), Vite 8, React 19.3, TypeScript 6.0,
Tailwind CSS 4.3 (`@tailwindcss/vite`), `motion` 14, `lucide-react`, Vitest 5 + Testing Library + jsdom,
ESLint 10 (flat config) + typescript-eslint 8, Prettier 3, GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-10-10-omniscience-design.md`

## Global Constraints

- Tauri **2.x** only (not the 3.0 alphas): `tauri = "2.12"`, `tauri-build = "2.7"`, `@tauri-apps/cli ^2.12.2`, `@tauri-apps/api ^2.12.3`.
- TypeScript pinned `~6.0.3`, because typescript-eslint 8.71 requires `typescript <6.1.0`.
- Bundle identifier: `io.github.joezhuo2.omniscience`. Product name: `Omniscience`. Version `0.1.0`.
- Window: `decorations: false`, `transparent: true`, `alwaysOnTop: true`, `resizable: false`, `shadow: false`, initial size 400 x 42 logical px.
- **No taskbar button, no Alt+Tab entry, no Dock icon** (maintainer request, 2026-10-10): `skipTaskbar: true`; the Windows HWND gets `WS_EX_TOOLWINDOW` with `WS_EX_APPWINDOW` cleared before first show; macOS uses `ActivationPolicy::Accessory`. Do **not** add `WS_EX_NOACTIVATE`, because keyboard input must keep working. A minimal tray (left-click toggle; menu Show / Hide, Quit) ships in this branch so the app can always be closed (Task 3b).
- `macOSPrivateApi: true` (needed for macOS transparency). This requires the `macos-private-api` feature on the `tauri` crate.
- Dark only. The page background must stay transparent (`html, body, #root`).
- Commits: Conventional Commits, small, on branch `feat/scaffold`. Never push. The maintainer merges.
- Update `README.md` and the `[Unreleased]` section of `CHANGELOG.md` in this branch.
- Package manager: npm. Node 24.

## Review Focus

1. **Transparent corners:** the rounded pill's corners must show the desktop, not white or black boxes. Pinned by a CSS test (Task 1) and a manual check (Task 5).
2. **High-DPI displays (125/150/200% scaling):** the window must be horizontally centered and the top margin must scale. Pinned by a `top_center` test with scaled margin (Task 3).
3. **Primary monitor not at origin / negative coordinates** (secondary monitor left of primary): the overlay must still land on the primary monitor's top-center. Pinned by a negative-offset test (Task 3).
4. **Taskbar docked at the top of the screen:** the overlay must sit below it, using the work area, not the full monitor. Pinned by a work-area y-offset test (Task 3).
5. **Launch flash:** the window must not appear at the default position and then jump. It is created hidden and shown after positioning. Manual check (Task 5).

---

## File Structure

```
.gitattributes                 LF line endings, binary icons
.prettierrc.json / .prettierignore
eslint.config.js               flat config
package.json                   scripts + pinned deps
index.html                     Vite entry
vite.config.ts                 Vite + React + Tailwind + Vitest config (Tauri-friendly dev server)
tsconfig.json                  app + tests (src/)
tsconfig.node.json             vite.config.ts, eslint.config.js
app-icon.svg                   source logo for `tauri icon`
src/
  main.tsx                     React root
  App.tsx                      placeholder pill
  App.test.tsx                 smoke test
  index.css                    Tailwind import + transparent page
  index.css.test.ts            pins transparent background rule
  test/setup.ts                jest-dom matchers
src-tauri/
  Cargo.toml, build.rs
  tauri.conf.json              window/bundle config
  capabilities/default.json    core permissions for `main`
  icons/                       generated
  src/main.rs                  binary entry
  src/lib.rs                   Builder + setup
  src/window.rs                top-center positioning (pure fn + Tauri glue), unit-tested
scripts/measure.ps1            memory/CPU sampler for docs/perf.md
docs/testing.md                manual checklist
docs/perf.md                   measurement method + results table
.github/workflows/ci.yml       lint/test/build on Windows + macOS
```

---

### Task 0: Create the branch

- [ ] **Step 1: Branch from main**

```bash
git checkout main
git checkout -b feat/scaffold
```

---

### Task 1: Frontend toolchain (Vite + React + TS + Tailwind + Vitest)

**Files:**
- Create: `package.json`, `index.html`, `vite.config.ts`, `tsconfig.json`, `tsconfig.node.json`, `src/main.tsx`, `src/App.tsx`, `src/index.css`, `src/test/setup.ts`
- Test: `src/App.test.tsx`, `src/index.css.test.ts`

**Interfaces:**
- Produces: `App` default export (placeholder; replaced in `feat/pill`). npm scripts `dev`, `build`, `typecheck`, `test`, `test:watch`, `tauri` (used by Tauri config and CI).

- [ ] **Step 1: Write `package.json`**

```json
{
  "name": "omniscience",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "license": "MIT",
  "scripts": {
    "dev": "vite",
    "build": "npm run typecheck && vite build",
    "preview": "vite preview",
    "typecheck": "tsc --noEmit -p tsconfig.json && tsc --noEmit -p tsconfig.node.json",
    "test": "vitest run",
    "test:watch": "vitest",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.12.3",
    "lucide-react": "^1.55.0",
    "motion": "^14.1.0",
    "react": "^19.3.0",
    "react-dom": "^19.3.0"
  },
  "devDependencies": {
    "@tailwindcss/vite": "^4.3.3",
    "@tauri-apps/cli": "^2.12.2",
    "@testing-library/jest-dom": "^7.0.1",
    "@testing-library/react": "^16.3.3",
    "@types/node": "^24.19.2",
    "@types/react": "^19.3.0",
    "@types/react-dom": "^19.3.0",
    "@vitejs/plugin-react": "^6.1.2",
    "jsdom": "^30.1.2",
    "tailwindcss": "^4.3.3",
    "typescript": "~6.0.3",
    "vite": "^8.3.4",
    "vitest": "^5.0.3"
  }
}
```

- [ ] **Step 2: Install**

Run: `npm install`
Expected: completes with no peer-dependency errors (warnings are OK). This creates `package-lock.json`.

- [ ] **Step 3: Write `index.html`**

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Omniscience</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

- [ ] **Step 4: Write `vite.config.ts`**

```ts
/// <reference types="vitest/config" />
import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

const host = process.env.TAURI_DEV_HOST;

// https://v2.tauri.app/start/frontend/vite/
export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
    watch: { ignored: ['**/src-tauri/**'] },
  },
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  build: {
    target: process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_ENV_DEBUG ? 'oxc' : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
  test: {
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/**/*.test.{ts,tsx}'],
  },
});
```

> Note: Vite 8 uses Rolldown/Oxc. If `minify: 'oxc'` is rejected by the installed Vite, use `minify: true`.

- [ ] **Step 5: Write `tsconfig.json` and `tsconfig.node.json`**

`tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "noUncheckedIndexedAccess": true,
    "isolatedModules": true,
    "verbatimModuleSyntax": true,
    "skipLibCheck": true,
    "noEmit": true,
    "types": ["vite/client", "node"]
  },
  "include": ["src"]
}
```

`tsconfig.node.json`:

```json
{
  "compilerOptions": {
    "target": "ES2023",
    "lib": ["ES2023"],
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "skipLibCheck": true,
    "noEmit": true,
    "allowJs": true,
    "types": ["node"]
  },
  "include": ["vite.config.ts", "eslint.config.js"]
}
```

- [ ] **Step 6: Write the test setup and the failing tests**

`src/test/setup.ts`:

```ts
import '@testing-library/jest-dom/vitest';
```

`src/App.test.tsx`:

```tsx
import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import App from './App';

describe('App', () => {
  it('renders the overlay pill', () => {
    render(<App />);
    expect(screen.getByTestId('pill')).toBeInTheDocument();
  });
});
```

`src/index.css.test.ts`:

```ts
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
```

- [ ] **Step 7: Run tests to verify they fail**

Run: `npm test`
Expected: FAIL. Both files fail because `./App` cannot be resolved and `index.css` does not exist (ENOENT).

- [ ] **Step 8: Implement `index.css`, `App.tsx`, `main.tsx`**

`src/index.css`:

```css
@import "tailwindcss";

html,
body,
#root {
  background: transparent;
  margin: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

body {
  font-family:
    Inter, "Segoe UI Variable", "Segoe UI", system-ui, -apple-system, sans-serif;
  color: #e2e8f0;
  user-select: none;
  -webkit-font-smoothing: antialiased;
}
```

`src/App.tsx`:

```tsx
export default function App() {
  return (
    <div className="flex h-full w-full items-start justify-center">
      <div
        data-testid="pill"
        className="flex h-[42px] w-full items-center justify-center rounded-full border border-slate-700/50 bg-slate-900/85 text-xs font-medium tracking-wide text-slate-300"
      >
        Omniscience
      </div>
    </div>
  );
}
```

`src/main.tsx`:

```tsx
import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import App from './App';
import './index.css';

const root = document.getElementById('root');
if (!root) throw new Error('#root element missing from index.html');

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
```

- [ ] **Step 9: Run tests and typecheck**

Run: `npm test`
Expected: PASS, 3 tests.

Run: `npm run typecheck`
Expected: exits 0 with no output.

- [ ] **Step 10: Build the frontend**

Run: `npx vite build`
Expected: `dist/index.html` and `dist/assets/*` produced, no errors.

- [ ] **Step 11: Commit**

```bash
git add package.json package-lock.json index.html vite.config.ts tsconfig.json tsconfig.node.json src
git commit -m "build: scaffold vite react typescript frontend with tailwind and vitest"
```

---

### Task 2: Lint, format, and line endings

**Files:**
- Create: `eslint.config.js`, `.prettierrc.json`, `.prettierignore`, `.gitattributes`
- Modify: `package.json` (scripts + devDependencies)

**Interfaces:**
- Produces: npm scripts `lint`, `format`, `format:check` (used by CI in Task 4).

- [ ] **Step 1: Install lint/format deps**

```bash
npm install -D eslint@^10.12.0 @eslint/js@^10.0.1 typescript-eslint@^8.71.1 eslint-plugin-react-hooks@^7.1.1 eslint-plugin-react-refresh@^0.5.7 globals@^17.13.0 prettier@^3.9.10
```

Expected: installs cleanly.

- [ ] **Step 2: Write `eslint.config.js`**

```js
import js from '@eslint/js';
import { defineConfig, globalIgnores } from 'eslint/config';
import reactHooks from 'eslint-plugin-react-hooks';
import reactRefresh from 'eslint-plugin-react-refresh';
import globals from 'globals';
import tseslint from 'typescript-eslint';

export default defineConfig([
  globalIgnores(['dist', 'coverage', 'src-tauri/target', 'src-tauri/gen', 'src/types/generated']),
  {
    files: ['**/*.{ts,tsx}'],
    extends: [
      js.configs.recommended,
      tseslint.configs.recommended,
      reactHooks.configs.flat.recommended,
      reactRefresh.configs.vite,
    ],
    languageOptions: {
      ecmaVersion: 2022,
      globals: globals.browser,
    },
  },
]);
```

> If `reactHooks.configs.flat.recommended` or `reactRefresh.configs.vite` is undefined in the installed versions, check each plugin's README for the flat-config export name and use that. Do not drop the plugin.

- [ ] **Step 3: Write Prettier config**

`.prettierrc.json`:

```json
{
  "singleQuote": true,
  "semi": true,
  "trailingComma": "all",
  "printWidth": 100
}
```

`.prettierignore`:

```
dist
coverage
node_modules
package-lock.json
src-tauri/target
src-tauri/gen
src-tauri/icons
src/types/generated
docs/superpowers
CHANGELOG.md
```

- [ ] **Step 4: Add scripts to `package.json`**

Add to `"scripts"`:

```json
"lint": "eslint .",
"format": "prettier --write .",
"format:check": "prettier --check ."
```

- [ ] **Step 5: Write `.gitattributes`**

```
* text=auto eol=lf
*.png binary
*.ico binary
*.icns binary
```

- [ ] **Step 6: Format and lint**

Run: `npm run format`, then `npm run lint`
Expected: lint exits 0. Formatting may touch existing files.

Run: `npm run format:check && npm test`
Expected: "All matched files use Prettier code style!" and tests PASS.

- [ ] **Step 7: Renormalize line endings and commit**

```bash
git add --renormalize .
git add eslint.config.js .prettierrc.json .prettierignore .gitattributes package.json package-lock.json src index.html vite.config.ts tsconfig.json tsconfig.node.json README.md
git commit -m "build: add eslint, prettier, and lf line endings"
```

---

### Task 3: Tauri backend and top-center window

**Files:**
- Create: `src-tauri/Cargo.toml`, `src-tauri/build.rs`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`, `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/window.rs`, `app-icon.svg`, `src-tauri/icons/*` (generated)
- Test: unit tests inside `src-tauri/src/window.rs`

**Interfaces:**
- Produces:
  - `window::top_center(area_x: i32, area_y: i32, area_width: u32, window_width: u32, margin: i32) -> (i32, i32)`, pure, physical pixels.
  - `window::place_top_center(window: &tauri::WebviewWindow) -> tauri::Result<()>`.
  - `window::TOP_MARGIN: f64 = 8.0` (logical px).
  - Window label `"main"` (used by every later branch).

- [ ] **Step 1: Write `src-tauri/Cargo.toml`**

```toml
[package]
name = "omniscience"
version = "0.1.0"
description = "Dynamic Island-style system monitor overlay"
authors = ["Omniscience contributors"]
license = "MIT"
repository = "https://github.com/joezhuo2/omniscience"
edition = "2024"
rust-version = "1.90"

[lib]
name = "omniscience_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2.7", features = [] }

[dependencies]
tauri = { version = "2.12", features = ["macos-private-api"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[profile.release]
codegen-units = 1
lto = true
opt-level = 3
strip = true
```

> `panic` stays at the default (`unwind`). `feat/telemetry` restarts a panicked collector thread, which requires unwinding.

- [ ] **Step 2: Write `src-tauri/build.rs` and `src-tauri/src/main.rs`**

`build.rs`:

```rust
fn main() {
    tauri_build::build()
}
```

`src/main.rs`:

```rust
// Prevents an extra console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    omniscience_lib::run()
}
```

- [ ] **Step 3: Write the failing positioning tests in `src-tauri/src/window.rs`**

```rust
//! Overlay window placement.

/// Top-left position (physical px) that horizontally centers a window at the top of a monitor
/// work area, `margin` physical px below its top edge.
pub fn top_center(
    area_x: i32,
    area_y: i32,
    area_width: u32,
    window_width: u32,
    margin: i32,
) -> (i32, i32) {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centers_on_monitor_at_origin() {
        assert_eq!(top_center(0, 0, 1920, 400, 8), (760, 8));
    }

    #[test]
    fn rounds_down_on_odd_remainder() {
        assert_eq!(top_center(0, 0, 1921, 400, 8), (760, 8));
    }

    #[test]
    fn handles_monitor_left_of_origin() {
        assert_eq!(top_center(-2560, 0, 2560, 400, 8), (-1480, 8));
    }

    #[test]
    fn respects_work_area_below_top_taskbar() {
        assert_eq!(top_center(0, 48, 1920, 400, 8), (760, 56));
    }

    #[test]
    fn uses_physical_sizes_at_150_percent_scale() {
        // 400 logical px window and 8 logical px margin at 1.5x on a 2880 px wide panel.
        assert_eq!(top_center(0, 0, 2880, 600, 12), (1140, 12));
    }

    #[test]
    fn window_wider_than_area_overhangs_evenly() {
        assert_eq!(top_center(0, 0, 300, 400, 0), (-50, 0));
    }
}
```

Also create a minimal `src/lib.rs` so the crate compiles for tests:

```rust
mod window;

pub fn run() {}
```

- [ ] **Step 4: Write `tauri.conf.json` and capabilities (needed for the crate to compile)**

`src-tauri/tauri.conf.json`:

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Omniscience",
  "version": "0.1.0",
  "identifier": "io.github.joezhuo2.omniscience",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "macOSPrivateApi": true,
    "windows": [
      {
        "label": "main",
        "title": "Omniscience",
        "width": 400,
        "height": 42,
        "decorations": false,
        "transparent": true,
        "alwaysOnTop": true,
        "resizable": false,
        "maximizable": false,
        "minimizable": false,
        "shadow": false,
        "skipTaskbar": true,
        "visible": false,
        "focus": false
      }
    ],
    "security": {
      "csp": "default-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:"
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "category": "Utility",
    "shortDescription": "Dynamic Island-style system monitor overlay",
    "license": "MIT",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

`src-tauri/capabilities/default.json`:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Core permissions for the overlay window",
  "windows": ["main"],
  "permissions": ["core:default"]
}
```

- [ ] **Step 5: Create the icon source and generate icons**

`app-icon.svg`:

```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024">
  <rect width="1024" height="1024" rx="224" fill="#0f172a"/>
  <circle cx="512" cy="512" r="300" fill="none" stroke="#1e293b" stroke-width="72"/>
  <circle cx="512" cy="512" r="300" fill="none" stroke="#22d3ee" stroke-width="72"
          stroke-linecap="round" stroke-dasharray="1319 1885" transform="rotate(-90 512 512)"/>
  <circle cx="512" cy="512" r="64" fill="#a78bfa"/>
</svg>
```

Run: `npx tauri icon app-icon.svg`
Expected: `src-tauri/icons/` populated (`32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.icns`, `icon.ico`, Store logos, and `android/`, `ios/` subfolders).

Then remove the mobile icon sets (desktop-only app):

```bash
rm -rf src-tauri/icons/android src-tauri/icons/ios
```

- [ ] **Step 6: Build the frontend once (Tauri's compile step requires `dist/`), then run the tests to verify they fail**

Run: `npx vite build` (creates `dist/`)
Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: compiles (first build takes a few minutes), then 6 tests FAIL with `not implemented`.

- [ ] **Step 7: Implement `top_center`**

Replace the `unimplemented!()` body:

```rust
pub fn top_center(
    area_x: i32,
    area_y: i32,
    area_width: u32,
    window_width: u32,
    margin: i32,
) -> (i32, i32) {
    let offset = (i64::from(area_width) - i64::from(window_width)).div_euclid(2);
    (area_x + offset as i32, area_y + margin)
}
```

> `div_euclid` gives floor division, so a window wider than the area overhangs evenly (`-100 / 2 = -50`) and odd remainders round down.

- [ ] **Step 8: Run tests to verify they pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: 6 passed.

- [ ] **Step 9: Add the Tauri glue to `window.rs` (above the `tests` module)**

```rust
use tauri::{PhysicalPosition, WebviewWindow};

/// Gap between the top of the work area and the overlay, in logical px.
pub const TOP_MARGIN: f64 = 8.0;

/// Moves `window` to the top-center of the primary monitor's work area.
/// Does nothing if no primary monitor is reported.
pub fn place_top_center(window: &WebviewWindow) -> tauri::Result<()> {
    let Some(monitor) = window.primary_monitor()? else {
        return Ok(());
    };
    let area = monitor.work_area();
    let size = window.outer_size()?;
    let margin = (TOP_MARGIN * monitor.scale_factor()).round() as i32;
    let (x, y) = top_center(
        area.position.x,
        area.position.y,
        area.size.width,
        size.width,
        margin,
    );
    window.set_position(PhysicalPosition::new(x, y))
}
```

> If `Monitor::work_area()` does not exist in the resolved Tauri version, use `monitor.position()` / `monitor.size()` instead and add a line to `docs/testing.md` noting that a top-docked taskbar is not respected yet.

- [ ] **Step 10: Wire `lib.rs`**

```rust
mod window;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let main = app
                .get_webview_window("main")
                .expect("window \"main\" is declared in tauri.conf.json");
            window::place_top_center(&main)?;
            main.show()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Omniscience");
}
```

- [ ] **Step 11: fmt, clippy, tests**

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: clippy clean, 6 passed.

- [ ] **Step 12: Run the app**

Run: `npm run tauri dev`
Expected: a 400 x 42 dark rounded pill reading "Omniscience" appears at the top-center of the primary monitor, about 8 px below the top. There is no title bar, the corners show the desktop behind, and it stays above other windows when you click elsewhere. No taskbar entry exists. Close it with Ctrl+C in the terminal (the tray arrives in Task 3b).

- [ ] **Step 13: Commit (two commits)**

```bash
git add app-icon.svg src-tauri/icons
git commit -m "chore: add app icon source and generated desktop icons"
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/build.rs src-tauri/tauri.conf.json src-tauri/capabilities src-tauri/src
git commit -m "feat(window): add tauri shell with transparent top-center overlay window"
```

---

### Task 3b: Tool window (no taskbar / Alt+Tab) and minimal tray

**Files:**
- Create: `src-tauri/src/platform.rs`, `src-tauri/src/tray.rs`
- Modify: `src-tauri/Cargo.toml` (tauri `tray-icon` feature, `windows-sys`), `src-tauri/src/lib.rs`
- Test: unit tests inside `src-tauri/src/platform.rs`

**Interfaces:**
- Produces:
  - `platform::tool_window_ex_style(ex_style: isize) -> isize`: pure. Adds `WS_EX_TOOLWINDOW`, clears `WS_EX_APPWINDOW`, keeps every other bit.
  - `platform::prepare_overlay(window: &tauri::WebviewWindow)`: applies it to the HWND on Windows, no-op elsewhere.
  - `tray::init(app: &tauri::AppHandle) -> tauri::Result<()>` and `tray::toggle_main(app: &tauri::AppHandle)`. `feat/app-shell` extends the menu and reuses `toggle_main` for the hotkey.

- [ ] **Step 1: Add dependencies to `src-tauri/Cargo.toml`**

Change the tauri line and add a Windows-only section:

```toml
tauri = { version = "2.12", features = ["macos-private-api", "tray-icon"] }

[target.'cfg(windows)'.dependencies]
windows-sys = { version = "0.61", features = ["Win32_Foundation", "Win32_UI_WindowsAndMessaging"] }
```

- [ ] **Step 2: Write the failing tests in `src-tauri/src/platform.rs`**

```rust
//! Platform-specific window tweaks.

/// `WS_EX_TOOLWINDOW`: no taskbar button and no Alt+Tab entry.
pub const WS_EX_TOOLWINDOW: isize = 0x0000_0080;
/// `WS_EX_APPWINDOW`: forces a taskbar button. It must be cleared or it wins over TOOLWINDOW.
pub const WS_EX_APPWINDOW: isize = 0x0004_0000;

/// Extended window style for the overlay.
pub fn tool_window_ex_style(ex_style: isize) -> isize {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;

    const WS_EX_TOPMOST: isize = 0x0000_0008;
    const WS_EX_LAYERED: isize = 0x0008_0000;

    #[test]
    fn adds_toolwindow() {
        assert_eq!(tool_window_ex_style(0) & WS_EX_TOOLWINDOW, WS_EX_TOOLWINDOW);
    }

    #[test]
    fn clears_appwindow() {
        assert_eq!(tool_window_ex_style(WS_EX_APPWINDOW) & WS_EX_APPWINDOW, 0);
    }

    #[test]
    fn keeps_other_bits() {
        let style = WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_APPWINDOW;
        assert_eq!(
            tool_window_ex_style(style),
            WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_TOOLWINDOW
        );
    }

    #[test]
    fn is_idempotent() {
        let once = tool_window_ex_style(WS_EX_APPWINDOW);
        assert_eq!(tool_window_ex_style(once), once);
    }
}
```

Add `mod platform;` to `lib.rs`.

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml platform`
Expected: 4 tests FAIL with `not implemented`.

- [ ] **Step 4: Implement**

```rust
pub fn tool_window_ex_style(ex_style: isize) -> isize {
    (ex_style | WS_EX_TOOLWINDOW) & !WS_EX_APPWINDOW
}
```

Then add the HWND glue below it:

```rust
/// Hides the overlay from the taskbar and Alt+Tab. Call before the window is first shown.
#[cfg(windows)]
pub fn prepare_overlay(window: &tauri::WebviewWindow) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GWL_EXSTYLE, GetWindowLongPtrW, SetWindowLongPtrW,
    };
    let Ok(hwnd) = window.hwnd() else { return };
    let hwnd = hwnd.0 as windows_sys::Win32::Foundation::HWND;
    // SAFETY: hwnd is a live window owned by this process, used on the thread that created it.
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, tool_window_ex_style(style));
    }
}

#[cfg(not(windows))]
pub fn prepare_overlay(_window: &tauri::WebviewWindow) {}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: 10 passed (6 window + 4 platform).

- [ ] **Step 6: Write `src-tauri/src/tray.rs`**

```rust
//! Tray icon: the app's only presence (no taskbar button, no Alt+Tab entry).

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

const SHOW_HIDE: &str = "show_hide";
const QUIT: &str = "quit";

pub fn init(app: &AppHandle) -> tauri::Result<()> {
    let menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, SHOW_HIDE, "Show / Hide Omniscience", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, QUIT, "Quit", true, None::<&str>)?,
        ],
    )?;
    let mut builder = TrayIconBuilder::with_id("main")
        .tooltip("Omniscience")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            SHOW_HIDE => toggle_main(app),
            QUIT => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_main(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

/// Shows the overlay if hidden, hides it if shown.
pub fn toggle_main(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let result = match window.is_visible() {
        Ok(true) => window.hide(),
        _ => window.show(),
    };
    if let Err(e) = result {
        eprintln!("toggle overlay: {e}");
    }
}
```

> If `show_menu_on_left_click` does not exist in the resolved Tauri version, use `menu_on_left_click(false)`.

- [ ] **Step 7: Wire `lib.rs`**

```rust
mod platform;
mod tray;
mod window;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let main = app
                .get_webview_window("main")
                .expect("window \"main\" is declared in tauri.conf.json");
            platform::prepare_overlay(&main);
            window::place_top_center(&main)?;
            tray::init(app.handle())?;
            main.show()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Omniscience");
}
```

- [ ] **Step 8: fmt, clippy, tests, run**

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: clippy clean, 10 passed.

Run: `npm run tauri dev`
Expected:
- The pill appears top-center with no taskbar button, and it does not appear in Alt+Tab.
- A tray icon (the app icon) appears in the notification area.
- Left-click toggles the pill.
- Right-click shows "Show / Hide Omniscience" and "Quit", and Quit exits the app.

- [ ] **Step 9: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src
git commit -m "feat(window): hide overlay from taskbar and alt-tab and add minimal tray"
```

Add to `docs/testing.md` (Task 5) under the window section:

```markdown
- [ ] No taskbar button. Not listed in Alt+Tab or Win+Tab.
- [ ] Tray icon present. Left-click hides/shows the pill. Right-click menu: Show / Hide, Quit. Quit exits fully (no omniscience.exe left in Task Manager).
- [ ] After restarting Explorer (Task Manager > Windows Explorer > Restart), the tray icon comes back.
```

---

### Task 4: CI workflow

**Files:**
- Create: `.github/workflows/ci.yml`

**Interfaces:**
- Consumes: npm scripts `lint`, `format:check`, `build`, `test`, `tauri`. Cargo manifest at `src-tauri/Cargo.toml`.

- [ ] **Step 1: Write `.github/workflows/ci.yml`**

```yaml
name: CI

on:
  push:
  pull_request:

concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true

jobs:
  check:
    name: Check (${{ matrix.os }})
    strategy:
      fail-fast: false
      matrix:
        os: [windows-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v5

      - uses: actions/setup-node@v5
        with:
          node-version: 24
          cache: npm

      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - uses: swatinem/rust-cache@v2
        with:
          workspaces: './src-tauri -> target'

      - run: npm ci
      - run: npm run lint
      - run: npm run format:check
      - run: npm test
      # Typechecks and produces dist/, which tauri::generate_context! needs at compile time.
      - run: npm run build

      - run: cargo fmt --manifest-path src-tauri/Cargo.toml --check
      - run: cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
      - run: cargo test --manifest-path src-tauri/Cargo.toml

      - run: npm run tauri build -- --no-bundle
```

- [ ] **Step 2: Validate locally (the workflow can't run until it's pushed)**

Run each command from the workflow in order, from `npm ci` through `npm run tauri build -- --no-bundle`.
Expected: all exit 0. The release binary is at `src-tauri/target/release/omniscience.exe`.

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: add lint, test, and build workflow for windows and macos"
```

---

### Task 5: Measurement script, docs, README, CHANGELOG

**Files:**
- Create: `scripts/measure.ps1`, `docs/testing.md`, `docs/perf.md`
- Modify: `README.md`, `CHANGELOG.md`

**Interfaces:**
- Produces: `scripts/measure.ps1 -Seconds <n>` prints average and peak private working set (MB) and CPU % for `omniscience.exe` plus all descendant processes (WebView2). Every later branch uses it to update `docs/perf.md`.

- [ ] **Step 1: Write `scripts/measure.ps1`**

```powershell
<#
.SYNOPSIS
  Samples memory and CPU of a running Omniscience instance, including its WebView2 child processes.
.DESCRIPTION
  Memory = sum of "Working Set - Private" (what Task Manager's Memory column shows) across
  omniscience.exe and all its descendants. CPU = sum of % Processor Time, normalized to
  total logical processors (0-100%, like Task Manager).
.EXAMPLE
  ./scripts/measure.ps1 -Seconds 30
#>
param(
  [int]$Seconds = 30,
  [string]$ProcessName = 'omniscience'
)

$root = Get-Process -Name $ProcessName -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $root) { Write-Error "$ProcessName is not running"; exit 1 }

function Get-Tree([int]$rootId) {
  $all = Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId
  $ids = [System.Collections.Generic.HashSet[int]]::new()
  [void]$ids.Add($rootId)
  do {
    $added = $false
    foreach ($p in $all) {
      if ($ids.Contains([int]$p.ParentProcessId) -and $ids.Add([int]$p.ProcessId)) { $added = $true }
    }
  } while ($added)
  return $ids
}

$cores = [Environment]::ProcessorCount
$memSamples = @()
$cpuSamples = @()

for ($i = 0; $i -lt $Seconds; $i++) {
  $tree = Get-Tree $root.Id
  $perf = Get-CimInstance Win32_PerfFormattedData_PerfProc_Process |
    Where-Object { $tree.Contains([int]$_.IDProcess) }
  $memSamples += ($perf | Measure-Object -Property WorkingSetPrivate -Sum).Sum / 1MB
  $cpuSamples += ($perf | Measure-Object -Property PercentProcessorTime -Sum).Sum / $cores
  Start-Sleep -Seconds 1
}

$tree = Get-Tree $root.Id
[pscustomobject]@{
  Processes   = $tree.Count
  MemAvgMB    = [math]::Round(($memSamples | Measure-Object -Average).Average, 1)
  MemPeakMB   = [math]::Round(($memSamples | Measure-Object -Maximum).Maximum, 1)
  CpuAvgPct   = [math]::Round(($cpuSamples | Measure-Object -Average).Average, 2)
  CpuPeakPct  = [math]::Round(($cpuSamples | Measure-Object -Maximum).Maximum, 2)
  Samples     = $Seconds
} | Format-List
```

- [ ] **Step 2: Measure the bare scaffold baseline**

Run the release binary from Task 4: start `src-tauri/target/release/omniscience.exe`, wait 10 s, then:

Run: `powershell -ExecutionPolicy Bypass -File scripts/measure.ps1 -Seconds 30`
Expected: prints `Processes`, `MemAvgMB`, `MemPeakMB`, `CpuAvgPct`, `CpuPeakPct`. Record the values. Close the app afterwards.

- [ ] **Step 3: Write `docs/perf.md` with the measured numbers**

```markdown
# Performance

Budget (spec §7): collapsed idle at most **50 MB** total private working set across the process
tree, and below **0.1% CPU**.

## Method

1. Build release: `npm run tauri build -- --no-bundle`.
2. Start `src-tauri/target/release/omniscience.exe` and leave it alone for 10 s.
3. Run `powershell -ExecutionPolicy Bypass -File scripts/measure.ps1 -Seconds 30`.
4. Record the averages below. Note the state (collapsed, expanded, or hidden).

Reference machine: Ryzen AI 9 HX 370, Radeon 890M + RTX 5060 Laptop, 32 GB RAM, Windows 11.

## Results

| Branch | State | Processes | Mem avg (MB) | Mem peak (MB) | CPU avg (%) | Notes |
|---|---|---|---|---|---|---|
| feat/scaffold | static pill, no telemetry | <n> | <avg> | <peak> | <cpu> | Bare Tauri + WebView2 baseline |
```

Replace `<n>`, `<avg>`, `<peak>`, `<cpu>` with the values printed in Step 2. These are measured numbers, not placeholders to leave in.

- [ ] **Step 4: Write `docs/testing.md`**

```markdown
# Manual test checklist

Run before handing off each branch. Record results in the branch handoff notes.

## Window (feat/scaffold)

- [ ] Pill appears at top-center of the primary monitor, about 8 px below the top of the work area.
- [ ] No jump on launch (window is not visible at another position first).
- [ ] Corners of the pill show the desktop. No white or black boxes.
- [ ] No title bar or window frame.
- [ ] Stays above other windows after clicking another app.
- [ ] Correct position at 100% and 150% display scaling (Settings > Display > Scale), restarting the app after each change.
- [ ] With a second monitor placed left of the primary, the pill still appears on the primary monitor.
```

- [ ] **Step 5: Update `README.md`**

Replace the `> **Status:**` line with:

```markdown
> **Status:** pre-alpha. The app shell runs; telemetry and UI are in progress.
```

Insert before `## Design`:

````markdown
## Development

Prerequisites: [Node.js 24+](https://nodejs.org), [Rust (stable)](https://rustup.rs), and the
[Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS (WebView2 is
preinstalled on Windows 11).

```bash
npm install
npm run tauri dev
```

| Command | What it does |
|---|---|
| `npm run tauri dev` | Run the app with hot reload |
| `npm test` | Frontend unit tests (Vitest) |
| `cargo test --manifest-path src-tauri/Cargo.toml` | Rust unit tests |
| `npm run lint` / `npm run format` | ESLint / Prettier |
| `npm run tauri build` | Release build and installers |

Performance numbers: [docs/perf.md](docs/perf.md). Manual test checklist: [docs/testing.md](docs/testing.md).
````

- [ ] **Step 6: Update `CHANGELOG.md` `[Unreleased]` → `### Added`**

Append these bullets:

```markdown
- Tauri 2 app shell with a frameless, transparent, always-on-top window placed at the top-center of the primary monitor.
- React 19, TypeScript, Tailwind CSS v4, and Vitest frontend scaffold.
- ESLint, Prettier, rustfmt, and clippy configuration.
- GitHub Actions CI for Windows and macOS (lint, test, build).
- Memory/CPU measurement script and performance log.
```

- [ ] **Step 7: Format check, then commit (two commits)**

Run: `npm run format:check`
Expected: passes. If not, run `npm run format`.

```bash
git add scripts/measure.ps1 docs/perf.md docs/testing.md
git commit -m "docs: add perf measurement script, baseline, and manual test checklist"
git add README.md CHANGELOG.md
git commit -m "docs: document development setup and scaffold changes"
```

---

### Task 6: Branch handoff

- [ ] **Step 1: Full verification**

```bash
npm run lint
npm run format:check
npm test
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: all exit 0.

- [ ] **Step 2: Walk through the `docs/testing.md` window checklist manually** and note the results.

- [ ] **Step 3: Report to the maintainer:** the commit list (`git log --oneline main..feat/scaffold`), the checklist results, the perf baseline, and any deviations from this plan. Stop. Do not start `feat/telemetry` until the maintainer merges and says to continue.
