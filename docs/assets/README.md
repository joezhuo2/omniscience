# README media

These images show the current Omniscience React frontend rendered in Chromium on Windows.
Telemetry is illustrative, based on `src/test/fixtures.ts`, rather than a live hardware capture.
The demo uses a 16-thread sample CPU, an active discrete GPU and private example IP addresses.
Tauri IPC is mocked for capture; native tray, window positioning and hardware sampling are not
part of the demo.

- `omniscience-banner.png`: wordmark with the repository's `app-icon.svg`.
- `demo.gif`: pill expansion, GPU card expansion/minimization, RAM sorting, Settings and collapse.
- `pill.png`: collapsed overlay.
- `dashboard.png`: expanded dashboard at the top of its scroll area.
- `dashboard-details.png`: expanded dashboard scrolled down to expose Network and Top processes.
- `settings.png`: settings view with default values except auto-check for updates, disabled for capture.

The screenshots use the app's existing layout and styles on a dark background. The GIF is
600 pixels wide and loops automatically. All media is stored locally so the README does not
depend on an external image host.
