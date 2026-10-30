---
id: evidence:kinekino/20261029114601
title: "Selected subtitle range: actual combined SDK/CLI gate"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Selected subtitle range: actual combined SDK/CLI gate

2026-10-20. Evidence root `runtime/verification/subtitle-range-api-r2-cli-r1/`. [Gate plan](../../../plans/player/subtitle-range-cli-gate.md), [contract](../../player/application/subtitle-range-export.md).

## Exact input and deliverable

PM allocated public-p3-parent-r1 snapshot plus single heavy. The actual execution root was `%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/public-p3-parent-r1`, accepted SDK union58 r2 (manifest95618D9F71902A623C56FEBC67F4CF270952C4F1FF61FED31BC3062B64C6A267, producer49443cf), not merely the older independent development checkout. Pre-overlay drift was empty, eight original files and five absent paths retained, and exact subtitle package7+API6 were overlaid with fresh mtime. expected13/overlay13/final-source13 all SHA256 `C662D124333BFA14CA5CDBD30D2ED78AC60227D381062C51E8058913C991FE8D`. Original package/source6 manifests remain unchanged.

Fresh CLI pre/post/final SHA256 `40CCB875E077D7BBA39A1AB96D3C6E16E80615C90470D66D35E8AA119911AAC4`, matching every range fixture cli_sha256. Preserved executable: `runtime/verification/subtitle-range-api-r2-cli-r1/final-bin/player-cli.exe`. This binary supports subtitles.export_range through the common package use case when file adapter is available. Live root API integration/central commit and any later merged executable are separate; no GUI acceptance is claimed.

## Actual results

| Stage | Result | Elapsed ms |
| --- | --- | ---: |
| Fresh player-cli build | exit0 | 102283 |
| New active range CLI | 4 passed,0 failed/ignored; 19 actual CLI requests | 33484 |
| Existing root conversion | 9 passed,1 external corpus ignored | 37687 shared receipt |
| Additional workspace plugin conversion target | 3 passed, distinct tests | same conversion receipt |
| Original capability parity | 1 passed,1 failed, exit101; retained | 43676 |
| Repaired capability parity | 2 passed,0 failed, exit0 | 39528 |
| Default all-target strict Clippy | exit0 | 31527 |
| Cargo fmt check | exit0 | 2963 |

The root acceptance targets are 4+9+2=15 passing tests; the additional workspace plugin3 are reported separately (18 executed passing tests across these final targets). Original failed parity and earlier package12 are not added as extra passes. Package CLI placeholder and external corpus remain ignored/unexecuted.

Range coverage executed: strict schema/known-supported-discovery, unsupported context no write, unavailable storage SDK path, typed object-safe worker/client vs CLI exact output, Unicode/distinct cue order, both clips and selected-start rebase, collision/no-overwrite/original immutability, explicit precision-loss approval, empty/collapsed/reversed/overflow/numeric/unknown wire rejection, VTT→SRT prefix and outside-selection speaker loss approval. Range-files retains four synthetic fixture directories, 19 call JSONs with actual request/response/exit and actual source/output files, each files.json containing file byte lengths/SHA256 and tested CLI identity.

Repaired capability evidence has 72 command schemas,66 unsupported dispatches,3 malformed output-review requests and1 discovery:142 actual CLI calls. SDK and CLI responses match. These are separate from range19 and conversion CLI calls; no total gate subprocess count is invented. Capability harness's pinned executable copy is retained in capability-files-r2.

## First failure and minimal test-only repair

Original unsupported-051.json shows SDK and CLI both returned InvalidArgument/exit2 for `{}` passed to proposals.inspect_output_review_target, missing proposal_id. Existing output-review order is context→strict deserialization→typed supported check. This differs from generic proposal adapter-first handling but is not itself a product defect; original parity assumed all unsupported commands must return Unsupported for malformed empty payload.

PM and SDK owner approved exact1 test-only fixture repair, preserving product code and source13. `runtime/coordination/subtitle-range-parity-fixture-r1/source1.json` SHA526DE9151C49E198A20E863F548A94C4DC8ED4CCDD1F4C8998A039A7F40D636D; repaired tests/sdk_capability_parity.rs SHA2A244426B79E8A411FD073375D2EB2676FDE78AE4C6B8E1F5DC6008DE6D721D1; repair.diff SHA87F5C8B12279F757B19D3A08C4FC7071EF5DA2986D8E0A376B813DF50BE246BD.

Only three output-review commands receive explicit well-formed fixtures; record supplies current criteria identity, syntactically valid typed target/decision and expected_revision2. Each original malformed request is separately asserted as identical SDK/CLI InvalidArgument/exit2. Record malformed failure may come from missing expected revision in context; it is not described as necessarily a payload deserialization error. No blanket acceptance of arbitrary errors, exclusion of commands or production guard reordering occurred. Original parity logs/receipt/CLI bytes are retained; only parity reran, with new evidence directory, before remaining strict/fmt. Previously passing range/conversion were not needlessly repeated.

Pre-repair snapshot file bytes for this unexpected test repair were not independently copied before replacement. repair-before/tests/sdk_capability_parity.rs is a reconstruction from unchanged root source version, SHA19D9F0D7696AA50254AC0B65967F9FB4DDF64E75E99961876905F20AE25C1E38; do not claim that hash was independently observed from the pre-repair snapshot. Original failure source/test output and minimal Git diff are preserved. Source13 before bytes remain separately captured.

## Protection, ownership and limits

Final protected583 delta[] and source13 exact equality. tests/sdk_capability_parity.rs was never part of original protected583, so adding this repair does not reduce protection to582; final scope is source13+repair1 and unchanged protected583. Root Cargo/lock, SDK native58/producer394, UI C21/launcher and bridge sources were not modified. No PNG overlay or live root/index changes.

Canonical runner SHA692C22751FAC7F9999F2D833571DAB6605AFD58E66711370494E62DD8A62C463; every receipt affinity3/logical_cpus2/BelowNormal16384/descendants=true; native existing cache/offline/locked/j1/testthreads1. No new runner adoption, cache, model download, inference, worker or GUI execution. Seven original receipts including failure are terminal;14 recorded launcher/child PIDs absent and final-owned=[]. Heavy and snapshot explicitly returned to PM; snapshot retains exact subtitle13+repair1 for central integration, not rolled back or replaced from an independent whole tree.

Remaining test limits: no dedicated assertion of UnavailableStorage's capability entry; precision-success path checks approval/success but not its exact bytes independently. These do not become extra success claims. New VTT→SRT/speaker/outside-selection tests now executed, so those earlier source-only gaps are resolved for these specific fixtures. Native surface/GUI/full product tests and later PNG/API merge remain separate.

Skills: modoo.today requirement-to-testable-stories applied normal/failure/boundary assertions; code-change-review-verification applied source-vs-execution claims and first-failure preservation. Rust fixture repair reused typed contracts and current criteria rather than changing product policy or adding non-Rust validation logic.
