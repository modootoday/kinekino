#!/usr/bin/env bash
# Serves model3d/ and captures key poses of the glb via headless Chrome into shots/.
set -euo pipefail; cd "$(dirname "$0")/.."
python3 -m http.server 18765 >/dev/null 2>&1 & SRV=$!; trap 'kill $SRV' EXIT; sleep 1
shot(){ /usr/bin/google-chrome --headless --no-sandbox --use-angle=swiftshader --enable-unsafe-swiftshader --hide-scrollbars \
  --window-size=480,480 --virtual-time-budget=4000 --screenshot="shots/$1.png" "http://localhost:18765/viewer/index.html?shot=1&$2" 2>/dev/null; }
shot 01_static 'clip=idle&t=0'; shot 02_reveal 'clip=reveal&t=0.7'; shot 03_blink 'clip=blink&t=0.3'
shot 04_tail_wave_a 'clip=tail_wave&t=0.4'; shot 05_tail_wave_b 'clip=tail_wave&t=0.8'; shot 06_yaw20 'clip=blink&t=0&yaw=20'; shot 07_yaw40 'clip=blink&t=0&yaw=40'; shot 08_static_yaw40 'clip=idle&t=0&yaw=40'
# Tail physics: sudden 35° body roll at 0.2s, spring on vs off (frame sequence for squash check).
mkdir -p shots/physics
for t in 0.2 0.25 0.3 0.35 0.45 0.6 0.9; do shot "physics/on_$t" "clip=idle&t=$t&whip=35&spring=1"; shot "physics/off_$t" "clip=idle&t=$t&whip=35&spring=0"; done
for t in 0.25 0.4 0.6; do shot "physics/startled_on_$t" "clip=startled&t=$t&spring=1"; done
