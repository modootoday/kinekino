---
id: change:kinekino/20261116111105
title: Dependabot holds windows-core to the windows crate generation
status: active
created: 2026-11-16T11:11:05Z
updated: 2026-11-16T11:11:05Z
rel:
  depends_on: [sot:kinekino/shared-automation-rules, adr:kinekino/20261029201516, adr:kinekino/20261116062843]
  governs: [.github/dependabot.yml]
---

# Dependabot holds windows-core to the windows crate generation

## Change

`.github/dependabot.yml` ignores `windows-core` from 0.63 on.

Dependabot proposed `windows-core` 0.62.2 → 0.100.0 (pull request 2, closed). Measured on that branch, `cargo check --locked -p kinekino-player --target x86_64-pc-windows-msvc` failed with E0277 because two `windows_core` versions were in the graph; the same check passes on `main`. `windows-core` 0.100.0 has no matching `windows` release: the newest `windows` is 0.62.2 and needs `windows-core ^0.62`, and `webview2-com` 0.39.1 needs `windows` and `windows-core ^0.62`. The required checks run on Linux, so that pull request was green while the Windows build broke.

`windows`, `windows-core` and `webview2-com` move together. The ignore rule is removed when `windows` and `webview2-com` publish the next generation.

## Migration

None.
