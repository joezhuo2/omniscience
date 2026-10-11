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

### CPU fix (before tagging 0.1.0)

Two causes, each confirmed by removing it in an otherwise identical build and re-measuring. CPU here
is from `TotalProcessorTime` deltas over 20 s windows with nothing else polling (`measure.ps1`'s
once-a-second WMI queries load the machine enough to move the rings, so it inflates CPU now that
ring cost depends on load changes; use it for memory).

1. **Ring transitions.** Each of the six arcs eased `stroke-dashoffset` over 400 ms on every tick.
   With software compositing every frame repaints the overlay, so the renderer and GPU process were
   busy for 40% of each second. Removing the transition took the renderer from ~9% to ~0.35% of one
   core and the GPU process from ~5% to ~0.3%.
2. **Hover poll.** Four Tauri window getters every 80 ms, each a round trip through the event loop,
   woke the main thread ~50 times a second. Calling Win32 directly took the Rust process from ~2.7%
   to ~0.8% of one core (main thread 1.4% to 0.16%).

Ring animation options tried (collapsed, all cores, hover fix in place):

| Ring animation                                     | Total CPU (%)  |
| -------------------------------------------------- | -------------- |
| None                                               | 0.075-0.09     |
| 400 ms ease, changes under 3 points snap           | 0.13-0.66      |
| 400 ms `steps(6)`, changes under 3 points snap     | 0.17-0.28      |
| **240 ms `steps(4)`, changes under 3 points snap** | **0.065-0.13** |

Shipped: the last row, plus arcs rounded to whole percent. Final full release build (LTO), three
20 s windows while the system was at 12-16% load: **0.15-0.25% total**, of which Rust ~0.04%,
renderer 0.07-0.12%, GPU process 0.03-0.07%. Memory from `measure.ps1`: 47.4 MB average, 66.7 MB
peak. The original build measured 0.57-0.74% (up to 2.2%) the same way. Ring cost scales with how
often load swings by 3+ points, so a quiet machine sits at about the 0.1% budget and a busy one
above it.
