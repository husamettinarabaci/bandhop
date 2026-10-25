# Desktop notifications

Bandhop is a headless Rust CLI. It uses the cross-desktop `org.freedesktop.Notifications` protocol via the optional `notify-send` command from `libnotify`. No KDE library or graphical application is required.

On every successful `play`, `stop`, or station change, Bandhop submits a **normal-urgency** notification. The current station name is included. The desktop environment controls whether, where, and how long a pop-up appears.

## Verify the notification server

1. Install `libnotify` (Arch/CachyOS: `sudo pacman -S --needed libnotify`).
2. From your ordinary desktop terminal (not `sudo`), run:

   ```sh
   notify-send -u normal -a Bandhop "Bandhop test" "Direct desktop notification"
   ```
3. Test Bandhop through its running daemon:

   ```sh
   bandhop notify test
   ```
4. Play a station and switch to the next station with F13/F14, or use `bandhop play` and `bandhop next`.

## No visible popup?

- In KDE Plasma, review **System Settings → Notifications**, especially **Do Not Disturb**, per-application filters, fullscreen suppression, and pop-up location. KDE can suppress notifications even after their delivery succeeds.
- Verify the current settings: `bandhop config show`. For each event, check `enabled`, `on_play`, `on_stop`, and `on_change` are `true`.
- Review the daemon's delivery errors: `journalctl --user -u bandhop.service -n 100 --no-pager`.
- The daemon uses the existing session D-Bus address. If `DBUS_SESSION_BUS_ADDRESS` is unavailable, it falls back to `$XDG_RUNTIME_DIR/bus`. The user must be logged in to a graphical session for desktop notifications.
- Test KDE notification popups with the `notify-send` example above before changing Bandhop settings.

`bandhop notify test` reports acceptance by the command-line notification sender, **not guaranteed visibility**. If playback works but the notification service is absent, playback continues and a diagnostic is written to the service journal.
