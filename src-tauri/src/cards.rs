use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::{AppHandle, Manager};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct CustomCardRow {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct CustomCard {
    pub id: String,
    pub title: String,
    pub rows: Vec<CustomCardRow>,
    pub refresh_s: Option<u32>,
    pub error: Option<String>,
}

const MAX_FILE: u64 = 64 * 1024;
const MAX_RESPONSE: usize = 256 * 1024;
const MAX_TITLE: usize = 60;
const MAX_ROWS: usize = 12;
const MAX_LABEL: usize = 40;
const MAX_VALUE: usize = 120;
const TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Deserialize)]
struct RowDef {
    label: String,
    value: String,
}

#[derive(Deserialize)]
struct SourceDef {
    url: String,
    refresh_s: Option<u32>,
}

#[derive(Deserialize)]
struct CardDef {
    title: Option<String>,
    rows: Option<Vec<RowDef>>,
    source: Option<SourceDef>,
}

#[derive(Debug, PartialEq)]
struct Source {
    host: String,
    port: u16,
    authority: String,
    path: String,
}

fn truncate(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

fn card_id(stem: &str) -> String {
    stem.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect()
}

fn parse_source(url: &str) -> Result<Source, String> {
    let invalid = || "Source URL must be http:// to localhost.".to_string();
    let rest = url.trim().strip_prefix("http://").ok_or_else(invalid)?;
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let (authority, tail) = rest.split_at(end);
    if authority.contains('@') {
        return Err("Source URL must not contain credentials.".into());
    }
    let (host, port) = if let Some(after) = authority.strip_prefix('[') {
        let (h, p) = after.split_once(']').ok_or_else(invalid)?;
        if p.is_empty() {
            (format!("[{h}]"), None)
        } else {
            (
                format!("[{h}]"),
                Some(p.strip_prefix(':').ok_or_else(invalid)?),
            )
        }
    } else {
        match authority.split_once(':') {
            Some((h, p)) => (h.to_string(), Some(p)),
            None => (authority.to_string(), None),
        }
    };
    let host = host.to_ascii_lowercase();
    if !["127.0.0.1", "localhost", "[::1]"].contains(&host.as_str()) {
        return Err(invalid());
    }
    let port = match port {
        None => 80,
        Some(p) => p
            .parse::<u16>()
            .ok()
            .filter(|p| *p != 0)
            .ok_or_else(|| "Source URL has an invalid port.".to_string())?,
    };
    let path = if tail.starts_with('/') {
        tail.to_string()
    } else {
        format!("/{tail}")
    };
    Ok(Source {
        host: host.trim_matches(['[', ']']).to_string(),
        port,
        authority: authority.to_string(),
        path,
    })
}

fn decode_chunked(mut body: &[u8]) -> Result<Vec<u8>, String> {
    let bad = || "Malformed chunked response.".to_string();
    let mut out = Vec::new();
    loop {
        let eol = body.windows(2).position(|w| w == b"\r\n").ok_or_else(bad)?;
        let line = std::str::from_utf8(&body[..eol]).map_err(|_| bad())?;
        let size = usize::from_str_radix(line.split(';').next().unwrap_or("").trim(), 16)
            .map_err(|_| bad())?;
        body = &body[eol + 2..];
        if size == 0 {
            return Ok(out);
        }
        if body.len() < size {
            return Err(bad());
        }
        out.extend_from_slice(&body[..size]);
        body = body[size..].strip_prefix(b"\r\n").ok_or_else(bad)?;
    }
}

fn parse_response(raw: &[u8]) -> Result<serde_json::Value, String> {
    if raw.len() > MAX_RESPONSE {
        return Err("Response is too large.".into());
    }
    let split = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or("Invalid HTTP response.")?;
    let head = String::from_utf8_lossy(&raw[..split]);
    let body = &raw[split + 4..];
    let mut lines = head.lines();
    let status = lines
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .ok_or("Invalid HTTP response.")?;
    if status != "200" {
        return Err(format!("Source returned HTTP {status}."));
    }
    let header = |name: &str| {
        lines
            .clone()
            .filter_map(|l| l.split_once(':'))
            .find(|(k, _)| k.trim().eq_ignore_ascii_case(name))
            .map(|(_, v)| v.trim().to_ascii_lowercase())
    };
    let decoded;
    let body = if header("transfer-encoding").is_some_and(|v| v.contains("chunked")) {
        decoded = decode_chunked(body)?;
        &decoded[..]
    } else if let Some(n) = header("content-length").and_then(|v| v.parse::<usize>().ok()) {
        &body[..n.min(body.len())]
    } else {
        body
    };
    serde_json::from_slice(body).map_err(|_| "Source did not return JSON.".to_string())
}

fn fetch(src: &Source) -> Result<serde_json::Value, String> {
    let addrs = (src.host.as_str(), src.port)
        .to_socket_addrs()
        .map_err(|_| "Could not resolve the source.".to_string())?;
    let mut stream = addrs
        .filter_map(|a| TcpStream::connect_timeout(&a, TIMEOUT).ok())
        .next()
        .ok_or("Could not connect to the source.")?;
    stream
        .set_read_timeout(Some(TIMEOUT))
        .and_then(|_| stream.set_write_timeout(Some(TIMEOUT)))
        .map_err(|e| e.to_string())?;
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nAccept: application/json\r\nConnection: close\r\n\r\n",
        src.path, src.authority
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|_| "Could not send the request.".to_string())?;
    let mut raw = Vec::new();
    stream
        .take(MAX_RESPONSE as u64 + 1)
        .read_to_end(&mut raw)
        .map_err(|_| "The source did not respond in time.".to_string())?;
    parse_response(&raw)
}

