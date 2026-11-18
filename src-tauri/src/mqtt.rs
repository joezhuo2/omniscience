use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::io::{self, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::settings::IntegrationSettings;
use crate::telemetry::Telemetry;
use crate::telemetry::types::Snapshot;

const KEEP_ALIVE_S: u16 = 60;
const PING_EVERY: Duration = Duration::from_secs(30);
const TICK: Duration = Duration::from_millis(200);
const IO_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone)]
struct Config {
    host: String,
    port: u16,
    username: String,
    password: String,
    topic: String,
    interval: Duration,
    discovery: bool,
}

impl Config {
    fn from_settings(s: &IntegrationSettings) -> Option<Self> {
        (s.mqtt_enabled && !s.mqtt_host.is_empty()).then(|| Self {
            host: s.mqtt_host.clone(),
            port: s.mqtt_port,
            username: s.mqtt_username.clone(),
            password: s.mqtt_password.clone(),
            topic: s.mqtt_topic.clone(),
            interval: Duration::from_secs(u64::from(s.mqtt_interval_s)),
            discovery: s.mqtt_discovery,
        })
    }

    fn same_as(&self, o: &Self) -> bool {
        self.host == o.host
            && self.port == o.port
            && self.username == o.username
            && self.password == o.password
            && self.topic == o.topic
            && self.interval == o.interval
            && self.discovery == o.discovery
    }
}

struct Running {
    config: Config,
    stop: Arc<AtomicBool>,
}

static WORKER: Mutex<Option<Running>> = Mutex::new(None);

pub fn sync(telemetry: &Arc<Telemetry>, settings: &IntegrationSettings) {
    let wanted = Config::from_settings(settings);
    let mut slot = WORKER.lock().unwrap_or_else(|e| e.into_inner());
    match (&*slot, &wanted) {
        (Some(r), Some(w)) if r.config.same_as(w) => return,
        (None, None) => return,
        _ => {}
    }
    if let Some(old) = slot.take() {
        old.stop.store(true, Ordering::Relaxed);
    }
    if let Some(config) = wanted {
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = stop.clone();
        let telemetry = telemetry.clone();
        let cfg = config.clone();
        let spawned = std::thread::Builder::new()
            .name("mqtt".into())
            .spawn(move || run(&cfg, &thread_stop, &|| telemetry.latest()));
        if spawned.is_ok() {
            *slot = Some(Running { config, stop });
        }
    }
}

fn run(cfg: &Config, stop: &AtomicBool, latest: &dyn Fn() -> Option<Snapshot>) {
    let mut backoff = 2u64;
    while !stop.load(Ordering::Relaxed) {
        let started = Instant::now();
        let id = client_id();
        if let Err(e) = session(cfg, &id, stop, latest) {
            eprintln!("mqtt: {} (retry in {backoff}s)", e.kind());
        }
        if started.elapsed() > Duration::from_secs(60) {
            backoff = 2;
        }
        let until = Instant::now() + Duration::from_secs(backoff);
        while Instant::now() < until && !stop.load(Ordering::Relaxed) {
            std::thread::sleep(TICK);
        }
        backoff = (backoff * 2).min(60);
    }
}

fn client_id() -> String {
    let n = RandomState::new().build_hasher().finish();
    format!("omniscience-{:08x}", n as u32)
}

