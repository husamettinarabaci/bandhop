# Installation and operation

## From a downloaded ZIP

```sh
unzip bandhop-0.1.2-source.zip
cd bandhop-0.1.2
sudo pacman -S --needed mpv libnotify
./install.sh
```

Use your existing `rustup` installation (`rustup default stable`). Do not install the conflicting `rust` package alongside `rustup`.

For optional daemon autostart on login:

```sh
./install.sh --enable-service
bandhop service status
bandhop status  # should report stopped
```

The systemd service runs as the **current user**. No root service, lingering, or automatic playback. Session audio may depend on a logged-in user audio server (PipeWire/PulseAudio).

## Operation

```sh
bandhop stations add "My Station" "https://DIRECT-STREAM-URL"
bandhop stations list
bandhop toggle
bandhop next
bandhop toggle
```

On a host with no enabled daemon, run `bandhop listen` in a terminal instead. You can then send `bandhop toggle` from another terminal while it is running.

## Troubleshooting

- `bandhop: ... not running`: `bandhop listen` or `bandhop service enable`.
- `mpv not found`: install `mpv`.
- No sound: verify the direct URL with `mpv 'https://DIRECT-STREAM-URL'`; check output device and network.
- Notifications absent: run `bandhop notify test` and consult [NOTIFICATIONS.md](NOTIFICATIONS.md); inspect the user service journal.
- Daemon problems: `journalctl --user -u bandhop.service -n 50 --no-pager`.
- Shortcut not firing: verify F13/F14 in KDE and in VIA; check conflicts.
- Config parse error: `bandhop config validate` and consult CONFIGURATION.md.

## Uninstall

For installs from source, `./uninstall.sh` preserves user configuration. `./uninstall.sh --purge` irreversibly removes the Bandhop config and selection state. For package-managed installs, use the package manager.

## Upgrading an existing user-service installation

If upgrading from v0.1.0, first stop the old user daemon. The old
controller may lose ownership of still-running mpv players. The systemd
user service has `KillMode=control-group`, so stopping it closes its streams.

```sh
systemctl --user stop bandhop.service
cargo test --locked
./scripts/smoke-test.sh
./install.sh --enable-service
bandhop --version
bandhop status
```

`bandhop status` should show a selected station in the stopped state.
Previously saved stations are not overwritten. Run `bandhop toggle` only
after the test and upgrade succeed.
