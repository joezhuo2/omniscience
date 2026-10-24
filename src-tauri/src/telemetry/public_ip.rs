//! The public IPv4 address, from one DNS query to OpenDNS (`myip.opendns.com` answered by its own
//! resolver with the address the query came from). Plain UDP, so no HTTP client is needed.

use std::net::{Ipv4Addr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const RESOLVER: (Ipv4Addr, u16) = (Ipv4Addr::new(208, 67, 222, 222), 53);
const NAME: &str = "myip.opendns.com";
const TIMEOUT: Duration = Duration::from_secs(3);
/// The address rarely changes; look it up again at most this often.
const EVERY: Duration = Duration::from_secs(600);

/// A DNS query for the A record of `name` with transaction `id`, recursion desired.
pub fn query(id: u16, name: &str) -> Vec<u8> {
    let mut q = Vec::with_capacity(18 + name.len());
    q.extend_from_slice(&id.to_be_bytes());
    q.extend_from_slice(&[0x01, 0x00, 0, 1, 0, 0, 0, 0, 0, 0]);
    for label in name.split('.') {
        q.push(label.len() as u8);
        q.extend_from_slice(label.as_bytes());
    }
    q.extend_from_slice(&[0, 0, 1, 0, 1]);
    q
}

/// The first A record in `resp`, the answer to `query` (whose question it repeats).
pub fn parse_answer(resp: &[u8], query: &[u8]) -> Option<Ipv4Addr> {
    let u16_at = |i: usize| Some(u16::from_be_bytes([*resp.get(i)?, *resp.get(i + 1)?]));
    if resp.get(..2)? != query.get(..2)? || resp.get(3)? & 0x0F != 0 {
        return None;
    }
    let answers = u16_at(6)?;
    let mut i = query.len();
    for _ in 0..answers {
        // Owner name: a compression pointer or a run of labels.
        loop {
            let len = *resp.get(i)?;
            if len & 0xC0 == 0xC0 {
                i += 2;
                break;
            }
            i += 1 + len as usize;
            if len == 0 {
                break;
            }
        }
        let (kind, rdlen) = (u16_at(i)?, u16_at(i + 8)? as usize);
        let data = resp.get(i + 10..i + 10 + rdlen)?;
        if kind == 1 && rdlen == 4 {
            return Some(Ipv4Addr::new(data[0], data[1], data[2], data[3]));
        }
        i += 10 + rdlen;
    }
    None
}

/// Asks OpenDNS for this machine's public address.
pub fn lookup() -> Option<Ipv4Addr> {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).ok()?;
    socket.set_read_timeout(Some(TIMEOUT)).ok()?;
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.subsec_nanos());
    let id = (nanos ^ std::process::id()) as u16;
    let q = query(id, NAME);
    socket.send_to(&q, RESOLVER).ok()?;
    let mut buf = [0u8; 512];
    let n = socket.recv(&mut buf).ok()?;
    parse_answer(&buf[..n], &q)
}

/// The last looked-up address, refreshed in the background at most every ten minutes.
#[derive(Default)]
pub struct PublicIp {
    latest: Arc<Mutex<Option<Ipv4Addr>>>,
    fetched: Option<Instant>,
}

impl PublicIp {
    /// Starts a lookup on its own thread unless a recent one exists.
    pub fn refresh(&mut self) {
        if self.fetched.is_some_and(|t| t.elapsed() < EVERY) {
            return;
        }
        self.fetched = Some(Instant::now());
        let out = self.latest.clone();
        let spawned = std::thread::Builder::new()
            .name("public-ip".into())
            .spawn(move || {
                let ip = lookup();
                *out.lock().unwrap_or_else(|e| e.into_inner()) = ip;
            });
        if let Err(e) = spawned {
            eprintln!("spawn public IP lookup: {e}");
        }
    }

    pub fn latest(&self) -> Option<Ipv4Addr> {
        *self.latest.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(q: &[u8], rcode: u8, answers: &[(u16, &[u8])]) -> Vec<u8> {
        let mut r = q.to_vec();
        r[2] = 0x81;
        r[3] = 0x80 | rcode;
        r[7] = answers.len() as u8;
        for (kind, data) in answers {
            r.extend_from_slice(&[0xC0, 0x0C]);
            r.extend_from_slice(&kind.to_be_bytes());
            r.extend_from_slice(&[0, 1, 0, 0, 0, 60]);
            r.extend_from_slice(&(data.len() as u16).to_be_bytes());
            r.extend_from_slice(data);
        }
        r
    }

    #[test]
    fn builds_a_standard_query() {
        let q = query(0xABCD, "a.bc");
        assert_eq!(
            q,
            [
                0xAB, 0xCD, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, b'a', 2, b'b', b'c', 0, 0, 1, 0, 1
            ]
        );
    }

    #[test]
    fn reads_the_a_record() {
        let q = query(7, NAME);
        let r = response(&q, 0, &[(1, &[203, 0, 113, 9])]);
        assert_eq!(parse_answer(&r, &q), Some(Ipv4Addr::new(203, 0, 113, 9)));
    }

    #[test]
    fn skips_records_that_are_not_a() {
        let q = query(7, NAME);
        let r = response(&q, 0, &[(5, b"\x01x\x00"), (1, &[198, 51, 100, 1])]);
        assert_eq!(parse_answer(&r, &q), Some(Ipv4Addr::new(198, 51, 100, 1)));
    }

    #[test]
    fn rejects_errors_mismatched_ids_and_truncation() {
        let q = query(7, NAME);
        assert_eq!(parse_answer(&response(&q, 3, &[]), &q), None);
        let other = response(&query(8, NAME), 0, &[(1, &[1, 2, 3, 4])]);
        assert_eq!(parse_answer(&other, &q), None);
        let full = response(&q, 0, &[(1, &[1, 2, 3, 4])]);
        assert_eq!(parse_answer(&full[..full.len() - 2], &q), None);
    }

    /// Network-dependent: `cargo test public_ip_live -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn public_ip_live() {
        println!("{:?}", lookup());
    }
}
