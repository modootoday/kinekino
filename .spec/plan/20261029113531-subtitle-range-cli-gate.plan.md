---
id: plan:kinekino/20261029113531
title: Subtitle range API r2 CLI gate
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Subtitle range API r2 CLI gate

Completed on explicit subsequent PM allocation: [actual verification](../../sots/verification/application/subtitle-range-cli-2026-10-20.md). Instructions below preserve the prepared plan and its original stop-on-failure policy. Actual input was SDK58r2+subtitle13 and approved parity fixture repair1; original parity101 retained, final strict/fmt passed, source13/protected583 unchanged, slot and snapshot returned. No further execution is implied by this document.

Prepared only; no overlay/Cargo/CLI execution authorized until PM assigns the slot after SDK native and build4 gates. Source r2 remains frozen. The actual execution base is the SDK-accepted public-p3-parent-r1 snapshot with final union58/r2, then an explicitly allocated exact subtitle union13 overlay. The old b0+Meari1df development checkout alone is no longer the planned CLI acceptance environment. PNG API is excluded. Root live files/index/Cargo are never overwritten; PM must centrally merge shared API diffs.

## Input identity

Exact development source checkout: `%USERPROFILE%/.codex/worktrees/subtitle-range-export/com.kinekino.player`.
Root base: `b0c19f17a59f54a6469361b05b7fbae3ef349a85`.
Accepted package commit in live root: `eaeb45460d5663ea96b8291abc7d73065e9b5c83`. This independent checkout contains the same seven package file bytes as edits over the older base; it is not claimed to have that commit as HEAD.

Package manifest: `runtime/coordination/subtitle-range-export-r1/source7.json`, SHA256 `B092B98BB9D08FC021D7A37ED8EDADF7895973F8D967F7443E3CA0413233B3B2`.
API manifest: `runtime/coordination/subtitle-range-api-r2/source6.json`, SHA256 `329BBB60D0AF56F34B4BD20173BABAADAF2C43D619E88A9315BCD4B602629815`. API manifest has no source_root field; checkout is specified above.

Exact union13, with each expected SHA supplied by its immutable manifest:

1. packages/media-subtitles/src/application/convert.rs
2. packages/media-subtitles/src/application/export_range.rs
3. packages/media-subtitles/src/application/mod.rs
4. packages/media-subtitles/src/domain/subtitles/mod.rs
5. packages/media-subtitles/src/domain/subtitles/range.rs
6. packages/media-subtitles/tests/export_range.rs
7. packages/media-subtitles/tests/export_range_cli.rs
8. src/application/subtitles/mod.rs
9. src/application/api/client.rs
10. src/application/api/capabilities.rs
11. src/application/api/queries.rs
12. src/application/api/schema.rs
13. tests/subtitle_range_cli.rs

Recheck these hashes and base before/after execution; source7 is unchanged by API6. Approved local environment restoration is Meari base gitlink `1df44812c11e61b7e8c000324e35ce461b0a53b0`, tree `0efb1c6f336694ca0289ddd9771d6db9da256ff5`, archive SHA256 `EDF5ECEC03A62B8644918081477385BE047DDB86FF7FA3C5BB5B29A2E443B78F`, extracted only into this checkout/packages/meari. Original package compile-before-source failure test101/173ms remains in the prior package evidence directory. No submodule update or network is needed.

### Actual combined execution base and read-only preparation

Execution root will be `%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/public-p3-parent-r1`, currently PM-owned. Its accepted input is C21+b0c+launcher2, producer49443cf overlay53 and final SDK5 r2; union58 manifest `runtime/quality-review/sdk-shared-native-child-r1/r2-union58.json` SHA256 `95618D9F71902A623C56FEBC67F4CF270952C4F1FF61FED31BC3062B64C6A267`. SDK5 final pin `FC7BF70F94EF7B0A8705A3B1DD936B5E3CBB806AEEBC152D7774782445C13946`. PM live HEAD observed `def8a2e59ad7ac94ae166ff5443ed3bd0c8c867b`; the snapshot is not falsely labeled as a Git checkout at that HEAD. The old restored Meari1df applies only to earlier independent package testing; combined execution preserves accepted producer494 instead.

Read-only preparation stored under `runtime/coordination/subtitle-range-combined-prep-r1/`:

