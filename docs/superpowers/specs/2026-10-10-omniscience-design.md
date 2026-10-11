# Omniscience — Design Spec

- **Date:** 2026-10-10
- **Status:** Draft, awaiting review
- **License:** MIT ("Omniscience contributors"), open source
- **Repository:** [joezhuo2/omniscience](https://github.com/joezhuo2/omniscience)
- **Bundle identifier:** `io.github.joezhuo2.omniscience`

## 1. Purpose and success criteria

Omniscience is a lightweight, open-source system monitor that lives at the top of the screen as a
"Dynamic Island" style pill and expands into a compact dashboard on hover. It is intended for public
release on GitHub with installers and auto-update.

Success means:

1. The collapsed pill shows six live metrics at a glance with threshold coloring.
2. The expanded dashboard shows processes (with safe kill), GPU/power, and a per-core heatmap.
3. The pill to dashboard transition is a smooth spring morph with no window flicker.
4. Idle cost (collapsed, 1 s polling) stays **at or below 50 MB total private working set** (whole
   process tree, as Task Manager reports it) and **below 0.1% CPU**.
5. No admin rights, kernel drivers, or third-party tools are required. Missing sensors degrade to `--`.
6. Windows is the tested platform. macOS builds in CI and is labeled **experimental, untested**.

## 2. Technology stack

| Layer | Choice |
|---|---|
| Shell | Tauri 2 |
| Backend | Rust (stable), `sysinfo`, `windows` crate (PDH, Win32), `nvml-wrapper` (runtime-loaded), `ts-rs` |
| Frontend | React 19, TypeScript, Vite, Tailwind CSS v4, `motion` (`motion/react`, formerly Framer Motion), Lucide React, Zustand |
| Plugins | `tauri-plugin-store`, `tauri-plugin-autostart`, `tauri-plugin-global-shortcut`, `tauri-plugin-updater`, `tauri-plugin-process` |
| Tests | `cargo test`, Vitest, Testing Library |
| Tooling | npm, ESLint, Prettier, `cargo fmt`, `cargo clippy` |
| CI/CD | GitHub Actions (`windows-latest`, `macos-latest`), `tauri-apps/tauri-action` for releases |

Dependency versions are pinned to the latest stable releases at scaffold time.

## 3. Window model

- `decorations: false`, `transparent: true`, `alwaysOnTop: true`, `skipTaskbar: true`, `resizable: false`,
  `shadow: false`. macOS requires `macOSPrivateApi: true` for transparency (excludes the Mac App Store,
  which is acceptable).
- **Collapsed size:** about 400 x 42 px. **Expanded size:** about 400 x 560 px. Final values are tuned in
  `feat/morph`.
- **Morph strategy: snap the window, animate the content.** On expand, Rust resizes the native window to
  the expanded size instantly (keeping its top-center anchor) and the React content springs open inside
  it. On collapse, the content springs closed first and the window shrinks when the animation completes.
  Per-frame native resizing is avoided because it flickers with transparent windows on Windows.
- **Glass look: faux glass.** A dark pill at about 85% opacity with a subtle border, gradient, and inner
  highlight. CSS `backdrop-filter` cannot blur the desktop behind a webview, and native Acrylic/vibrancy
  does not suit a rounded pill, so neither is used.
- **Position:** defaults to top-center of the primary monitor. The user drags it by the expanded
  dashboard header (the pill itself is not draggable, because hover expands it). The position is saved
  per monitor configuration (keyed by the monitor layout) and validated on startup so the window never
  restores off-screen.
- **Theme:** dark only.

## 4. Telemetry engine (Rust)

### 4.1 Collector

A single background thread owns all sensor handles and runs a fixed-interval loop (default 1 s,
configurable 0.5 to 5 s). Each tick it builds a `Snapshot` and emits it to the frontend as the Tauri
event `telemetry://tick`. The frontend never polls.

Sampling is tiered:

- **Core tier** (always, while visible): CPU total %, RAM used/total, disk active %, network down/up
  bytes per second, iGPU %, dGPU %, package watts.
- **Detail tier** (only while expanded): per-core load %, per-core clock MHz, per-core watts, thermal
  zone temperatures, per-GPU VRAM/shared memory and temperature, power breakdown (APU/CPU/GPU/NPU), top
  processes.

Detail sampling starts on `set_expanded(true)` and stops on `set_expanded(false)`. When it stops, the
process list and detail buffers are dropped (memory is released, not cached). Because per-core and
per-process CPU percentages need two samples, the first detail payload arrives about one interval after
expanding. The frontend shows skeletons until then.

While the window is hidden (hotkey, tray, or fullscreen auto-hide), the loop pauses entirely.

If the collector thread panics, a supervisor restarts it. The frontend shows a stale-data indicator if no
tick arrives for 3 s.

### 4.2 Snapshot shape

All sensor-derived fields are optional. A missing or failed sensor yields `null`, and the failure is
logged once (no repeated warnings).

```rust
struct Snapshot {
    timestamp_ms: u64,
    core: CoreMetrics,
    detail: Option<DetailMetrics>, // present only while expanded and warmed up
}

struct CoreMetrics {
    cpu: Metric,                 // percent
    ram: MemoryMetric,           // used_bytes, total_bytes, percent, level
    disk: Metric,                // active-time percent
    net: NetMetric,              // down_bps, up_bps, down_pct, up_pct (vs rolling peak), level
    igpu: Option<Metric>,
    dgpu: Option<Metric>,
    package_watts: Option<f32>,
}

struct Metric { percent: f32, level: Level }
enum Level { Normal, Warning, Critical }

struct DetailMetrics {
    cores: Vec<CoreSample>,      // id, usage_pct, freq_mhz, watts: Option<f32>, level
    temps: Vec<TempSample>,      // label (e.g. "ACPI THRM"), celsius
    gpus: Vec<GpuDetail>,        // name, kind (Integrated/Discrete), usage_pct,
                                 // mem_used_bytes, mem_total_bytes, temp_c: Option<f32>
    power: Option<PowerBreakdown>, // apu, cpu, gpu, npu watts, each Option<f32>
    processes: Vec<ProcessGroup>,  // name, pids, cpu_pct, mem_bytes
}
```

TypeScript types are generated from these structs with `ts-rs` into `src/types/generated/`, so the two
sides cannot drift. Exact field names may be refined during implementation. The structure above is the
contract.

### 4.3 Threshold logic

`classify(percent, warn, crit) -> Level` lives in Rust and is the only source of truth:

- `percent <= warn` (default 70): `Normal`
- `warn < percent <= crit` (default 90): `Warning`
- `percent > crit`: `Critical`

The thresholds are user-configurable. The frontend maps `Level` to colors and never re-derives it.

### 4.4 Sources

| Metric | Windows | macOS |
|---|---|---|
| CPU total / per-core %, per-core MHz | `sysinfo` | `sysinfo` |
| RAM | `sysinfo` | `sysinfo` |
| Network down/up | `sysinfo` networks (sum of non-loopback interfaces) | `sysinfo` |
| Disk active % | PDH `\PhysicalDisk(_Total)\% Idle Time`, so active = 100 - idle | IOKit block-storage busy time (experimental) |
| iGPU / dGPU % | PDH `\GPU Engine(*)\Utilization Percentage`, grouped by adapter LUID, max over engine types; LUID mapped to adapter name and kind via DXGI | `--` |
| GPU memory | PDH `\GPU Adapter Memory(*)\Dedicated Usage` / `Shared Usage`; NVML for NVIDIA totals | `--` |
| dGPU temp | NVML, **detail tier only and only if PDH shows the dGPU active** (NVML calls can wake a sleeping Optimus dGPU) | `--` |
| Package / APU / CPU / GPU / NPU watts | PDH `\Energy Meter(*)\Power` (milliwatts), no admin needed | `--` |
| Per-core watts | PDH `\Energy Meter(RAPL_Package0_CoreN_CORE)\Power` where present | `--` |
| Temperatures | PDH `\Thermal Zone Information(*)\High Precision Temperature` (tenths of Kelvin) | `--` |

Per-core temperature is not available on AMD or through Windows without a kernel driver, so it is not
displayed. Temperatures are shown as package/zone readings and labeled that way.

**Network rolling peak:** each direction's ring percent = current bps / max(bps over the last 60 s,
1 MB/s floor). The ring color uses the same thresholds on this percent.

