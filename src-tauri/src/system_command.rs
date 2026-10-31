//! Bounded, hidden execution for built-in Windows management tools.
#[cfg(windows)]
pub fn powershell(script: &str, timeout: std::time::Duration) -> Result<String, String> {
    use std::io::Read;
    use std::os::windows::process::CommandExt;
    use std::process::{Command, Stdio};
    use std::time::Instant;

    let root = std::env::var_os("SystemRoot").ok_or("Windows directory is unavailable")?;
    let exe = std::path::PathBuf::from(root).join("System32/WindowsPowerShell/v1.0/powershell.exe");
    let script = format!(
        "$ErrorActionPreference='Stop'; [Console]::OutputEncoding=[Text.UTF8Encoding]::new(); {script}"
    );
    let mut child = Command::new(exe)
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &script,
        ])
        .creation_flags(0x0800_0000)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let mut stdout = child.stdout.take().ok_or("No command output")?;
    let mut stderr = child.stderr.take().ok_or("No command error stream")?;
    let output = std::thread::spawn(move || {
        let mut buf = String::new();
        stdout.read_to_string(&mut buf).map(|_| buf)
    });
    let error = std::thread::spawn(move || {
        let mut buf = String::new();
        stderr.read_to_string(&mut buf).map(|_| buf)
    });
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if start.elapsed() < timeout => {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            other => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = output.join();
                let _ = error.join();
                return Err(match other {
                    Err(e) => e.to_string(),
                    _ => "The system request timed out. Try again.".into(),
                });
            }
        }
    };
    let output = output
        .join()
        .map_err(|_| "Output reader failed")?
        .map_err(|e| e.to_string())?;
    let error = error
        .join()
        .map_err(|_| "Error reader failed")?
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(output
            .trim_start_matches(char::from_u32(0xfeff).unwrap())
            .trim()
            .to_owned())
    } else {
        Err(if error.trim().is_empty() {
            format!("System request failed ({status})")
        } else {
            error.trim().chars().take(600).collect()
        })
    }
}