fn read_def(path: &Path) -> Result<CardDef, String> {
    let meta = std::fs::metadata(path).map_err(|_| "Could not read the card file.".to_string())?;
    if meta.len() > MAX_FILE {
        return Err("Card file is larger than 64 KB.".into());
    }
    let bytes = std::fs::read(path).map_err(|_| "Could not read the card file.".to_string())?;
    serde_json::from_slice(&bytes).map_err(|e| format!("Invalid card JSON: {e}"))
}

fn resolve_source(def: &CardDef) -> Result<Option<(Source, u32)>, String> {
    def.source
        .as_ref()
        .map(|s| {
            Ok((
                parse_source(&s.url)?,
                s.refresh_s.unwrap_or(10).clamp(2, 3600),
            ))
        })
        .transpose()
}

fn load_card(id: &str, path: &Path) -> CustomCard {
    let loaded = read_def(path).and_then(|def| resolve_source(&def).map(|s| (def, s)));
    match loaded {
        Ok((def, source)) => CustomCard {
            id: id.to_string(),
            title: truncate(def.title.as_deref().unwrap_or(id), MAX_TITLE),
            rows: def
                .rows
                .unwrap_or_default()
                .into_iter()
                .take(MAX_ROWS)
                .map(|r| CustomCardRow {
                    label: truncate(&r.label, MAX_LABEL),
                    value: truncate(&r.value, MAX_VALUE),
                })
                .collect(),
            refresh_s: source.map(|(_, r)| r),
            error: None,
        },
        Err(error) => CustomCard {
            id: id.to_string(),
            title: id.to_string(),
            rows: Vec::new(),
            refresh_s: None,
            error: Some(error),
        },
    }
}

fn card_files(dir: &Path) -> Vec<(String, PathBuf)> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<(std::ffi::OsString, PathBuf)> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.extension()
                    .is_some_and(|x| x.eq_ignore_ascii_case("json"))
        })
        .filter_map(|p| Some((p.file_name()?.to_os_string(), p)))
        .collect();
    files.sort();
    files
        .into_iter()
        .filter_map(|(_, p)| {
            let id = card_id(&p.file_stem()?.to_string_lossy());
            (!id.is_empty()).then_some((id, p))
        })
        .collect()
}

fn list_in(dir: &Path) -> Vec<CustomCard> {
    card_files(dir)
        .iter()
        .map(|(id, path)| load_card(id, path))
        .collect()
}

fn data_in(dir: &Path, id: &str) -> Result<serde_json::Value, String> {
    let (_, path) = card_files(dir)
        .into_iter()
        .find(|(i, _)| i == id)
        .ok_or("Unknown card.")?;
    let def = read_def(&path)?;
    let (source, _) = resolve_source(&def)?.ok_or("This card has no data source.")?;
    fetch(&source)
}

fn cards_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|d| d.join("cards"))
        .map_err(|e| e.to_string())
}

