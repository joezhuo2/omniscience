# Graph Report - omniscience  (2026-11-03)

## Corpus Check
- 178 files · ~72,918 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 10 file(s) not represented in the graph (top: (none) 5, .css 2, .toml 1)

## Summary
- 1681 nodes · 3479 edges · 116 communities (75 shown, 41 thin omitted)
- Extraction: 96% EXTRACTED · 4% INFERRED · 0% AMBIGUOUS · INFERRED: 122 edges (avg confidence: 0.85)
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
- settings.rs
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
- charge_limit.rs
- types.rs
- Omniscience — Design Spec
- metrics.ts
- ConfigView.tsx
- compilerOptions
- driver_updates.rs
- sections.ts
- with
- useNativeControl
- MetricStrip.tsx
- windows.rs
- super
- ProcessIo
- vitest
- ProcessList.tsx
- project.yml
- ConfigView.tsx
- check
- Manual test checklist
- compilerOptions
- tray.rs
- README.md
- Telemetry
- Changelog
- platform.rs
- window.rs
- File Structure
- hover.rs
- Asus
- Card.tsx
- asus.rs
- default.json
- RollingPeak
- dependencies
- System configuration
- commands.rs
- fullscreen.rs
- DriverUpdatesCard.tsx
- Results
- Features
- Thresholds
- Releasing
- .prettierrc.json
- Installation
- package.json
- Development
- check-version.mjs
- store.ts
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
- shell.rs
- eslint.config.js
- omniscience
- KillOutcome
- detail
- @tailwindcss/vite
- .sample
- BatterySaverCard.tsx

## God Nodes (most connected - your core abstractions)
1. `Changelog` - 45 edges
2. `vitest` - 34 edges
3. `Card()` - 34 edges
4. `useTelemetryStore` - 29 edges
5. `@tauri-apps/api` - 24 edges
6. `react` - 23 edges
7. `@testing-library/react` - 23 edges
8. `Shell` - 20 edges
9. `Positions` - 19 edges
10. `Telemetry` - 17 edges

## Surprising Connections (you probably didn't know these)
- `Task 1: Frontend toolchain (Vite + React + TS + Tailwind + Vitest)` --references--> `App()`  [INFERRED]
  docs/superpowers/plans/2026-10-10-feat-scaffold.md → src/App.tsx
- `10. Testing` --references--> `Ring()`  [INFERRED]
  docs/superpowers/specs/2026-10-10-omniscience-design.md → src/components/Ring.tsx
- `4.5 Process management` --references--> `KillReport`  [INFERRED]
  docs/superpowers/specs/2026-10-10-omniscience-design.md → src/types/generated/KillReport.ts
- `4.3 Threshold logic` --references--> `Level`  [INFERRED]
  docs/superpowers/specs/2026-10-10-omniscience-design.md → src/types/generated/Level.ts
- `9. Repository workflow` --references--> `Ring()`  [INFERRED]
  docs/superpowers/specs/2026-10-10-omniscience-design.md → src/components/Ring.tsx

## Import Cycles
- None detected.

## Communities (116 total, 41 thin omitted)

### Community 0 - "sampler.rs"
Cohesion: 0.08
Nodes (21): CORE_COUNTERS, CPU_BASE, CPU_LIMIT, CPU_PERF, Detail, DETAIL_COUNTERS, DGPU_ACTIVE_PCT, DISK_IDLE (+13 more)

### Community 1 - "positions.rs"
Cohesion: 0.08
Nodes (36): A, B, clamp_into(), clamp_keeps_an_oversized_window_at_the_top_left(), clamp_leaves_an_inside_window_alone(), clamp_pulls_a_window_back_inside(), does_not_lift_a_window_that_fits(), fit() (+28 more)

### Community 2 - "gpu_mode.rs"
Cohesion: 0.10
Nodes (23): apply(), apply_device(), boot_id(), crate::asus::Asus, get_gpu_mode(), GpuConfig, GpuDevice, GpuMode (+15 more)

### Community 4 - "diagnostics.rs"
Cohesion: 0.09
Nodes (34): AppInfo, available(), build(), control(), control_reads_support_and_error_from_a_status(), controls(), ControlStatus, core() (+26 more)

### Community 5 - "settings.rs"
Cohesion: 0.27
Nodes (7): load(), path(), sanitizes_cpu_temp_limits(), sanitizes_out_of_range_values(), save(), Settings, trims_the_ping_target()

