#!/usr/bin/env bash
# Low-priority single headless capture helper: tools/shot1.sh OUT.png 'query' [size]. Run shots sequentially; caller starts/stops the server.
nice -n 15 timeout 90 /usr/bin/google-chrome --headless --no-sandbox --use-angle=swiftshader --enable-unsafe-swiftshader --hide-scrollbars \
  --window-size=${3:-420},${3:-420} --virtual-time-budget=4000 --screenshot="$1" "http://localhost:18765/viewer/index.html?shot=1&$2" 2>/dev/null
