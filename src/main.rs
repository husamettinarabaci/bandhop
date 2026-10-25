//! Bandhop: zero-dependency Rust internet radio control for Linux.
mod config;
mod daemon;
mod notifications;
mod paths;
mod service;

use config::Station;
use paths::AppResult;
use std::env;
use std::fs;
use std::io::IsTerminal;
use std::process::{Command, ExitCode};

const HELP: &str = r#"Bandhop 0.1.2 — keyboard-driven internet radio for Linux

USAGE
  bandhop init                   Create an empty config in the XDG directory
  bandhop listen                 Foreground session; p=toggle, n=next, q=quit
  bandhop daemon                 Run the optional user daemon (no autoplay)
  bandhop play|stop|toggle|next  Control an active session
  bandhop status                 Show the currently selected station/state
  bandhop notify test            Send a test desktop notification

STATIONS
  bandhop stations list
  bandhop stations add "NAME" "https://stream.example.org/live"
  bandhop stations remove ID
  bandhop stations move ID POSITION    (1-based position)

CONFIG
  bandhop config path|show|validate|edit|reload

OPTIONAL SYSTEMD SERVICE
  bandhop service install|enable|disable|status|uninstall

KDE GLOBAL KEYS
  bandhop keys show|check|setup
  Default: F13 = toggle, F14 = next. Keys are registered in Plasma settings.
  The CLI does not silently rewrite KDE's running shortcut database.

All station changes are saved in ~/.config/bandhop/config.toml (XDG-aware).
Playback does NOT start automatically when the daemon starts.
"#;

fn usage_error(message: &str) -> String { format!("{message}\n\nRun `bandhop --help` for usage.") }

fn run(args: &[String]) -> AppResult<()> {
    if args.is_empty() || matches!(args[0].as_str(), "--help" | "-h" | "help") {
        println!("{HELP}"); return Ok(());
    }
    if matches!(args[0].as_str(), "--version" | "-V") {
        println!("bandhop {}", env!("CARGO_PKG_VERSION")); return Ok(());
    }
    match args[0].as_str() {
        "init" if args.len() == 1 => {
            let _ = config::load()?;
            println!("Configuration: {}", paths::config_file()?.display());
            println!("Add a stream with: bandhop stations add \"Station Name\" \"https://...\"");
        }
        "listen" if args.len() == 1 => {
            if !std::io::stdin().is_terminal() { return Err("listen requires an interactive terminal".into()); }
            daemon::run(true)?;
        }
        "daemon" if args.len() == 1 => daemon::run(false)?,
        "play" | "stop" | "toggle" | "next" | "status" if args.len() == 1 => {
            println!("{}", daemon::send(&args[0])?);
        }
        "stations" => stations(&args[1..])?,
        "config" => config_command(&args[1..])?,
        "service" => service_command(&args[1..])?,
        "keys" => keys_command(&args[1..])?,
        "notify" if args.len() == 2 && args[1] == "test" => { println!("{}", daemon::send("notify-test")?); },
        _ => return Err(usage_error("Unknown command or invalid number of arguments")),
    }
    Ok(())
}

fn stations(args: &[String]) -> AppResult<()> {
    if args.len() == 1 && args[0] == "list" {
        let cfg = config::load()?;
        let state = paths::state_file().ok().and_then(|p| fs::read_to_string(p).ok());
        if cfg.stations.is_empty() { println!("No stations. Add one using `bandhop stations add`." ); }
        for (index, s) in cfg.stations.iter().enumerate() {
            let selected = if state.as_deref().is_some_and(|id| id.trim() == s.id) { "*" } else { " " };
            println!("{selected} {:>2}. {:<24} {:<18} {}", index + 1, s.id, s.name, s.url);
        }
        return Ok(());
    }
    if args.len() == 3 && args[0] == "add" {
        let mut cfg = config::load()?;
        let name = args[1].trim();
        if name.is_empty() { return Err("Station name is empty".into()); }
        let base = config::slug(name);
        let mut id = base.clone();
        let mut suffix = 2;
        while cfg.stations.iter().any(|s| s.id == id) {
            id = format!("{base}-{suffix}"); suffix += 1;
        }
        cfg.stations.push(Station { id: id.clone(), name: name.into(), url: args[2].clone() });
        config::save(&cfg)?;
        daemon::try_reload();
        println!("Added station: {id}");
        return Ok(());
    }
    if args.len() == 2 && args[0] == "remove" {
        let mut cfg = config::load()?;
        let before = cfg.stations.len();
        cfg.stations.retain(|s| s.id != args[1]);
        if cfg.stations.len() == before { return Err(format!("No such station: {}", args[1])); }
        config::save(&cfg)?;
        daemon::try_reload();
        println!("Removed: {}", args[1]);
        return Ok(());
    }
    if args.len() == 3 && args[0] == "move" {
        let mut cfg = config::load()?;
        let from = cfg.stations.iter().position(|s| s.id == args[1])
            .ok_or_else(|| format!("No such station: {}", args[1]))?;
        let position = args[2].parse::<usize>().map_err(|_| "POSITION must be a positive number".to_string())?;
        if position < 1 || position > cfg.stations.len() { return Err("POSITION is outside station list".into()); }
        let station = cfg.stations.remove(from);
        cfg.stations.insert(position - 1, station);
        config::save(&cfg)?;
        daemon::try_reload();
        println!("Moved {} to position {}", args[1], position);
        return Ok(());
    }
    Err(usage_error("Usage: bandhop stations list|add NAME URL|remove ID|move ID POSITION"))
}

