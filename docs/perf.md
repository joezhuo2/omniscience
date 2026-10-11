# Performance

Budget (spec §7): collapsed idle at most **50 MB** total private working set across the process
tree, and below **0.1% CPU**.

## Method

1. Build release: `npm run tauri build -- --no-bundle`.
2. Start `src-tauri/target/release/omniscience.exe` and leave it alone for 10 s.
3. Run `powershell -ExecutionPolicy Bypass -File scripts/measure.ps1 -Seconds 30`.
4. Record the averages below. CPU is integrated from processor-time deltas over the window.
   Rows before this fix used the formatted counter, which under-reports small loads. Note the state (collapsed, expanded, or hidden).

Reference machine: Ryzen AI 9 HX 370, Radeon 890M + RTX 5060 Laptop, 32 GB RAM, Windows 11,
1440 x 900 logical desktop.

## Results

| Branch        | State                        | Processes | Mem avg (MB) | Mem peak (MB) | CPU avg (%) | Notes                                                                                                                                                                                              |
| ------------- | ---------------------------- | --------- | ------------ | ------------- | ----------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| feat/scaffold | static pill, no telemetry    | 7         | 96.1         | 96.6          | 0.007       | Bare Tauri + WebView2 baseline, Tauri default browser args (re-measured with the corrected CPU method)                                                                                             |
| feat/scaffold | static pill, no telemetry    | 7         | 59.2         | 60.0          | 0.02        | Experiment only (not shipped): `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` adds `--disable-gpu`                                                                                                        |
| main (0.1.0)  | collapsed idle, telemetry on | 7         | 40.6         | 67.1          | 0.815       | Release build, shipped `--disable-gpu`, 1 s polling. Second 30 s window after a 70 s warm-up (first window: 44.7 / 73.1 / 0.797). **Memory average within budget but peaks over it; CPU 8x over.** |

### Baseline breakdown (default args)

| Process                | Private WS (MB) |
| ---------------------- | --------------- |
| omniscience.exe (Rust) | 4.3             |
| WebView2 browser       | 23.3            |
| WebView2 GPU process   | 35.5            |
| WebView2 renderer      | 12.9            |
| WebView2 utility x2    | 8.7             |
| WebView2 crashpad      | 1.8             |

The GPU process is the largest single cost. `--disable-gpu` (software compositing) removes most of
it. Whether to ship that flag depends on how smooth the spring morph stays without GPU compositing,
which gets checked in `feat/morph`.

### 0.1.0 collapsed idle breakdown

Per process over 20 s, private bytes (`PrivateMemorySize64`, a little above private working set):

| Process                | CPU (%) | Private (MB) |
| ---------------------- | ------- | ------------ |
| omniscience.exe (Rust) | 0.18    | 13.7         |
| WebView2 browser       | 0.25    | 32.2         |
| WebView2 GPU process   | 0.51    | 19.0         |
| WebView2 renderer      | 0.79    | 48.7         |
| WebView2 utility x2    | 0.00    | 18.2         |
| WebView2 crashpad      | 0.00    | 2.9          |

Most CPU is in the renderer and GPU process, i.e. painting. Likely causes, not yet confirmed: each
ring's 400 ms `stroke-dashoffset` transition repaints for 40% of every 1 s tick with software
compositing, and the collapsed dashboard may still re-render behind `opacity: 0`. Expanded and
hidden states were not measured in this run because the island did not expand from a synthetic
cursor move.
