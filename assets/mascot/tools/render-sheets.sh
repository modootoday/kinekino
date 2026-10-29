#!/usr/bin/env bash
# Keyframe sheets (PNG) for part clips, combos and look-at. One headless Chrome at a time, niced; frames in /tmp/kk-frames.
set -u; cd "$(dirname "$0")/.."; F=/tmp/kk-frames; rm -rf $F; mkdir -p $F shots/v6/parts
nice -n 15 python3 tools/record-server.py & SRV=$!; sleep 1
shoot() { # spec times
  local key=${1/:/_}
  nice -n 15 /usr/bin/google-chrome --headless --no-sandbox --use-angle=swiftshader --enable-unsafe-swiftshader --hide-scrollbars \
    --window-size=400,500 --force-device-scale-factor=1 --remote-debugging-port=0 --user-data-dir=/tmp/kk-chrome \
    "http://localhost:18765/viewer/index.html?sheet=$1&times=$2&yaw=${YAW:-0}&tilt=${TILT:-0}" >/dev/null 2>&1 & local CH=$!
  for s in $(seq 180); do [ -f $F/$key/DONE ] && break; sleep 1; done
  kill $CH 2>/dev/null; wait $CH 2>/dev/null; echo "$1 $(ls $F/$key 2>/dev/null | grep -c png)"
}
for spec in "$@"; do shoot ${spec%%@*} ${spec#*@}; done
kill $SRV
