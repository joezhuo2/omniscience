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