**GPU discovery:** adapters come from PDH instance names (LUIDs) joined with DXGI adapter descriptions.
Integrated vs. discrete is decided by DXGI dedicated video memory size and vendor (an adapter with
less than 1 GiB dedicated memory counts as integrated). Machines with a single GPU show one GPU ring.
Machines with none show both GPU rings in the missing state.

### 4.5 Process management

- Processes are grouped by executable name (case-insensitive). Each group sums CPU % and memory. The
  dashboard shows the top 8 by the selected sort (CPU or RAM).
- `kill_process_group(name: String) -> KillReport` kills every PID in the group, subject to the guard:
  - **Blocklist** (case-insensitive, `.exe` optional): `system`, `idle`, `registry`, `smss`, `csrss`,
    `wininit`, `winlogon`, `services`, `lsass`, `lsaiso`, `svchost`, `explorer`, `dwm`, `fontdrvhost`,
    `sihost`, `ctfmon`, `memcompression`, `secure system`, `audiodg`, `spoolsv`, `msmpeng`,
    `launchd`, `kernel_task`, `windowserver`, `loginwindow`, `systemuiserver`, `dock`, `finder`,
    `mds`, `coreaudiod`.
  - PIDs `0` and `4` are always refused.
  - Omniscience's own PID (and its WebView2 child processes) are refused.
