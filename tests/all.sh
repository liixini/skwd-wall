#!/usr/bin/env sh
DIR="$(cd "$(dirname "$0")" && pwd)"
rc=0

echo "== unit: view logic =="
"$DIR/run.sh" || rc=1

echo "== e2e: real daemon (static/video/WE/multi-monitor/sequence) =="
"$DIR/e2e/run.sh" || rc=1

echo "== e2e: engine routing (skwd-paper vs awww) =="
"$DIR/e2e/engine.sh" || rc=1

echo "== e2e: WE scene renderer spawn (--assets-dir regression) =="
"$DIR/e2e/we_render.sh" || rc=1

echo "== e2e: external matugen config change =="
"$DIR/e2e/matugen.sh" || rc=1

if [ "$rc" -eq 0 ]; then
  echo "ALL SUITES PASS"
else
  echo "SOME SUITES FAILED"
fi
exit $rc
