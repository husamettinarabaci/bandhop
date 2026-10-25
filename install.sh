#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
if [[ $# -gt 1 || ( $# -eq 1 && "${1:-}" != "--enable-service" ) ]]; then
  echo 'Usage: ./install.sh [--enable-service]' >&2; exit 2
fi
for cmd in cargo mpv install; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Missing requirement: $cmd" >&2; exit 1
  fi
done
cargo build --release --locked
mkdir -p "$HOME/.local/bin"
# Atomic replacement avoids ETXTBSY when a previous Bandhop binary is running.
STAGED_BIN="$(mktemp "$HOME/.local/bin/.bandhop-install.XXXXXX")"
trap 'rm -f "$STAGED_BIN"' EXIT
install -m 0755 target/release/bandhop "$STAGED_BIN"
BIN="$HOME/.local/bin/bandhop"
mv -f "$STAGED_BIN" "$BIN"
"$BIN" init
if [[ $# -eq 1 ]]; then
  "$BIN" service enable
fi
# `systemctl enable --now` does not restart an already-running service.
# Restart so every new CLI talks to the matching, updated daemon binary.
if command -v systemctl >/dev/null 2>&1 && systemctl --user is-active --quiet bandhop.service; then
  systemctl --user restart bandhop.service
  printf 'Restarted the Bandhop user daemon using the updated binary. Playback is OFF.\n'
fi
printf '\nInstalled: %s\n' "$BIN"
printf 'Next: %s stations add "My Radio" "https://DIRECT-STREAM-URL"\n' "$BIN"
printf 'Keys: %s keys setup\n' "$BIN"
printf 'If bandhop is not found, add ~/.local/bin to PATH or restart the shell.\n'
