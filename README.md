# Omniscience

A lightweight, open-source system monitor that floats at the top of your screen as a
Dynamic Island–style pill and expands into a compact dashboard on hover.

> **Status:** pre-alpha. All v1 features are implemented; no release has been published yet.

## Features

- **Pill:** a black Dynamic Island–style pill (styled after [islet](https://github.com/joezhuo2/islet))
  with six live rings (CPU, RAM, iGPU, dGPU, Disk, and a download/upload Network ring), colored by
  load: the metric's accent up to 70%, amber above 70%, red above 90%.
- **Hover to expand** into a dashboard with:
  - top processes grouped by app, with a CPU/RAM sort and a safe two-step "End?" button
    (critical OS processes are refused);
  - a GPU & Power card: per-GPU usage, VRAM (`5.2 / 8.0 GB`), temperature, and package watts
    with an APU/CPU/GPU/NPU breakdown where the hardware reports it;
  - a per-core CPU heatmap with tooltips (usage, clock, per-core watts, package temperature).
- **Tray only:** no taskbar button and no Alt+Tab entry. The tray menu has launch at startup,
  auto-check for updates, hide over fullscreen apps, check for updates now, settings, and quit.
- **Global show/hide hotkey** (default `Ctrl+Shift+O`), drag-to-move with the position
  remembered per monitor layout, and auto-hide over fullscreen apps.
- **No admin rights, drivers, or third-party tools.** Sensors come from Windows performance
  counters and D3DKMT; a sleeping laptop dGPU is never woken. Missing sensors show `--`.
- **In-app updates** from GitHub Releases, with a Yes/No prompt.

## Footprint

The target is at most 50 MB of RAM and below 0.1% CPU while collapsed and idle. Measured numbers
are in [docs/perf.md](docs/perf.md).

## Platforms

| Platform      | Status                                                                 |
| ------------- | ---------------------------------------------------------------------- |
| Windows 10/11 | Primary, tested                                                        |
| macOS         | Experimental. Builds in CI but is untested, and some sensors show `--` |

## Tech stack

Tauri 2, Rust, React 19, TypeScript, Tailwind CSS v4, Lucide. Theme and morph animation follow
[islet](https://github.com/joezhuo2/islet).

## Development

Prerequisites: [Node.js 24+](https://nodejs.org), [Rust (stable)](https://rustup.rs), and the
[Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS (WebView2 is
preinstalled on Windows 11).

```bash
npm install
npm run tauri dev
```

The overlay has no taskbar button. Use the tray icon to show, hide, or quit it.

| Command                                           | What it does                                                   |
| ------------------------------------------------- | -------------------------------------------------------------- |
| `npm run tauri dev`                               | Run the app with hot reload                                    |
| `npm test`                                        | Frontend unit tests (Vitest)                                   |
| `cargo test --manifest-path src-tauri/Cargo.toml` | Rust unit tests                                                |
| `npm run lint` / `npm run format`                 | ESLint / Prettier                                              |
| `npm run tauri build`                             | Release build and installers                                   |
| `cargo test` (as above)                           | Also regenerates the TypeScript types in `src/types/generated` |

Performance numbers: [docs/perf.md](docs/perf.md). Manual test checklist:
[docs/testing.md](docs/testing.md). Releasing and updater keys: [docs/releasing.md](docs/releasing.md).
`

## Design

See the [design spec](docs/superpowers/specs/2026-10-10-omniscience-design.md).

## License

[MIT](LICENSE) © Omniscience contributors