- `KillReport` lists the result per PID: `Killed`, `Protected`, `AccessDenied`, `NotFound`, or
  `Failed(message)`. Processes owned by other users or running elevated will normally report
  `AccessDenied`, because the app does not run as admin.
- **UI:** the first click turns the row's X into a red "End?" button, and a second click within 3 s
  kills. The row is removed optimistically. If any PID is not `Killed` or `NotFound`, the row returns
  and a toast explains why.

## 5. Frontend

### 5.1 State

A small Zustand store holds the latest `Snapshot`, the UI state machine, settings, and toasts.
Components subscribe to narrow selectors, so a tick re-renders only what changed. One hook,
`useTelemetry`, subscribes to `telemetry://tick`.

UI state machine: `collapsed`, `expanding`, `expanded`, `collapsing`, `hidden`.

- Cursor enters the pill and stays 250 ms: `expanding`. Calls `set_expanded(true)`, then animates.
- Cursor leaves for 400 ms, or the user presses Esc: `collapsing`. Animates, then calls
  `set_expanded(false)` on animation complete, then `collapsed`.
- Re-entering during the 400 ms grace period cancels the collapse.

### 5.2 Collapsed pill

- About 400 x 42 px, rounded-full, faux glass.
- Six rings, **no text**: CPU, RAM, iGPU, dGPU, Disk, Net. Each ring has a Lucide icon inside.
- Net is a concentric ring: outer = download, inner = upload.
- Ring color: the per-metric accent when `Normal`, orange `#F97316` when `Warning`, red `#EF4444` when
  `Critical` with a soft pulse. The pulse is a CSS animation that runs only while critical, so idle CPU
  stays near zero.
- Accents ("cool neon"): CPU `#22D3EE`, RAM `#A78BFA`, iGPU `#34D399`, dGPU `#A3E635`,
  Disk `#38BDF8`, Net `#F472B6`.
- Ring values change with a short CSS transition on `stroke-dashoffset` per tick. There are no
  always-running animation loops.
- A missing sensor shows its ring dimmed and dashed.
- Hover gives a subtle glow and lift before expansion.

### 5.3 Expanded dashboard

The dashboard mounts on expand, shows skeleton loaders for every detail-tier section until the first
detail payload arrives, and **unmounts on collapse**.

Top to bottom:

1. **Header (drag region):** "Omniscience" wordmark, package watts badge, hottest temperature badge,
   settings gear.
