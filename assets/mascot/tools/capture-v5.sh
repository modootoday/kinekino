#!/usr/bin/env bash
set -u; cd "$(dirname "$0")/..";S=tools/shot1.sh; mkdir -p shots/v5/angles shots/v5/presets shots/v5/static
for y in 0 20 40 60 90 -60; do $S shots/v5/angles/yaw_$y.png "clip=reveal&t=1.1&yaw=$y"; done
for t in 15 -15; do $S shots/v5/angles/tilt_$t.png "clip=reveal&t=1.1&tilt=$t&yaw=20"; done
declare -A T=([idle_breath]="0 1.0 1.8 3.5" [alert]="0.2 0.42 0.6 1.5" [curious]="0.4 0.9 1.5 2.2" [tickle]="0.4 0.55 0.8 1.2"
  [content]="0.8 1.4 2.6 4.5" [startled]="0.05 0.2 0.5 1.4" [ear_perk]="0.3 0.4 0.55 0.85" [ear_flutter]="0.32 0.36 0.5 0.9")
for c in "${!T[@]}"; do y=0; [[ $c == curious || $c == startled ]] && y=25; for t in ${T[$c]}; do $S "shots/v5/presets/${c}_$t.png" "clip=$c&t=$t&yaw=$y" 360; done; done
for y in 0 40 60; do $S shots/v5/static/yaw_$y.png "clip=idle&t=0&yaw=$y"; done
