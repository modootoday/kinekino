#!/usr/bin/env bash
# Render each clip to out/video/CLIP.mp4 (720x720, 30fps, H.264 yuv420p) + all-presets.mp4. One headless Chrome at a time, niced.
set -u; cd "$(dirname "$0")/.."; F=/tmp/kk-frames; rm -rf $F; mkdir -p out/video $F
nice -n 15 python3 tools/record-server.py & SRV=$!; sleep 1
declare -A D=([idle]=2 [reveal]=2 [blink]=2 [tail_wave]=2 [sequence]=7 [idle_breath]=4 [alert]=2.4 [curious]=3 [tickle]=2 [content]=4 [startled]=2.4 [ear_perk]=2 [ear_flutter]=2)
ORDER="idle reveal blink tail_wave idle_breath alert curious tickle content startled ear_perk ear_flutter sequence"
for c in $ORDER; do
  nice -n 15 /usr/bin/google-chrome --headless --no-sandbox --use-angle=swiftshader --enable-unsafe-swiftshader --hide-scrollbars \
    --window-size=720,900 --force-device-scale-factor=1 --remote-debugging-port=0 --user-data-dir=/tmp/kk-chrome \
    "http://localhost:18765/viewer/index.html?rec=$c&dur=${D[$c]}&yaw=20&tilt=8" >/dev/null 2>&1 & CH=$!
  for s in $(seq 600); do [ -f $F/$c/DONE ] && break; sleep 1; done
  kill $CH; wait $CH 2>/dev/null
  ffmpeg -loglevel error -y -framerate 30 -i $F/$c/%04d.png -c:v libx264 -pix_fmt yuv420p -crf 18 -movflags +faststart out/video/$c.mp4
  echo "$c $(ls $F/$c | wc -l)"
done
printf "file '%s.mp4'\n" $ORDER > out/video/list.txt
(cd out/video && ffmpeg -loglevel error -y -f concat -safe 0 -i list.txt -c copy all-presets.mp4 && rm list.txt)
kill $SRV; rm -rf $F /tmp/kk-chrome
