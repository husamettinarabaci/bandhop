//! XDG-compliant paths. No files are placed in shared /tmp directories.
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

pub type AppResult<T> = Result<T, String>;

fn home() -> AppResult<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME is not set".to_string())
}

fn xdg_dir(env_name: &str, fallback: &str) -> AppResult<PathBuf> {
    match env::var_os(env_name) {
        Some(path) if !path.is_empty() => {
            let path = PathBuf::from(path);
            if path.is_absolute() { Ok(path) } else { Err(format!("{env_name} must be absolute")) }
        }
        _ => Ok(home()?.join(fallback)),
    }
}

pub fn config_file() -> AppResult<PathBuf> {
    Ok(xdg_dir("XDG_CONFIG_HOME", ".config")?.join("bandhop/config.toml"))
}

pub fn state_file() -> AppResult<PathBuf> {
    Ok(xdg_dir("XDG_STATE_HOME", ".local/state")?.join("bandhop/state"))
}

pub fn service_file() -> AppResult<PathBuf> {
    Ok(xdg_dir("XDG_CONFIG_HOME", ".config")?.join("systemd/user/bandhop.service"))
}

pub fn runtime_dir() -> AppResult<PathBuf> {
    let base = match env::var_os("XDG_RUNTIME_DIR") {
        Some(path) if !path.is_empty() => {
            let p = PathBuf::from(path);
            if !p.is_absolute() { return Err("XDG_RUNTIME_DIR must be absolute".into()); }
            p
        }
        _ => xdg_dir("XDG_STATE_HOME", ".local/state")?.join("bandhop"),
    };
    let path = base.join("bandhop-runtime");
    fs::create_dir_all(&path).map_err(|e| format!("Cannot create {}: {e}", path.display()))?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700))
        .map_err(|e| format!("Cannot protect runtime directory: {e}"))?;
    Ok(path)
}

pub fn socket_file() -> AppResult<PathBuf> { Ok(runtime_dir()?.join("control.sock")) }

pub fn create_parent(path: &std::path::Path) -> AppResult<()> {
    let parent = path.parent().ok_or("Path has no parent")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())
}