fn config_command(args: &[String]) -> AppResult<()> {
    if args.len() != 1 { return Err(usage_error("Usage: bandhop config path|show|edit|validate|reload")); }
    match args[0].as_str() {
        "path" => println!("{}", paths::config_file()?.display()),
        "show" => { let _ = config::load()?; print!("{}", fs::read_to_string(paths::config_file()?).map_err(|e| e.to_string())?); }
        "validate" => { let cfg = config::load()?; println!("Configuration valid: {} stations", cfg.stations.len()); }
        "reload" => println!("{}", daemon::send("reload")?),
        "edit" => {
            let _ = config::load()?;
            let editor = env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
            // EDITOR is an executable name, NOT an untrusted shell command.
            let status = Command::new(&editor).arg(paths::config_file()?)
                .status().map_err(|e| format!("Cannot start editor '{editor}': {e}. Set EDITOR to an executable."))?;
            if !status.success() { return Err(format!("Editor exited with {status}")); }
            let cfg = config::load()?;
            println!("Configuration valid: {} stations", cfg.stations.len());
            daemon::try_reload();
        }
        _ => return Err(usage_error("Usage: bandhop config path|show|edit|validate|reload")),
    }
    Ok(())
}

fn service_command(args: &[String]) -> AppResult<()> {
    if args.len() != 1 { return Err(usage_error("Usage: bandhop service install|enable|disable|status|uninstall")); }
    match args[0].as_str() {
        "install" => service::install(),
        "enable" => service::enable(),
        "disable" => service::disable(),
        "status" => service::status(),
        "uninstall" => service::uninstall(),
        _ => Err(usage_error("Usage: bandhop service install|enable|disable|status|uninstall")),
    }
}

fn keys_command(args: &[String]) -> AppResult<()> {
    if args.len() != 1 { return Err(usage_error("Usage: bandhop keys show|check|setup")); }
    match args[0].as_str() {
        "show" => {
            println!("F13: bandhop toggle\nF14: bandhop next");
            println!("Physical keyboard: assign KC_F13 and KC_F14 in VIA/QMK.");
        }
        "check" => {
            let path = paths::config_file()?.parent().and_then(|p| p.parent()).map(|p| p.join("kglobalshortcutsrc"));
            if let Some(path) = path {
                let text = fs::read_to_string(&path).unwrap_or_default();
                let matches: Vec<&str> = text.lines().filter(|line| !line.trim_start().starts_with('#') &&
                    (line.to_ascii_lowercase().contains("f13") || line.to_ascii_lowercase().contains("f14"))).collect();
                if matches.is_empty() { println!("No F13/F14 matches in {}", path.display()); }
                else { println!("Potential shortcut assignments:"); for line in matches { println!("  {line}"); } }
            }
            println!("This is not definitive. KDE Settings must confirm that both keys are free.");
        }
        "setup" => {
            let binary = env::current_exe().map_err(|e| e.to_string())?;
            println!("1. In VIA/QMK, assign KC_F13 and KC_F14 to two physical keys.");
            println!("2. KDE Plasma: System Settings > Keyboard > Shortcuts > Add New > Command or Script.");
            println!("3. Assign F13 to: {} toggle", binary.display());
            println!("4. Assign F14 to: {} next", binary.display());
            println!("5. Check for conflicts in KDE; save and press each key to test.");
            println!("Wayland global shortcuts are owned by the compositor. Bandhop does not capture keystrokes.");
        }
        _ => return Err(usage_error("Usage: bandhop keys show|check|setup")),
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => { eprintln!("bandhop: {e}"); ExitCode::FAILURE }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn usage_contains_keybindings() { assert!(HELP.contains("F13")); assert!(HELP.contains("F14")); }
    #[test] fn config_defaults_to_silent() { let _cfg = config::Config::default(); /* engine owns play state */ }
}
