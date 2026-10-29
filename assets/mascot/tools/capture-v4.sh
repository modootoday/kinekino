#!/usr/bin/env bash
# Sequential, low-priority capture set for the slime-look review (angles, tilts, preset strips, tail physics). Assumes tools/serve.py on :18765.
set -u; cd "$(dirname "$0")/.."; S=tools/shot1.sh; mkdir -p shots/v4/angles shots/v4/presets shots/v4/physics
for y in 0 20 40 60 90 -60; do $S shots/v4/angles/yaw_$y.png "clip=reveal&t=1.1&yaw=$y"; done
for t in 15 -15; do $S shots/v4/angles/tilt_$t.png "clip=reveal&t=1.1&tilt=$t&yaw=20"; done
declare -A T=([idle_breath]="0 1.0 1.8 3.5" [alert]="0.2 0.42 0.6 1.5" [curious]="0.4 0.9 1.5 2.2" [tickle]="0.4 0.55 0.8 1.2"
  [content]="0.8 1.4 2.6 4.5" [startled]="0.05 0.2 0.5 1.4" [ear_perk]="0.3 0.4 0.55 0.85" [ear_flutter]="0.32 0.36 0.5 0.9")
for c in "${!T[@]}"; do y=0; [[ $c == curious || $c == startled ]] && y=25; for t in ${T[$c]}; do $S "shots/v4/presets/${c}_$t.png" "clip=$c&t=$t&yaw=$y" 360; done; done
for t in 0.2 0.25 0.3 0.35 0.45 0.6 0.9; do $S shots/v4/physics/on_$t.png "clip=idle&t=$t&whip=35&spring=1" 300; $S shots/v4/physics/off_$t.png "clip=idle&t=$t&whip=35&spring=0" 300; done
