# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Project license (MIT), README, and changelog.
- Design spec for Omniscience v1.
- Tauri 2 app shell with a frameless, transparent, always-on-top window placed at the top-center of the primary monitor.
- Tray icon with Show / Hide and Quit. The overlay has no taskbar button and does not appear in Alt+Tab.
- React 19, TypeScript, Tailwind CSS v4, and Vitest frontend scaffold.
- ESLint, Prettier, rustfmt, and clippy configuration.
- GitHub Actions CI for Windows and macOS (lint, test, build).
- Memory/CPU measurement script, window-style probe, and performance log.
- Telemetry engine: one background thread samples CPU, RAM, disk active time, network, per-GPU usage, package power and temperatures each second and emits a typed snapshot; per-core, per-GPU and process detail is sampled only while the dashboard is open.
- GPU names, VRAM and temperature via D3DKMT and power via Windows Energy Meter counters, with no admin rights, no drivers and no NVML (a sleeping laptop dGPU is never woken).
- Guarded `kill_process_group` command that refuses critical OS processes, PIDs 0 and 4, and Omniscience's own process tree.
- Collapsed pill: six icon rings (CPU, RAM, iGPU, dGPU, Disk, and a concentric download/upload Network ring) with no text, colored by threshold (accent, orange above 70%, pulsing red above 90%) and dashed when a sensor is missing.
- Expanded dashboard: header with package watts and hottest temperature, labeled metric strip, GPU & Power card (per-GPU usage, VRAM like `5.2 / 8.0 GB`, temperature, APU/CPU/GPU/NPU watts), per-core CPU heatmap with hover tooltips (usage, clock, per-core watts, package temperature), and skeleton loaders until detail data arrives.
- Top processes grouped by app with a CPU/RAM sort toggle and a two-step "End?" button; rows disappear immediately and come back with a toast if the kill is refused.
