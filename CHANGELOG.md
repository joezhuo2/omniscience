# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2026-10-24

### Added

- The overlay snaps to nearby screen edges and to the top-center of its monitor when you drop it ([#5](https://github.com/joezhuo2/omniscience/issues/5)).
- Its position is now also remembered per monitor, so it returns to its spot on a monitor even after the rest of the monitor layout changes.

### Fixed

- The overlay can no longer be left partly off-screen: after a drag, and when a saved position is restored, it is pulled fully onto the nearest monitor.
- Opening the dashboard near the bottom of a monitor lifts it so the whole dashboard is visible, and drops it back on collapse.
- Settings > Reset position also forgets per-monitor positions.

## [0.1.0] - 2026-10-11

First public release.

### Added

- Dynamic Island–style overlay: a black, frameless, transparent, always-on-top pill at the top-center of the primary monitor, themed after islet with iOS system colors. It has no taskbar button and does not appear in Alt+Tab.
- Collapsed pill: six icon rings (CPU, RAM, iGPU, dGPU, Disk, and a concentric download/upload Network ring) with no text, colored by threshold (accent, amber above 70%, pulsing red above 90%) and dashed when a sensor is missing.
- Hover to expand: the island opens the moment the cursor reaches the pill with a 280 ms clip-path morph and a delayed content fade, and starts collapsing the moment the cursor leaves; coming back mid-collapse reverses it at once. Esc also closes it. Clicks on the transparent margins pass through to the window underneath.
- Expanded dashboard: header with package watts and hottest temperature, labeled metric strip (CPU with its live clock in GHz, RAM like `15/31 GB`, download and upload on two lines), GPU & Power card (per-GPU usage, VRAM like `5.2 / 8.0 GB`, temperature, APU/CPU/GPU/NPU watts), per-core CPU heatmap with hover tooltips (usage, clock, per-core watts, package temperature), and skeleton loaders until detail data arrives.
- Top processes grouped by app with a CPU/RAM sort toggle, 3 rows by default and a scrollable "Show more" list, and a two-step "End?" button. Critical OS processes, PIDs 0 and 4, and Omniscience's own process tree are refused; the row comes back with a toast explaining why.
- Amber indicator in the dashboard header when telemetry stops arriving.
- Telemetry engine: one background thread samples CPU, RAM, disk active time, network, per-GPU usage, package power and temperatures each second; per-core, per-GPU and process detail is sampled only while the dashboard is open.
- GPU names, VRAM and temperature via D3DKMT and power via Windows Energy Meter counters, with no admin rights, no drivers and no NVML, so a sleeping laptop dGPU is never woken.
- Tray menu: Show / Hide, Launch at startup, Auto-check for updates, Hide over fullscreen apps (on by default), Check for updates now, Settings…, Quit.
- Settings view (gear in the dashboard header or tray > Settings…): warning/critical thresholds with step arrows, a segmented update-interval picker (0.5-5 s), a show/hide hotkey recorder (default Ctrl+Shift+O), iOS-style switches for the tray toggles, and Reset position. Settings are saved to the app config folder.
- Drag the overlay by the pill (the row of rings); its position is remembered per monitor layout and never restored off-screen.
- The overlay hides itself while a fullscreen app covers its monitor and comes back afterwards. Telemetry pauses and WebView2 memory is trimmed whenever the overlay is hidden.
- WebView2 runs with software compositing and a low memory target while collapsed, to keep idle memory down.
- A second launch reveals the running instance instead of starting another.
- In-app updates: with auto-check on, a launch-time check against GitHub Releases opens the island with "vX available. Update? Yes / No" until answered; Yes downloads, verifies, installs and restarts. Tray > Check for updates now reports when you are up to date.
- Windows installers (NSIS and MSI) and an experimental universal macOS dmg, built and drafted to GitHub Releases with signed updater artifacts when a version tag is pushed.
- GitHub Actions CI for Windows and macOS (lint, format check, tests, clippy, release build).
- MIT license, README, design spec, manual test checklist, and performance measurement scripts.

[Unreleased]: https://github.com/joezhuo2/omniscience/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/joezhuo2/omniscience/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/joezhuo2/omniscience/releases/tag/v0.1.0
