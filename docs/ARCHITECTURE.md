# Architecture

## Objectives

Deliver an uncomplicated, keyboard-driven Linux radio controller; optional long-running service; no autoplay; desktop-agnostic CLI; KDE/Wayland shortcuts through native global shortcut facilities.

## Components

```text
VIA/QMK physical keys (KC_F13 / KC_F14)
                |
KDE Plasma global shortcuts (manual binding)
                |
          bandhop CLI
                |
  per-user Unix domain socket (private XDG runtime directory)
                |
         bandhop daemon/listen
           |          |
         mpv       notify-send
           |          |
       Internet    Freedesktop
         radio     notifications
```

## Single executable, two modes

`bandhop listen` starts a local control server and provides an interactive stdin loop. `bandhop daemon` starts the same control server with no interactive stdin; systemd is optional. Both modes use the same config file and station state. Starting a second instance is rejected while the first is alive. Stale socket files are removed only if they are Unix sockets and the previous listener is not accepting connections.

## Startup invariant

The daemon creates `Controller { player: None }`. This is independent of `state`, which holds only the selected station ID. Restarting the daemon must never resume playback. `play` or `toggle` is the only action that starts `mpv`; `next` while stopped changes station selection without starting audio.

## Playback / control model

To minimize dependencies, v0.1 starts mpv as a child only when playback is requested and terminates it when changing or stopping. On a user service, systemd `KillMode=control-group` additionally ensures child processes are cleaned up when the unit stops. The control connection is local, line-based and bounded; requests are `play`, `stop`, `toggle`, `next`, `status`, and `reload`. Stream URLs are passed as distinct process arguments, never interpolated into a shell command.

There is no HTTP control endpoint. There is no authentication for peer processes running as the same user. IPC needs no network connectivity.

## Config ownership

`~/.config/bandhop/config.toml` is owned by the CLI and editable by hand. CLI changes are written through a temporary file and rename. The daemon reloads on request. Selected station ID is persisted to `$XDG_STATE_HOME` separately. Any station can be reordered and `next` wraps from the final entry to the first.

## Notifications

Uses freedesktop-compatible `notify-send`. A fixed replacement ID updates the active notification rather than stacking multiple toasts. A missing notification utility does not prevent playback.

## Platform / future direction

The core is Linux-specific due to Unix-domain sockets; the service manager integration is systemd-specific but optional. Keybinding integration is compositor-owned on Wayland, so v0.1 only provides explicit commands and setup guidance. Potential future improvements: fully standard TOML parser, MPRIS service, `mpv` IPC and reconnection, richer stream metadata, dynamic station search, enhanced KDE shortcut registration.
