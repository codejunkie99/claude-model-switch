use crate::commands::pid_file_path;
use anyhow::{bail, Context, Result};
use std::process::Command;

pub fn start_daemon(port: u16) -> Result<()> {
    let pid_path = pid_file_path()?;

    if pid_path.exists() {
        let pid_str = std::fs::read_to_string(&pid_path)?;
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            let status = Command::new("kill").arg("-0").arg(pid.to_string()).status();
            if status.map(|s| s.success()).unwrap_or(false) {
                bail!(
                    "Proxy already running (PID {}). Stop it first with: claude-model-switch stop",
                    pid
                );
            }
            std::fs::remove_file(&pid_path)?;
        }
    }

    let exe = std::env::current_exe().context("Could not determine executable path")?;

    let child = Command::new(&exe)
        .arg("start")
        .arg("--port")
        .arg(port.to_string())
        .arg("--foreground")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("Failed to spawn proxy process")?;

    let pid = child.id();
    std::fs::write(&pid_path, pid.to_string())?;
    println!("Proxy started on http://127.0.0.1:{} (PID {})", port, pid);
    Ok(())
}

pub fn stop_daemon() -> Result<()> {
    let pid_path = pid_file_path()?;

    if !pid_path.exists() {
        bail!("Proxy is not running (no PID file found).");
    }

    let pid_str = std::fs::read_to_string(&pid_path)?;
    let pid: u32 = pid_str.trim().parse().context("Invalid PID file")?;

    let status = Command::new("kill").arg(pid.to_string()).status()?;
    std::fs::remove_file(&pid_path)?;

    if status.success() {
        println!("Proxy stopped (PID {}).", pid);
    } else {
        println!("Process {} was not running. Cleaned up PID file.", pid);
    }
    Ok(())
}