### Community 6 - "devDependencies"
Cohesion: 0.13
Nodes (15): devDependencies, prettier@^3.9.10, tailwindcss@^4.3.3, @testing-library/jest-dom@^7.0.1, @testing-library/react@^16.3.3, @types/node@^24.19.2, @types/react@^19.3.0, @types/react-dom@^19.3.0 (+7 more)

### Community 7 - "App.tsx"
Cohesion: 0.06
Nodes (44): @tauri-apps/plugin-updater@^2.13.3, @tauri-apps/api, @tauri-apps/plugin-updater, App(), NoopResizeObserver, useFitWindow(), Toasts(), HotkeyRecorder() (+36 more)

### Community 8 - "telemetry.ts"
Cohesion: 0.14
Nodes (17): Battery, BatteryState, CoreMetrics, CoreSample, DetailMetrics, FanSpeeds, GpuDetail, GpuKind (+9 more)

### Community 9 - "display.rs"
Cohesion: 0.13
Nodes (28): apply_auto(), apply_mode(), auto_rate(), change(), compatible(), DisplayConfig, DisplayStatus, enum_mode() (+20 more)

### Community 10 - "scripts"
Cohesion: 0.17
Nodes (12): scripts, build, dev, format, format:check, lint, preview, tauri (+4 more)

### Community 11 - "parse.rs"
Cohesion: 0.13
Nodes (29): clock_mhz(), computes_live_clock_from_base_and_performance(), converts_units(), core_watts(), decikelvin_to_c(), EnergyReadings, gpu_usage_by_adapter(), gpu_usage_by_pid() (+21 more)

### Community 12 - "cpu_boost.rs"
Cohesion: 0.06
Nodes (34): battery_saver_live(), BatterySaverStatus, ENERGY_SAVER_STATE, ESBATTTHRESHOLD, get_battery_saver(), on(), read(), set_battery_saver() (+26 more)

### Community 13 - "tauri.conf.json"
Cohesion: 0.07
Nodes (27): app, macOSPrivateApi, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl (+19 more)

### Community 14 - "power_mode.rs"
Cohesion: 0.09
Nodes (31): apply(), BALANCED, EFFICIENCY, get_effective(), get_power_mode(), guid_of(), guid_string(), LegacyMode (+23 more)

### Community 15 - "Dashboard.tsx"
Cohesion: 0.19
Nodes (16): Dashboard(), SECTIONS, withDetail, Header(), MetricStrip(), NetworkCard(), SensorDetails(), StaleDot() (+8 more)

### Community 16 - "processes.rs"
Cohesion: 0.08
Nodes (40): battery_live(), cycle_count_without_capacity_data(), DRAINING, driver_status(), DriverStatus, estimate(), falls_back_to_the_os_estimate_without_a_rate(), health_from_design_and_full_charge_capacity() (+32 more)

### Community 17 - "charge_limit.rs"
Cohesion: 0.10
Nodes (23): ChargeLimit, ChargeLimitStatus, clamp(), clamps_to_supported_range(), get_charge_limit(), MAX, MIN, Preferences (+15 more)

### Community 18 - "types.rs"
Cohesion: 0.16
Nodes (17): Battery, BatteryState, Charging, Discharging, Idle, CoreSample, DetailMetrics, FanSpeeds (+9 more)

### Community 19 - "Omniscience — Design Spec"
Cohesion: 0.10
Nodes (21): 10. Testing, 11. Out of scope (v1), 12. Known risks, 1. Purpose and success criteria, 2. Technology stack, 3. Window model, 5.1 State, 5.2 Collapsed pill (+13 more)

### Community 20 - "metrics.ts"
Cohesion: 0.14
Nodes (22): Pill(), Arc, ArcCircle(), Ring(), RingProps, ACCENTS, CRITICAL, levelColor() (+14 more)

### Community 21 - "ConfigView.tsx"
Cohesion: 0.17
Nodes (16): CpuBoostCard(), CpuBoostStatus, modes, supported, Limits, PowerLimitsCard(), PowerLimitsStatus, supported (+8 more)

### Community 22 - "compilerOptions"
Cohesion: 0.11
Nodes (17): compilerOptions, isolatedModules, jsx, lib, module, moduleResolution, noEmit, noFallthroughCasesInSwitch (+9 more)

### Community 23 - "driver_updates.rs"
Cohesion: 0.19
Nodes (16): accepts_only_catalog_update_guids(), get_system_update_status(), OfferedUpdate, one_background_job_retains_its_result_until_rechecked(), open_update_download(), scan(), scan_driver_updates(), ScanPhase (+8 more)

