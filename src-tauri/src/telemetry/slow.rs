#[cfg(any(windows, test))]
use super::types::PowerEventKind;
use super::types::{BluetoothBattery, PhysicalDisk, PowerEvent};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SlowData {
    pub disks: Vec<PhysicalDisk>,
    pub bluetooth: Vec<BluetoothBattery>,
    pub metered: Option<bool>,
    pub power_events: Vec<PowerEvent>,
}

#[cfg(any(windows, test))]
mod parse {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize, Default)]
    #[serde(default)]
    struct Raw {
        disks: Vec<RawDisk>,
        bluetooth: Vec<RawBluetooth>,
        metered: Option<bool>,
        power_events: Vec<RawEvent>,
    }

    #[derive(Deserialize)]
    struct RawDisk {
        name: Option<String>,
        kind: Option<String>,
        health: Option<String>,
        temp_c: Option<f32>,
        wear_pct: Option<f32>,
    }

    #[derive(Deserialize)]
    struct RawBluetooth {
        name: Option<String>,
        percent: Option<u32>,
    }

    #[derive(Deserialize)]
    struct RawEvent {
        ts_ms: Option<u64>,
        kind: Option<String>,
    }

    pub fn parse_slow(json: &str) -> Option<SlowData> {
        let raw: Raw = serde_json::from_str(json.trim_start_matches('\u{feff}').trim()).ok()?;
        Some(SlowData {
            disks: raw
                .disks
                .into_iter()
                .filter_map(|d| {
                    Some(PhysicalDisk {
                        name: d.name.filter(|n| !n.is_empty())?,
                        kind: d.kind.filter(|k| !k.is_empty()),
                        health: d.health.filter(|h| !h.is_empty()),
                        temp_c: d.temp_c.filter(|t| t.is_finite() && *t > 0.0),
                        wear_pct: d.wear_pct.filter(|w| w.is_finite() && *w >= 0.0),
                    })
                })
                .collect(),
            bluetooth: raw
                .bluetooth
                .into_iter()
                .filter_map(|b| {
                    Some(BluetoothBattery {
                        name: b.name.filter(|n| !n.is_empty())?,
                        percent: b.percent?.min(100) as u8,
                    })
                })
                .collect(),
            metered: raw.metered,
            power_events: raw
                .power_events
                .into_iter()
                .filter_map(|e| {
                    let kind = match e.kind?.as_str() {
                        "sleep" => PowerEventKind::Sleep,
                        "wake" => PowerEventKind::Wake,
                        _ => return None,
                    };
                    Some(PowerEvent {
                        ts_ms: e.ts_ms?,
                        kind,
                    })
                })
                .take(10)
                .collect(),
        })
    }
}

#[cfg(windows)]
const SCRIPT: &str = r#"
$ErrorActionPreference='SilentlyContinue'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new()
$r=[ordered]@{disks=@();bluetooth=@();metered=$null;power_events=@()}
try{
  $r.disks=@(Get-PhysicalDisk | ForEach-Object {
    $c=$null
    try{ $c=$_ | Get-StorageReliabilityCounter -ErrorAction Stop }catch{}
    $k=if("$($_.BusType)" -eq 'NVMe'){'NVMe'}elseif("$($_.MediaType)" -in @('','Unspecified')){$null}else{"$($_.MediaType)"}
    [ordered]@{name="$($_.FriendlyName)";kind=$k;health="$($_.HealthStatus)";temp_c=$c.Temperature;wear_pct=$c.Wear}
  })
}catch{}
try{
  $r.bluetooth=@(Get-PnpDevice -Class Bluetooth -PresentOnly | ForEach-Object {
    $p=Get-PnpDeviceProperty -InstanceId $_.InstanceId -KeyName '{104EA319-6EE2-4701-BD47-8DDBF425BBE5} 2'
    if($p -and $null -ne $p.Data){ [ordered]@{name="$($_.FriendlyName)";percent=[int]$p.Data} }
  })
}catch{}
try{
  [void][Windows.Networking.Connectivity.NetworkInformation,Windows.Networking.Connectivity,ContentType=WindowsRuntime]
  $p=[Windows.Networking.Connectivity.NetworkInformation]::GetInternetConnectionProfile()
  if($p){ $r.metered=("$($p.GetConnectionCost().NetworkCostType)" -ne 'Unrestricted') }
}catch{}
try{
  $s=(Get-Date).AddDays(-7)
  $e=@()
  $e+=@(Get-WinEvent -FilterHashtable @{LogName='System';ProviderName='Microsoft-Windows-Kernel-Power';Id=42;StartTime=$s} | ForEach-Object { [ordered]@{ts_ms=[DateTimeOffset]::new($_.TimeCreated).ToUnixTimeMilliseconds();kind='sleep'} })
  $e+=@(Get-WinEvent -FilterHashtable @{LogName='System';ProviderName='Microsoft-Windows-Power-Troubleshooter';Id=1;StartTime=$s} | ForEach-Object { [ordered]@{ts_ms=[DateTimeOffset]::new($_.TimeCreated).ToUnixTimeMilliseconds();kind='wake'} })
  $r.power_events=@($e | Sort-Object { $_.ts_ms } -Descending | Select-Object -First 10)
}catch{}
$r | ConvertTo-Json -Depth 5 -Compress
"#;

