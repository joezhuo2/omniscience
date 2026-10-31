# System configuration

Open the dashboard and choose **Config**. **Info** contains the live monitoring cards;
switching tabs keeps the process search, sorting and card state for that dashboard session.
The gear icon still opens application settings.

## Driver and software updates

Choose **Check now** in **Driver & software updates**. Omniscience queries the computer's
configured Windows Update service, including optional drivers and Microsoft software.
The check runs only when requested and may take up to three minutes.

Only updates offered to this computer appear. **Download** opens the matching
[Microsoft Update Catalog](https://www.catalog.update.microsoft.com/) entry in the
default browser, where you can review and download the package. Omniscience does not
install drivers or updates automatically.

The up-to-date message applies to Windows Update's catalog. It does not check every
third-party program or compare against every manufacturer's website. Failed, partial,
offline and timed-out checks show an error and can be retried; they never count as an
up-to-date result. This check is currently available on Windows only.

The query uses Microsoft's
[Windows Update Agent search API](https://learn.microsoft.com/en-us/windows/win32/api/wuapi/nf-wuapi-iupdatesearcher-search).
