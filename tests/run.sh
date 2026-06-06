#!/usr/bin/env sh
DIR="$(cd "$(dirname "$0")" && pwd)"
export SKWD_WALL_QML="$(cd "$DIR/../qml" && pwd)"
RT="$(mktemp -d)"
export XDG_RUNTIME_DIR="$RT"
unset WAYLAND_DISPLAY

QT_QPA_PLATFORM=offscreen qs -p "$DIR/qs" >"$RT/out.log" 2>&1
code=$?
sed 's/\x1b\[[0-9;]*m//g' "$RT/out.log" | grep -aE 'QMLTEST|  - ' || true
rm -rf "$RT"
exit $code
