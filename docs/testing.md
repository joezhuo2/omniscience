# Manual test checklist

Run before every release, and before handing off a branch that touches the area. Test the release
build (`npm run tauri build`), not `tauri dev`. Record results in the release log at the bottom.

Helpers:

- `powershell -ExecutionPolicy Bypass -File scripts/window-style.ps1` prints the overlay's position, size,
  and extended styles. Expected: `TOOLWINDOW=True`, `APPWINDOW=False`, `TOPMOST=True`.
- `powershell -ExecutionPolicy Bypass -File scripts/measure.ps1 -Seconds 30` measures memory and CPU
  (see `docs/perf.md`).

## Window

- [ ] Pill appears at top-center of the primary monitor, about 8 px below the top of the work area.
- [ ] No jump on launch (window is not visible at another position first).
- [ ] Corners of the pill show the desktop. No white or black boxes.
- [ ] No title bar or window frame.
- [ ] Stays above other windows after clicking another app.
- [ ] No taskbar button. Not listed in Alt+Tab or Win+Tab.
- [ ] `scripts/window-style.ps1` reports `TOOLWINDOW=True`, `APPWINDOW=False`, `TOPMOST=True`.
- [ ] Clicks on the transparent area beside the pill reach the window underneath.
- [ ] Correct position at 100% and 150% display scaling (Settings > Display > Scale), restarting the app after each change.
- [ ] With a second monitor placed left of the primary, the pill still appears on the primary monitor.

## Pill

- [ ] Rings: CPU, RAM, iGPU, optional dGPU, Disk, Network (download outside, upload inside). No text. With an absent or sleeping dGPU, five rings stay centered in the collapsed pill; waking it restores six. The expanded dashboard always keeps the dGPU indicator. A sampled 0% dGPU stays visible.
- [ ] Rings move with load (start a heavy task and watch CPU climb).
- [ ] A ring turns amber above the warning threshold and pulses red above the critical one (lower the thresholds in Settings to force it).
- [ ] A ring with no sensor is dashed, not stuck at 0%.

## Dashboard

