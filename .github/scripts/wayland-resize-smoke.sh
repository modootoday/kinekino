#!/usr/bin/env bash
# M6-5b: the player window on a headless sway (Wayland only, no Xwayland), resized by the
# compositor and by an edge drag; wayland_resize_probe.py measures resize-to-frame latency.
# Usage: wayland-resize-smoke.sh <kinekino binary> <output dir>
set -euo pipefail
bin=$(realpath "$1")
out=$(realpath -m "$2")
mkdir -p "$out"
here=$(cd "$(dirname "$0")" && pwd)

export XDG_RUNTIME_DIR="$(mktemp -d)"
chmod 700 "$XDG_RUNTIME_DIR"
unset DISPLAY WAYLAND_DISPLAY
cat > "$out/sway.conf" <<CONF
xwayland disable
output HEADLESS-1 resolution 1600x1000 position 0 0 bg #ff00ff solid_color
default_border none
default_floating_border none
focus_follows_mouse no
for_window [title=".*"] floating enable
CONF
WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1 \
  sway -d -c "$out/sway.conf" > "$out/sway.log" 2>&1 &
sway_pid=$!
trap 'kill "${app_pid:-}" "$sway_pid" 2>/dev/null || true' EXIT
for _ in $(seq 100); do
  sock=$(ls "$XDG_RUNTIME_DIR"/wayland-* 2>/dev/null | grep -v lock | head -1 || true)
  ipc=$(ls "$XDG_RUNTIME_DIR"/sway-ipc.* 2>/dev/null | head -1 || true)
  [ -n "$sock" ] && [ -n "$ipc" ] && break
  sleep 0.1
done
[ -n "${sock:-}" ] || { echo "sway did not start"; cat "$out/sway.log"; exit 1; }
export WAYLAND_DISPLAY=$(basename "$sock") SWAYSOCK="$ipc"
swaymsg -t get_version | tee "$out/sway-version.json"
swaymsg -t get_outputs > "$out/outputs.json"

clip="$out/clip.mp4"
ffmpeg -loglevel error -y -f lavfi -i testsrc2=size=1280x720:rate=30 -t 120 -pix_fmt yuv420p "$clip"

export KINEKINO_TRACE_RESIZE=1 WINIT_UNIX_BACKEND=wayland
"$bin" watch "$clip" > "$out/player.stdout" 2> "$out/player.stderr" &
app_pid=$!
status=0
python3 "$here/wayland_resize_probe.py" --pid "$app_pid" --out "$out" || status=$?
kill -0 "$app_pid" 2>/dev/null && echo "player still running after probe: yes" | tee -a "$out/summary.txt" \
  || { echo "player exited during probe" | tee -a "$out/summary.txt"; status=1; }
# Did sway give up waiting for the client (transaction timeout) instead of the client
# answering the resize? Counted from the debug log.
echo "sway transaction timeouts: $(grep -c 'timed out' "$out/sway.log" || true)" | tee -a "$out/summary.txt"
kill "$app_pid" 2>/dev/null || true
wait "$app_pid" 2>/dev/null || true
rm -f "$clip"
exit "$status"
