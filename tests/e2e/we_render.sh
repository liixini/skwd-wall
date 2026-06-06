#!/usr/bin/env sh
set -u
DIR="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$DIR/../.." && pwd)"

DAEMON="${SKWD_DAEMON_BIN:-}"
[ -x "$DAEMON" ] || DAEMON="$REPO/../skwd-daemon/target/release/skwd-daemon"
[ -x "$DAEMON" ] || DAEMON="$(command -v skwd-daemon || true)"
if [ ! -x "$DAEMON" ]; then
  echo "WE-RENDER ERROR: skwd-daemon binary not found (set SKWD_DAEMON_BIN)"
  exit 3
fi

fails=0
note() { echo "  - $1"; fails=$((fails + 1)); }

TMP="$(mktemp -d)"
export HOME="$TMP/home"
export XDG_RUNTIME_DIR="$TMP/run"
export XDG_DATA_HOME="$TMP/home/.local/share"
export XDG_CONFIG_HOME="$TMP/home/.config"
export XDG_CACHE_HOME="$TMP/home/.cache"
export SKWD_HEADLESS=1
unset WAYLAND_DISPLAY
WP="$HOME/Pictures/Wallpapers"
WE="$HOME/we"
SCENE="$WE/900900900"
CFG="$XDG_CONFIG_HOME/skwd-wall"
FAKE="$TMP/bin"
ASSETS="$HOME/.local/share/Steam/steamapps/common/wallpaper_engine/assets"
LWE_LOG="$TMP/lwe.log"
mkdir -p "$WP" "$SCENE" "$CFG" "$FAKE" "$XDG_RUNTIME_DIR" "$ASSETS"
trap 'kill "${DPID:-0}" 2>/dev/null; rm -rf "$TMP"' EXIT INT TERM

magick -size 16x16 xc:red "$WP/alpha.png" 2>/dev/null || convert -size 16x16 xc:red "$WP/alpha.png"
echo '{"title":"Scene","type":"scene","file":"scene.pkg","preview":"preview.gif"}' > "$SCENE/project.json"
head -c 256 /dev/urandom > "$SCENE/scene.pkg"
magick -size 16x16 xc:green "$SCENE/preview.gif" 2>/dev/null || convert -size 16x16 xc:green "$SCENE/preview.gif"

printf '#!/bin/sh\necho "$@" >> "%s"\n' "$LWE_LOG" > "$FAKE/linux-wallpaperengine"
chmod +x "$FAKE/linux-wallpaperengine"
export PATH="$FAKE:$PATH"

printf '{"features":{"steam":true,"matugen":false},"paths":{"steamWorkshop":"%s"},"restoreOnStartup":false}' "$WE" > "$CFG/config.json"

"$DAEMON" > "$TMP/daemon.log" 2>&1 &
DPID=$!
python3 "$DIR/wait_ready.py" "$XDG_RUNTIME_DIR/skwd/daemon.sock" 2 30 >/dev/null || { echo "WE-RENDER ERROR: daemon did not cache items"; exit 4; }

python3 - "$XDG_RUNTIME_DIR/skwd/daemon.sock" <<'PY'
import socket, json, sys
s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.connect(sys.argv[1])
s.sendall((json.dumps({"method": "wall.apply", "params": {"type": "we", "we_id": "900900900", "screens": ["DP-1", "DP-2"]}, "id": 1}) + "\n").encode())
b = b""
s.settimeout(25)
while b"\n" not in b:
    b += s.recv(65536)
PY
sleep 2

echo "WE-RENDER scene apply test"
[ -s "$LWE_LOG" ] || note "linux-wallpaperengine was NOT spawned for a scene WE"
grep -q -- "--assets-dir" "$LWE_LOG" 2>/dev/null || note "scene WE spawned WITHOUT --assets-dir (assets fallback regression)"
grep -q "wallpaper_engine/assets" "$LWE_LOG" 2>/dev/null || note "--assets-dir does not point at the WE assets dir"
grep -q -- "--screen-root" "$LWE_LOG" 2>/dev/null || note "spawn missing --screen-root"
grep -q "900900900" "$LWE_LOG" 2>/dev/null || note "spawn missing the WE id"
if grep -q -- "--bg" "$LWE_LOG" 2>/dev/null; then
  note "spawn uses --bg <id> (lwe r594 renders that as a WINDOW; the id must be the trailing positional arg)"
fi
[ "$(awk 'END{print $NF}' "$LWE_LOG" 2>/dev/null)" = "900900900" ] || note "WE id must be the trailing positional argument, got: $(awk 'END{print $NF}' "$LWE_LOG" 2>/dev/null)"
SCREENS_N=$(grep -o -- "--screen-root" "$LWE_LOG" 2>/dev/null | wc -l)
[ "$SCREENS_N" -eq 2 ] || note "expected 2 --screen-root for a 2-monitor apply, got $SCREENS_N"
CLAMP_N=$(grep -o -- "--clamp" "$LWE_LOG" 2>/dev/null | wc -l)
[ "$CLAMP_N" -eq 1 ] || note "--clamp must appear exactly ONCE (lwe rejects duplicate --clamp), got $CLAMP_N"

if [ "$fails" -eq 0 ]; then
  echo "WE-RENDER PASS (scene WE: --assets-dir + --screen-root + positional id, no --bg)"
  exit 0
fi
echo "WE-RENDER FAIL ($fails)"
echo "--- spawn args seen ---"
cat "$LWE_LOG" 2>/dev/null | head
exit 1
