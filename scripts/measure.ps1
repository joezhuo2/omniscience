<#
.SYNOPSIS
  Samples memory and CPU of a running Omniscience instance, including its WebView2 child processes.
.DESCRIPTION
  Memory = sum of "Working Set - Private" (what Task Manager's Memory column shows) across
  omniscience.exe and all its descendants, sampled once per second. CPU = total processor time
  consumed by the tree over the whole window, divided by elapsed time x logical processors
  (0-100%, like Task Manager). This integrates over the window instead of sampling the
  formatted counter, which truncates each process to whole percents of one core.
.EXAMPLE
  ./scripts/measure.ps1 -Seconds 30
#>
param(
  [int]$Seconds = 30,
  [string]$ProcessName = 'omniscience',
  [int]$ProcessId = 0
)

$root = if ($ProcessId) { Get-Process -Id $ProcessId -ErrorAction SilentlyContinue }
        else { Get-Process -Name $ProcessName -ErrorAction SilentlyContinue | Select-Object -First 1 }
if (-not $root) { Write-Error "process not running"; exit 1 }

function Get-Tree([int]$rootId) {
  $all = Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId
  $ids = [System.Collections.Generic.HashSet[int]]::new()
  [void]$ids.Add($rootId)
  do {
    $added = $false
    foreach ($p in $all) {
      if ($ids.Contains([int]$p.ParentProcessId) -and $ids.Add([int]$p.ProcessId)) { $added = $true }
    }
  } while ($added)
  return $ids
}

$cores = [Environment]::ProcessorCount
$memSamples = @()
# CPU seconds per process at the first time we saw it, and the latest reading.
$cpuStart = @{}
$cpuLast = @{}
$clock = [Diagnostics.Stopwatch]::StartNew()

for ($i = 0; $i -lt $Seconds; $i++) {
  $tree = Get-Tree $root.Id
  $perf = Get-CimInstance Win32_PerfFormattedData_PerfProc_Process |
    Where-Object { $tree.Contains([int]$_.IDProcess) }
  $memSamples += ($perf | Measure-Object -Property WorkingSetPrivate -Sum).Sum / 1MB
  foreach ($id in $tree) {
    $p = Get-Process -Id $id -ErrorAction SilentlyContinue
    if (-not $p) { continue }
    $t = $p.TotalProcessorTime.TotalSeconds
    if (-not $cpuStart.ContainsKey($id)) { $cpuStart[$id] = $t }
    $cpuLast[$id] = $t
  }
  Start-Sleep -Seconds 1
}

$elapsed = $clock.Elapsed.TotalSeconds
$cpuSeconds = 0.0
foreach ($id in $cpuLast.Keys) { $cpuSeconds += $cpuLast[$id] - $cpuStart[$id] }

$tree = Get-Tree $root.Id
[pscustomobject]@{
  Processes  = $tree.Count
  MemAvgMB   = [math]::Round(($memSamples | Measure-Object -Average).Average, 1)
  MemPeakMB  = [math]::Round(($memSamples | Measure-Object -Maximum).Maximum, 1)
  CpuAvgPct  = [math]::Round($cpuSeconds / ($elapsed * $cores) * 100, 3)
  Seconds    = [math]::Round($elapsed, 1)
} | Format-List
