#[cfg(any(windows, test))]
use super::types::PowerEventKind;
use super::types::{PhysicalDisk, PowerEvent};

#[cfg(any(windows, test))]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SlowData {
    pub disks: Vec<PhysicalDisk>,
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
const PRELUDE: &str = r#"
$ErrorActionPreference='SilentlyContinue'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new()
$r=[ordered]@{}
"#;

#[cfg(windows)]
const DISKS: &str = r#"
try{
  $r.disks=@(Get-PhysicalDisk | ForEach-Object {
    $c=$null
    try{ $c=$_ | Get-StorageReliabilityCounter -ErrorAction Stop }catch{}
    $k=if("$($_.BusType)" -eq 'NVMe'){'NVMe'}elseif("$($_.MediaType)" -in @('','Unspecified')){$null}else{"$($_.MediaType)"}
    [ordered]@{name="$($_.FriendlyName)";kind=$k;health="$($_.HealthStatus)";temp_c=$c.Temperature;wear_pct=$c.Wear}
  })
}catch{}
"#;

#[cfg(windows)]
const METERED: &str = r#"
try{
  [void][Windows.Networking.Connectivity.NetworkInformation,Windows.Networking.Connectivity,ContentType=WindowsRuntime]
  $p=[Windows.Networking.Connectivity.NetworkInformation]::GetInternetConnectionProfile()
  if($p){ $r.metered=("$($p.GetConnectionCost().NetworkCostType)" -ne 'Unrestricted') }
}catch{}
"#;

#[cfg(windows)]
const POWER_EVENTS: &str = r#"
try{
  $s=(Get-Date).AddDays(-7)
  $e=@()
  $e+=@(Get-WinEvent -FilterHashtable @{LogName='System';ProviderName='Microsoft-Windows-Kernel-Power';Id=42;StartTime=$s} | ForEach-Object { [ordered]@{ts_ms=[DateTimeOffset]::new($_.TimeCreated).ToUnixTimeMilliseconds();kind='sleep'} })
  $e+=@(Get-WinEvent -FilterHashtable @{LogName='System';ProviderName='Microsoft-Windows-Power-Troubleshooter';Id=1;StartTime=$s} | ForEach-Object { [ordered]@{ts_ms=[DateTimeOffset]::new($_.TimeCreated).ToUnixTimeMilliseconds();kind='wake'} })
  $r.power_events=@($e | Sort-Object { $_.ts_ms } -Descending | Select-Object -First 10)
}catch{}
"#;

/// A cold Storage module load alone can take 15 s.
#[cfg(windows)]
const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);

#[cfg(windows)]
fn run_probe(body: &str, stop: &std::sync::atomic::AtomicBool) -> Option<SlowData> {
    use std::io::Read;
    use std::os::windows::process::CommandExt;
    use std::process::{Command, Stdio};
    use std::sync::atomic::Ordering;
    use std::time::{Duration, Instant};

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    const BELOW_NORMAL_PRIORITY_CLASS: u32 = 0x0000_4000;
    let script = format!("{PRELUDE}{body}$r | ConvertTo-Json -Depth 5 -Compress");
    let root = std::env::var_os("SystemRoot")?;
    let exe = std::path::PathBuf::from(root).join("System32/WindowsPowerShell/v1.0/powershell.exe");
    let mut child = Command::new(exe)
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &script,
        ])
        .creation_flags(CREATE_NO_WINDOW | BELOW_NORMAL_PRIORITY_CLASS)
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
            Ok(None) if start.elapsed() < TIMEOUT && !stop.load(Ordering::Relaxed) => {
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
pub fn disks(stop: &std::sync::atomic::AtomicBool) -> Option<Vec<PhysicalDisk>> {
    run_probe(DISKS, stop).map(|d| d.disks)
}

#[cfg(windows)]
pub fn metered(stop: &std::sync::atomic::AtomicBool) -> Option<Option<bool>> {
    run_probe(METERED, stop).map(|d| d.metered)
}

#[cfg(windows)]
pub fn power_events(stop: &std::sync::atomic::AtomicBool) -> Option<Vec<PowerEvent>> {
    run_probe(POWER_EVENTS, stop).map(|d| d.power_events)
}

#[cfg(not(windows))]
pub fn disks(_stop: &std::sync::atomic::AtomicBool) -> Option<Vec<PhysicalDisk>> {
    None
}

#[cfg(not(windows))]
pub fn metered(_stop: &std::sync::atomic::AtomicBool) -> Option<Option<bool>> {
    None
}

#[cfg(not(windows))]
pub fn power_events(_stop: &std::sync::atomic::AtomicBool) -> Option<Vec<PowerEvent>> {
    None
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
        println!("{:#?}", super::disks(&stop));
        println!("{:#?}", super::metered(&stop));
        let events = super::power_events(&stop);
        println!("{events:#?}");
        assert!(events.is_some());
    }
}
