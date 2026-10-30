---
id: evidence:kinekino/20261029114138
title: Accepted Card canonical integration r1 — 2026-10-24
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Accepted Card canonical integration r1 — 2026-10-24

Evidence root: `runtime/verification/accepted-card-canonical-integration-r1/`.
Canonical adoption remains uncommitted; PM owns the root index. No stage, commit, push, GUI acceptance, new renderer run, or product-source repair was performed in this gate.

## Scope and results

- Canonical graph: source61 plus locked Cargo.lock B2E3D540665FFC8E68D24D0A6C531D2B983658FB19B5ABBA525659BA1ACC371D. `02-canonical-supply62.json` records exact supplied sources. Root parent f2365796824de011edcc52ac20ef653969eada45; Meari f85a68df7a6a8fee239d9655895ee15d5eaa5510.
- Candidate fmt passed. Default full ran once, including doctests: **1163 passed, 0 failed, 76 ignored**, 39 result lines. Card application5 and strict PNG inspection2 passed. Original `05-default-full.*` and extracted `08-default-results.json`, `18-default-totals.json` retain the evidence. Ignored actual-media tests were not run here.
- Default root all-target Clippy with `-D warnings` passed: `07-default-strict.*`.
- Feature library mapper was compiled, selected by its exact name, and passed once: `09-feature-mapper-build.*`, `10-mapper-list.*`, `11-mapper.*`. Exact name: `infrastructure::card_export::render::tests::parser_error_fields_and_heap_identity_survive_root_mapping`.
- Feature SDK/CLI parity actual existing2 passed once: `12-feature-parity-build.*`, `13-parity-list.*`, `14-feature-parity.*`, `parity-feature/`. The actual Cargo normal CLI producer was checked against the runtime alias before execution. Original assertions were unchanged. This legacy harness uses `wait_with_output`; no new supervised-lifetime claim is made.
- Feature all-target strict uses the PM-authorized compositional reuse in `06-feature-strict-reuse.json`: 59/62 source paths unchanged, production delta0; SDK test fixture2 covered by r3 selected strict, renderer test1 covered by its native strict. This is explicit reuse, not a fresh all-target feature invocation.
- Prior renderer actual r2 and root actual r1 results remain separate evidence. Template approval, glyph/readability/visual quality and unsupported cancellation/timeout capabilities are not promoted by these tests.

## Evidence exceptions and corrections

1. Initial admission's 5 differing rows represented 3 already-authorized canonical/PM paths. `00-admission-reconciliation.json` and fresh baselines retain this reconciliation; no product test retry resulted.
2. PowerShell missing-brace parse errors occurred before the artifact-copy/build command and before the final cache-audit command executed. `08-setup-correction.txt` and `19-setup-correction.txt` record them. No Cargo, scan or file mutation ran from those malformed commands; corrected orchestration continued without repeating product tests.
3. The artifact-copy convention derived `player-cli.pdb` from the hyphenated EXE; the actual CLI PDB uses `player_cli.pdb`. Initial final-producer lookup therefore stopped before restoration. `15-producer-lookup-correction.txt` preserves that failure. The exact current deps PDB was then copied directly and verified in `15-producers4.json`. The **default CLI PDB was not independently preserved before feature compilation; its default hash was not observed, and recovery is not claimed**. Default EXE/Cargo JSON/full/parity evidence is retained. PM explicitly accepted this debug-symbol evidence exception; no reconstructed PDB or test rerun was used.
4. The `09` feature mapper no-run invocation did **not** save a fresh RAM/GPU/disk/writer preflight. Its CPU2/mask3/BelowNormal descendant-job runner admission and terminal receipt are retained; they do not establish the missing resource measurements. Later invocations have their own fresh preflight. No retrospective measurement or rerun fills this gap.
5. Recorded PID45456 was the terminal mapper-list child (receipt closed 11:21:02.7389998Z). The present `conhost` was created at 11:22:13.6224485Z, so the numeric PID was reused. `17-recorded-PIDs.json` retains the original present observation; `17-PID-reuse.json` records classification. No action was taken against that process. PID coverage is the recorded launcher/child and download fixture identities only.

## Restoration and retained evidence

`15-restoration.txt` records exact S preimage62 and alias4 restoration. New normal producers remain in their producer namespace and have independent exact copies referenced by `15-producers4.json`. Worker2 were observed unchanged in SHA/mtime/FileID and were not written or restored. Alias unlink-and-move restoration intentionally changes FileIDs while retaining original bytes/mtime; original S file identities and original absences are audited separately.

Final audit files: `16-protection-audit.json`, `16-S-restored62.json`, `17-download-close6.json`, `17-command-receipts.json`, `17-EOF.json`, `17-recorded-PIDs.json`, `18-carry-returned.json`, `18-exceptions.json`, and final slot receipt. The final slot receipt distinguishes remaining observations from passed checks. Download close6 include the original intentional forced2; they are not all nonforced. No Meari mirror cleanup was performed; its previously isolated five-path cleanup exception remains separate.

Final audits passed: protection203, carry210, root pins6, read assets48 have zero differences; S22 existing SHA/mtime/FileID and original absences40 restored. Canonical62 remains exact. Cache11 differences are limited to four alias FileID exceptions and 14 metadata fields on four new normal producers; unclassified differences0. Worker2 and original cached library remain unchanged. New independent files19 extend carry210 to **229**. Streams30 were exclusively opened at EOF; nine original command receipts are terminal/exit0. Recorded numeric PIDs24 include the one later conhost reuse described above. `19-staged-paths.txt` is empty and `19-diff-check.txt` reports no diff-check error. Root application/mod.rs WIP and Meari docs WIP remain untouched.

## Applied skill provenance

Read and applied modoo.today `modootoday/software-qa/test-fixture-and-mock-build` body SHA a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100; `modootoday/software-qa/code-change-review-verification` body SHA 9f73865f96ec0cefcf3a4023b294400258473640df3b1d2add03dcca8192d64b and required `references/verdict-table.md` SHA 9d9619a829ae24d73645ef5f7c8126c8257caeddac80800632866257008ad2a9. Applied definition-based fixtures, original failure preservation, and distinct source/type/list/actual/quality verdicts. No foreign-language application/test policy was introduced.