### Community 24 - "sections.ts"
Cohesion: 0.25
Nodes (16): SectionLayout(), finish(), SortableSection(), layout(), order(), CONFIG_SECTION_ORDER, DEFAULT_SECTION_ORDER, moveSection() (+8 more)

### Community 25 - "with"
Cohesion: 0.06
Nodes (35): GITHUB_TOKEN, TAURI_SIGNING_PRIVATE_KEY, TAURI_SIGNING_PRIVATE_KEY_PASSWORD, args, platform, jobs, release, include (+27 more)

### Community 26 - "useNativeControl"
Cohesion: 0.23
Nodes (10): DisplayCard(), DisplayStatus, supported, Args, sameArgs(), Undo, useNativeControl(), ChargeLimitBar() (+2 more)

### Community 27 - "MetricStrip.tsx"
Cohesion: 0.15
Nodes (27): lucide-react, barColor(), BatteryCard(), HealthRow(), summary(), cellStyle(), CpuHeatmap(), GpuPowerCard() (+19 more)

### Community 28 - "windows.rs"
Cohesion: 0.10
Nodes (12): Core, Adapter, adapter_perf(), adapters(), adapters_live(), at_thermal_limit(), classifies_adapter_kind_by_dedicated_memory(), INTEGRATED_BELOW (+4 more)

### Community 29 - "super"
Cohesion: 0.05
Nodes (35): powershell(), preserves_utf8_output_and_reports_failed_commands(), terminates_a_stalled_management_process(), converts_units_of_100_rpm(), EVERY, FanSource, ignores_bits_above_the_low_byte(), keeps_each_fan_independent() (+27 more)

### Community 30 - "ProcessIo"
Cohesion: 0.18
Nodes (4): accumulates_only_attributed_payload_bytes(), Counts, ProcessIo, thread_owner()

### Community 31 - "vitest"
Cohesion: 0.17
Nodes (9): @testing-library/react, vitest, noHealth, FanCard(), FanRow(), formatRpm(), SENSORS, sampleDetail (+1 more)

### Community 32 - "ProcessList.tsx"
Cohesion: 0.24
Nodes (13): ProcessList(), onKill(), Row(), SortToggle(), COLLAPSED_PROCESSES, CONFIRM_MS, killFailure(), processRate() (+5 more)

### Community 33 - "project.yml"
Cohesion: 0.08
Nodes (25): activation_command, activation_command_timeout, added_modes, agent_interface, default_modes, encoding, excluded_apis, excluded_tools (+17 more)

### Community 34 - "ConfigView.tsx"
Cohesion: 0.19
Nodes (13): ConfigView(), GpuModeCard(), GpuStatus, Mode, modes, supported, SectionToggleProvider(), base (+5 more)

### Community 35 - "check"
Cohesion: 0.08
Nodes (23): name, runs-on, steps, strategy, concurrency, cancel-in-progress, group, jobs (+15 more)

### Community 36 - "Manual test checklist"
Cohesion: 0.12
Nodes (15): CPU boost, Dashboard, Display configuration, GPU configuration, Install, Manual test checklist, Pill, Power limits (+7 more)

### Community 37 - "compilerOptions"
Cohesion: 0.17
Nodes (11): compilerOptions, allowJs, lib, module, moduleResolution, noEmit, skipLibCheck, strict (+3 more)

### Community 38 - "tray.rs"
Cohesion: 0.12
Nodes (12): AUTO_CHECK_UPDATES, CHECK_UPDATE_EVENT, CHECK_UPDATES, HIDE_OVER_FULLSCREEN, init(), LAUNCH_AT_STARTUP, on_menu(), OPEN_SYSTEM_UPDATES_EVENT (+4 more)

### Community 39 - "README.md"
Cohesion: 0.13
Nodes (12): README media, Acknowledgements, Configuration, Contributing, License, Performance, Platform support, Privacy and permissions (+4 more)

### Community 40 - "Telemetry"
Cohesion: 0.18
Nodes (3): Control, Telemetry, TICK_EVENT

### Community 41 - "Changelog"
Cohesion: 0.14
Nodes (14): [0.1.15] - 2026-10-30, [0.2.15] - 2026-11-02, [0.2.16] - 2026-11-02, [0.2.1] - 2026-10-31, [0.2.20] - 2026-11-02, [0.2.6] - 2026-10-31, Added, Added (+6 more)

### Community 42 - "platform.rs"
Cohesion: 0.13
Nodes (14): run(), adds_toolwindow(), clears_appwindow(), enforce_tool_window(), is_idempotent(), keeps_other_bits(), mouse_down(), prepare_overlay() (+6 more)

