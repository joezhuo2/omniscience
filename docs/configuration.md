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

## Power mode

Requires Windows 11. Sets the Windows power mode overlay, shown in Settings as Best power
efficiency, Balanced and Best performance.

| Mode        | Behavior                                            |
| ----------- | --------------------------------------------------- |
| Auto        | Efficiency on battery; Performance when plugged in. |
| Efficiency  | Best power efficiency.                              |
| Balanced    | Windows default balance of power and performance.   |
| Performance | Best performance.                                   |

Windows keeps a separate mode for battery and plugged in, so a manual choice applies to
the current power source. The selection is saved and reapplied when Omniscience starts;
Auto reapplies whenever the power source changes, even with the overlay closed. The card
always shows the mode Windows reports, so changes made elsewhere are visible.

The endpoint identifiers follow the public ASUS WMI protocol documented in
[Linux's ASUS WMI header](https://github.com/torvalds/linux/blob/master/include/linux/platform_data/x86/asus-wmi.h).

## CPU boost

Off, On and Aggressive set the processor performance boost mode (PERFBOOSTMODE) of the
active Windows power plan for both plugged-in and battery use. Omniscience stores nothing:
it always shows the active plan's value, so switching plans changes what is shown. If the
two values differ, the plugged-in value is shown with a notice. Other modes (such as
Efficient enabled) are named with no button selected. Windows only.

The update query uses Microsoft's
[Windows Update Agent search API](https://learn.microsoft.com/en-us/windows/win32/api/wuapi/nf-wuapi-iupdatesearcher-search).