- [ ] Moving the cursor onto the pill opens the dashboard at once with the clip-path morph. No flicker or white flash.
- [ ] Moving the cursor away collapses it at once; coming back mid-collapse reverses it.
- [ ] Esc closes it.
- [ ] Header shows the app name, stale indicator and Settings only. Package watts appear once in GPU & Power, CPU temperature under the CPU ring; missing readings show `--` or `—`.
- [ ] Dashboard is 480 logical pixels wide inside a 500-pixel window. All cards have 12-pixel internal padding on every side; GPU rows, battery bars, core cells, adapter rows and process controls remain inside their borders. Long header controls wrap without overlap. Expand all cards and Show more: cards keep their full height and the dashboard scrolls to the last control. Check at 100% and 150% scaling.
- [ ] Click each card heading or focus it and press Space/Enter to minimize and expand GPU & Power, Battery, CPU cores, Network and Top processes independently. Minimized cards keep their heading and key summary; rings stay visible and the overlay shrinks to fit.
- [ ] Click each card heading with a pointer: details fold and fade over 180 ms, the chevron rotates, and surrounding cards/overlay follow the changing height without a gap or jump. Toggle again mid-transition: it reverses smoothly. Collapsed controls cannot receive focus. Reduced motion and Space/Enter toggles change instantly. Reopening with saved minimized cards does not animate on mount.
- [ ] Minimize several cards, close/reopen the dashboard, then restart the app: those cards stay minimized. Expand them to undo. Process search text and sort selection survive minimizing and expanding the card within the same dashboard session.
- [ ] Drag each card from its background, title, body and controls into a new position, including minimized cards and across the scroll area. Titles still minimize/expand, and search, sorting and End buttons remain usable. Reopen the dashboard and restart the app: the chosen order is retained.
- [ ] Focus a card and press Space/Enter, move with Up/Down, then drop with Space/Enter. Escape cancels the drag without changing the saved order or closing the overlay; a later Escape closes it normally. Focus returns to the card. Space/Enter within child inputs and buttons keep their normal behavior.
- [ ] During mouse dragging, neighboring cards slide into place over 180 ms while the grabbed card follows the pointer directly. Repeat with reduced motion enabled: cards still reorder with no sliding transition. Keyboard moves are instant.
- [ ] Reorder with no battery present, then check a laptop/battery-present fixture: the battery keeps its saved relative position. Invalid order preferences fall back to all known sections without duplicates.
- [ ] Metric strip: CPU % with GHz on a second line, RAM like `15/31 GB`, download/upload on two lines.
- [ ] GPU & Power card lists each GPU with usage, VRAM and temperature, and the watt breakdown.
- [ ] A dGPU under load shows its fan speed in rpm (if its driver reports one); a GPU at its thermal limit shows a red Throttling chip.
- [ ] On a laptop, the Battery card shows the charge and, on battery, the drain in watts and time left (close to the Windows battery flyout); charging shows the charge rate and time to full; plugged in at a charge limit shows "Plugged in, not charging". On a desktop the card is absent.
- [ ] With Show more open, the dashboard scrolls inside the window and keeps a black inset above the rounded bottom edge at every scroll position. Scroll to the end and verify the last row and Show less button are fully reachable.
- [ ] With a touchpad, scroll past both ends of the dashboard, Settings and expanded process list: content stays within the overlay without bounce; intermediate rows remain reachable.
- [ ] CPU heatmap has one cell per logical core. Hovering a cell shows usage, clock, watts and temperature.
- [ ] Compare CPU cores against `\Processor Information(_Total)\% Performance Limit`: raw 100% shows no badge; raw 75% means Throttled −25%. Missing/zero raw readings hide the badge. Check AC, battery and sustained load; the tooltip explains this is a Windows power/platform estimate.
- [ ] Network card shows the first adapter, IPv4 and ping on one line, with no repeated download/upload rates. Extra adapters stay listed; ping appears only once (`--` with the network unplugged).
- [ ] Skeletons show briefly on first open, then real data.
- [ ] Top processes are grouped by app. The CPU/RAM/GPU/Disk/Network toggle re-sorts. 3 rows by default; "Show more" expands a scrollable list.
- [ ] On Windows with permission to trace, generate disk reads/writes and TCP/UDP traffic: Disk and Network tabs show bytes/second and put the busiest app first. Search and Show more retain the selected sort. Without tracing permission, rates show `—` with an explanation, never fabricated zeroes.
- [ ] Collapse or hide the overlay and verify its private `Omniscience-ProcessIo-*` ETW session stops (`logman query -ets`); reopening starts a fresh session without stale traffic.
- [ ] On the GPU tab, a game or video player leads the list with GPU usage close to Task Manager's GPU column and its GPU memory.
- [ ] End? then confirm on a harmless app (e.g. Notepad) closes it.
- [ ] End on a critical process (e.g. `csrss`, `System`) or on Omniscience itself is refused: the row comes back with a toast.
- [ ] Dragging the pill (row of rings) moves the overlay; the dashboard header no longer drags. After restarting the app, it reopens at the new position.
- [ ] Dropping the pill half off a screen edge pulls it fully back on screen once the mouse is released.
- [ ] Dropping it within about 24 px of an edge, or of the top-center, snaps it exactly there.
- [ ] With the pill near the bottom of a monitor, hovering lifts the window so the whole dashboard is visible; it drops back on collapse.
- [ ] On a second monitor, unplugging and replugging another monitor (a new layout) still brings the pill back to its spot on that monitor.

## Settings

