#!/usr/bin/env bash
# Install a pinned gitleaks release into DIR after checking it against the release's checksums file.
# usage: install-gitleaks.sh <version without v> <dir>
set -euo pipefail
version="$1"; dir="$2"
case "$(uname -s)-$(uname -m)" in
  Linux-x86_64) arch=linux_x64 ;;
  Linux-aarch64) arch=linux_arm64 ;;
  *) echo "unsupported host $(uname -s)-$(uname -m)" >&2; exit 1 ;;
esac
name="gitleaks_${version}_${arch}.tar.gz"
base="https://github.com/gitleaks/gitleaks/releases/download/v${version}"
work="$(mktemp -d)"; trap 'rm -rf "$work"' EXIT
curl -fsSL --retry 3 -o "$work/$name" "$base/$name"
curl -fsSL --retry 3 -o "$work/checksums.txt" "$base/gitleaks_${version}_checksums.txt"
expected="$(awk -v n="$name" '$2 == n {print $1}' "$work/checksums.txt")"
[ -n "$expected" ] || { echo "no checksum for $name" >&2; exit 1; }
actual="$(sha256sum "$work/$name" | awk '{print $1}')"
[ "$expected" = "$actual" ] || { echo "checksum mismatch for $name: $actual != $expected" >&2; exit 1; }
mkdir -p "$dir"
tar -xzf "$work/$name" -C "$dir" gitleaks
"$dir/gitleaks" version