- expected13.json SHA `C662D124333BFA14CA5CDBD30D2ED78AC60227D381062C51E8058913C991FE8D`: exact expected overlay paths/hashes from frozen manifests.
- before13.json SHA `232B5BCD76DB5D7DDB457B1F75F7CA129E2E27EABA1D27CBAB840E63686D9136`: current snapshot eight present files with original SHA/size and five absent files. Existing bytes copied into before/ outside the snapshot. No overlay performed.
- protected-current.json SHA `3B812AD8515D9F49A48B06E77BC9AA4D2D42578ADE928BD6D08B1D1486DFB918`: 583 unique current protected paths from prior protected527, final SDK union58 and producer394 references, excluding the intentional overlay13 paths. This is a current-byte baseline for later comparisons, not a claim of independently rerunning SDK tests.

Absent5: package application/export_range.rs, domain/subtitles/range.rs, package tests/export_range.rs and export_range_cli.rs, root tests/subtitle_range_cli.rs. Present8: package convert.rs/application mod/domain mod, root subtitle mod, API client/capabilities/queries/schema. Their before bytes and hashes are individually in before13.json; no nonexistent file receives a fabricated old hash. Read-only client/capability diffs after ignoring line-ending differences show solely the intended range method/command and storage-capability extension, with no PNG additions.

Protected set includes root Cargo.toml SHA `97B3F652B039A9D99931C51BF344A0B0AD25CDC32C15052F5276E1B753DEAE25`, Cargo.lock SHA `7CC98A9406B50CD03BA69B4FE7B95914DE1F5A1771985A4831EA24918C567B38`, Meari producer/native lifecycle, SDK native child/router/binding/dispatcher files, UI C21/launcher/frame and existing bridge sources/manifests. Runtime/build caches are not source overlay targets. Recheck source13, before13 and protected baseline immediately after allocation because the snapshot is still PM-owned; any unexpected drift stops overlay for central reconciliation.

Combined gate uses root snapshot Cargo manifest and existing `.../native` target cache for player-cli and root tests/default strict. The separate SDK product bridge uses tools/player-ui-sdk-bridge manifest and `sdk-product-parent-consumer` cache; those are not the CLI execution target, and earlier bridge product4 success cannot substitute for range CLI acceptance. No new cache/target or source download is planned. Root default CLI validation on SDK58 does not repeat native/bridge35 tests or prove PNG, GUI/OS parent lifecycle acceptance.

## Serial commands after allocation

Run only after PM assigns snapshot overlay and execution ownership, from the exact combined snapshot after expected union13 is applied and all 583 protected paths remain unchanged. Create a new evidence root; do not overwrite an earlier attempt. Canonical existing runner SHA256 `692C22751FAC7F9999F2D833571DAB6605AFD58E66711370494E62DD8A62C463`; existing native cache, 2CPU/BelowNormal/descendants, offline/locked/j1 and test threads1. Do not use build4 merely because its preceding gate passed; execution budget remains PM's explicit allocation.

```powershell
$taskRunner='D:/workspace/@myso-kr/com.kinekino.player/runtime/quality-review/frame-document-cli-r1/final-bin/resource-runner.exe'
$taskExecution='%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/public-p3-parent-r1'
Set-Location -LiteralPath $taskExecution
$taskEvidence='D:/workspace/@myso-kr/com.kinekino.player/runtime/verification/subtitle-range-api-r2-cli-r1'
New-Item -ItemType Directory -Path $taskEvidence
$env:CARGO_TARGET_DIR='%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/native'
$env:RUST_TEST_THREADS='1'
$env:SUBTITLE_RANGE_EVIDENCE_DIR="$taskEvidence/range-files"
New-Item -ItemType Directory -Path $env:SUBTITLE_RANGE_EVIDENCE_DIR
$env:SDK_CAPABILITY_EVIDENCE_DIR="$taskEvidence/capability-files"
```

The range harness expects its evidence root already exists; parity harness itself creates capability-files, so do not precreate that child. Both must be unused paths. Preserve runner stdout/stderr/receipt for every step, inspect terminal/exit before advancing. Stop on first failure, retain original evidence, and obtain a separate source pin for any actual code repair.