2. **Metric strip:** the six rings, larger, with values: CPU `23%`, RAM `12.4 / 31.1 GB`, iGPU `4%`,
   dGPU `12%`, Disk `3%`, Net `↓1.2 MB/s ↑80 KB/s`.
3. **GPU & Power card:** one row per GPU with name, usage, memory (`5.2 / 8.0 GB` dedicated for
   discrete, shared usage for integrated), and temperature if available. Below it, package watts with
   an APU/CPU/GPU/NPU breakdown where available, otherwise `-- W`.
4. **CPU heatmap:** a responsive grid with one cell per logical core, colored by load (accent, then
   orange, then red, with intensity scaled by percent). The hover tooltip shows Core ID, usage %, clock
   MHz, per-core watts (if available), and package/zone temperature labeled "package".
5. **Processes:** a CPU/RAM sort toggle and the top 8 groups with name, instance count, CPU %, and
   memory, plus the two-step kill button.

### 5.4 Settings view

The gear replaces the dashboard body with the settings view. A back arrow returns.

- Warning and critical thresholds (default 70 / 90)
- Poll interval (0.5 to 5 s, default 1 s)
- Global hotkey (default `Ctrl+Shift+O` / `Cmd+Shift+O`) with rebinding
- Toggles: launch at startup, auto-check for updates, hide over fullscreen apps
- Reset position

Settings are saved with `tauri-plugin-store` and applied live.

### 5.5 Toasts and update banner

- Toasts appear inside the dashboard bottom edge and auto-dismiss after 4 s.
- **Update banner:** when auto-check is on and a newer version is found at launch, the pill expands
  into a slim banner reading "v{x} available. Update? Yes / No". Yes downloads, installs, and
  relaunches. No dismisses it until the next launch.

## 6. App shell

### 6.1 Tray

There is no taskbar entry and no window chrome, so the tray icon is the main control surface.
Right-click menu:

- Show / Hide Omniscience
- ☐ Launch at startup
- ☐ Auto-check for updates (default on)
- ☑ Hide over fullscreen apps (default on)
- Check for updates now
- Settings…
- Quit

The checkbox items share state with the settings view.

### 6.2 Fullscreen auto-hide (Windows)

A separate lightweight timer (about every 2 s) runs whenever this option is enabled. It keeps running
while the overlay is hidden for fullscreen, because the collector is paused then. It reads the
foreground window's rect via `GetForegroundWindow` / `MonitorFromWindow` and compares it with the
monitor rect, excluding the desktop and shell windows. When a fullscreen window covers the overlay's monitor, the app hides the overlay and
restores it afterwards. On macOS this is a no-op (experimental platform).

### 6.3 Hidden state

When hidden by the hotkey, the tray, or fullscreen auto-hide, the collector pauses and WebView2 is
suspended (`ICoreWebView2_3::TrySuspend`). It resumes on show.

## 7. Memory and CPU budget

- **Target:** collapsed idle at or below 50 MB total private working set across the process tree
  (Tauri plus all WebView2 processes), and below 0.1% CPU.
- **Techniques:**
  - Detail tier only while expanded. Process data and detail buffers are dropped on collapse.
  - The dashboard is unmounted on collapse.
  - WebView2 `MemoryUsageTargetLevel` (`ICoreWebView2_19`) is set to `Low` while collapsed or hidden
    and to `Normal` while expanded.
  - The webview is suspended while hidden.
  - `sysinfo` refreshes only the kinds each tier needs, and buffers are reused.
  - No NVML calls in the core tier.
  - No continuous animations except the critical pulse.
  - Lean bundle: tree-shaken Lucide, `motion` used through its React entry, no UI kit.
- **Measurement:** `docs/perf.md` records idle collapsed, expanded, and hidden memory and CPU, measured
  on the reference machine (Ryzen AI 9 HX 370, Radeon 890M + RTX 5060 Laptop, 32 GB, Windows 11) at
  the end of each branch that affects it.

## 8. Release and distribution

