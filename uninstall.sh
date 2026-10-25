#!/usr/bin/env bash
set -euo pipefail
if [[ $# -gt 1 || ( $# -eq 1 && "${1:-}" != "--purge" ) ]]; then
  echo 'Usage: ./uninstall.sh [--purge]' >&2; exit 2
fi
BIN="$HOME/.local/bin/bandhop"
if [[ -x "$BIN" ]]; then
  "$BIN" service uninstall || echo 'Warning: could not remove systemd user service' >&2
else
  systemctl --user disable --now bandhop.service 2>/dev/null || true
  rm -f "${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user/bandhop.service"
  systemctl --user daemon-reload 2>/dev/null || true
fi
rm -f "$BIN"
if [[ ${1:-} == '--purge' ]]; then
  rm -rf "${XDG_CONFIG_HOME:-$HOME/.config}/bandhop" "${XDG_STATE_HOME:-$HOME/.local/state}/bandhop"
fi
echo 'Uninstalled Bandhop. (If installed via pacman, use pacman for removal.)'
