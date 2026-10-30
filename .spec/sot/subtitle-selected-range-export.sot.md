---
id: sot:kinekino/subtitle-selected-range-export
title: Subtitle selected-range export
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Subtitle selected-range export

2026-10-20 PM assigned source implementation; functional acceptance pending.

## Ownership and source

Independent checkout: `%USERPROFILE%/.codex/worktrees/subtitle-range-export/com.kinekino.player`, base `b0c19f17a59f54a6469361b05b7fbae3ef349a85`.
Exact seven source files: `runtime/coordination/subtitle-range-export-r1/source7.json`, SHA256 `B092B98BB9D08FC021D7A37ED8EDADF7895973F8D967F7443E3CA0413233B3B2`. Snapshot lives beside manifest under `source/`.
PM explicitly approved minimal `application/convert.rs` shared helper extraction. Existing ConvertSubtitles DTO/schema/result, errors, parser, loss approval, render and publication remain the shared implementation; ordinary conversion supplies an identity transform. No root SDK/Cargo/index changes.

## Contract

Proposed central command `subtitles.export_range` is currently unregistered/unsupported. Package hook: `application::export_range::{execute, execute_typed, schema}`; typed request `ExportSubtitleRange`. Existing conversion fields plus required `range: {in_us: decimal-string-u64, out_us: decimal-string-u64}`. Result reuses ConversionResult.

Select strict half-open overlap, clip both ends, subtract selected range start with checked integer arithmetic. Preserve cue order and text, input file and original CueDocument. No-overwrite uses existing SubtitleStorage publication and media-file-publication, with no new storage policy.

Invalid/reversed/empty selection, invalid cue interval, arithmetic failure, and any cue collapsing after the existing renderer's millisecond truncation return InvalidArgument before publication, even with allow_loss=true. No silent dropping or empty successful exports. Nonintegral millisecond endpoints add timestamp_precision_reduced; existing allow_loss approval applies. Parser losses from the complete input remain conservatively reported, including losses outside the selected range.

## Evidence and limits

Written: three domain tests, four package integration tests using real SubtitleFiles, one explicitly ignored fresh-process CLI integration test. These cover overlap boundaries, original preservation, Unicode, checked u64 extremes, precision approval, invalid/empty/collapsed inputs, no-overwrite and ordinary conversion regression.
Observed: exact-source rustfmt --check and git diff --check exit0. Cargo fmt/test/Clippy and all test execution remain unperformed under central allocation restrictions. The ignored CLI test requires central command registration plus freshly built, pinned SUBTITLE_RANGE_CLI_EXE; ignored is not a pass or support claim. No inference/download/worker/GUI execution.

### Subsequent central package gate

PM allocated the single heavy slot. Evidence: `runtime/verification/subtitle-range-export-r1/`. Original `test.*` failed before compilation (exit101/173ms): independent checkout lacked Meari source needed to resolve the root workspace blitz-paint patch. Failure is preserved. PM approved local Git archive restoration of exact base gitlink `1df44812c11e61b7e8c000324e35ce461b0a53b0`, tree `0efb1c6f336694ca0289ddd9771d6db9da256ff5`. Archive SHA256 `EDF5ECEC03A62B8644918081477385BE047DDB86FF7FA3C5BB5B29A2E443B78F`; destination is solely this independent checkout/packages/meari, non-link directory. Archive entry listing had no absolute/parent-traversal entries. Root Meari and Cargo files were not modified, network/download0.

Unchanged source7 then passed test-r2 (14309ms), all-target strict Clippy (4299ms), cargo fmt --check (1615ms). Package totals: existing parser/render domain5 + new range domain3 + real-file integration4 =12 passed,0 failed. Integration4 includes one ordinary conversion/loss-gate regression; it is not root freshCLI conversion regression. CLI1 ignored/0 executed, doc0 tests not counted. No new package source fixes were required.

