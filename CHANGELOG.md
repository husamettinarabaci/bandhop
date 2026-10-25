## [0.1.2] - 2026-10-25

### Fixed
- Use normal notification urgency instead of low priority; KDE Plasma may suppress low-priority popups.
- Report `notify-send` execution failures in the user service journal instead of discarding errors.
- Fall back to the per-user D-Bus socket address if the session-bus environment variable is absent.

### Added
- `bandhop notify test` to check notification delivery through the active Bandhop daemon.
- Notification troubleshooting guide and mock integration assertions.
- GitHub publication checklist (repository must be created before publishing).

# Changelog

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

- Validate the v0.1.2 fix on CachyOS before publishing the first public GitHub release.

## [0.1.2] - Playback process hotfix (2026-10-25)

### Fixed
- Correctly interpret `Child::try_wait()` returning `Ok(None)` as a running `mpv` process.
- Prevent orphaned simultaneous radio streams caused by `stop`, `toggle`, and `next` losing player ownership.
- Reinstall the binary with an atomic rename and restart an active user daemon after upgrading.

### Tests
- Add unit regressions for running and finished child process polling.
- Extend mock-mpv smoke tests to enforce one-player-at-a-time, clean channel handoff, and complete stop.


## [0.1.0] - Pre-release source snapshot (2026-10-25)

### Added
- Rust CLI for ordered station management, playback control, and config validation.
- Foreground and optional systemd user-service modes.
- F13/F14 KDE global shortcut setup guide.
- Freedesktop notifications through notify-send.
- Local Arch/CachyOS packaging workflow and CI templates.

### Known limitations
- No Radio Browser API client, Shortwave library integration, or stream reconnection.
- No verified compiled binary included in source snapshot.
