# Omniscience

A lightweight, open-source system monitor that floats at the top of your screen as a
Dynamic Island–style pill and expands into a compact dashboard on hover.

> **Status:** pre-alpha, in design. Nothing to install yet.

## Planned features

- Six live rings in the pill (CPU, RAM, iGPU, dGPU, Disk, Network), colored by load
  (normal, warning above 70%, critical above 90%)
- Hover to expand into a dashboard with:
  - Top processes, grouped by app, with a safe two-step "End task"
  - GPU and power card (VRAM, per-GPU usage, package watts)
  - Per-core CPU heatmap with tooltips
- Tray menu: launch at startup, auto-update, hide over fullscreen apps
- Global show/hide hotkey
- No admin rights or kernel drivers required
- Idle footprint target: at most 50 MB RAM and below 0.1% CPU

## Platforms

| Platform | Status |
|---|---|
| Windows 10/11 | Primary, tested |
| macOS | Experimental. Builds in CI but is untested, and some sensors show `--` |

## Tech stack

Tauri 2, Rust, React 19, TypeScript, Tailwind CSS v4, Motion, Lucide.

## Design

See the [design spec](docs/superpowers/specs/2026-10-10-omniscience-design.md).

## License

[MIT](LICENSE) © Omniscience contributors
