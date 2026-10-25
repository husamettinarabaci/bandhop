//! Small, intentionally strict TOML *subset* for an auditable zero-dependency build.
//! Supports only the documented fields/sections. Rejects unknown keys rather than
//! silently ignoring an accidentally misspelled setting.
use crate::paths::{self, AppResult};
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Station {
    pub id: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct Notifications {
    pub enabled: bool,
    pub on_play: bool,
    pub on_stop: bool,
    pub on_change: bool,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub volume: u8,
    pub notifications: Notifications,
    pub stations: Vec<Station>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            volume: 60,
            notifications: Notifications {
                enabled: true, on_play: true, on_stop: true, on_change: true,
            },
            stations: Vec::new(),
        }
    }
}

fn unquote(s: &str) -> AppResult<String> {
    let s = s.trim();
    if !(s.starts_with('"') && s.ends_with('"') && s.len() >= 2) {
        return Err("Expected a double-quoted string".into());
    }
    let mut output = String::new();
    let mut escaped = false;
    for c in s[1..s.len() - 1].chars() {
        if escaped {
            match c {
                '"' => output.push('"'),
                '\\' => output.push('\\'),
                'n' => output.push('\n'),
                'r' => output.push('\r'),
                't' => output.push('\t'),
                _ => return Err(format!("Unsupported escape sequence: \\{c}")),
            }
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' {
            return Err("Unescaped quote in string".into());
        } else {
            output.push(c);
        }
    }
    if escaped { return Err("Incomplete escape sequence".into()); }
    Ok(output)
}

fn quoted(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

fn parse_bool(s: &str) -> AppResult<bool> {
    match s.trim() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err("Expected true or false".into()),
    }
}

pub fn parse(text: &str) -> AppResult<Config> {
    let mut cfg = Config::default();
    let mut section = "";
    let mut seen = HashSet::new();
    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        if line == "[playback]" || line == "[notifications]" {
            section = if line == "[playback]" { "playback" } else { "notifications" };
            continue;
        }
        if line == "[[stations]]" {
            section = "station";
            cfg.stations.push(Station { id: String::new(), name: String::new(), url: String::new() });
            seen.clear();
            continue;
        }
        let result: AppResult<()> = (|| {
            let (key, val) = line.split_once('=').ok_or("Expected key = value")?;
            let key = key.trim();
            if !seen.insert((section.to_string(), key.to_string())) {
                return Err(format!("Duplicate field: {key}"));
            }
            let val = val.trim();
            match (section, key) {
                ("playback", "volume") => {
                    cfg.volume = val.parse::<u8>().map_err(|_| "volume must be an integer 0..100")?;
                }
                ("notifications", "enabled") => cfg.notifications.enabled = parse_bool(val)?,
                ("notifications", "on_play") => cfg.notifications.on_play = parse_bool(val)?,
                ("notifications", "on_stop") => cfg.notifications.on_stop = parse_bool(val)?,
                ("notifications", "on_change") => cfg.notifications.on_change = parse_bool(val)?,
                ("station", "id") => cfg.stations.last_mut().unwrap().id = unquote(val)?,
                ("station", "name") => cfg.stations.last_mut().unwrap().name = unquote(val)?,
                ("station", "url") => cfg.stations.last_mut().unwrap().url = unquote(val)?,
                _ => return Err(format!("Unknown field or section: {section}.{key}")),
            }
            Ok(())
        })();
        result.map_err(|e| format!("Line {}: {e}", i + 1))?;
    }
    validate(&cfg)?;
    Ok(cfg)
}

