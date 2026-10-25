# Contributing to Bandhop

Contributions, bug reports and documentation improvements are welcome once the public repository is online. Use English for issues, code, commit messages and documentation.

## Development environment

- Linux, Rust stable >= 1.74, Cargo, mpv; libnotify/notify-send for desktop notifications.
- `cargo test --locked`
- `cargo fmt --all`
- `cargo clippy --locked --all-targets`
- `./scripts/smoke-test.sh`

Follow conventional commit prefixes where useful (`feat:`, `fix:`, `docs:`, `test:`). Keep patches focused, preserve backward compatibility for the CLI/config file, and include tests for state transitions. Do not commit credentials, real personal configuration, or binaries.

## Pull requests

Explain the use case, include tests, document changes and update CHANGELOG.md. Test on a non-production user account. Any proposed changes to KDE keybinding automation must respect Wayland's compositor security model.

By contributing you agree that your contributions may be distributed under MIT OR Apache-2.0.
