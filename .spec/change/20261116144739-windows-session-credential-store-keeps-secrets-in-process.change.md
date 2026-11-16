---
id: change:kinekino/20261116144739
title: Windows session credential store keeps secrets in process
status: active
created: 2026-11-16T14:47:39Z
updated: 2026-11-16T14:47:39Z
rel:
  depends_on: [adr:kinekino/20261029155443, adr:kinekino/20261030030000, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions]
  governs: [crates/kinekino-platform-windows/src/session.rs, crates/kinekino-player/src/infrastructure/frame_capture/process.rs]
---

# Windows session credential store keeps secrets in process

## Change
- Windows `SessionCredentialStore` was a placeholder that refused every call with `Unavailable`. The product's credential selection falls back to the session store exactly when the OS store is `Unavailable` (there is no Windows Credential Manager adapter yet), so on Windows the fallback store refused too and `kinekino-app --test credential_selection auto_falls_back_to_session_and_keeps_the_reason` failed (Windows run, issue 45). The Windows store is now the same process-lifetime map as the Unix one, with the shared empty-key check and a contract test.
- The frame-capture test `successful_run_removes_its_evidence_root` ran `/bin/echo`, which has no drive on Windows, so the owned launcher refused it as a non-absolute executable. The test now re-runs its own test binary (an absolute path on every OS) as the child, through a new ignored fixture `echo_child`, so it needs no platform `cfg` outside the adapter crates.

## Migration
- None. No persisted data or public API changes. The Credential Manager adapter stays unwritten; `open_os_store` still reports `Unavailable` on Windows.
