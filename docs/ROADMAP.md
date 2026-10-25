# Roadmap

## v0.1.0 — core prototype

- Rust CLI: station CRUD/order, playback control and config validation
- Optional user daemon, foreground session, KDE notification support
- Local installer, local Arch package toolchain, docs and tests

## v0.1.1 — playback-process hotfix

- Correct handling of running mpv processes and one-player-at-a-time regression tests
- Safe in-place upgrades and user daemon restart

## v0.2.0 — practical station discovery

- Radio Browser station search from the terminal
- Direct station add by station UUID
- Stream URL health checks and reconnect policies
- Easier config key mapping inspection

## v0.3.0 — deeper desktop integration

- MPRIS control and metadata
- More portable shortcut setup across Wayland compositors
- User-configurable service/notification behavior
- Production CI enforcement and distro package distribution

No timeline or API compatibility is guaranteed in pre-release versions.
