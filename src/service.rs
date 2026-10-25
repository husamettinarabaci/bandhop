//! Optional per-user systemd integration. Does not require root privileges.
use crate::paths::{self, AppResult};
use std::fs;
use std::process::Command;

fn run_systemctl(args: &[&str]) -> AppResult<()> {
    let status = Command::new("systemctl").arg("--user").args(args)
        .status().map_err(|e| format!("Failed to run systemctl --user: {e}"))?;
    if status.success() { Ok(()) } else { Err(format!("systemctl --user {} failed ({status})", args.join(" "))) }
}

fn escape_systemd_path(s: &str) -> AppResult<String> {
    if s.chars().any(|c| c == '\n' || c == '\r') { return Err("Unsupported newline in executable path".into()); }
    // In a systemd unit, %% is a literal percent and quotes escape whitespace.
    Ok(format!("\"{}\"", s.replace('%', "%%").replace('\\', "\\\\").replace('"', "\\\"")))
}

pub fn install() -> AppResult<()> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let path = paths::service_file()?;
    paths::create_parent(&path)?;
    let executable = escape_systemd_path(&exe.to_string_lossy())?;
    let unit = format!("[Unit]\nDescription=Bandhop user internet radio controller\nDocumentation=man:bandhop(1)\n\n[Service]\nType=simple\nExecStart={executable} daemon\nRestart=on-failure\nRestartSec=2\nKillMode=control-group\n\n[Install]\nWantedBy=default.target\n");
    fs::write(&path, unit).map_err(|e| format!("Cannot write {}: {e}", path.display()))?;
    run_systemctl(&["daemon-reload"])?;
    println!("Installed user service: {}", path.display());
    println!("Enable it with: bandhop service enable");
    Ok(())
}

pub fn enable() -> AppResult<()> {
    install()?;
    run_systemctl(&["enable", "--now", "bandhop.service"])?;
    println!("Bandhop daemon enabled. Playback is OFF until you issue `bandhop play`/`toggle`.");
    Ok(())
}

pub fn disable() -> AppResult<()> {
    run_systemctl(&["disable", "--now", "bandhop.service"])?;
    println!("Bandhop user service disabled.");
    Ok(())
}

pub fn uninstall() -> AppResult<()> {
    let _ = run_systemctl(&["disable", "--now", "bandhop.service"]);
    let path = paths::service_file()?;
    if path.exists() { fs::remove_file(&path).map_err(|e| e.to_string())?; }
    run_systemctl(&["daemon-reload"])?;
    println!("Removed Bandhop user service. Configuration and stations are preserved.");
    Ok(())
}

pub fn status() -> AppResult<()> {
    run_systemctl(&["--no-pager", "status", "bandhop.service"])
}
