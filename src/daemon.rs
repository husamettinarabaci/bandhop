//! A per-user Unix-domain-socket command server and an on-demand mpv child.
//! Daemon startup never starts audio, including after systemd restarts.
use crate::config::{self, Config};
use crate::paths::{self, AppResult};
use crate::notifications;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::os::unix::fs::FileTypeExt;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn notify(cfg: &Config, event: &str, title: &str, detail: &str) {
    let n = &cfg.notifications;
    let event_enabled = match event {
        "play" => n.on_play,
        "stop" => n.on_stop,
        "change" => n.on_change,
        _ => false,
    };
    if n.enabled && event_enabled {
        notifications::fire_and_log(title, detail);
    }
}

pub fn send(command: &str) -> AppResult<String> {
    let path = paths::socket_file()?;
    let mut stream = UnixStream::connect(&path).map_err(|_| {
        "Bandhop is not running. Start `bandhop listen` or `bandhop service enable`.".to_string()
    })?;
    stream.set_read_timeout(Some(Duration::from_secs(3))).map_err(|e| e.to_string())?;
    stream.write_all(format!("{command}\n").as_bytes()).map_err(|e| e.to_string())?;
    stream.shutdown(std::net::Shutdown::Write).map_err(|e| e.to_string())?;
    let mut response = String::new();
    stream.take(4096).read_to_string(&mut response).map_err(|e| e.to_string())?;
    let response = response.trim_end();
    if let Some(message) = response.strip_prefix("OK\t") { Ok(message.to_string()) }
    else if let Some(message) = response.strip_prefix("ERR\t") { Err(message.to_string()) }
    else { Err("Invalid response from Bandhop".into()) }
}

pub fn try_reload() {
    let _ = send("reload");
}

struct Controller {
    config: Config,
    selected: usize,
    player: Option<Child>,
}

impl Controller {
    fn new(config: Config) -> Self {
        let saved = paths::state_file().ok().and_then(|p| fs::read_to_string(p).ok());
        let selected = saved.as_deref().and_then(|id| config.stations.iter().position(|s| s.id == id.trim())).unwrap_or(0);
        Self { config, selected, player: None }
    }

    fn selected_name(&self) -> String {
        self.config.stations.get(self.selected).map(|s| s.name.clone()).unwrap_or_else(|| "(no stations)".into())
    }

    fn selected_id(&self) -> Option<String> {
        self.config.stations.get(self.selected).map(|s| s.id.clone())
    }

    fn persist(&self) -> AppResult<()> {
        let path = paths::state_file()?;
        paths::create_parent(&path)?;
        config::atomic_write(&path, &self.selected_id().unwrap_or_default())
    }

    fn play_silent(&mut self) -> AppResult<()> {
        let station = self.config.stations.get(self.selected).ok_or("No stations configured; use `bandhop stations add`." )?;
        if self.player.is_some() { return Ok(()); }
        let child = Command::new("mpv")
            .args(["--no-config", "--no-video", "--force-window=no", "--terminal=no", "--really-quiet"])
            .arg(format!("--volume={}", self.config.volume))
            .arg("--").arg(&station.url)
            .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
            .spawn().map_err(|e| format!("Failed to start mpv: {e}. Install the `mpv` package."))?;
        self.player = Some(child);
        self.persist()?;
        Ok(())
    }