```powershell
& $taskRunner "$taskEvidence/build.stdout" "$taskEvidence/build.stderr" "$taskEvidence/build.receipt.json" cargo build --manifest-path "$taskExecution/Cargo.toml" --bin player-cli --offline --locked -j1
$taskCli="$env:CARGO_TARGET_DIR/debug/player-cli.exe"
Get-FileHash -LiteralPath $taskCli -Algorithm SHA256 | ConvertTo-Json | Set-Content -LiteralPath "$taskEvidence/cli-pre.json" -Encoding utf8
& $taskRunner "$taskEvidence/range.stdout" "$taskEvidence/range.stderr" "$taskEvidence/range.receipt.json" cargo test --manifest-path "$taskExecution/Cargo.toml" --test subtitle_range_cli --offline --locked -j1 -- --test-threads=1
& $taskRunner "$taskEvidence/conversion.stdout" "$taskEvidence/conversion.stderr" "$taskEvidence/conversion.receipt.json" cargo test --manifest-path "$taskExecution/Cargo.toml" --test subtitle_conversion --offline --locked -j1 -- --test-threads=1
& $taskRunner "$taskEvidence/parity.stdout" "$taskEvidence/parity.stderr" "$taskEvidence/parity.receipt.json" cargo test --manifest-path "$taskExecution/Cargo.toml" --test sdk_capability_parity --offline --locked -j1 -- --test-threads=1
Get-FileHash -LiteralPath $taskCli -Algorithm SHA256 | ConvertTo-Json | Set-Content -LiteralPath "$taskEvidence/cli-post.json" -Encoding utf8
& $taskRunner "$taskEvidence/strict.stdout" "$taskEvidence/strict.stderr" "$taskEvidence/strict.receipt.json" cargo clippy --manifest-path "$taskExecution/Cargo.toml" --offline --locked -j1 --all-targets -- -D warnings
& $taskRunner "$taskEvidence/fmt.stdout" "$taskEvidence/fmt.stderr" "$taskEvidence/fmt.receipt.json" cargo fmt --manifest-path "$taskExecution/Cargo.toml" --check
```

These are sequential step instructions, not a script to run blindly after a failed predecessor. Strict/fmt scope must remain within the final PM allocation. Compare cli-pre/cli-post and each range fixture's cli_sha256; all must match. Range tests use CARGO_BIN_EXE_player-cli, not a stale copied executable or env-selected substitute. Archive the exact tested executable after passing and record its hash if a user/SDK runnable copy is delivered. No extra inference, downloads, worker builds or GUI runs.

## Acceptance and evidence

Active range target has four tests and 19 fresh CLI processes if all finish:

| Test | CLI requests | Evidence |
| --- | ---: | --- |
| typed client/clip/no-overwrite | 2 | exact distinct cue text/order, both clips, 0 rebase, typed worker output equivalence, collision5, source preserved |
| precision/invalid/empty | 8 | NeedsInput3, explicit approval success, empty/collapsed/reversed/overflow/numeric/unknown-field2, no unintended files |
| discovery/schema/context | 5 | capabilities/schema, strict required payload/range, three Unsupported context3 and no files/source unchanged; unavailable adapter is an additional SDK call |
| VTT→SRT/speaker/outside loss | 4 | prefix output bytes, outside-selection parser loss still needs approval, source unchanged |

Every actual new-range CLI call saves request/response/exit in its retained fixture directory. Each fixture files.json lists input/output/call file bytes and SHA256 plus actual CLI path/hash; retained files allow independent inspection. Failed/nonwriting request evidence is its response/exit plus explicit absence assertions, not an invented missing-file hash. Optional evidence is mandatory in this gate. No per-request publication receipt is fabricated; these are test receipts plus actual filesystem bytes/digests.

Keep denominator groups separate: range4 (new), existing subtitle_conversion9 active +1 external corpus ignored at current source, sdk_capability_parity2 active. Runtime stdout is authoritative if counts differ; never add ignored corpus or package CLI placeholder to pass count. Parity makes additional CLI calls beyond the new range19; report separately rather than claiming19 is total gate processes. Existing package unit8 includes range3, so never count range3 again as additional to8. Package integration4 and package CLI ignored1 belong to the earlier package gate, not this new root CLI acceptance.

Known remaining test limitations: UnavailableStorage capability status has no direct dedicated assertion, and precision-success output bytes are not directly asserted (success/loss gate is asserted, baseline output bytes are covered by another case). Keep these limits after a pass. No command support claim until fresh CLI/typed/schema/error agreement and original preservation/no-overwrite assertions actually pass. Root live integration requires central diff merge and its own input identity; independent checkout success does not prove a later merged executable.

At completion record source13/base/executable pre/post identity, original handles/terminal receipts and absence of owned processes, then explicitly return the single heavy slot to PM. Do not kill unrelated processes or advance another session's gate. Update the SOT and remaining plan with observed facts only.

Skill application: modoo.today `modootoday/product-planning/requirement-to-testable-stories` body previously read, applied normal/failure/boundary acceptance and explicit source/execution dependencies. `modootoday/software-qa/code-change-review-verification` body/reference previously read, used to separate source assertions, actual execution, unsupported/ignored cases and retained-artifact evidence. No new non-Rust application/validation logic is introduced; PowerShell snippets only launch allocated existing tools and record file metadata.