#[cfg(windows)]
fn run_probe(stop: &std::sync::atomic::AtomicBool) -> Option<SlowData> {
    use std::io::Read;
    use std::os::windows::process::CommandExt;
    use std::process::{Command, Stdio};
    use std::sync::atomic::Ordering;
    use std::time::{Duration, Instant};

    let root = std::env::var_os("SystemRoot")?;
    let exe = std::path::PathBuf::from(root).join("System32/WindowsPowerShell/v1.0/powershell.exe");
    let mut child = Command::new(exe)
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            SCRIPT,
        ])
        .creation_flags(0x0800_0000)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mut stdout = child.stdout.take()?;
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout.read_to_end(&mut buf);
        let _ = tx.send(buf);
    });
    let start = Instant::now();
    let done = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status.success(),
            Ok(None)
                if start.elapsed() < Duration::from_secs(20) && !stop.load(Ordering::Relaxed) =>
            {
                std::thread::sleep(Duration::from_millis(100));
            }
            _ => break false,
        }
    };
    if !done {
        let _ = child.kill();
        let _ = child.wait();
        return None;
    }
    let buf = rx.recv_timeout(Duration::from_secs(2)).ok()?;
    parse::parse_slow(&String::from_utf8_lossy(&buf))
}

#[cfg(windows)]
pub struct SlowProbe {
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    data: std::sync::Arc<std::sync::Mutex<SlowData>>,
}

#[cfg(windows)]
impl SlowProbe {
    pub fn start() -> Self {
        use std::sync::atomic::Ordering;
        use std::sync::{Arc, Mutex};
        use std::time::Duration;

        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let data = Arc::new(Mutex::new(SlowData::default()));
        let (thread_stop, thread_data) = (stop.clone(), data.clone());
        std::thread::spawn(move || {
            while !thread_stop.load(Ordering::Relaxed) {
                if let Some(next) = run_probe(&thread_stop)
                    && let Ok(mut guard) = thread_data.lock()
                {
                    *guard = next;
                }
                for _ in 0..300 {
                    if thread_stop.load(Ordering::Relaxed) {
                        return;
                    }
                    std::thread::sleep(Duration::from_millis(200));
                }
            }
        });
        Self { stop, data }
    }

    pub fn latest(&self) -> SlowData {
        self.data.lock().map(|d| d.clone()).unwrap_or_default()
    }
}

#[cfg(windows)]
impl Drop for SlowProbe {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

#[cfg(not(windows))]
pub struct SlowProbe;

#[cfg(not(windows))]
impl SlowProbe {
    pub fn start() -> Self {
        Self
    }

    pub fn latest(&self) -> SlowData {
        SlowData::default()
    }
}

#[cfg(test)]
mod tests {
    use super::parse::parse_slow;
    use super::*;

    #[test]
    fn parses_full_payload() {
        let json = r#"{"disks":[{"name":"Samsung","kind":"NVMe","health":"Healthy","temp_c":41,"wear_pct":3}],
            "bluetooth":[{"name":"Mouse","percent":80},{"name":"Pad","percent":250}],
            "metered":true,
            "power_events":[{"ts_ms":2000,"kind":"wake"},{"ts_ms":1000,"kind":"sleep"},{"ts_ms":5,"kind":"x"}]}"#;
        let d = parse_slow(json).unwrap();
        assert_eq!(d.disks.len(), 1);
        assert_eq!(d.disks[0].kind.as_deref(), Some("NVMe"));
        assert_eq!(d.disks[0].temp_c, Some(41.0));
        assert_eq!(d.disks[0].wear_pct, Some(3.0));
        assert_eq!(d.bluetooth[1].percent, 100);
        assert_eq!(d.metered, Some(true));
        assert_eq!(d.power_events.len(), 2);
        assert_eq!(d.power_events[0].kind, PowerEventKind::Wake);
    }

    #[test]
    fn parses_nulls_and_empty() {
        let d = parse_slow(
            r#"{"disks":[{"name":"D","kind":null,"health":null,"temp_c":null,"wear_pct":null}],"bluetooth":[{"name":"x","percent":null}],"metered":null,"power_events":[]}"#,
        )
        .unwrap();
        assert_eq!(d.disks[0].kind, None);
        assert_eq!(d.disks[0].temp_c, None);
        assert!(d.bluetooth.is_empty());
        assert_eq!(d.metered, None);
        assert_eq!(
            parse_slow(r#"{"disks":[],"bluetooth":[]}"#).unwrap(),
            SlowData::default()
        );
    }

    #[test]
    fn caps_events_and_rejects_garbage() {
        let events: Vec<String> = (0..15)
            .map(|i| format!(r#"{{"ts_ms":{i},"kind":"sleep"}}"#))
            .collect();
        let json = format!(r#"{{"power_events":[{}]}}"#, events.join(","));
        assert_eq!(parse_slow(&json).unwrap().power_events.len(), 10);
        assert!(parse_slow("").is_none());
        assert!(parse_slow("not json").is_none());
        assert!(parse_slow("[1,2]").is_none());
    }

    #[cfg(windows)]
    #[test]
    #[ignore]
    fn live_probe() {
        let stop = std::sync::atomic::AtomicBool::new(false);
        let data = super::run_probe(&stop);
        println!("{data:#?}");
        assert!(data.is_some());
    }
}