- **CI** (`.github/workflows/ci.yml`): on push and pull request, run lint, typecheck, Vitest,
  `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, and `tauri build` (no bundle upload) on
  `windows-latest` and `macos-latest`.
- **Release** (`.github/workflows/release.yml`): on `v*` tags, `tauri-action` builds the `.msi` and
  NSIS `.exe` (Windows) and `.dmg` (macOS, universal), signs the update artifacts with the updater key,
  and publishes a GitHub Release including `latest.json`.
- **Updater endpoint:** `https://github.com/joezhuo2/omniscience/releases/latest/download/latest.json`.
- **Signing:** updater artifacts are signed with a Tauri updater keypair. The private key and password
  are stored as GitHub secrets (`TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`), and
  the README documents how to generate them. OS code signing (Authenticode, Apple notarization) is out
  of scope, so the README warns about SmartScreen and Gatekeeper prompts.
- These workflows run once commits are pushed to GitHub. Pushing is done by the maintainer;
  development commits stay local until then.

## 9. Repository workflow

- `main` starts with: LICENSE, README stub, CHANGELOG ([Keep a Changelog](https://keepachangelog.com)),
  `.gitignore`, this spec, and the implementation plan.
- There is one feature branch at a time. Each is built with small Conventional Commits. When a branch is
  done, work stops; the maintainer merges it and requests the next one.
- Every branch updates README (features, setup, status) and the CHANGELOG `Unreleased` section.

Branches, in order:

1. `feat/scaffold`: Tauri 2 + Vite + React 19 + TS + Tailwind v4 + motion + Lucide; window config;
   ESLint/Prettier/Vitest; `cargo fmt`/`clippy`; CI workflow.
2. `feat/telemetry`: collector with tiers, sources, threshold logic, rolling peak, kill guard, `ts-rs`
   types, tick event.
3. `feat/pill`: store, `useTelemetry`, `Ring` (single and concentric), threshold colors, missing-sensor
   state.
4. `feat/dashboard`: metric strip, GPU & Power card, heatmap with tooltips, process list with two-step
   kill and rollback, skeletons, toasts.
5. `feat/morph`: hover state machine, `set_expanded` window snap, spring transitions
   (`type: "spring", stiffness: 300, damping: 25`), header drag, per-monitor position persistence,
   WebView2 memory level switching.
6. `feat/app-shell`: tray menu, autostart, global hotkey, settings store and view, fullscreen auto-hide,
   suspend while hidden.
7. `feat/updater`: updater plugin, launch banner, release workflow, signing docs.

## 10. Testing

- **Rust unit tests, written test-first:** `classify` boundaries (70/70.1/90/90.1), rolling peak
  window and floor, unit conversions (mW to W, tenths of Kelvin to °C), PDH instance-name parsing
  (LUID, engine type, RAPL core index), GPU integrated/discrete classification, process grouping, kill
  guard (blocklist with and without `.exe`, case-insensitivity, PIDs 0/4, own PID).
- **Frontend (Vitest + Testing Library):** byte and rate formatters, level-to-color mapping, hover state
  machine timing (fake timers), optimistic kill and rollback reducer, `Ring` rendering at the edges and
  in the missing state.
- **Manual:** `docs/testing.md` holds a checklist for sensors, window behavior, the morph, tray, hotkey,
  fullscreen hide, and memory/CPU. The results are reported at each branch handoff.
- macOS is verified only by CI compile and tests.

## 11. Out of scope (v1)

- OS code signing and notarization
- Light theme
- Per-core temperature
- Kernel drivers or LibreHardwareMonitor/HWiNFO integration
- Linux
- Historical graphs and logging
- Localization
- Mac App Store distribution

## 12. Known risks

- **WebView2 memory floor:** the 50 MB idle target depends on `MemoryUsageTargetLevel` being effective.
  A baseline is measured at the end of `feat/pill` (before memory-level switching exists) so problems
  surface before the rest of the UI is built. It is re-measured after `feat/morph`.
- **Sensor availability varies by machine:** Energy Meter and thermal zone counters depend on OEM
  firmware. Every field degrades to `--`.
- **Transparent windows on Windows:** known WebView2 and DWM quirks with transparency on some GPU
  drivers. Mitigated by avoiding native resizing during animation.
- **Untested macOS:** CI only guarantees it compiles. It is labeled experimental.