### Community 43 - "window.rs"
Cohesion: 0.19
Nodes (15): set_overlay_height(), centers_on_monitor_at_origin(), clamp_height(), clamps_requested_heights(), COLLAPSED_SIZE, EXPANDED_MAX_HEIGHT, handles_monitor_left_of_origin(), place_top_center() (+7 more)

### Community 44 - "File Structure"
Cohesion: 0.15
Nodes (12): feat/scaffold Implementation Plan, File Structure, Global Constraints, Review Focus, Task 0: Create the branch, Task 1: Frontend toolchain (Vite + React + TS + Tailwind + Vitest), Task 2: Lint, format, and line endings, Task 3: Tauri backend and top-center window (+4 more)

### Community 45 - "hover.rs"
Cohesion: 0.08
Nodes (26): collapsed_counts_only_the_pill(), expanded_counts_the_whole_window(), HOVER_EVENT, inside(), PILL_H, PILL_W, POLL, POLL_HIDDEN (+18 more)

### Community 47 - "Card.tsx"
Cohesion: 0.23
Nodes (9): @dnd-kit/sortable, react, SectionToggleContext, Toggle, useSectionToggle(), SectionDragContext, useSectionDrag(), CardSectionId (+1 more)

### Community 48 - "asus.rs"
Cohesion: 0.11
Nodes (8): CHARGE_LIMIT, CPU_FAN, GPU_ECO, GPU_FAN, GPU_MUX, PPT_FPPT, PPT_SPL, PPT_SPPT

### Community 49 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 50 - "RollingPeak"
Cohesion: 0.31
Nodes (6): floor_applies_when_quiet(), ignores_negative_and_nan(), MB, new_peak_reads_full(), old_peak_expires_after_window(), RollingPeak

### Community 51 - "dependencies"
Cohesion: 0.13
Nodes (15): dependencies, @dnd-kit/core@^6.3.1, @dnd-kit/sortable@^10.0.0, @dnd-kit/utilities@^3.2.2, lucide-react@^1.55.0, react@^19.3.0, react-dom@^19.3.0, @tauri-apps/api@^2.12.3 (+7 more)

### Community 52 - "System configuration"
Cohesion: 0.25
Nodes (7): CPU boost, Display, Driver and software updates, GPU mode, Power limits, Power mode, System configuration

### Community 53 - "commands.rs"
Cohesion: 0.25
Nodes (6): get_settings(), kill_process_group(), reset_position(), set_expanded(), set_settings(), set_memory_low()

### Community 54 - "fullscreen.rs"
Cohesion: 0.42
Nodes (8): covers(), detects_exact_and_oversized_fullscreen(), fullscreen_app_over(), ignores_windows_that_leave_any_edge_uncovered(), is_shell_class(), MON, recognizes_desktop_and_taskbar_classes(), Rect

### Community 55 - "DriverUpdatesCard.tsx"
Cohesion: 0.32
Nodes (6): DriverUpdatesCard(), OfferedUpdate, ScanStatus, checking, complete, idle

### Community 56 - "Results"
Cohesion: 0.29
Nodes (6): 0.1.0 collapsed idle breakdown, Baseline breakdown (default args), CPU fix (before tagging 0.1.0), Method, Performance, Results

### Community 57 - "Features"
Cohesion: 0.29
Nodes (7): Features, Lives in the tray, No extra software, System configuration, The dashboard, The pill, Updates

### Community 58 - "Thresholds"
Cohesion: 0.17
Nodes (13): classify(), classify_boundaries(), classify_respects_custom_thresholds(), CoreMetrics, Level, Critical, Normal, Warning (+5 more)

### Community 60 - "Releasing"
Cohesion: 0.40
Nodes (4): Cutting a release, One-time setup: updater signing key, Releasing, Unsigned builds

### Community 61 - ".prettierrc.json"
Cohesion: 0.40
Nodes (4): printWidth, semi, singleQuote, trailingComma

### Community 62 - "Installation"
Cohesion: 0.40
Nodes (5): Build from source, Installation, macOS (experimental), Uninstalling, Windows 10/11

### Community 63 - "package.json"
Cohesion: 0.17
Nodes (11): jsdom@^30.1.2, @tauri-apps/cli@^2.12.2, typescript@~6.0.3, license, name, private, type, version (+3 more)

### Community 64 - "Development"
Cohesion: 0.50
Nodes (4): Development, Further reading, Project layout, Tech stack

