# Build and verification status

Generated: 2026-10-25

## Verified in source-generation environment

- ZIP source layout and executable shell script flags
- Bash syntax (`bash -n`) for install, uninstall, smoke test and packaging scripts
- Parseability of `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, and example configuration as TOML
- GitHub Actions workflow YAML parseability
- Required project and license files present

## Not yet verified here

- **Compilation with `cargo build --locked`** (Rust toolchain unavailable in this environment)
- Rust unit tests (`cargo test --locked`)
- Integration tests (`./scripts/smoke-test.sh`)
- Actual playback over a verified live radio stream
- KDE Plasma 6 / Wayland F13 and F14 bindings via VIA/QMK
- Local `makepkg` build and install on CachyOS
- GitHub release and CachyOS distribution acceptance

## First validation on CachyOS

```bash
sudo pacman -S --needed mpv libnotify base-devel
cargo build --locked
cargo test --locked
./scripts/smoke-test.sh
./install.sh
bandhop keys setup
```

If building fails, do not tag or publish the package until errors are fixed. This is a complete source repository intended for end-to-end validation, not a prebuilt or formally verified release.

## v0.1.2 hotfix verification

- Root cause confirmed from user-provided CachyOS logs and source inspection: `Ok(None)` from `Child::try_wait` was interpreted as an exit.
- Added Rust unit regressions for running and exited child processes.
- Strengthened mock-mpv process-lifecycle smoke test and upgrade installer.
- **The Rust compiler is not installed in this artifact-creation environment.** Rust compilation, unit execution, smoke execution, and real CachyOS playback must be completed by the user before release.

## v0.1.2 pending desktop verification

- The v0.1.1 audio controller, 10 Rust unit tests, smoke tests, stress tests, and physical key bindings were verified by the user on CachyOS.
- v0.1.2 changes desktop notifications only; source-generation checks here do not substitute for Cargo compilation or real KDE notification testing.
- On CachyOS, verify `bandhop notify test`, `bandhop play`, `bandhop next`, and `bandhop stop` after installing v0.1.2. Review `journalctl --user -u bandhop.service` if notifications do not appear.
- The GitHub repository has not yet been created. Do not claim this package is already available in GitHub Releases or AUR.
