# Omniscience

A lightweight, open-source system monitor that floats at the top of your screen as a
Dynamic Island–style pill and expands into a compact dashboard on hover.

> **Status:** pre-alpha. The app shell runs; telemetry and UI are in progress.

## Planned features

- Six live rings in the pill (CPU, RAM, iGPU, dGPU, Disk, Network), colored by load
  (normal, warning above 70%, critical above 90%)
- Hover to expand into a dashboard with:
  - Top processes, grouped by app, with a safe two-step "End task"
  - GPU and power card (VRAM, per-GPU usage, package watts)
  - Per-core CPU heatmap with tooltips
- Lives in the tray only: no taskbar button, no Alt+Tab entry
- Tray menu: launch at startup, auto-update, hide over fullscreen apps
- Global show/hide hotkey
- No admin rights or kernel drivers required
- Idle footprint target: at most 50 MB RAM and below 0.1% CPU

## Platforms

| Platform      | Status                                                                 |
| ------------- | ---------------------------------------------------------------------- |
| Windows 10/11 | Primary, tested                                                        |
| macOS         | Experimental. Builds in CI but is untested, and some sensors show `--` |

## Tech stack

Tauri 2, Rust, React 19, TypeScript, Tailwind CSS v4, Motion, Lucide.

## Development

Prerequisites: [Node.js 24+](https://nodejs.org), [Rust (stable)](https://rustup.rs), and the
[Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS (WebView2 is
preinstalled on Windows 11).

```bash
npm install
npm run tauri dev
```

The overlay has no taskbar button. Use the tray icon to show, hide, or quit it.

| Command                                           | What it does                 |
| ------------------------------------------------- | ---------------------------- |
| `npm run tauri dev`                               | Run the app with hot reload  |
| `npm test`                                        | Frontend unit tests (Vitest) |
| `cargo test --manifest-path src-tauri/Cargo.toml` | Rust unit tests              |
| `npm run lint` / `npm run format`                 | ESLint / Prettier            |
| `npm run tauri build`                             | Release build and installers |

Performance numbers: [docs/perf.md](docs/perf.md). Manual test checklist: [docs/testing.md](docs/testing.md).

## Design

See the [design spec](docs/superpowers/specs/2026-10-10-omniscience-design.md).

## License

[MIT](LICENSE) © Omniscience contributors