Canonical runner SHA256 `692C22751FAC7F9999F2D833571DAB6605AFD58E66711370494E62DD8A62C463`, native existing cache, offline/locked/-j1 and RUST_TEST_THREADS1. All receipts terminal=true, affinity3/logical_cpus2/BelowNormal16384/descendants=true. Successful runner/child handles: test-r2 108300/112896, strict72876/120644, fmt113012/90264; original failure46132/88072. These exact processes were absent after completion; owned heavy empty. All seven source hashes still match the frozen manifest and Cargo.toml/Cargo.lock diff is empty.

Actual real-file test asserted exact source/output bytes and preserved output after no-overwrite rejection. Fixture cleanup intentionally removed the temporary files, so separate input/output SHA artifacts were not retained; source pin plus exact-byte assertions and raw test receipt are the available evidence, not an independent artifact-digest receipt. This limitation is reported to PM. Domain fixture uses identical text across cues, so it alone does not distinguish text ordering; real-file clip fixture uses distinct texts and checks exact output. VTT-to-SRT range export, speaker policy and parser losses outside the selected range have not received direct new export tests. Root CLI integration and root regression gate remain pending.

## Skill application

Source: modoo.today `modootoday/product-planning/requirement-to-testable-stories`, full body read, no additional references supplied. Applied Given/When/Then normal/failure/boundary acceptance and explicit dependencies to range selection, precision approval and publication tests. Unrelated domain search matches were rejected. Rust examples and all new tests remain Rust. Review skill `modootoday/software-qa/code-change-review-verification` distinguishes source claims from executed evidence; no functional completion asserted.

## SDK/API/CLI integration candidate

PM accepted package7 as root commit `eaeb45460d5663ea96b8291abc7d73065e9b5c83` and assigned six additional source paths solely in the independent checkout: subtitle module reexport, API client/capabilities/queries/schema, root `tests/subtitle_range_cli.rs`. Frozen manifest `runtime/coordination/subtitle-range-api-r1/source6.json` SHA256 `125A96D378A156B2030D80E498D3EB7FAA460CDC4F4C091A192E7A038F9D7032`; files are copied beside it under source/. Package7 and existing conversion methods/DTO remain unchanged.

New object-safe ApplicationClient::export_subtitle_range returns existing ConversionResult through shared review_call transport. Known command and adapter-dependent capability, strict request schema/example and dispatch use the same storage availability and unsupported revision/idempotency/timeout guards as conversion. No GUI command or shell execution is used; existing conversion error text/order remains unchanged.

Four active root integration tests are written: typed worker client/freshCLI exact output equivalence + no-overwrite/source preservation; loss approval/empty/collapsed/invalid wire; capability/schema/unavailable adapter/context agreement; VTT→SRT prefix and outside-selection speaker loss approval. These are source-only candidates, not executed proof. The original package CLI ignored test remains separate and was not converted into a package pass. Optional SUBTITLE_RANGE_EVIDENCE_DIR retains synthetic input/output bytes, each actual CLI request/response/exit code and per-fixture files.json with file/CLI SHA256. Central gate must create this evidence root before running. Without this option fixtures are removed normally.

Observed only direct rustfmt --check and git diff --check exit0 for integration6. Cargo/root freshCLI and existing conversion/parity regression are pending the SDK native gate and explicit PM allocation. No live root/index/Cargo/shared snapshot modifications.

Current integration pin is r2: `runtime/coordination/subtitle-range-api-r2/source6.json`, SHA256 `329BBB60D0AF56F34B4BD20173BABAADAF2C43D619E88A9315BCD4B602629815`. Only the root test file differs from r1: explicit required-payload schema assertion and original-file equality after all Unsupported context requests. Implementation five files and package7 unchanged. r1 is preserved, not overwritten; execution remains pending.

Subsequent actual combined gate completed without changing r2: [verification](../../verification/application/subtitle-range-cli-2026-10-20.md). Tested SDK58r2 snapshot plus subtitle13 supports this CLI command with available storage; range4/19actualCLI, existing root conversion9, parity2 and default strict/fmt passed. Parity required a separately pinned test-only valid-input fixture repair; original failure retained. Frozen executable40CCB875…11AAC4, retained per-request/file-digest evidence, protected583 delta0 and owned[]; PM received heavy/snapshot. Live root API integration/any merged binary acceptance remain separate.