fn session(
    cfg: &Config,
    id: &str,
    stop: &AtomicBool,
    latest: &dyn Fn() -> Option<Snapshot>,
) -> io::Result<()> {
    let addr = (cfg.host.as_str(), cfg.port)
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "host did not resolve"))?;
    let mut sock = TcpStream::connect_timeout(&addr, IO_TIMEOUT)?;
    sock.set_nodelay(true)?;
    sock.set_read_timeout(Some(IO_TIMEOUT))?;
    sock.set_write_timeout(Some(IO_TIMEOUT))?;

    let availability = format!("{}/availability", cfg.topic);
    let state_topic = format!("{}/state", cfg.topic);
    sock.write_all(&connect_packet(
        id,
        &cfg.username,
        &cfg.password,
        &availability,
        KEEP_ALIVE_S,
    ))?;
    let mut ack = [0u8; 4];
    sock.read_exact(&mut ack)?;
    if ack[0] != 0x20 || ack[1] != 2 || ack[3] != 0 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "connection refused",
        ));
    }
    sock.set_read_timeout(Some(TICK))?;

    sock.write_all(&publish_packet(&availability, b"online", true))?;
    if cfg.discovery {
        for sensor in SENSORS {
            let topic = discovery_topic(&cfg.topic, sensor.key);
            let body = discovery_payload(&cfg.topic, sensor).to_string();
            sock.write_all(&publish_packet(&topic, body.as_bytes(), true))?;
        }
    }

    let mut last_write = Instant::now();
    let mut last_state: Option<Instant> = None;
    let mut sink = [0u8; 64];
    while !stop.load(Ordering::Relaxed) {
        if last_state.is_none_or(|t| t.elapsed() >= cfg.interval)
            && let Some(snap) = latest()
        {
            let body = state_json(&snap).to_string();
            sock.write_all(&publish_packet(&state_topic, body.as_bytes(), true))?;
            last_write = Instant::now();
            last_state = Some(last_write);
        }
        if last_write.elapsed() >= PING_EVERY {
            sock.write_all(&[0xC0, 0])?;
            last_write = Instant::now();
        }
        match sock.read(&mut sink) {
            Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
            Ok(_) => {}
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) => {}
            Err(e) => return Err(e),
        }
    }
    let _ = sock.write_all(&[0xE0, 0]);
    Ok(())
}

fn remaining_length(mut n: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(4);
    loop {
        let mut b = (n % 128) as u8;
        n /= 128;
        if n > 0 {
            b |= 0x80;
        }
        out.push(b);
        if n == 0 {
            return out;
        }
    }
}

fn put_str(buf: &mut Vec<u8>, s: &str) {
    buf.extend_from_slice(&(s.len() as u16).to_be_bytes());
    buf.extend_from_slice(s.as_bytes());
}

fn frame(first: u8, body: &[u8]) -> Vec<u8> {
    let mut out = vec![first];
    out.extend(remaining_length(body.len()));
    out.extend_from_slice(body);
    out
}

fn connect_packet(
    id: &str,
    username: &str,
    password: &str,
    will_topic: &str,
    keep_alive: u16,
) -> Vec<u8> {
    let mut flags = 0x02 | 0x04 | 0x20;
    if !username.is_empty() {
        flags |= 0x80;
        if !password.is_empty() {
            flags |= 0x40;
        }
    }
    let mut body = Vec::new();
    put_str(&mut body, "MQTT");
    body.push(4);
    body.push(flags);
    body.extend_from_slice(&keep_alive.to_be_bytes());
    put_str(&mut body, id);
    put_str(&mut body, will_topic);
    put_str(&mut body, "offline");
    if !username.is_empty() {
        put_str(&mut body, username);
        if !password.is_empty() {
            put_str(&mut body, password);
        }
    }
    frame(0x10, &body)
}

fn publish_packet(topic: &str, payload: &[u8], retain: bool) -> Vec<u8> {
    let mut body = Vec::new();
    put_str(&mut body, topic);
    body.extend_from_slice(payload);
    frame(0x30 | u8::from(retain), &body)
}

fn round2(x: f32) -> f64 {
    (f64::from(x) * 100.0).round() / 100.0
}

fn state_json(s: &Snapshot) -> Value {
    let c = &s.core;
    let opt = |v: Option<f32>| v.map(round2);
    json!({
        "cpu_percent": round2(c.cpu.percent),
        "cpu_temp_c": opt(c.cpu_temp_c),
        "memory_percent": round2(c.ram.percent),
        "memory_used_bytes": c.ram.used_bytes,
        "memory_total_bytes": c.ram.total_bytes,
        "igpu_percent": opt(c.igpu.map(|m| m.percent)),
        "dgpu_percent": opt(c.dgpu.map(|m| m.percent)),
        "igpu_temp_c": opt(c.igpu_temp_c),
        "dgpu_temp_c": opt(c.dgpu_temp_c),
        "disk_percent": opt(c.disk.map(|m| m.percent)),
        "net_down_bps": c.net.down_bps.round(),
        "net_up_bps": c.net.up_bps.round(),
        "package_watts": opt(c.package_watts),
    })
}

struct Sensor {
    key: &'static str,
    name: &'static str,
    unit: &'static str,
    device_class: Option<&'static str>,
}

