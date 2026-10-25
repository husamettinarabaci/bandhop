# KDE Plasma / VIA keyboard shortcuts

Bandhop has **no embedded global keylogger**. KDE Plasma captures shortcuts globally on Wayland. VIA/QMK supplies virtual keycodes, not the playback logic.

## Default layout

- `KC_F13` → **F13** → `bandhop toggle` (on/off)
- `KC_F14` → **F14** → `bandhop next` (circular station selection)

## Setup

1. In VIA, assign `KC_F13` and `KC_F14` to two unused physical keys.
2. Execute `bandhop keys check`. This looks for potential assignments in `~/.config/kglobalshortcutsrc` but is not definitive.
3. Execute `bandhop keys setup` to print the full absolute executable path and commands for this installation.
4. In KDE Plasma **System Settings → Keyboard → Shortcuts → Add New → Command or Script**, register two command shortcuts.
5. Assign physical F13 to `.../bandhop toggle` and physical F14 to `.../bandhop next`.
6. Confirm any KDE conflict warning before saving. Test from an unrelated application window.

### Why no automatic keyboard rewrite?

Directly editing KGlobalAccel files or changing live bindings through experimental D-Bus methods is not a reliable, portable KDE Plasma 6/Wayland shortcut workflow. A text-only `keys setup` command provides unambiguous setup instructions instead of falsely claiming keys were registered. This makes the actual supported capability transparent.

The F13/F14 mapping is the **project default**, not a technical requirement for the CLI. A user may bind different combinations in the desktop shortcut UI.
