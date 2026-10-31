# System configuration

Open the dashboard and choose **Config**. **Info** contains the live monitoring cards;
switching tabs keeps the process search, sorting and card state for that dashboard session.
The gear icon still opens application settings. Config cards can be collapsed or expanded
with their headings; their collapsed state is remembered across dashboard reopening and app
restarts. Collapsed controls are removed from keyboard navigation.

## Driver and software updates

Choose **Check now** in **Driver & software updates**, or right-click the tray icon and choose
**Check for system updates**. Both actions use the same background job. The tray action also
opens Config and expands the update card.

Omniscience queries the computer's configured Windows Update service, including optional
drivers and Microsoft software. The check runs only when requested and may take up to three
minutes. Closing or collapsing the card, switching tabs, and hiding the overlay do not cancel
it. Reopening the card shows the current check or its finished results. Starting another check
while one is running joins the existing job. Results are kept until the next check or app exit;
quitting the application stops the job.

Only updates offered to this computer appear. **Download** opens the matching
[Microsoft Update Catalog](https://www.catalog.update.microsoft.com/) entry in the
default browser, where you can review and download the package. Omniscience does not
install drivers or updates automatically.

The up-to-date message applies to Windows Update's catalog. It does not check every
third-party program or compare against every manufacturer's website. Failed, partial,
offline and timed-out checks show an error and can be retried; they never count as an
up-to-date result. This check is currently available on Windows only.

## GPU mode

Requires Windows and an ASUS ROG laptop whose installed ASUS System Control Interface
exposes the GPU controls. Unsupported controls are disabled with an explanation.

| Mode | Behavior                                                                    |
| ---- | --------------------------------------------------------------------------- |
| Auto | iGPU only on battery; hybrid graphics when plugged in.                      |
| Med  | Both GPUs available through hybrid graphics.                                |
| Min  | iGPU only, with the dGPU disabled. Close apps using the dGPU first.         |
| Max  | Routes the display through the dGPU on laptops with a supported MUX switch. |

MUX changes require a Windows restart. Omniscience reports this and pauses further GPU
changes until a restart; it does not restart Windows for you. Selecting Min or Auto from
Max first switches back to hybrid display routing, then completes the requested mode
after restart.

The selected mode is saved. Auto responds to power-source changes while Omniscience runs,
including when the overlay is closed or hidden. It is off until you choose it. Firmware
refusals and permission errors stay visible; Omniscience does not terminate GPU apps.
If Windows denies access to the ASUS device, administrator access may be needed.
Other manufacturers and macOS currently have no GPU-switching backend.

The endpoint identifiers follow the public ASUS WMI protocol documented in
[Linux's ASUS WMI header](https://github.com/torvalds/linux/blob/master/include/linux/platform_data/x86/asus-wmi.h).

## Display

Requires Windows. Config > Display controls the built-in panel (or the primary display if
the panel cannot be identified) and shows its name. The refresh row has Auto plus one button
per rate available at the current resolution. Auto uses the lowest rate of at least 60 Hz on
battery and the highest rate when plugged in, and responds to power-source changes while
Omniscience runs; it is off until you choose it, and picking a rate turns it off. The
resolution row lists the resolutions Windows offers at the current color depth and
orientation. Changing resolution keeps the refresh rate when available, otherwise uses the
highest one (or Auto's choice). Windows remembers the resolution itself; Omniscience only
saves whether Auto is on. Rejected modes are reported and the actual state is re-read.

The update query uses Microsoft's
[Windows Update Agent search API](https://learn.microsoft.com/en-us/windows/win32/api/wuapi/nf-wuapi-iupdatesearcher-search).
