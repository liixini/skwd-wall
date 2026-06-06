#!/usr/bin/env sh
set -u
DIR="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$DIR/../.." && pwd)"

DAEMON="${SKWD_DAEMON_BIN:-}"
[ -x "$DAEMON" ] || DAEMON="$REPO/../skwd-daemon/target/release/skwd-daemon"
[ -x "$DAEMON" ] || DAEMON="$(command -v skwd-daemon || true)"
if [ ! -x "$DAEMON" ]; then
  echo "E2E ERROR: skwd-daemon binary not found (set SKWD_DAEMON_BIN)"
  exit 3
fi

TMP="$(mktemp -d)"
export HOME="$TMP/home"
export XDG_RUNTIME_DIR="$TMP/run"
export XDG_DATA_HOME="$TMP/home/.local/share"
export XDG_CONFIG_HOME="$TMP/home/.config"
export XDG_CACHE_HOME="$TMP/home/.cache"
export SKWD_HEADLESS=1
export SKWD_WALL_QML="$(cd "$REPO/qml" && pwd)"
unset WAYLAND_DISPLAY
WP="$HOME/Pictures/Wallpapers"
WE="$HOME/we/777000777"
CFG="$XDG_CONFIG_HOME/skwd-wall"
mkdir -p "$WP" "$WE" "$CFG" "$XDG_RUNTIME_DIR" "$XDG_DATA_HOME" "$XDG_CACHE_HOME"

magick -size 16x16 xc:red "$WP/alpha.png" 2>/dev/null || convert -size 16x16 xc:red "$WP/alpha.png"
magick -size 16x16 xc:green "$WP/bravo.png" 2>/dev/null || convert -size 16x16 xc:green "$WP/bravo.png"
magick -size 16x16 xc:blue "$WP/charlie.png" 2>/dev/null || convert -size 16x16 xc:blue "$WP/charlie.png"
touch -d "2020-01-01T00:00:00" "$WP/alpha.png"
touch -d "2020-01-02T00:00:00" "$WP/bravo.png"
touch -d "2020-01-03T00:00:00" "$WP/charlie.png"
ffmpeg -f lavfi -i color=c=purple:s=32x32:d=1 -pix_fmt yuv420p -y "$WP/clip.mp4" >/dev/null 2>&1
echo '{"title":"TestScene","type":"scene","file":"scene.pkg","preview":"preview.gif"}' > "$WE/project.json"
magick -size 16x16 xc:orange "$WE/preview.gif" 2>/dev/null || convert -size 16x16 xc:orange "$WE/preview.gif"

printf '{"features":{"steam":true},"paths":{"steamWorkshop":"%s/we"},"restoreOnStartup":false}' "$HOME" > "$CFG/config.json"

"$DAEMON" > "$TMP/daemon.log" 2>&1 &
DPID=$!
cleanup() { kill "$DPID" 2>/dev/null; rm -rf "$TMP"; }
trap cleanup EXIT INT TERM

if ! python3 "$DIR/wait_ready.py" "$XDG_RUNTIME_DIR/skwd/daemon.sock" 5 40; then
  echo "E2E ERROR: daemon did not cache 5 wallpapers"
  tail -20 "$TMP/daemon.log"
  exit 4
fi

log="$TMP/qs.log"
QT_QPA_PLATFORM=offscreen qs -p "$DIR/qs" > "$log" 2>&1
code=$?
sed 's/\x1b\[[0-9;]*m//g' "$log" | grep -aE 'E2E|  - ' || true
exit $code
