#!/usr/bin/env sh
set -u
DIR="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$DIR/../.." && pwd)"

DAEMON="${SKWD_DAEMON_BIN:-}"
[ -x "$DAEMON" ] || DAEMON="$REPO/../skwd-daemon/target/release/skwd-daemon"
[ -x "$DAEMON" ] || DAEMON="$(command -v skwd-daemon || true)"
if [ ! -x "$DAEMON" ]; then
  echo "ENGINE ERROR: skwd-daemon binary not found (set SKWD_DAEMON_BIN)"
  exit 3
fi

fails=0
note() { echo "  - $1"; fails=$((fails + 1)); }

apply_one() {
  python3 - "$1" "$2" <<'PY'
import socket, json, sys
s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.connect(sys.argv[1])
s.sendall((json.dumps({"method": "wall.apply", "params": {"type": "static", "path": sys.argv[2]}, "id": 1}) + "\n").encode())
b = b""
s.settimeout(25)
while b"\n" not in b:
    b += s.recv(65536)
PY
}

run_engine() {
  engine="$1"
  TMP="$(mktemp -d)"
  export HOME="$TMP/home"
  export XDG_RUNTIME_DIR="$TMP/run"
  export XDG_DATA_HOME="$TMP/home/.local/share"
  export XDG_CONFIG_HOME="$TMP/home/.config"
  export XDG_CACHE_HOME="$TMP/home/.cache"
  export SKWD_HEADLESS=1
  unset WAYLAND_DISPLAY
  WP="$HOME/Pictures/Wallpapers"
  CFG="$XDG_CONFIG_HOME/skwd-wall"
  FAKE="$TMP/bin"
  mkdir -p "$WP" "$CFG" "$FAKE" "$XDG_RUNTIME_DIR"
  magick -size 16x16 xc:red "$WP/alpha.png" 2>/dev/null || convert -size 16x16 xc:red "$WP/alpha.png"

  PAPER_LOG="$TMP/paper.log"
  AWWW_LOG="$TMP/awww.log"
  printf '#!/bin/sh\necho "$@" >> "%s"\n' "$PAPER_LOG" > "$FAKE/skwd-paper-still"
  cp "$FAKE/skwd-paper-still" "$FAKE/skwd-paper"
  printf '#!/bin/sh\n[ "$1" = "query" ] && exit 0\necho "$@" >> "%s"\nexit 0\n' "$AWWW_LOG" > "$FAKE/awww"
  chmod +x "$FAKE/skwd-paper-still" "$FAKE/skwd-paper" "$FAKE/awww"

  printf '{"features":{"matugen":false},"paper":{"engine":"%s"},"restoreOnStartup":false}' "$engine" > "$CFG/config.json"

  export SKWD_PAPER_STILL_BIN="$FAKE/skwd-paper-still"
  export SKWD_PAPER_BIN="$FAKE/skwd-paper"
  export PATH="$FAKE:$PATH"

  "$DAEMON" > "$TMP/daemon.log" 2>&1 &
  DPID=$!
  python3 "$DIR/wait_ready.py" "$XDG_RUNTIME_DIR/skwd/daemon.sock" 1 30 >/dev/null
  apply_one "$XDG_RUNTIME_DIR/skwd/daemon.sock" "$HOME/Pictures/Wallpapers/alpha.png"
  sleep 1
  kill "$DPID" 2>/dev/null

  if [ "$engine" = "skwd-paper" ]; then
    [ -s "$PAPER_LOG" ] || note "skwd-paper: renderer skwd-paper-still was NOT invoked"
    grep -q "alpha.png" "$PAPER_LOG" 2>/dev/null || note "skwd-paper: invocation missing wallpaper path"
    [ ! -s "$AWWW_LOG" ] || note "skwd-paper: awww was unexpectedly invoked"
  else
    [ -s "$AWWW_LOG" ] || note "awww: renderer awww was NOT invoked"
    grep -q "img" "$AWWW_LOG" 2>/dev/null || note "awww: 'awww img' not invoked"
    grep -q "alpha.png" "$AWWW_LOG" 2>/dev/null || note "awww: invocation missing wallpaper path"
    [ ! -s "$PAPER_LOG" ] || note "awww: skwd-paper-still unexpectedly invoked"
  fi
  rm -rf "$TMP"
}

echo "ENGINE routing test (stubbed renderers)"
run_engine "skwd-paper"
run_engine "awww"
if [ "$fails" -eq 0 ]; then
  echo "ENGINE PASS (skwd-paper -> skwd-paper-still, awww -> awww img)"
  exit 0
fi
echo "ENGINE FAIL ($fails)"
exit 1
