---
id: change:kinekino/20261116084158
title: Release archives follow one payload list checked by xtask
status: active
created: 2026-11-16T08:41:58Z
updated: 2026-11-16T08:41:58Z
rel:
  depends_on: [adr:kinekino/20261109101726, adr:kinekino/20261029201516, adr:kinekino/20261116062843]
---

# Release archives follow one payload list checked by xtask

## Change

The release archive contents moved from a hard-coded loop in `.github/scripts/package.sh` to `.github/scripts/package-payload.txt`. `cargo xtask package-check` (`xtask/src/package_check/`) compares a finished archive with that list. The release workflow's Package step now runs the check right after `package.sh`, so a release (or dry run) fails when the archive and the list differ. The archive contents are unchanged: three binaries plus LICENSE, README.md and THIRD_PARTY.md.

## Migration

Add a file to release archives by adding a line to the payload list; nothing else needs to change. Slice (c), the `.deb` builder, will read the same list.
