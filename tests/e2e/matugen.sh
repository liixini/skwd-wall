#!/usr/bin/env sh
set -u
DIR="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$DIR/../.." && pwd)"

DAEMON="${SKWD_DAEMON_BIN:-}"
[ -x "$DAEMON" ] || DAEMON="$REPO/../skwd-daemon/target/release/skwd-daemon"
[ -x "$DAEMON" ] || DAEMON="$(command -v skwd-daemon || true)"
if [ ! -x "$DAEMON" ]; then
  echo "MATUGEN ERROR: skwd-daemon binary not found (set SKWD_DAEMON_BIN)"
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
s.settimeout(30)
while b"\n" not in b:
    b += s.recv(65536)
PY
}

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
MATCFG="$HOME/.config/matugen/config.toml"
EXT_LOG="$TMP/ext-matugen.log"
mkdir -p "$WP" "$CFG" "$FAKE" "$XDG_RUNTIME_DIR" "$HOME/.config/matugen"
trap 'kill "${DPID:-0}" 2>/dev/null; rm -rf "$TMP"' EXIT INT TERM

magick -size 16x16 xc:red "$WP/alpha.png" 2>/dev/null || convert -size 16x16 xc:red "$WP/alpha.png"
printf '#!/bin/sh\nexit 0\n' > "$FAKE/matugen"
printf '#!/bin/sh\necho "$@" >> "%s"\n' "$EXT_LOG" > "$FAKE/ext-matugen"
chmod +x "$FAKE/matugen" "$FAKE/ext-matugen"
echo '# dummy matugen config' > "$MATCFG"
export PATH="$FAKE:$PATH"

printf '{"features":{"matugen":true},"defaultMatugenConfig":"%s","restoreOnStartup":false}' "$MATCFG" > "$CFG/config.json"

"$DAEMON" > "$TMP/daemon.log" 2>&1 &
DPID=$!
python3 "$DIR/wait_ready.py" "$XDG_RUNTIME_DIR/skwd/daemon.sock" 1 30 >/dev/null || { echo "MATUGEN ERROR: daemon did not start"; exit 4; }

echo "MATUGEN external-command config test"

apply_one "$XDG_RUNTIME_DIR/skwd/daemon.sock" "$WP/alpha.png"
[ ! -s "$EXT_LOG" ] || note "before config change: external command ran unexpectedly (default should use matugen)"

printf '{"features":{"matugen":true},"defaultMatugenConfig":"%s","externalMatugenCommand":"ext-matugen ran path=%%path%% scheme=%%scheme%% mode=%%mode%% idx=%%index%%","restoreOnStartup":false}' "$MATCFG" > "$CFG/config.json"
sleep 2

grep -q "config] reloaded" "$TMP/daemon.log" || note "daemon did not reload config.json after change"

apply_one "$XDG_RUNTIME_DIR/skwd/daemon.sock" "$WP/alpha.png"
[ -s "$EXT_LOG" ] || note "after config change: external matugen command was NOT invoked"
grep -q "alpha.png" "$EXT_LOG" 2>/dev/null || note "external matugen: %path% not substituted with wallpaper path"
grep -q "scheme=" "$EXT_LOG" 2>/dev/null || note "external matugen: %scheme% not substituted"
grep -q "mode=" "$EXT_LOG" 2>/dev/null || note "external matugen: %mode% not substituted"

if [ "$fails" -eq 0 ]; then
  echo "MATUGEN PASS (config reload + external command invoked with substitution)"
  exit 0
fi
echo "MATUGEN FAIL ($fails)"
exit 1
