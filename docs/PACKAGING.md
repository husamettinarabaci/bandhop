# Arch Linux / CachyOS packaging and publication plan

This source ZIP is ready for **local package preparation**, not yet published in the AUR or any distro repository. Do not represent the release templates as approved distro packages.

## Local PKGBUILD package

On Arch/CachyOS:

```sh
sudo pacman -S --needed base-devel mpv
# Use existing rustup + stable toolchain if already installed (do not install both rust and rustup).
./scripts/build-arch-package.sh
```

This produces a source tarball, computes its actual SHA-256 hash, renders a local `PKGBUILD` with that hash, and invokes `makepkg -s`. The resulting `bandhop-*.pkg.tar.zst` can be installed locally via `sudo pacman -U ...`. Inspect the PKGBUILD before doing so. Packages include `/usr/bin/bandhop`, documentation, man page, licenses, and a dormant systemd **user** unit. Package installation must **not** enable or start the service automatically.

## After publishing the GitHub repository

1. Choose and verify the repository and crate names. Replace `OWNER` in Cargo.toml and README.
2. Run `cargo fmt --all`, `cargo test --locked`, `cargo clippy --all-targets`, and `./scripts/smoke-test.sh`; test live streams on CachyOS/KDE/Wayland.
3. Set formatting CI to blocking and verify all jobs succeed.
4. Push source to a public GitHub repository, tag a signed/verified `v0.1.2` release if possible.
5. Use `packaging/arch/PKGBUILD.release.in`, substituting the actual owner, version, and SHA-256 checksum for the GitHub release source archive.
6. Build and test the release PKGBUILD with `makepkg -s` and install in a clean environment. Regenerate `.SRCINFO` using `makepkg --printsrcinfo > .SRCINFO` when preparing an AUR package.
7. Submit to the AUR (AUR maintainer credentials and packaging responsibility required); do not claim AUR acceptance before it exists.
8. Once mature and available through AUR, submit a **Package Request** to CachyOS's distribution tracker. Acceptance into CachyOS official repositories is up to its maintainers.

Official CachyOS guidance: https://wiki.cachyos.org/cachyos_basic/faq/ (Submitting Package Requests to CachyOS). Request tracker: https://github.com/CachyOS/distribution/issues

Arch Rust packaging guidance: https://wiki.archlinux.org/title/Rust_package_guidelines

## Package contents

- `packaging/arch/PKGBUILD.local.in` — local-source, checksum-verified template
- `packaging/arch/PKGBUILD.release.in` — verified source release template (owner/hash placeholders)
- `packaging/arch/bandhop.service` — optional systemd user unit for package-managed installation
- `scripts/build-arch-package.sh` — creates source tarball and a local PKGBUILD

The user controls whether to enable the service. CLI installation and package installation are alternatives; avoid mixing their uninstall processes.