pub fn validate(cfg: &Config) -> AppResult<()> {
    if cfg.volume > 100 { return Err("volume must be between 0 and 100".into()); }
    let mut ids = HashSet::new();
    for s in &cfg.stations {
        if s.id.is_empty() || !s.id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
            return Err(format!("Invalid station id {:?} (use a-z, 0-9, -)", s.id));
        }
        if !ids.insert(&s.id) { return Err(format!("Duplicate station id: {}", s.id)); }
        if s.name.trim().is_empty() || s.name.chars().any(|c| c == '\n' || c == '\r') {
            return Err(format!("Invalid station name for {}", s.id));
        }
        let host = s.url.strip_prefix("https://").or_else(|| s.url.strip_prefix("http://"))
            .and_then(|rest| rest.split(|c| c == '/' || c == '?' || c == '#').next()).unwrap_or("");
        if host.is_empty() || host.starts_with(':') || s.url.chars().any(|c| c.is_whitespace() || c.is_control()) {
            return Err(format!("Invalid HTTP(S) stream URL for {}", s.id));
        }
    }
    Ok(())
}

pub fn render(cfg: &Config) -> String {
    let mut out = String::from("# Bandhop configuration — a documented TOML subset.\n# Edit or use `bandhop stations ...`, then `bandhop config reload`.\n\n");
    out.push_str(&format!("[playback]\nvolume = {}\n\n", cfg.volume));
    out.push_str(&format!("[notifications]\nenabled = {}\non_play = {}\non_stop = {}\non_change = {}\n",
        cfg.notifications.enabled, cfg.notifications.on_play, cfg.notifications.on_stop, cfg.notifications.on_change));
    for s in &cfg.stations {
        out.push_str(&format!("\n[[stations]]\nid = {}\nname = {}\nurl = {}\n",
            quoted(&s.id), quoted(&s.name), quoted(&s.url)));
    }
    out
}

pub fn load() -> AppResult<Config> {
    let path = paths::config_file()?;
    if !path.exists() { save(&Config::default())?; }
    let txt = fs::read_to_string(&path).map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
    parse(&txt)
}

pub fn save(cfg: &Config) -> AppResult<()> {
    validate(cfg)?;
    let path = paths::config_file()?;
    paths::create_parent(&path)?;
    atomic_write(&path, &render(cfg))
}

pub fn atomic_write(path: &Path, text: &str) -> AppResult<()> {
    use std::os::unix::fs::OpenOptionsExt;
    let tmp = path.with_extension(format!("{}.tmp", std::process::id()));
    let result: AppResult<()> = (|| {
        let mut file = fs::OpenOptions::new().create_new(true).write(true).mode(0o600)
            .open(&tmp).map_err(|e| format!("Cannot create {}: {e}", tmp.display()))?;
        file.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        fs::rename(&tmp, path).map_err(|e| format!("Cannot save {}: {e}", path.display()))?;
        Ok(())
    })();
    if result.is_err() { let _ = fs::remove_file(&tmp); }
    result
}

pub fn slug(name: &str) -> String {
    let mut s = String::new();
    let mut dash = false;
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            s.push(c.to_ascii_lowercase()); dash = false;
        } else if !dash && !s.is_empty() {
            s.push('-'); dash = true;
        }
    }
    let s = s.trim_end_matches('-').to_string();
    if s.is_empty() { "station".into() } else { s }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn round_trip() {
        let mut c = Config::default();
        c.stations.push(Station { id: "npr".into(), name: "NPR \\\"News\\\"".into(), url: "https://example.com/a?b=1".into() });
        assert_eq!(parse(&render(&c)).unwrap().stations, c.stations);
    }
    #[test] fn reject_duplicate() {
        assert!(parse("[[stations]]\nid = \"one\"\nname = \"A\"\nurl = \"https://example.com\"\n[[stations]]\nid = \"one\"\nname = \"B\"\nurl = \"https://example.com\"").is_err());
    }
    #[test] fn reject_shell_like_url() {
        let mut c = Config::default();
        c.stations.push(Station { id: "one".into(), name: "One".into(), url: "https://host/evil\nfile".into() });
        assert!(validate(&c).is_err());
    }
    #[test] fn slug_basic() { assert_eq!(slug("NPR News / USA"), "npr-news-usa"); }
}