### Community 66 - "store.ts"
Cohesion: 0.22
Nodes (9): 4.1 Collector, 4.2 Snapshot shape, 4.3 Threshold logic, 4.4 Sources, 4.5 Process management, 4. Telemetry engine (Rust), TelemetryState, Snapshot (+1 more)

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

### Community 106 - "shell.rs"
Cohesion: 0.17
Nodes (3): OPEN_SETTINGS_EVENT, SETTINGS_EVENT, TrayChecks

### Community 108 - "eslint.config.js"
Cohesion: 0.15
Nodes (12): eslint@^10.12.0, @eslint/js@^10.0.1, eslint-plugin-react-hooks@^7.1.1, eslint-plugin-react-refresh@^0.5.7, globals@^17.13.0, typescript-eslint@^8.71.1, eslint, @eslint/js (+4 more)

### Community 111 - "KillOutcome"
Cohesion: 0.25
Nodes (8): KillOutcome, AccessDenied, Failed, Killed, NotFound, Protected, KillReport, KillResult

### Community 112 - "detail"
Cohesion: 0.33
Nodes (4): performance_limit_is_available_capacity_not_loss(), performance_loss_pct(), CoreSample, detail()

### Community 113 - "@tailwindcss/vite"
Cohesion: 0.29
Nodes (6): @tailwindcss/vite@^4.3.3, vite@^8.3.4, @vitejs/plugin-react@^6.1.2, @tailwindcss/vite, vite, @vitejs/plugin-react

### Community 114 - ".sample"
Cohesion: 0.38
Nodes (5): labels_acpi_zones(), zone_label(), collapsed_snapshot_has_no_detail(), detail_appears_on_second_expanded_tick_and_drops_on_collapse(), snapshot_live()

### Community 115 - "BatterySaverCard.tsx"
Cohesion: 0.47
Nodes (4): BatterySaverCard(), BatterySaverStatus, supported, thresholds

## Knowledge Gaps
- **29 isolated node(s):** `omniscience`, `Idle`, `Checking`, `Complete`, `Failed` (+24 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 554 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **41 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Omniscience — Design Spec` connect `Omniscience — Design Spec` to `store.ts`?**
  _High betweenness centrality (0.062) - this node is a cross-community bridge._
- **What connects `omniscience`, `Idle`, `Checking` to the rest of the system?**
  _29 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `sampler.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.07777777777777778 - nodes in this community are weakly interconnected._
- **Why does `Changelog` connect `Changelog` to `README.md`, `[0.1.1] - 2026-10-24`, `[0.1.4] - 2026-10-24`, `[0.1.8] - 2026-10-27`, `[0.1.9] - 2026-10-29`, `[0.2.10] - 2026-10-31`, `[0.1.0] - 2026-10-11`, `[0.1.10] - 2026-10-30`, `[0.1.11] - 2026-10-30`, `[0.1.12] - 2026-10-30`, `[0.1.13] - 2026-10-30`, `[0.1.14] - 2026-10-30`, `[0.1.16] - 2026-10-30`, `[0.1.17] - 2026-10-30`, `[0.1.18] - 2026-10-30`, `[0.1.19] - 2026-10-30`, `[0.1.2] - 2026-10-24`, `[0.1.3] - 2026-10-24`, `[0.1.5] - 2026-10-26`, `[0.1.6] - 2026-10-26`, `[0.1.7] - 2026-10-27`, `[0.2.0] - 2026-10-31`, `[0.2.11] - 2026-10-31`, `[0.2.12] - 2026-10-31`, `[0.2.13] - 2026-11-02`, `[0.2.14] - 2026-11-02`, `[0.2.17] - 2026-11-02`, `[0.2.18] - 2026-11-02`, `[0.2.19] - 2026-11-02`, `[0.2.21] - 2026-11-02`, `[0.2.22] - 2026-11-02`, `[0.2.2] - 2026-10-31`, `[0.2.3] - 2026-10-31`, `[0.2.4] - 2026-10-31`, `[0.2.5] - 2026-10-31`, `[0.2.7] - 2026-10-31`, `[0.2.8] - 2026-10-31`, `[0.2.9] - 2026-10-31`?**
  _High betweenness centrality (0.041) - this node is a cross-community bridge._
- **Should `positions.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.07948568088836938 - nodes in this community are weakly interconnected._
- **Why does `9. Repository workflow` connect `Omniscience — Design Spec` to `metrics.ts`?**
  _High betweenness centrality (0.039) - this node is a cross-community bridge._
- **Should `gpu_mode.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.0975609756097561 - nodes in this community are weakly interconnected._