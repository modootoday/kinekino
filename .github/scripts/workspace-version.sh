#!/usr/bin/env bash
# Print [workspace.package] version from the root Cargo.toml (the release tag is v<this>).
set -euo pipefail
awk '/^\[workspace\.package\]/{s=1;next} /^\[/{s=0} s&&/^version *=/{gsub(/.*= *"|".*/,"");print;exit}' "${1:-Cargo.toml}"