const fn sensor(
    key: &'static str,
    name: &'static str,
    unit: &'static str,
    device_class: Option<&'static str>,
) -> Sensor {
    Sensor {
        key,
        name,
        unit,
        device_class,
    }
}

const SENSORS: &[Sensor] = &[
    sensor("cpu_percent", "CPU usage", "%", None),
    sensor("cpu_temp_c", "CPU temperature", "°C", Some("temperature")),
    sensor("memory_percent", "Memory usage", "%", None),
    sensor("memory_used_bytes", "Memory used", "B", Some("data_size")),
    sensor("memory_total_bytes", "Memory total", "B", Some("data_size")),
    sensor("igpu_percent", "Integrated GPU usage", "%", None),
    sensor("dgpu_percent", "Discrete GPU usage", "%", None),
    sensor(
        "igpu_temp_c",
        "Integrated GPU temperature",
        "°C",
        Some("temperature"),
    ),
    sensor(
        "dgpu_temp_c",
        "Discrete GPU temperature",
        "°C",
        Some("temperature"),
    ),
    sensor("disk_percent", "Disk activity", "%", None),
    sensor("net_down_bps", "Network download", "B/s", Some("data_rate")),
    sensor("net_up_bps", "Network upload", "B/s", Some("data_rate")),
    sensor("package_watts", "Package power", "W", Some("power")),
];

fn device_id(topic: &str) -> String {
    let tail: String = topic
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    format!("omniscience_{tail}")
}

fn discovery_topic(topic: &str, key: &str) -> String {
    format!("homeassistant/sensor/{}/{key}/config", device_id(topic))
}

