<#
.SYNOPSIS
  Samples memory and CPU of a running Omniscience instance, including its WebView2 child processes.
.DESCRIPTION
  Memory = sum of "Working Set - Private" (what Task Manager's Memory column shows) across
  omniscience.exe and all its descendants. CPU = sum of % Processor Time, normalized to
  total logical processors (0-100%, like Task Manager).
.EXAMPLE
  ./scripts/measure.ps1 -Seconds 30
#>
param(
  [int]$Seconds = 30,
  [string]$ProcessName = 'omniscience'
)

$root = Get-Process -Name $ProcessName -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $root) { Write-Error "$ProcessName is not running"; exit 1 }

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
$cpuSamples = @()

for ($i = 0; $i -lt $Seconds; $i++) {
  $tree = Get-Tree $root.Id
  $perf = Get-CimInstance Win32_PerfFormattedData_PerfProc_Process |
    Where-Object { $tree.Contains([int]$_.IDProcess) }
  $memSamples += ($perf | Measure-Object -Property WorkingSetPrivate -Sum).Sum / 1MB
  $cpuSamples += ($perf | Measure-Object -Property PercentProcessorTime -Sum).Sum / $cores
  Start-Sleep -Seconds 1
}

$tree = Get-Tree $root.Id
[pscustomobject]@{
  Processes   = $tree.Count
  MemAvgMB    = [math]::Round(($memSamples | Measure-Object -Average).Average, 1)
  MemPeakMB   = [math]::Round(($memSamples | Measure-Object -Maximum).Maximum, 1)
  CpuAvgPct   = [math]::Round(($cpuSamples | Measure-Object -Average).Average, 2)
  CpuPeakPct  = [math]::Round(($cpuSamples | Measure-Object -Maximum).Maximum, 2)
  Samples     = $Seconds
} | Format-List
