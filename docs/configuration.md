# System configuration

Config is off by default. Turn on **System config** in Settings (gear icon) to show the
**Info**/**Config** switch and the tray's **Check for system updates**. While it is off, the
switch and the Battery card's charge limit are hidden, Config commands are rejected, and GPU
mode, power limits, power mode, display and charge limit automation do nothing. Saved Config choices stay on disk and resume when it is turned
back on; nothing is reset on disable, though ASUS firmware drops the charge limit at the next reboot
until System config is back on.

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

| Mode | Behavior                                                            |
| ---- | ------------------------------------------------------------------- |
| Auto | iGPU only on battery; hybrid graphics when plugged in.              |
| Med  | Both GPUs available through hybrid graphics.                        |
| Min  | iGPU only, with the dGPU disabled. Close apps using the dGPU first. |

Omniscience never routes the display through the dGPU. If firmware is already in that state,
the card says so, and any mode switches back to hybrid display routing. That MUX change
requires a Windows restart: Omniscience reports this and pauses further GPU changes until a
restart; it does not restart Windows for you. Min or Auto completes after the restart.

The selected mode is saved. Auto responds to power-source changes while Omniscience runs,
including when the overlay is closed or hidden. It is off until you choose it. Firmware
refusals and permission errors stay visible; Omniscience does not terminate GPU apps.
If Windows denies access to the ASUS device, administrator access may be needed.
Other manufacturers and macOS currently have no GPU-switching backend.

## Power mode

Requires Windows 11. Sets the Windows power mode overlay, shown in Settings as Best power
efficiency, Balanced and Best performance.

| Mode        | Behavior                                          |
| ----------- | ------------------------------------------------- |
| Efficiency  | Best power efficiency.                            |
| Balanced    | Windows default balance of power and performance. |
| Performance | Best performance.                                 |

The card has Plugged in and Battery tabs (the tab for the current power source is marked
and selected by default). Each tab saves its own mode. Choosing a mode on the current
source's tab applies it immediately and verifies it; choosing on the other tab only saves
it. Saved profiles are applied when Omniscience starts and whenever the power source
changes, even with the overlay closed. Until a mode is chosen nothing is written, and the
card shows the mode Windows reports. A previously saved Auto becomes Performance plugged in
and Efficiency on battery.

The endpoint identifiers follow the public ASUS WMI protocol documented in
[Linux's ASUS WMI header](https://github.com/torvalds/linux/blob/master/include/linux/platform_data/x86/asus-wmi.h).

## CPU boost

Off, On and Aggressive set the processor performance boost mode (PERFBOOSTMODE) of the
active Windows power plan. The Plugged in tab reads and writes only the plan's plugged-in
value and the Battery tab only its battery value; the tab for the current power source is
selected by default. Omniscience stores nothing: it always shows the active plan's values,
so switching plans changes what is shown. Other modes (such as Efficient enabled) are named
with no button selected. Windows only.

## Power limits

On supported ASUS ROG laptops, sets the CPU sustained (SPL) and boost (SPPT) wattage from
5 to 80 W, with separate Plugged in and Battery tabs. Each tab's limits are saved. Releasing
a slider on the current power source's tab writes to firmware immediately; on the other tab
it only saves. The matching profile is applied at launch and whenever the power source
changes. Reset clears the selected tab's profile. A previously saved single pair is used for
both sources.

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
