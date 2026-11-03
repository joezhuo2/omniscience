# Graph Report - omniscience  (2026-11-03)

## Corpus Check
- 175 files · ~113,977 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 1536 nodes · 3049 edges · 114 communities (79 shown, 35 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 17 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `d3ec4701`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- sampler.rs
- positions.rs
- gpu_mode.rs
- Shell
- diagnostics.rs
- Telemetry
- devDependencies
- App.tsx
- telemetry.ts
- display.rs
- scripts
- parse.rs
- cpu_boost.rs
- tauri.conf.json
- power_mode.rs
- Dashboard.tsx
- processes.rs
- PowerLimits
- types.rs
- Omniscience — Design Spec
- metrics.ts
- ConfigView.tsx
- compilerOptions
- driver_updates.rs
- sections.ts
- charge_limit.rs
- useNativeControl
- MetricStrip.tsx
- battery.rs
- FanSource
- ProcessIo
- Dashboard.test.tsx
- ProcessList.tsx
- public_ip.rs
- UndoButton.test.tsx
- Pinger
- Manual test checklist
- compilerOptions
- SensorDetails.tsx
- README.md
- SettingsView.tsx
- Changelog
- platform.rs
- window.rs
- File Structure
- .update
- Asus
- Card.tsx
- config_gate.rs
- default.json
- RollingPeak
- hover.rs
- System configuration
- power_mode/tests.rs
- fullscreen.rs
- DriverUpdatesCard.tsx
- Results
- Features
- GpuModeCard.tsx
- powershell
- Releasing
- .prettierrc.json
- Installation
- display/tests.rs
- Development
- check-version.mjs
- [0.1.1] - 2026-10-24
- [0.1.4] - 2026-10-24
- [0.1.8] - 2026-10-27
- [0.1.9] - 2026-10-29
- [0.2.10] - 2026-10-31
- [0.1.0] - 2026-10-11
- [0.1.10] - 2026-10-30
- [0.1.11] - 2026-10-30
- [0.1.12] - 2026-10-30
- [0.1.13] - 2026-10-30
- [0.1.14] - 2026-10-30
- [0.1.16] - 2026-10-30
- [0.1.17] - 2026-10-30
- [0.1.18] - 2026-10-30
- [0.1.19] - 2026-10-30
- [0.1.2] - 2026-10-24
- [0.1.3] - 2026-10-24
- [0.1.5] - 2026-10-26
- [0.1.6] - 2026-10-26
- [0.1.7] - 2026-10-27
- [0.2.0] - 2026-10-31
- [0.2.11] - 2026-10-31
- [0.2.12] - 2026-10-31
- [0.2.13] - 2026-11-02
- [0.2.14] - 2026-11-02
- [0.2.17] - 2026-11-02
- [0.2.18] - 2026-11-02
- [0.2.19] - 2026-11-02
- [0.2.21] - 2026-11-02
- [0.2.22] - 2026-11-02
- [0.2.2] - 2026-10-31
- [0.2.3] - 2026-10-31
- [0.2.4] - 2026-10-31
- [0.2.5] - 2026-10-31
- [0.2.7] - 2026-10-31
- [0.2.8] - 2026-10-31
- [0.2.9] - 2026-10-31
- omniscience

## God Nodes (most connected - your core abstractions)
1. `Changelog` - 45 edges
2. `useTelemetryStore` - 29 edges
3. `Positions` - 21 edges
4. `Shell` - 21 edges
5. `Card()` - 20 edges
6. `Sampler` - 19 edges
7. `useNativeControl()` - 17 edges
8. `useSettingsStore` - 17 edges
9. `Telemetry` - 16 edges
10. `Detail` - 16 edges

## Surprising Connections (you probably didn't know these)
- `run()` --indirect_call--> `nextPhase()`  [INFERRED]
  src/lib/island.test.ts → src/lib/island.ts
- `read()` --calls--> `check()`  [INFERRED]
  src-tauri/src/battery_saver.rs → src-tauri/src/cpu_boost.rs
- `write_threshold()` --calls--> `check()`  [INFERRED]
  src-tauri/src/battery_saver.rs → src-tauri/src/cpu_boost.rs
- `set_expanded()` --references--> `Positions`  [EXTRACTED]
  src-tauri/src/commands.rs → src-tauri/src/positions.rs
- `set_expanded()` --references--> `Telemetry`  [EXTRACTED]
  src-tauri/src/commands.rs → src-tauri/src/telemetry/mod.rs

## Import Cycles
- None detected.

## Communities (114 total, 35 thin omitted)

### Community 0 - "sampler.rs"
Cohesion: 0.06
Nodes (53): Battery, CoreSample, GpuDetail, NetAdapter, NetMetric, Networks, PDH_HCOUNTER, PDH_HQUERY (+45 more)

### Community 1 - "positions.rs"
Cohesion: 0.06
Nodes (31): AtomicI32, AtomicU64, From, Monitor, PhysicalPosition, clamp_into(), fit(), forget_clears_the_layout_and_its_monitors() (+23 more)

### Community 2 - "gpu_mode.rs"
Cohesion: 0.10
Nodes (35): Cell, RefCell, ac_power(), apply(), apply_device(), boot_id(), crate::asus::Asus, get_gpu_mode() (+27 more)

### Community 3 - "Shell"
Cohesion: 0.09
Nodes (34): CheckMenuItem, Menu, MenuItem, get_settings(), kill_process_group(), reset_position(), AppHandle, KillReport (+26 more)

### Community 4 - "diagnostics.rs"
Cohesion: 0.10
Nodes (44): Display, Serialize, AppInfo, available(), build(), control(), control_reads_support_and_error_from_a_status(), controls() (+36 more)

### Community 5 - "Telemetry"
Cohesion: 0.09
Nodes (26): Condvar, load(), path(), AppHandle, Default, Duration, Option, PathBuf (+18 more)

### Community 6 - "devDependencies"
Cohesion: 0.05
Nodes (41): eslint, @eslint/js, eslint-plugin-react-hooks, eslint-plugin-react-refresh, globals, jsdom, devDependencies, eslint (+33 more)

### Community 7 - "App.tsx"
Cohesion: 0.10
Nodes (26): App(), NoopResizeObserver, useFitWindow(), UpdateBanner(), HOVER_EVENT, Timer, useIsland(), useTelemetry() (+18 more)

### Community 8 - "telemetry.ts"
Cohesion: 0.12
Nodes (19): TelemetryState, Battery, BatteryState, CoreMetrics, CoreSample, FanSpeeds, GpuDetail, GpuKind (+11 more)

### Community 9 - "display.rs"
Cohesion: 0.19
Nodes (33): DEVMODEW, apply_auto(), apply_mode(), auto_rate(), change(), compatible(), DisplayConfig, DisplayStatus (+25 more)

### Community 10 - "scripts"
Cohesion: 0.05
Nodes (38): @dnd-kit/core, @dnd-kit/sortable, @dnd-kit/utilities, lucide-react, dependencies, @dnd-kit/core, @dnd-kit/sortable, @dnd-kit/utilities (+30 more)

### Community 11 - "parse.rs"
Cohesion: 0.12
Nodes (31): clock_mhz(), computes_live_clock_from_base_and_performance(), core_watts(), decikelvin_to_c(), EnergyReadings, gpu_usage_by_adapter(), gpu_usage_by_pid(), gpu_usage_per_pid_takes_the_busiest_engine() (+23 more)

### Community 12 - "cpu_boost.rs"
Cohesion: 0.15
Nodes (29): battery_saver_live(), BatterySaverStatus, get_battery_saver(), on(), read(), Option, Result, String (+21 more)

### Community 13 - "tauri.conf.json"
Cohesion: 0.06
Nodes (33): https://github.com/joezhuo2/omniscience/releases/latest/download/latest.json, icons/128x128@2x.png, icons/128x128.png, icons/32x32.png, icons/icon.icns, icons/icon.ico, app, macOSPrivateApi (+25 more)

### Community 14 - "power_mode.rs"
Cohesion: 0.20
Nodes (25): GUID, apply(), get_effective(), get_power_mode(), guid_of(), guid_string(), LegacyMode, mode_of() (+17 more)

### Community 15 - "Dashboard.tsx"
Cohesion: 0.18
Nodes (18): barColor(), BatteryCard(), HealthRow(), summary(), cellStyle(), CpuHeatmap(), Dashboard(), SECTIONS (+10 more)

### Community 16 - "processes.rs"
Cohesion: 0.15
Nodes (25): HashSet, KillResult, ProcessGroup, busy_thread_reads_near_one_core_after_priming(), descendants(), finds_descendants(), groups(), groups_and_normalizes_cpu_usage() (+17 more)

### Community 17 - "PowerLimits"
Cohesion: 0.22
Nodes (18): apply(), get_power_limits(), Limits, normalize(), PowerLimits, PowerLimitsStatus, Preferences, reset_power_limits() (+10 more)

### Community 18 - "types.rs"
Cohesion: 0.17
Nodes (23): Battery, BatteryState, CoreMetrics, CoreSample, DetailMetrics, FanSpeeds, GpuDetail, GpuKind (+15 more)

### Community 19 - "Omniscience — Design Spec"
Cohesion: 0.07
Nodes (26): 10. Testing, 11. Out of scope (v1), 12. Known risks, 1. Purpose and success criteria, 2. Technology stack, 3. Window model, 4.1 Collector, 4.2 Snapshot shape (+18 more)

### Community 20 - "metrics.ts"
Cohesion: 0.17
Nodes (18): Pill(), Arc, ArcCircle(), Ring(), RingProps, ACCENTS, CRITICAL, levelColor() (+10 more)

### Community 21 - "ConfigView.tsx"
Cohesion: 0.15
Nodes (16): ConfigView(), CpuBoostCard(), CpuBoostStatus, modes, supported, Limits, PowerLimitsCard(), PowerLimitsStatus (+8 more)

### Community 22 - "compilerOptions"
Cohesion: 0.08
Nodes (23): DOM, DOM.Iterable, ES2022, src, vite/client, compilerOptions, isolatedModules, jsx (+15 more)

### Community 23 - "driver_updates.rs"
Cohesion: 0.20
Nodes (18): get_system_update_status(), OfferedUpdate, one_background_job_retains_its_result_until_rechecked(), open_update_download(), Mutex, Option, Result, Self (+10 more)

### Community 24 - "sections.ts"
Cohesion: 0.21
Nodes (15): SectionToggleProvider(), SectionLayout(), finish(), order(), CONFIG_SECTION_ORDER, ConfigSectionId, DEFAULT_SECTION_ORDER, moveSection() (+7 more)

### Community 25 - "charge_limit.rs"
Cohesion: 0.21
Nodes (16): ChargeLimit, ChargeLimitStatus, clamp(), get_charge_limit(), Preferences, read_hardware(), AppHandle, Mutex (+8 more)

### Community 26 - "useNativeControl"
Cohesion: 0.16
Nodes (14): BatterySaverCard(), BatterySaverStatus, supported, thresholds, DisplayCard(), DisplayStatus, supported, Args (+6 more)

### Community 27 - "MetricStrip.tsx"
Cohesion: 0.21
Nodes (17): FanRow(), GpuRow(), memoryText(), tempText(), valueLines(), ICONS, RingIcon(), formatBytes() (+9 more)

### Community 28 - "battery.rs"
Cohesion: 0.19
Nodes (15): battery_live(), cycle_count_without_capacity_data(), driver_status(), DriverStatus, estimate(), health_from_design_and_full_charge_capacity(), no_health_without_design_capacity(), plugged_in_and_not_charging_is_idle() (+7 more)

### Community 29 - "FanSource"
Cohesion: 0.17
Nodes (9): FanSpeeds, FanSource, AtomicBool, Drop, Mutex, Option, Self, source_stops_when_dropped_and_never_panics_without_a_sensor() (+1 more)

### Community 30 - "ProcessIo"
Cohesion: 0.18
Nodes (13): JoinHandle, KernelTrace, accumulates_only_attributed_payload_bytes(), Counts, ProcessIo, AtomicBool, Drop, HashMap (+5 more)

### Community 31 - "Dashboard.test.tsx"
Cohesion: 0.17
Nodes (10): noHealth, withDetail, SettingsView(), defaults, SETTINGS_EVENT, SettingsState, sampleDetail, sampleSnapshot (+2 more)

### Community 32 - "ProcessList.tsx"
Cohesion: 0.23
Nodes (12): ProcessList(), onKill(), Row(), COLLAPSED_PROCESSES, CONFIRM_MS, killFailure(), processRate(), ProcessSort (+4 more)

### Community 33 - "public_ip.rs"
Cohesion: 0.22
Nodes (14): builds_a_standard_query(), lookup(), parse_answer(), PublicIp, query(), reads_the_a_record(), rejects_errors_mismatched_ids_and_truncation(), response() (+6 more)

### Community 34 - "UndoButton.test.tsx"
Cohesion: 0.21
Nodes (9): base, UndoButton(), Toasts(), Toast, ToastState, useToastStore, UndoEntry, UndoState (+1 more)

### Community 35 - "Pinger"
Cohesion: 0.16
Nodes (9): ping(), Pinger, resolve(), AtomicBool, Drop, Ipv4Addr, Mutex, Option (+1 more)

### Community 36 - "Manual test checklist"
Cohesion: 0.12
Nodes (15): CPU boost, Dashboard, Display configuration, GPU configuration, Install, Manual test checklist, Pill, Power limits (+7 more)

### Community 37 - "compilerOptions"
Cohesion: 0.12
Nodes (15): ES2023, eslint.config.js, vite.config.ts, compilerOptions, allowJs, lib, module, moduleResolution (+7 more)

### Community 38 - "SensorDetails.tsx"
Cohesion: 0.22
Nodes (10): Header(), SensorDetails(), StaleDot(), useStaleness(), CPU_TEMP_SOURCE, SensorInfo, SensorKey, SENSORS (+2 more)

### Community 39 - "README.md"
Cohesion: 0.13
Nodes (12): README media, Acknowledgements, Configuration, Contributing, License, Performance, Platform support, Privacy and permissions (+4 more)

### Community 40 - "SettingsView.tsx"
Cohesion: 0.20
Nodes (7): HotkeyRecorder(), INTERVALS, SWITCH, acceleratorFromEvent(), displayAccelerator(), KeyLike, MODIFIER_KEYS

### Community 41 - "Changelog"
Cohesion: 0.14
Nodes (14): [0.1.15] - 2026-10-30, [0.2.15] - 2026-11-02, [0.2.16] - 2026-11-02, [0.2.1] - 2026-10-31, [0.2.20] - 2026-11-02, [0.2.6] - 2026-10-31, Added, Added (+6 more)

### Community 42 - "platform.rs"
Cohesion: 0.18
Nodes (9): HWND, LPARAM, LRESULT, enforce_tool_window(), is_idempotent(), prepare_overlay(), WebviewWindow, tool_window_ex_style() (+1 more)

### Community 43 - "window.rs"
Cohesion: 0.19
Nodes (6): clamp_height(), place_top_center(), Result, WebviewWindow, set_height(), top_center()

### Community 44 - "File Structure"
Cohesion: 0.15
Nodes (12): feat/scaffold Implementation Plan, File Structure, Global Constraints, Review Focus, Task 0: Create the branch, Task 1: Frontend toolchain (Vite + React + TS + Tailwind + Vitest), Task 2: Lint, format, and line endings, Task 3: Tauri backend and top-center window (+4 more)

### Community 45 - ".update"
Cohesion: 0.27
Nodes (8): dropping_below_rearms_and_restarts_the_clock(), exactly_the_threshold_is_not_high(), metrics_are_tracked_independently(), reset_forgets_progress(), CoreMetrics, Option, Vec, Sustained

### Community 46 - "Asus"
Cohesion: 0.30
Nodes (6): HANDLE, Asus, Drop, Result, Self, String

### Community 47 - "Card.tsx"
Cohesion: 0.26
Nodes (8): SectionToggleContext, Toggle, useSectionToggle(), Card(), SectionDragContext, useSectionDrag(), CardSectionId, readSectionCollapsed()

### Community 48 - "config_gate.rs"
Cohesion: 0.20
Nodes (7): enabled(), require(), FnOnce, Option, Result, String, SourceWatch

### Community 49 - "default.json"
Cohesion: 0.18
Nodes (10): core:default, core:window:allow-start-dragging, main, process:allow-restart, updater:default, description, identifier, permissions (+2 more)

### Community 50 - "RollingPeak"
Cohesion: 0.33
Nodes (7): floor_applies_when_quiet(), ignores_negative_and_nan(), new_peak_reads_full(), old_peak_expires_after_window(), RollingPeak, Self, VecDeque

### Community 51 - "hover.rs"
Cohesion: 0.22
Nodes (6): inside(), Probe, AppHandle, Option, WebviewWindow, spawn()

### Community 52 - "System configuration"
Cohesion: 0.25
Nodes (7): CPU boost, Display, Driver and software updates, GPU mode, Power limits, Power mode, System configuration

### Community 53 - "power_mode/tests.rs"
Cohesion: 0.29
Nodes (3): Preferences, migrated(), migrates_saved_auto_and_manual_modes_to_both_sources()

### Community 54 - "fullscreen.rs"
Cohesion: 0.32
Nodes (4): Rect, covers(), fullscreen_app_over(), is_shell_class()

### Community 55 - "DriverUpdatesCard.tsx"
Cohesion: 0.32
Nodes (6): DriverUpdatesCard(), OfferedUpdate, ScanStatus, checking, complete, idle

### Community 56 - "Results"
Cohesion: 0.29
Nodes (6): 0.1.0 collapsed idle breakdown, Baseline breakdown (default args), CPU fix (before tagging 0.1.0), Method, Performance, Results

### Community 57 - "Features"
Cohesion: 0.29
Nodes (7): Features, Lives in the tray, No extra software, System configuration, The dashboard, The pill, Updates

### Community 58 - "GpuModeCard.tsx"
Cohesion: 0.38
Nodes (5): GpuModeCard(), GpuStatus, Mode, modes, supported

### Community 59 - "powershell"
Cohesion: 0.33
Nodes (5): powershell(), preserves_utf8_output_and_reports_failed_commands(), Duration, Result, String

### Community 60 - "Releasing"
Cohesion: 0.40
Nodes (4): Cutting a release, One-time setup: updater signing key, Releasing, Unsigned builds

### Community 61 - ".prettierrc.json"
Cohesion: 0.40
Nodes (4): printWidth, semi, singleQuote, trailingComma

### Community 62 - "Installation"
Cohesion: 0.40
Nodes (5): Build from source, Installation, macOS (experimental), Uninstalling, Windows 10/11

### Community 64 - "Development"
Cohesion: 0.50
Nodes (4): Development, Further reading, Project layout, Tech stack

### Community 67 - "[0.1.1] - 2026-10-24"
Cohesion: 0.67
Nodes (3): [0.1.1] - 2026-10-24, Added, Fixed

### Community 68 - "[0.1.4] - 2026-10-24"
Cohesion: 0.67
Nodes (3): [0.1.4] - 2026-10-24, Added, Fixed

### Community 69 - "[0.1.8] - 2026-10-27"
Cohesion: 0.67
Nodes (3): [0.1.8] - 2026-10-27, Added, Changed

### Community 70 - "[0.1.9] - 2026-10-29"
Cohesion: 0.67
Nodes (3): [0.1.9] - 2026-10-29, Added, Changed

### Community 71 - "[0.2.10] - 2026-10-31"
Cohesion: 0.67
Nodes (3): [0.2.10] - 2026-10-31, Added, Changed

## Knowledge Gaps
- **293 isolated node(s):** `singleQuote`, `semi`, `trailingComma`, `printWidth`, `name` (+288 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **35 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Telemetry` connect `Telemetry` to `Shell`, `diagnostics.rs`?**
  _High betweenness centrality (0.064) - this node is a cross-community bridge._
- **Why does `Positions` connect `positions.rs` to `Shell`?**
  _High betweenness centrality (0.042) - this node is a cross-community bridge._
- **Why does `set_expanded()` connect `Shell` to `positions.rs`, `Telemetry`?**
  _High betweenness centrality (0.023) - this node is a cross-community bridge._
- **What connects `singleQuote`, `semi`, `trailingComma` to the rest of the system?**
  _293 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `sampler.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.05898021308980213 - nodes in this community are weakly interconnected._
- **Should `positions.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.0625 - nodes in this community are weakly interconnected._
- **Should `gpu_mode.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.09693877551020408 - nodes in this community are weakly interconnected._