- [ ] Opens from the gear icon and from tray > Settings…; dashboard and settings fade into each other.
- [ ] Threshold step arrows, update-interval picker and switches all change the value immediately.
- [ ] Changing the ping target updates the Network card within a few seconds; clearing it hides the ping readout.
- [ ] Show public IP adds the public address to the Network card within a few seconds; off by default.
- [ ] Changes survive a restart (`%APPDATA%\io.github.joezhuo2.omniscience\settings.json` is updated).
- [ ] Hotkey recorder: record a new combination; the new one toggles the overlay and the old one no longer does.
- [ ] Reset position moves the overlay back to the top-center of the primary monitor.
- [ ] Deleting `settings.json`, or putting out-of-range values in it, still starts the app with defaults or clamped values.

## Tray and app shell

- [ ] Tray icon present. Left-click hides/shows the pill.
- [ ] Right-click menu: Show / Hide, Launch at startup, Auto-check for updates, Hide over fullscreen apps, Check for updates now, Settings…, Quit. Checkboxes match Settings.
- [ ] `Ctrl+Shift+O` hides and shows the overlay.
- [ ] After hiding and showing, `scripts/window-style.ps1` still reports `TOOLWINDOW=True`, `APPWINDOW=False`.
- [ ] Launch at startup: toggling it adds/removes Omniscience in Settings > Apps > Startup, and the app starts after sign-in.
- [ ] A fullscreen video or game on the overlay's monitor hides it; leaving fullscreen brings it back. With the toggle off, it stays.
- [ ] Launching a second copy reveals the running one; only one `omniscience.exe` stays running.
- [ ] After restarting Explorer (Task Manager > Windows Explorer > Restart), the tray icon comes back.
- [ ] Quit exits fully (no `omniscience.exe` or its WebView2 children left in Task Manager).

## Updates

- [ ] Config > Driver & software updates: start a check, close the dashboard, and reopen Config. It still shows Checking until the background job succeeds or reports an error; finished results remain available. No second scan starts while one is running.
- [ ] Tray > Check for system updates starts that same job, opens Config and expands the update card even if it was saved collapsed. Offered updates open their matching Microsoft Update Catalog entry. This is separate from the signed Omniscience app updater.
- [ ] Collapse/expand Config cards using their headings, reopen the dashboard and restart the app: disclosure state persists and collapsed controls cannot receive keyboard focus.

- [ ] Tray > Check for updates now with no newer release reports that you are up to date.
- [ ] With a newer published release, the island opens with "vX available. Update? Yes / No" and stays until answered.
- [ ] Yes downloads, installs and restarts into the new version. No dismisses it.
- [ ] With Auto-check off, no request goes to GitHub at launch.

## Install

- [ ] NSIS installer on a clean user: SmartScreen "More info > Run anyway" works, app starts, tray icon appears.
- [ ] MSI installer installs and starts.
- [ ] Uninstall from Settings > Apps removes the app; `settings.json` is kept.

## Release log

Record each pre-release run here: date, version or commit, machine, and anything that failed or was not
checked.

| Date       | Build             | Machine                     | Result                                                                                                                                                                                                                                                                                                                    |
| ---------- | ----------------- | --------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 2026-10-11 | 0.1.0 (`816bc11`) | Reference machine (perf.md) | Partial, automated only. Pass: window styles (`TOOLWINDOW`, no `APPWINDOW`, `TOPMOST`), top-center position, six rings render, second launch keeps one process. Not run: everything needing real mouse or keyboard input (a synthetic cursor move did not expand the island), installers, updates. Perf: CPU over budget. |

## v0.1.7–v0.1.8 regression checks

- Expand before detail telemetry arrives: Battery shows the same skeleton as other cards, then displays data on a laptop or disappears on a desktop.
- CPU/iGPU/dGPU inner rings update while collapsed; 52.5°C is half a circle, 105°C or higher is full.
- Missing temperature hides only the inner ring and shows `—` in the metric strip.
- Expanded CPU has usage, clock, temperature; GPUs have usage and temperature. Hover CPU temperature to see the ACPI proxy explanation.
- GPU & Power retains memory, fan, throttling and power but no temperature text.
- Confirm sleeping dGPU is not woken and check collapsed CPU/RAM usage after enabling temperature sampling.