fn open_folder(app: &AppHandle) -> Result<(), String> {
    let dir = cards_dir(app)?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let program = if cfg!(windows) {
        "explorer.exe"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    std::process::Command::new(program)
        .arg(&dir)
        .spawn()
        .and_then(|mut c| c.wait())
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_custom_cards(app: AppHandle) -> Result<Vec<CustomCard>, String> {
    tauri::async_runtime::spawn_blocking(move || Ok(list_in(&cards_dir(&app)?)))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_custom_card_data(id: String, app: AppHandle) -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(move || data_in(&cards_dir(&app)?, &id))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn open_custom_cards_folder(app: AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || open_folder(&app))
        .await
        .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    fn ok(body: &str) -> Vec<u8> {
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        )
        .into_bytes()
    }

    #[test]
    fn source_validation() {
        let s = parse_source("http://127.0.0.1:3000/stats?a=1").unwrap();
        assert_eq!(
            (s.host.as_str(), s.port, s.path.as_str()),
            ("127.0.0.1", 3000, "/stats?a=1")
        );
        assert_eq!(parse_source("http://localhost").unwrap().path, "/");
        assert_eq!(parse_source("http://localhost").unwrap().port, 80);
        let v6 = parse_source("http://[::1]:8080/x").unwrap();
        assert_eq!((v6.host.as_str(), v6.port), ("::1", 8080));
        assert_eq!(parse_source("http://[::1]/").unwrap().port, 80);
        for bad in [
            "https://127.0.0.1/",
            "http://example.com/",
            "http://127.0.0.1.evil.com/",
            "http://user:pw@127.0.0.1/",
            "http://127.0.0.1:0/",
            "http://127.0.0.1:99999/",
            "http://127.0.0.1:/",
            "ftp://localhost/",
            "localhost",
            "http://[::2]/",
        ] {
            assert!(parse_source(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn response_parsing() {
        assert_eq!(parse_response(&ok("{\"a\":1}")).unwrap()["a"], 1);
        let chunked = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\n{\"a\"\r\n4\r\n:42}\r\n0\r\n\r\n";
        assert_eq!(parse_response(chunked).unwrap()["a"], 42);
        assert_eq!(
            parse_response(b"HTTP/1.1 404 Not Found\r\n\r\n{}").unwrap_err(),
            "Source returned HTTP 404."
        );
        assert_eq!(
            parse_response(&ok("nope")).unwrap_err(),
            "Source did not return JSON."
        );
        let big = ok(&format!("[{}0]", "0,".repeat(MAX_RESPONSE)));
        assert_eq!(parse_response(&big).unwrap_err(), "Response is too large.");
        assert!(parse_response(b"garbage").is_err());
        assert!(
            parse_response(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\nzz\r\n").is_err()
        );
    }

    #[test]
    fn ids_and_limits() {
        assert_eq!(card_id("my card.v2_x-1"), "mycardv2_x-1");
        assert_eq!(card_id("é ü"), "");
        let dir = std::env::temp_dir().join(format!("omni-cards-limits-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let rows: Vec<String> = (0..20)
            .map(|_| {
                format!(
                    "{{\"label\":\"{}\",\"value\":\"{}\"}}",
                    "l".repeat(80),
                    "v".repeat(300)
                )
            })
            .collect();
        let json = format!(
            "{{\"title\":\"{}\",\"rows\":[{}],\"source\":{{\"url\":\"http://localhost/\",\"refresh_s\":1}}}}",
            "t".repeat(100),
            rows.join(",")
        );
        std::fs::write(dir.join("big.json"), json).unwrap();
        std::fs::write(dir.join("huge.json"), " ".repeat(70_000)).unwrap();
        std::fs::write(dir.join("é.json"), "{}").unwrap();
        let cards = list_in(&dir);
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(cards.len(), 2);
        assert_eq!(cards[0].title.len(), MAX_TITLE);
        assert_eq!(cards[0].rows.len(), MAX_ROWS);
        assert_eq!(cards[0].rows[0].label.len(), MAX_LABEL);
        assert_eq!(cards[0].rows[0].value.len(), MAX_VALUE);
        assert_eq!(cards[0].refresh_s, Some(2));
        assert_eq!(
            cards[1].error.as_deref(),
            Some("Card file is larger than 64 KB.")
        );
    }

    #[test]
    fn cards_from_folder_and_source() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || {
            let (mut conn, _) = listener.accept().unwrap();
            let mut buf = [0u8; 1024];
            let n = conn.read(&mut buf).unwrap();
            let req = String::from_utf8_lossy(&buf[..n]).to_string();
            conn.write_all(&ok("{\"queue\":{\"length\":7}}")).unwrap();
            req
        });
        let dir = std::env::temp_dir().join(format!("omni-cards-int-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let a = format!(
            "{{\"title\":\"Build\",\"rows\":[{{\"label\":\"Q\",\"value\":\"{{data.queue.length}}\"}}],\"source\":{{\"url\":\"http://127.0.0.1:{port}/stats\",\"refresh_s\":5}}}}"
        );
        std::fs::write(dir.join("b-build.json"), a).unwrap();
        std::fs::write(
            dir.join("a-local.json"),
            "{\"rows\":[{\"label\":\"x\",\"value\":\"y\"}]}",
        )
        .unwrap();
        std::fs::write(dir.join("c-broken.json"), "{not json").unwrap();
        std::fs::write(
            dir.join("d-https.json"),
            "{\"source\":{\"url\":\"https://127.0.0.1/\"}}",
        )
        .unwrap();
        std::fs::write(dir.join("notes.txt"), "x").unwrap();

        let cards = list_in(&dir);
        let ids: Vec<&str> = cards.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids, ["a-local", "b-build", "c-broken", "d-https"]);
        assert_eq!(cards[0].title, "a-local");
        assert_eq!(cards[0].refresh_s, None);
        assert_eq!(cards[1].title, "Build");
        assert_eq!(cards[1].refresh_s, Some(5));
        assert_eq!(cards[1].rows[0].value, "{data.queue.length}");
        assert!(cards[2].error.is_some() && cards[2].rows.is_empty());
        assert!(cards[3].error.is_some() && cards[3].refresh_s.is_none());

        let value = data_in(&dir, "b-build").unwrap();
        assert_eq!(value["queue"]["length"], 7);
        assert!(
            server
                .join()
                .unwrap()
                .starts_with("GET /stats HTTP/1.1\r\n")
        );
        assert_eq!(
            data_in(&dir, "a-local").unwrap_err(),
            "This card has no data source."
        );
        assert_eq!(data_in(&dir, "nope").unwrap_err(), "Unknown card.");
        assert!(data_in(&dir, "d-https").is_err());
        assert!(list_in(&dir.join("missing")).is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
