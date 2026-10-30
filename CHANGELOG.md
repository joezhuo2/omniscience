# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.11] - 2026-10-30

### Changed

- Center the collapsed pill rings without an empty dGPU slot when the discrete GPU is absent or asleep. The expanded dashboard keeps all six indicators. A sampled GPU at 0% remains visible; loading retains all six placeholders ([#21](https://github.com/joezhuo2/omniscience/issues/21)).

## [0.1.10] - 2026-10-30

### Fixed

- Disable touchpad overscroll bounce on the overlay, dashboard and expanded process list while preserving normal scrolling ([#20](https://github.com/joezhuo2/omniscience/issues/20)).

## [0.1.9] - 2026-10-29

### Added

- Search Top processes by app name or PID, with case-insensitive matching, skeleton loaders while typing, a clear button and an empty-results message ([#18](https://github.com/joezhuo2/omniscience/issues/18)).

### Changed

- Process search includes every running app group, including quiet apps outside the top CPU, RAM and GPU lists. Filtering happens before sorting and limiting displayed rows.

## [0.1.8] - 2026-10-27

### Added

- Inner CPU, iGPU and dGPU temperature rings, scaled to 105°C, with live readings in the collapsed pill ([#14](https://github.com/joezhuo2/omniscience/issues/14)).
- CPU temperature on a third metric-strip line and GPU temperatures on a second line; the CPU tooltip explains the ACPI system-temperature proxy.

### Changed

- Removed duplicate temperatures from the GPU & Power card. Missing temperatures hide the inner ring and show an em dash in the metric strip. Sleeping discrete GPUs remain unqueried.


## [0.1.7] - 2026-10-27

### Fixed

- Battery card uses the existing skeleton style until detail telemetry arrives, then hides on machines without a battery ([#15](https://github.com/joezhuo2/omniscience/issues/15)).

## [0.1.6] - 2026-10-26

### Fixed

- Expanded dashboard keeps a fixed bottom inset around its rounded corners while scrolling, including with Show more open ([#10](https://github.com/joezhuo2/omniscience/issues/10)).

## [0.1.5] - 2026-10-26

### Changed

- Network card shows the adapter, IPv4 address and ping latency on one line, without duplicate download/upload rates ([#11](https://github.com/joezhuo2/omniscience/issues/11)).

## [0.1.4] - 2026-10-24

### Added

- Battery card for laptops: charge, charging or draining with the rate in watts, and time left or time to full, read from the battery driver with no admin rights. Hidden on machines without a battery ([#2](https://github.com/joezhuo2/omniscience/issues/2)).

### Fixed

- A tall dashboard (for example with Show more open) now scrolls inside the window instead of being cut off at the bottom.

## [0.1.3] - 2026-10-24

### Added

- GPU tab in Top processes: each app's GPU usage (its busiest engine, as in Task Manager) and dedicated GPU memory, sortable like CPU and RAM ([#3](https://github.com/joezhuo2/omniscience/issues/3)).

## [0.1.2] - 2026-10-24

### Added

- GPU fan speed in the GPU & Power card, read from the driver like temperature ([#4](https://github.com/joezhuo2/omniscience/issues/4)).
- Throttle indicators: a Throttling chip on a GPU at its thermal limit, and a "Throttled −N%" chip on the CPU cores card when power or thermal limits hold the CPU back.
- Network card: each active adapter with its IPv4 address and live download/upload rates.
- Ping latency to a configurable target (Settings > Ping target, `1.1.1.1` by default, empty turns it off), measured only while the dashboard is open.
- Optional public IP (Settings > Show public IP, off by default), looked up with one DNS query to OpenDNS at most every ten minutes.

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

[Unreleased]: https://github.com/joezhuo2/omniscience/compare/v0.1.8...HEAD
[0.1.8]: https://github.com/joezhuo2/omniscience/compare/v0.1.7...v0.1.8
[0.1.7]: https://github.com/joezhuo2/omniscience/compare/v0.1.6...v0.1.7
[0.1.6]: https://github.com/joezhuo2/omniscience/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/joezhuo2/omniscience/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/joezhuo2/omniscience/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/joezhuo2/omniscience/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/joezhuo2/omniscience/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/joezhuo2/omniscience/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/joezhuo2/omniscience/releases/tag/v0.1.0