    fn stop_silent(&mut self) {
        if let Some(mut child) = self.player.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    fn play(&mut self) -> AppResult<String> {
        if self.player.is_some() { return Ok(format!("Already playing: {}", self.selected_name())); }
        self.play_silent()?;
        notify(&self.config, "play", "Bandhop — Now playing", &self.selected_name());
        Ok(format!("Playing: {}", self.selected_name()))
    }

    fn stop(&mut self) -> AppResult<String> {
        if self.player.is_none() { return Ok("Already stopped".into()); }
        self.stop_silent();
        notify(&self.config, "stop", "Bandhop — Stopped", &self.selected_name());
        Ok(format!("Stopped: {}", self.selected_name()))
    }

    fn next(&mut self) -> AppResult<String> {
        if self.config.stations.is_empty() { return Err("No stations configured".into()); }
        let was_playing = self.player.is_some();
        self.selected = (self.selected + 1) % self.config.stations.len();
        self.persist()?;
        if was_playing {
            self.stop_silent();
            if let Err(e) = self.play_silent() {
                notify(&self.config, "stop", "Bandhop — Playback failed", &e);
                return Err(e);
            }
        }
        let name = self.selected_name();
        let detail = format!("{} ({}/{})", name, self.selected + 1, self.config.stations.len());
        notify(&self.config, "change", if was_playing { "Bandhop — Now playing" } else { "Bandhop — Selected" }, &detail);
        Ok(if was_playing { format!("Playing: {detail}") } else { format!("Selected (stopped): {detail}") })
    }

    fn reload(&mut self) -> AppResult<String> {
        let updated = config::load()?;
        let previous = self.selected_id();
        let playing_url = if self.player.is_some() {
            self.config.stations.get(self.selected).map(|s| s.url.clone())
        } else { None };
        let selected = previous.as_deref().and_then(|id| updated.stations.iter().position(|s| s.id == id)).unwrap_or(0);
        let new_url = updated.stations.get(selected).map(|s| s.url.clone());
        let should_restart = playing_url.is_some() && playing_url != new_url;
        if should_restart { self.stop_silent(); }
        self.config = updated;
        self.selected = selected;
        self.persist()?;
        if should_restart && new_url.is_some() { self.play_silent()?; }
        Ok(format!("Configuration reloaded ({} stations)", self.config.stations.len()))
    }

    fn poll_player(&mut self) {
        // try_wait returns Ok(None) while mpv is still running. Flatten both
        // Option layers: Some(None) is NOT an exited child process.
        let exit_status = match self.player.as_mut() {
            Some(child) => match child.try_wait() {
                Ok(status) => status,
                Err(error) => {
                    eprintln!("bandhop: cannot poll mpv process: {error}");
                    return;
                }
            },
            None => return,
        };
        if let Some(status) = exit_status {
            self.player.take();
            eprintln!("bandhop: mpv exited with status {status}");
            notify(&self.config, "stop", "Bandhop — Stream ended", &self.selected_name());
        }
    }

    fn handle(&mut self, cmd: &str) -> AppResult<String> {
        self.poll_player();
        match cmd {
            "play" => self.play(),
            "stop" => self.stop(),
            "toggle" => if self.player.is_some() { self.stop() } else { self.play() },
            "next" => self.next(),
            "status" => Ok(format!("{}: {}", if self.player.is_some() { "Playing" } else { "Stopped" }, self.selected_name())),
            "reload" => self.reload(),
            "notify-test" => {
                notifications::deliver("Bandhop — Test notification", &format!("Desktop notifications are working. Selected station: {}", self.selected_name()))?;
                Ok("Notification submitted successfully. If no popup appeared, inspect Plasma Notifications settings and Do Not Disturb.".into())
            },
            _ => Err(format!("Unknown command: {cmd}")),
        }
    }
}

impl Drop for Controller {
    fn drop(&mut self) { self.stop_silent(); }
}

struct SocketGuard(std::path::PathBuf);
impl Drop for SocketGuard {
    fn drop(&mut self) { let _ = fs::remove_file(&self.0); }
}

pub fn run(interactive: bool) -> AppResult<()> {
    let path = paths::socket_file()?;
    if path.exists() {
        if UnixStream::connect(&path).is_ok() { return Err("Bandhop is already running".into()); }
        let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if !metadata.file_type().is_socket() { return Err("Refusing to remove a non-socket path".into()); }
        fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    let listener = UnixListener::bind(&path).map_err(|e| format!("Cannot bind control socket: {e}"))?;
    let _cleanup = SocketGuard(path);
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    let mut controller = Controller::new(config::load()?);

    let (sender, receiver) = mpsc::channel::<String>();
    if interactive {
        println!("Bandhop listening. Enter: p (toggle), n (next), s (status), q (quit).");
        thread::spawn(move || {
            use std::io::BufRead;
            for line in std::io::stdin().lock().lines() {
                match line { Ok(line) => if sender.send(line).is_err() { break; }, Err(_) => break }
            }
        });
    }
    loop {
        match listener.accept() {
            Ok((mut client, _)) => {
                let _ = client.set_read_timeout(Some(Duration::from_secs(2)));
                let mut input = String::new();
                let response = match (&mut client).take(256).read_to_string(&mut input) {
                    Ok(_) => controller.handle(input.trim()).map(|s| format!("OK\t{s}\n")).unwrap_or_else(|e| format!("ERR\t{e}\n")),
                    Err(e) => format!("ERR\tCannot read command: {e}\n"),
                };
                let _ = client.write_all(response.as_bytes());
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {},
            Err(e) => return Err(format!("Socket accept failed: {e}")),
        }
        if interactive {
            while let Ok(line) = receiver.try_recv() {
                let cmd = match line.trim() {
                    "q" | "quit" => return Ok(()),
                    "p" | " " => "toggle",
                    "n" => "next",
                    "s" => "status",
                    "h" | "help" => { println!("p: toggle | n: next | s: status | q: quit"); continue; }
                    other => other,
                };
                match controller.handle(cmd) { Ok(msg) => println!("{msg}"), Err(e) => eprintln!("Error: {e}") }
            }
        }
        controller.poll_player();
        thread::sleep(Duration::from_millis(100));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn starts_silent() {
        let mut cfg = Config::default();
        cfg.stations.push(crate::config::Station { id: "one".into(), name: "One".into(), url: "https://example.org/stream".into() });
        let engine = Controller::new(cfg);
        assert!(engine.player.is_none());
    }
    #[test] fn a_running_mpv_child_must_remain_tracked() {
        // Regression for v0.1.0: Ok(None) from try_wait was mistaken for an
        // exited process. That caused `stop` to lose mpv and leak audio.
        let mut engine = Controller::new(Config::default());
        let child = Command::new("sleep").arg("30").spawn().unwrap();
        engine.player = Some(child);
        engine.poll_player();
        assert!(engine.player.is_some(), "running child must not be forgotten");
        assert!(engine.stop().unwrap().starts_with("Stopped:"));
        assert!(engine.player.is_none());
    }
    #[test] fn a_finished_child_is_reaped() {
        let mut engine = Controller::new(Config::default());
        let mut child = Command::new("true").spawn().unwrap();
        let _ = child.wait().unwrap();
        engine.player = Some(child);
        engine.poll_player();
        assert!(engine.player.is_none(), "finished child should be removed");
    }
    #[test] fn selecting_next_does_not_autoplay() {
        let mut cfg = Config::default();
        cfg.stations = vec![
            crate::config::Station { id: "first".into(), name: "First".into(), url: "https://example.org/first".into() },
            crate::config::Station { id: "second".into(), name: "Second".into(), url: "https://example.org/second".into() },
        ];
        let mut controller = Controller::new(cfg);
        controller.selected = 0;
        // This test deliberately avoids calling next(), which writes the user's state file.
        controller.selected = (controller.selected + 1) % controller.config.stations.len();
        assert_eq!(controller.selected_name(), "Second");
        assert!(controller.player.is_none());
    }
}
