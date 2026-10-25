#!/usr/bin/env bash
# Safe smoke/integration tests using isolated XDG directories and a fake mpv.
# These tests never connect to the internet and do not modify the user's config.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
cargo build --quiet --locked
BIN="$PWD/target/debug/bandhop"
ROOT="$(mktemp -d)"
SERVICE_PID=''
cleanup() {
  if [[ -n "$SERVICE_PID" ]]; then
    "$BIN" stop >/dev/null 2>&1 || true
    kill "$SERVICE_PID" 2>/dev/null || true
    wait "$SERVICE_PID" 2>/dev/null || true
  fi
  # Reap any leaked test streams even when the assertions fail.
  if [[ -f "$ROOT/mpv-pids" ]]; then
    while read -r pid; do kill "$pid" 2>/dev/null || true; done < "$ROOT/mpv-pids"
  fi
  rm -rf "$ROOT"
}
trap cleanup EXIT
export XDG_CONFIG_HOME="$ROOT/config"
export XDG_STATE_HOME="$ROOT/state"
export XDG_RUNTIME_DIR="$ROOT/run"
mkdir -p "$XDG_RUNTIME_DIR" "$ROOT/mock-bin"
cat > "$ROOT/mock-bin/mpv" <<'MOCK'
#!/usr/bin/env bash
echo "$$" >> "$BANDHOP_MOCK_PID_LOG"
exec sleep 180
MOCK
cat > "$ROOT/mock-bin/notify-send" <<'MOCK'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$BANDHOP_MOCK_NOTIFY_LOG"
exit 0
MOCK
chmod +x "$ROOT/mock-bin/mpv" "$ROOT/mock-bin/notify-send"
export PATH="$ROOT/mock-bin:$PATH"
export BANDHOP_MOCK_PID_LOG="$ROOT/mpv-pids"
export BANDHOP_MOCK_NOTIFY_LOG="$ROOT/notifications"

"$BIN" init
"$BIN" stations add 'NPR Demo' 'https://example.com/news.mp3'
"$BIN" stations add 'BBC Demo' 'https://example.com/world.mp3'
"$BIN" stations list | grep -q 'npr-demo'
"$BIN" stations move bbc-demo 1
"$BIN" config validate
"$BIN" stations remove npr-demo
"$BIN" stations list | grep -q 'bbc-demo'
if "$BIN" stations add Unsafe 'file:///etc/passwd' > /dev/null 2>&1; then
  echo 'ERROR: rejected URL was accepted' >&2; exit 1
fi
"$BIN" keys show | grep -q F14

"$BIN" daemon &
SERVICE_PID=$!
READY=0
for _ in $(seq 1 50); do
  if "$BIN" status >/dev/null 2>&1; then READY=1; break; fi
  sleep 0.1
done
if [[ "$READY" != 1 ]]; then echo 'ERROR: daemon did not start' >&2; exit 1; fi
"$BIN" status | grep -q '^Stopped:'
"$BIN" notify test | grep -q '^Notification submitted successfully'
"$BIN" next | grep -q 'Selected (stopped):'
"$BIN" status | grep -q '^Stopped:'
"$BIN" toggle | grep -q '^Playing:'
sleep 0.2
"$BIN" status | grep -q '^Playing:'
mapfile -t first_pids < "$BANDHOP_MOCK_PID_LOG"
[[ ${#first_pids[@]} -eq 1 ]] || { echo 'ERROR: play spawned more than one stream' >&2; exit 1; }
kill -0 "${first_pids[0]}" || { echo 'ERROR: playing mpv exited early' >&2; exit 1; }
"$BIN" play | grep -q '^Already playing:'
sleep 0.2
[[ $(wc -l < "$BANDHOP_MOCK_PID_LOG") -eq 1 ]] || { echo 'ERROR: repeat play spawned another mpv' >&2; exit 1; }
"$BIN" next | grep -q '^Playing:'
sleep 0.2
"$BIN" status | grep -q '^Playing:'
mapfile -t all_pids < "$BANDHOP_MOCK_PID_LOG"
[[ ${#all_pids[@]} -eq 2 ]] || { echo 'ERROR: next did not replace exactly one mpv' >&2; exit 1; }
if kill -0 "${all_pids[0]}" 2>/dev/null; then echo 'ERROR: previous stream leaked after next' >&2; exit 1; fi
kill -0 "${all_pids[1]}" || { echo 'ERROR: next stream is not alive' >&2; exit 1; }
"$BIN" stop | grep -q '^Stopped:'
"$BIN" status | grep -q '^Stopped:'
if kill -0 "${all_pids[1]}" 2>/dev/null; then echo 'ERROR: mpv leaked after stop' >&2; exit 1; fi
"$BIN" next | grep -q '^Selected (stopped):'
"$BIN" status | grep -q '^Stopped:'
"$BIN" config reload | grep -q '^Configuration reloaded'
"$BIN" toggle | grep -q '^Playing:'
"$BIN" toggle | grep -q '^Stopped:'
for pid in "${all_pids[@]}"; do
  if kill -0 "$pid" 2>/dev/null; then echo "ERROR: leaked mpv process $pid" >&2; exit 1; fi
done

sleep 0.3
[[ -f "$BANDHOP_MOCK_NOTIFY_LOG" ]] || { echo 'ERROR: notify-send not called' >&2; exit 1; }
grep -q -- '--urgency=normal' "$BANDHOP_MOCK_NOTIFY_LOG" || { echo 'ERROR: notifications are not normal urgency' >&2; exit 1; }
grep -q 'Bandhop' "$BANDHOP_MOCK_NOTIFY_LOG" || { echo 'ERROR: notification app name missing' >&2; exit 1; }
grep -q 'Now playing' "$BANDHOP_MOCK_NOTIFY_LOG" || { echo 'ERROR: playback notification missing' >&2; exit 1; }
grep -q 'Stopped' "$BANDHOP_MOCK_NOTIFY_LOG" || { echo 'ERROR: stop notification missing' >&2; exit 1; }

printf 'Smoke/integration tests passed (isolated config, mocked mpv, no internet).\n'
