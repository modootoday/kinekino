#!/usr/bin/env bash
# Package the release binaries for one target as kinekino-<target>.tar.gz (zip on Windows)
# plus <archive>.sha256. usage: package.sh <target> <release-dir> <out-dir>
set -euo pipefail
target="$1"; bin_dir="$2"; out="$3"
name="kinekino-$target"
ext=""; case "$target" in *windows*) ext=".exe" ;; esac
rm -rf "${out:?}/$name"; mkdir -p "$out/$name"
# The file set comes from package-payload.txt; `cargo xtask package-check` verifies the result.
payload="$(dirname "$0")/package-payload.txt"
while read -r kind file _; do
  case "$kind" in
    ''|'#'*) continue ;;
    bin) cp "$bin_dir/$file$ext" "$out/$name/" ;;
    file) cp "$file" "$out/$name/" ;;
    *) echo "package.sh: unknown payload kind '$kind'" >&2; exit 1 ;;
  esac
done < "$payload"
cd "$out"
case "$target" in
  *windows*) archive="$name.zip"; rm -f "$archive"
    if command -v 7z >/dev/null; then 7z a -tzip "$archive" "$name" >/dev/null; else zip -qr "$archive" "$name"; fi ;;
  *) archive="$name.tar.gz"; tar -czf "$archive" "$name" ;;
esac
if command -v sha256sum >/dev/null; then sha256sum "$archive" > "$archive.sha256"; else shasum -a 256 "$archive" > "$archive.sha256"; fi
echo "$out/$archive"
