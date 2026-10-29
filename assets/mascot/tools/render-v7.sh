#!/usr/bin/env bash
# Keyframe sheets: one niced headless Chrome at a time. Args: "multi:GROUP|clips|fr", "combo:NAME@times" or "cam:cam.NAME@times".
# Camera defaults to the front view (YAW=0 TILT=0). ANGLES="15,40" adds one extra cut per angle (frames in KEY_yA/),
# which make-sheets-v8.py appends as extra columns for checking depth.
set -u; ANGLES=${ANGLES:-}; cd "$(dirname "$0")/.."; F=/tmp/kk-frames; mkdir -p $F
nice -n 15 python3 tools/record-server.py & SRV=$!; sleep 1
chrome() { nice -n 15 /usr/bin/google-chrome --headless --no-sandbox --use-angle=swiftshader --enable-unsafe-swiftshader --hide-scrollbars \
  --window-size=400,500 --force-device-scale-factor=1 --remote-debugging-port=0 --user-data-dir=/tmp/kk-chrome "$@"; }
stop() { kill $CH 2>/dev/null; pkill -f "user-data-dir=/tmp/kk-chrome"; wait $CH 2>/dev/null; sleep 1; }
wait_done() { for s in $(seq 900); do [ -f $F/$1/DONE ] && break; sleep 1; done; }
for spec in "$@"; do
  case $spec in
    multi:*) IFS='|' read -r head clips fr <<< "$spec"; key=${head/:/_}; rm -rf $F/$key
      chrome "http://localhost:18765/viewer/index.html?sheet=$head&clips=$clips&fr=$fr&yaw=${YAW:-0}&tilt=${TILT:-0}&glb=${GLB:-kinekino-mascot.glb}" >/dev/null 2>&1 & CH=$!; wait_done $key ;;
    cam:*) key=${spec%%@*}; key=${key/:/_}; rm -rf $F/$key
      chrome "http://localhost:18765/viewer/index.html?sheet=${spec%%@*}&times=${spec#*@}&yaw=0&tilt=0" >/dev/null 2>&1 & CH=$!; wait_done $key ;;
    combo:*) key=${spec%%@*}; key=${key/:/_}; rm -rf $F/$key
      chrome "http://localhost:18765/viewer/index.html?sheet=${spec%%@*}&times=${spec#*@}&yaw=${YAW:-0}&tilt=${TILT:-0}&glb=${GLB:-kinekino-mascot.glb}" >/dev/null 2>&1 & CH=$!; wait_done $key ;;
  esac
  stop
  echo "$spec -> $(ls $F/$key 2>/dev/null | grep -c png)"
  case $spec in multi:*|combo:*)
    for a in ${ANGLES//,/ }; do rm -rf $F/${key}_y$a
      if [[ $spec == multi:* ]]; then url="sheet=$head&clips=$clips&fr=0.5"; else t=${spec#*@}; url="sheet=${spec%%@*}&times=${t%%,*}"; fi
      chrome "http://localhost:18765/viewer/index.html?$url&tag=y$a&yaw=$a&tilt=${TILT:-0}&glb=${GLB:-kinekino-mascot.glb}" >/dev/null 2>&1 & CH=$!; wait_done ${key}_y$a; stop
      echo "$spec yaw $a -> $(ls $F/${key}_y$a 2>/dev/null | grep -c png)"
    done ;;
  esac
done
kill $SRV
