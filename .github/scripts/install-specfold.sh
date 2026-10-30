#!/usr/bin/env bash
# Install a specfold release into DIR after checking its .sha256 (same check as specfold's action).
# usage: install-specfold.sh <tag> <dir>
set -euo pipefail
tag="$1"; dir="$2"
case "$(uname -s)-$(uname -m)" in
  Linux-x86_64) target=x86_64-unknown-linux-gnu ;;
  Linux-aarch64) target=aarch64-unknown-linux-gnu ;;
  Darwin-x86_64) target=x86_64-apple-darwin ;;
  Darwin-arm64) target=aarch64-apple-darwin ;;
  *) echo "unsupported host $(uname -s)-$(uname -m)" >&2; exit 1 ;;
esac
name="specfold-$target"
base="https://github.com/modootoday/specfold/releases/download/$tag"
work="$(mktemp -d)"; trap 'rm -rf "$work"' EXIT
curl -fsSL --retry 3 -o "$work/$name.tar.gz" "$base/$name.tar.gz"
curl -fsSL --retry 3 -o "$work/$name.tar.gz.sha256" "$base/$name.tar.gz.sha256"
expected="$(awk '{print $1}' "$work/$name.tar.gz.sha256")"
if command -v sha256sum >/dev/null; then actual="$(sha256sum "$work/$name.tar.gz" | awk '{print $1}')"
else actual="$(shasum -a 256 "$work/$name.tar.gz" | awk '{print $1}')"; fi
[ "$expected" = "$actual" ] || { echo "checksum mismatch for $name: $actual != $expected" >&2; exit 1; }
mkdir -p "$dir"
tar -xzf "$work/$name.tar.gz" -C "$work"
cp "$work/$name/spec" "$work/$name/specfold" "$dir/"
"$dir/spec" --version
