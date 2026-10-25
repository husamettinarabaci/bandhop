# Bandhop

**A keyboard-driven, terminal-first internet radio player for Linux, written in Rust.**

[![CI](https://github.com/husamettinarabaci/bandhop/actions/workflows/ci.yml/badge.svg)](https://github.com/husamettinarabaci/bandhop/actions/workflows/ci.yml)

> **Pre-release source package (`v0.1.2` notification fix).** The GitHub repository is prepared for `husamettinarabaci/bandhop` but has not yet been created. The v0.1.1 audio and keyboard controls were tested on CachyOS; v0.1.2 notifications still require desktop verification before a public release.

Bandhop provides two operating modes from the same binary and station library. Launch it interactively from a terminal, or optionally enable a **systemd user service** so it is ready when you log in. The service **never autoplays** at login, on initial start, or after restart. Play only starts on an explicit user command.

No GUI. No account. No telemetry. No mandatory daemon. No Rust crates or internet access are required to build the source after a Rust toolchain is installed; streaming still requires internet access.

## Features

- Play, stop, toggle, and circular next-station selection
- Manage an ordered station library from the CLI, stored in a readable config file
- Optional foreground session (`bandhop listen`)
- Optional `systemd --user` service (`bandhop service enable`)
- Normal-urgency station-change, play, and stop notifications via `notify-send` on compatible desktops; CLI diagnostics with `bandhop notify test`
- KDE Plasma 6 / Wayland shortcut instructions for VIA/QMK **F13** and **F14**
- One local Unix socket for safe per-user control; the daemon does not listen on the network
- Remembers the last selected station but **not** a playing state
- XDG Base Directory compliant locations

## Requirements

Linux, Rust 1.74+ and Cargo (to compile), [`mpv`](https://mpv.io/) (runtime), and optionally `notify-send` from `libnotify` (notifications). `systemd --user` is required **only** for background-service mode. KDE is not required to run the CLI.

### Arch Linux / CachyOS

```sh
sudo pacman -S --needed mpv libnotify
# If using rustup instead of the rust package: rustup default stable
```

Verify `cargo --version` and `mpv --version`. On systems with `rustup`, use its stable toolchain rather than installing both competing Rust toolchain managers.

## Quick start

```sh
unzip bandhop-0.1.2-source.zip
cd bandhop-0.1.2
./install.sh

bandhop init
bandhop stations add "My English News Radio" "https://YOUR-ACTUAL-STREAM-URL"
bandhop stations list
bandhop listen
```

Inside `listen`: type `p` + Enter to toggle, `n` + Enter to advance, `s` + Enter for status, and `q` + Enter to quit. Note that `https://YOUR-ACTUAL-STREAM-URL` is a placeholder, **not a working radio stream**. Obtain the direct stream URL from the broadcaster.

### Background service, still silent at login

```sh
bandhop service enable        # installs and enables a user service
bandhop status                # Stopped, ready for a command
bandhop toggle                # starts selected station
bandhop next                  # advances, wraps around, KDE notification
bandhop toggle                # stops playback, service stays alive
bandhop service disable       # stops & disables the service
```

**Never run the service with sudo.** It belongs to your user session. The service starts when the user manager starts (usually at login), not necessarily before you log in. Lingering is not needed.

## Upgrading from v0.1.0 (important)

The v0.1.0 playback controller mistakenly interpreted a still-running mpv
process as exited, causing orphaned audio and multiple simultaneous streams.
**Stop the old user daemon before installing the hotfix:**

```sh
systemctl --user stop bandhop.service
cargo test --locked
./scripts/smoke-test.sh
./install.sh --enable-service
bandhop status  # Stopped: ... (no audio yet)
```

Existing stations and selected-station state are preserved. The installer
replaces the executable atomically and restarts a running user service when
applicable. Never use `sudo ./install.sh`.

## Default physical shortcuts

| Default key | Command | Behavior |
| --- | --- | --- |
| **F13** | `bandhop toggle` | Start/stop the currently selected station |
| **F14** | `bandhop next` | Select the next station, circularly; retain stopped state if stopped |

Use VIA/QMK to assign `KC_F13` and `KC_F14` to two keyboard keys, then configure global shortcuts **in KDE Plasma**. First run:

```sh
bandhop keys check
bandhop keys setup
```

In **System Settings → Keyboard → Shortcuts → Add New → Command or Script**, bind the commands shown by `bandhop keys setup` to F13 and F14. Inspect conflict warnings before saving. Under Wayland, **KDE owns global shortcut capture**; Bandhop deliberately does not try to listen to all keyboard events or directly edit KDE's live shortcut database. `keys check` is a preliminary text check, not a proof of no conflicts. More details: [docs/SHORTCUTS.md](docs/SHORTCUTS.md).

## CLI reference

```text
bandhop --help
bandhop init
bandhop listen
bandhop daemon
bandhop play
bandhop stop
bandhop toggle
bandhop next
bandhop status
bandhop stations list
bandhop stations add "Station Name" "https://host/live.mp3"
bandhop stations remove station-name
bandhop stations move station-name 1
bandhop config path
bandhop config show
bandhop config edit
bandhop config validate
bandhop config reload
bandhop service install
bandhop service enable
bandhop service disable
bandhop service status
bandhop service uninstall
bandhop keys show
bandhop keys check
bandhop keys setup
```

Commands `play|stop|toggle|next|status` talk to an active background or foreground session. They do not secretly spawn a persistent process. `listen` is the daemonless interactive mode. The `stations` commands update the config on disk and request a live reload if a session exists.

## Configuration

`$XDG_CONFIG_HOME/bandhop/config.toml` (default `~/.config/bandhop/config.toml`):

```toml
[playback]
volume = 60

[notifications]
enabled = true
on_play = true
on_stop = true
on_change = true

[[stations]]
id = "american-news"
name = "American News"
url = "https://YOUR-ACTUAL-STREAM-URL"
```

The application deliberately supports a **documented, strict subset of TOML**, not the entire TOML specification. One `[playback]`, one `[notifications]`, and repeated `[[stations]]` sections are supported; use standalone quoted strings and no inline comments. Unknown keys, duplicate IDs, malformed URLs and unrecognized syntax are errors. Refer to [docs/CONFIGURATION.md](docs/CONFIGURATION.md).

The selected station ID is stored separately under `$XDG_STATE_HOME/bandhop/state` (default `~/.local/state/bandhop/state`). The Unix control socket is under `$XDG_RUNTIME_DIR/bandhop-runtime/control.sock` (or a private fallback when the variable is unavailable).

When a station URL is removed or changed, a config reload may stop or replace that playing station. Pure ordering and unrelated settings reload without interrupting it.

## Install and uninstall

```sh
./install.sh                 # build and install into ~/.local/bin; daemon OFF
./install.sh --enable-service
./uninstall.sh               # remove user service and ~/.local/bin binary
./uninstall.sh --purge       # also delete Bandhop configuration and state
```

The install script never uses sudo. Ensure `~/.local/bin` is on your PATH, or reopen your shell. If you install through `pacman`, remove it using `pacman`, not `uninstall.sh`.

### Arch / CachyOS package development

`./scripts/build-arch-package.sh` builds a local source tarball with SHA-256 verification and calls `makepkg`. It requires Arch/CachyOS and `base-devel`. Distribution/publishing templates and policies are in [docs/PACKAGING.md](docs/PACKAGING.md). No package has been submitted to CachyOS yet.

## Testing

```sh
cargo test --locked
cargo build --locked --release
./scripts/smoke-test.sh
```

`smoke-test.sh` isolates XDG folders and exercises local configuration/CLI behavior; it does not require any real radio endpoint. Audio and shortcut integration must be tested manually on a KDE/Wayland machine.

## Security and privacy

Streams connect to URLs explicitly supplied by the user, and mpv is invoked using argument arrays (not a shell). The local control socket lives in a private per-user directory. No remote command port is opened; no login, analytics or telemetry are used. Treat stream URLs as untrusted and keep mpv updated. A station URL may disclose your public IP address to the broadcaster.

## Known v0.1 limitations

- No station directory discovery or direct import of Shortwave favorites
- No fully automatic KDE shortcut registration or remapping; setup is performed in KDE
- Stream metadata, reconnection, and MPRIS are not in v0.1
- Config file parser supports a constrained TOML subset
- Runtime audio requires the external `mpv` binary
- The source bundle is **not** a verified release binary; run Rust tests, playback checks and packaging checks before publishing

## Contributing / license

Contributions welcome after the repository is published. See [CONTRIBUTING.md](CONTRIBUTING.md) and [SECURITY.md](SECURITY.md). Dual-licensed under **MIT OR Apache-2.0**; see the included license files.

## Desktop notification troubleshooting (v0.1.2)

After updating, test the session notification server through the *daemon*:

```sh
bandhop notify test
```

Successful submission is not proof that Plasma displayed the popup: check **System Settings → Notifications** for Do Not Disturb, blocked applications, and pop-up filters. Bandhop sends **normal** priority notifications in this version, because KDE may suppress low-priority popups. See [docs/NOTIFICATIONS.md](docs/NOTIFICATIONS.md). If `notify-send` fails inside the service, the reason is written to the user journal (`journalctl --user -u bandhop.service`). The app still plays audio when notification delivery fails.
