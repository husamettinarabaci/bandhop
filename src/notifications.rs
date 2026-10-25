//! Freedesktop desktop notifications sent with `notify-send` (libnotify).
//! A user session and a notification daemon are optional: playback never
//! depends on desktop notifications being available.
use crate::paths::AppResult;
use std::{env, process::{Command, Stdio}};

pub fn deliver(title: &str, body: &str) -> AppResult<()> {
    let mut cmd = Command::new("notify-send");
    cmd.args([
        "--app-name=Bandhop",
        "--urgency=normal",
        "--expire-time=3500",
        "--icon=audio-headphones",
        title,
        body,
    ]);
    // User services normally inherit the session bus. If the environment
    // lacks this variable, the well-known user-bus socket is the fallback.
    if env::var_os("DBUS_SESSION_BUS_ADDRESS").is_none() {
        if let Some(dir) = env::var_os("XDG_RUNTIME_DIR") {
            let bus = std::path::PathBuf::from(dir).join("bus");
            cmd.env("DBUS_SESSION_BUS_ADDRESS", format!("unix:path={}", bus.display()));
        }
    }
    let output = cmd.stdin(Stdio::null())
        .output().map_err(|error| format!("cannot run notify-send: {error} (install libnotify)"))?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr);
        return Err(format!("notify-send failed ({}): {}", output.status, detail.trim()));
    }
    Ok(())
}

pub fn fire_and_log(title: &str, body: &str) {
    let title = title.to_owned();
    let body = body.to_owned();
    // Notify asynchronously so a stalled desktop notification service never
    // blocks playback, station switching, or shutdown.
    std::thread::spawn(move || {
        if let Err(error) = deliver(&title, &body) {
            eprintln!("bandhop: notification not delivered: {error}");
        }
    });
}

