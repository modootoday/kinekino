#!/usr/bin/env bash
# Captures a 4-frame sequence per preset clip into shots/presets/ and builds one strip per preset.
set -euo pipefail; cd "$(dirname "$0")/.."; mkdir -p shots/presets
python3 -m http.server 18766 >/dev/null 2>&1 & SRV=$!; trap 'kill $SRV 2>/dev/null || true' EXIT; sleep 1
shot(){ /usr/bin/google-chrome --headless --no-sandbox --use-angle=swiftshader --enable-unsafe-swiftshader --hide-scrollbars \
  --window-size=360,360 --virtual-time-budget=4000 --screenshot="shots/presets/$1_$2.png" "http://localhost:18766/viewer/index.html?shot=1&clip=$1&t=$2&yaw=${3:-0}" 2>/dev/null; }
declare -A T=([idle_breath]="0 1.0 1.8 3.5" [alert]="0.2 0.42 0.6 1.5" [curious]="0.4 0.9 1.5 2.2" [tickle]="0.4 0.55 0.8 1.2"
  [content]="0.8 1.4 2.6 4.5" [startled]="0.05 0.2 0.5 1.4" [ear_perk]="0.3 0.4 0.55 0.85" [ear_flutter]="0.32 0.36 0.5 0.9" [sequence]="0 1.4 1.7 2.86 4.1 5.5 6.9")
for c in "${!T[@]}"; do for t in ${T[$c]}; do shot "$c" "$t" "${YAW:-0}"; done; done
python3 tools/compose-strips.py