fn discovery_payload(topic: &str, s: &Sensor) -> Value {
    let dev = device_id(topic);
    let mut v = json!({
        "name": s.name,
        "unique_id": format!("{dev}_{}", s.key),
        "state_topic": format!("{topic}/state"),
        "value_template": format!("{{{{ value_json.{} }}}}", s.key),
        "unit_of_measurement": s.unit,
        "state_class": "measurement",
        "availability_topic": format!("{topic}/availability"),
        "device": {
            "identifiers": [dev],
            "name": "Omniscience",
            "manufacturer": "Omniscience",
            "sw_version": env!("CARGO_PKG_VERSION"),
        },
    });
    if let Some(class) = s.device_class {
        v["device_class"] = json!(class);
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::types::{CoreMetrics, Level, MemoryMetric, Metric, NetMetric};
    use std::net::TcpListener;

    fn snapshot() -> Snapshot {
        Snapshot {
            timestamp_ms: 1,
            core: CoreMetrics {
                cpu: Metric {
                    percent: 12.5,
                    level: Level::Normal,
                },
                cpu_mhz: None,
                cpu_temp_c: Some(55.0),
                igpu_temp_c: None,
                dgpu_temp_c: None,
                ram: MemoryMetric {
                    used_bytes: 8,
                    total_bytes: 16,
                    percent: 50.0,
                    level: Level::Normal,
                },
                disk: Some(Metric {
                    percent: 3.0,
                    level: Level::Normal,
                }),
                net: NetMetric {
                    down_bps: 1000.4,
                    up_bps: 20.0,
                    down_pct: 0.0,
                    up_pct: 0.0,
                    level: Level::Normal,
                },
                igpu: None,
                dgpu: None,
                package_watts: Some(15.25),
            },
            detail: None,
        }
    }

    #[test]
    fn remaining_length_boundaries() {
        assert_eq!(remaining_length(0), [0]);
        assert_eq!(remaining_length(127), [0x7F]);
        assert_eq!(remaining_length(128), [0x80, 0x01]);
        assert_eq!(remaining_length(16383), [0xFF, 0x7F]);
        assert_eq!(remaining_length(16384), [0x80, 0x80, 0x01]);
    }

    #[test]
    fn connect_without_credentials() {
        let p = connect_packet("cid", "", "", "t/availability", 60);
        let mut want = vec![0x10, 0];
        let mut body = vec![0, 4, b'M', b'Q', b'T', b'T', 4, 0x26, 0, 60];
        body.extend([0, 3, b'c', b'i', b'd', 0, 14]);
        body.extend(b"t/availability");
        body.extend([0, 7]);
        body.extend(b"offline");
        want[1] = body.len() as u8;
        want.extend(body);
        assert_eq!(p, want);
    }

    #[test]
    fn connect_with_credentials() {
        let p = connect_packet("cid", "bob", "pw", "t/availability", 60);
        assert_eq!(p[9], 0xE6);
        assert!(p.ends_with(&[0, 3, b'b', b'o', b'b', 0, 2, b'p', b'w']));
    }

    #[test]
    fn publish_sets_retain() {
        assert_eq!(
            publish_packet("a", b"xy", true),
            [0x31, 5, 0, 1, b'a', b'x', b'y']
        );
        assert_eq!(publish_packet("a", b"", false)[0], 0x30);
    }

    #[test]
    fn state_has_numbers_and_nulls() {
        let v = state_json(&snapshot());
        assert_eq!(v["cpu_percent"], 12.5);
        assert_eq!(v["memory_used_bytes"], 8);
        assert_eq!(v["net_down_bps"], 1000.0);
        assert_eq!(v["package_watts"], 15.25);
        assert!(v["igpu_percent"].is_null());
        assert!(v["dgpu_temp_c"].is_null());
        assert_eq!(v.as_object().unwrap().len(), 13);
    }

    #[test]
    fn discovery_payload_shape() {
        let s = SENSORS.iter().find(|s| s.key == "cpu_temp_c").unwrap();
        let v = discovery_payload("pc/one", s);
        assert_eq!(
            discovery_topic("pc/one", s.key),
            "homeassistant/sensor/omniscience_pc_one/cpu_temp_c/config"
        );
        assert_eq!(v["unique_id"], "omniscience_pc_one_cpu_temp_c");
        assert_eq!(v["state_topic"], "pc/one/state");
        assert_eq!(v["availability_topic"], "pc/one/availability");
        assert_eq!(v["value_template"], "{{ value_json.cpu_temp_c }}");
        assert_eq!(v["unit_of_measurement"], "°C");
        assert_eq!(v["device_class"], "temperature");
        assert_eq!(v["device"]["name"], "Omniscience");
        assert_eq!(v["device"]["sw_version"], env!("CARGO_PKG_VERSION"));
    }

    fn read_packet(sock: &mut TcpStream) -> (u8, Vec<u8>) {
        let mut first = [0u8; 1];
        sock.read_exact(&mut first).unwrap();
        let (mut len, mut mult) = (0usize, 1usize);
        loop {
            let mut b = [0u8; 1];
            sock.read_exact(&mut b).unwrap();
            len += usize::from(b[0] & 0x7F) * mult;
            mult *= 128;
            if b[0] & 0x80 == 0 {
                break;
            }
        }
        let mut body = vec![0u8; len];
        sock.read_exact(&mut body).unwrap();
        (first[0], body)
    }

    #[test]
    fn publishes_to_a_fake_broker() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let cfg = Config {
            host: "127.0.0.1".into(),
            port,
            username: "bob".into(),
            password: "pw".into(),
            topic: "pc".into(),
            interval: Duration::from_secs(60),
            discovery: false,
        };
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = stop.clone();
        let worker = std::thread::spawn(move || {
            session(&cfg, "omniscience-test", &worker_stop, &|| Some(snapshot()))
        });

        let (mut sock, _) = listener.accept().unwrap();
        sock.set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let want = connect_packet("omniscience-test", "bob", "pw", "pc/availability", 60);
        let (first, body) = read_packet(&mut sock);
        assert_eq!(first, want[0]);
        assert_eq!(body, want[2..]);
        sock.write_all(&[0x20, 2, 0, 0]).unwrap();

        let (first, body) = read_packet(&mut sock);
        assert_eq!(first, 0x31);
        assert_eq!(body, [&[0, 15][..], b"pc/availability", b"online"].concat());
        let (first, body) = read_packet(&mut sock);
        assert_eq!(first, 0x31);
        assert!(body.starts_with(&[0, 8, b'p', b'c', b'/', b's']));
        let json: Value = serde_json::from_slice(&body[10..]).unwrap();
        assert_eq!(json["cpu_percent"], 12.5);

        stop.store(true, Ordering::Relaxed);
        worker.join().unwrap().unwrap();
    }
}
