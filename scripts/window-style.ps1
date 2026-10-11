<#
.SYNOPSIS
  Prints the position, size, and taskbar/Alt+Tab-relevant extended styles of Omniscience's visible windows.
.DESCRIPTION
  Expected for the overlay: TOOLWINDOW=True, APPWINDOW=False, TOPMOST=True. Coordinates are as seen by
  this (DPI-unaware) PowerShell process, so they are in logical pixels on scaled displays.
.EXAMPLE
  ./scripts/window-style.ps1
#>
param([string]$ProcessName = 'omniscience')

Add-Type @"
using System; using System.Runtime.InteropServices; using System.Text;
public static class OmniWin {
  public struct RECT { public int L, T, R, B; }
  public delegate bool EnumProc(IntPtr h, IntPtr p);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc cb, IntPtr p);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern IntPtr GetWindowLongPtr(IntPtr h, int i);
  [DllImport("user32.dll")] public static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
}
"@

$proc = Get-Process -Name $ProcessName -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $proc) { Write-Error "$ProcessName is not running"; exit 1 }

$handles = [System.Collections.Generic.List[IntPtr]]::new()
[void][OmniWin]::EnumWindows({
    param($h, $x)
    $id = 0
    [void][OmniWin]::GetWindowThreadProcessId($h, [ref]$id)
    if ($id -eq $proc.Id -and [OmniWin]::IsWindowVisible($h)) { $handles.Add($h) }
    $true
  }, [IntPtr]::Zero)

foreach ($h in $handles) {
  $r = New-Object OmniWin+RECT
  [void][OmniWin]::GetWindowRect($h, [ref]$r)
  $title = New-Object Text.StringBuilder 256
  [void][OmniWin]::GetWindowText($h, $title, 256)
  $ex = [OmniWin]::GetWindowLongPtr($h, -20).ToInt64()
  [pscustomobject]@{
    Title      = $title.ToString()
    Position   = "$($r.L),$($r.T)"
    Size       = "$($r.R - $r.L)x$($r.B - $r.T)"
    ExStyle    = '0x{0:X}' -f $ex
    TOOLWINDOW = ($ex -band 0x80) -ne 0
    APPWINDOW  = ($ex -band 0x40000) -ne 0
    TOPMOST    = ($ex -band 0x8) -ne 0
  }
}
