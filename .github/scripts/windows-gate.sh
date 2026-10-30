#!/usr/bin/env bash
# Windows compile gate: clippy -D warnings on the crates the box gate checks
# (x86_64-pc-windows-msvc). Pass extra cargo args (e.g. --target, -j 1) after the script.
set -euo pipefail
crates=(core platform platform-windows process vad media-contracts jobs subtitles ui-plugins card-render inference inference-hf inference-local player host cli app settings-ui)
args=()
for c in "${crates[@]}"; do args+=(-p "kinekino-$c"); done
cargo clippy --locked "${args[@]}" --all-targets "$@" -- -D warnings
