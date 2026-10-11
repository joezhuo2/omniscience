# Manual test checklist

Run before handing off each branch. Record results in the branch handoff notes.

Helpers:

- `powershell -ExecutionPolicy Bypass -File scripts/window-style.ps1` prints the overlay's position, size,
  and extended styles. Expected: `TOOLWINDOW=True`, `APPWINDOW=False`, `TOPMOST=True`.
- `powershell -ExecutionPolicy Bypass -File scripts/measure.ps1 -Seconds 30` measures memory and CPU
  (see `docs/perf.md`).

## Window (feat/scaffold)

- [ ] Pill appears at top-center of the primary monitor, about 8 px below the top of the work area.
- [ ] No jump on launch (window is not visible at another position first).
- [ ] Corners of the pill show the desktop. No white or black boxes.
- [ ] No title bar or window frame.
- [ ] Stays above other windows after clicking another app.
- [ ] No taskbar button. Not listed in Alt+Tab or Win+Tab.
- [ ] Tray icon present. Left-click hides/shows the pill. Right-click menu: Show / Hide, Quit. Quit exits fully (no omniscience.exe left in Task Manager).
- [ ] After hiding and showing from the tray, `scripts/window-style.ps1` still reports `TOOLWINDOW=True`, `APPWINDOW=False`.
- [ ] After restarting Explorer (Task Manager > Windows Explorer > Restart), the tray icon comes back.
- [ ] Correct position at 100% and 150% display scaling (Settings > Display > Scale), restarting the app after each change.
- [ ] With a second monitor placed left of the primary, the pill still appears on the primary monitor.
