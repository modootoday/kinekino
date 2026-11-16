---
id: change:kinekino/20261116155827
title: Historical verifier spawns its verified program
status: active
created: 2026-11-16T15:58:27Z
updated: 2026-11-16T15:58:27Z
rel:
  depends_on: [adr:kinekino/20261116151137, change:kinekino/20261116153944, adr:kinekino/20261029155443, adr:kinekino/20261029163426, adr:kinekino/20261029164425, adr:kinekino/20261030030000, adr:kinekino/20261030120000, adr:kinekino/20261030180000, adr:kinekino/20261030190000, adr:kinekino/20261030200000, guide:kinekino/ignored-tests, plan:kinekino/20261030013900, sot:kinekino/conventions]
  governs: [crates/kinekino-player/src/infrastructure/historical_evidence/mod.rs, crates/kinekino-player/src/infrastructure/historical_evidence/tests.rs]
---

# Historical verifier spawns its verified program

## Change
- This completes slice (c) of [[adr:kinekino/20261116151137]] for the historical verifier.
- `HistoricalVerifier::verify` re-hashed its pinned executable through a guard but then spawned it by path. On POSIX another process could rename a file over that path between the hash and the spawn.
- It now asks `OsPolicy::pinned_program` for the program and spawns that while the guard is held. On Linux that is the verified handle.
- On POSIX, a verifier that is a `#!` script is refused with `InvalidArgument` before anything runs, and so is a file under a location other users can write. Windows keeps spawning the path under its share-mode guards.
- Tests:
  - `script_verifier_is_refused_where_sharing_is_advisory` fails without this change, where the script is spawned and the run fails with `EngineFailed`.
  - `pinned_verifier_runs_its_verified_program` runs a pinned native `true` through to its empty output. It skips when no `true` is on `PATH`.

## Migration
- None. A verifier must already be a native executable; one installed in a group- or world-writable location must move on POSIX.
