---
id: evidence:kinekino/20261029114547
title: Manual native skip negative/owner default CPU verification — 2026-10-24
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Manual native skip negative/owner default CPU verification — 2026-10-24

Status: CPU gate passed; private candidate remains unpromoted. No N/V/O actual, GUI, inference, download or full-suite execution. PM owns root/index. Root daff354372810824292935578869499e39fec5e4 unchanged.

Evidence gate: runtime/verification/manual-native-skip-default-cpu-r1/. Source handoff and approved plan: runtime/coordination/manual-native-skip-negative-owner-source-r1/HANDOFF.md and CPU-GATE-PLAN.md.

## Source and execution

source4 manifest 6FCEAAFCBF9D56B4C3DF3F134A4C167BE7220BF3C35556930F8372E7B7C93039; before4 69F19BC3BF9CCBFBABA4B2270F7118878FF2DD0555905D7D797F22D39B4352D8. Existing root13 plus private4 overlap parent1 => union16; required integration parents/helpers6 matched small-source hashes before and after. Root13 final hashes unchanged. Existing S/T reused, no broad sync, new target, dependency changes or cache-wide content hashes.

Sequential conservative2 owned Job receipts report logicalCPU2, affinity3, BelowNormal16384 and descendants covered. Cargo offline/locked/j1, thread1. fmt check exit0/4276ms; default all-target clippy -D warnings exit0/10726ms; default native_skip_cli no-run JSON exit0/50978ms. Exact JSON artifacts features=[]: integration test=true, CLI and player test=false. No fallback bin build (0).

New ignored names each appear exactly once in both full and ignored lists:
- actual_export::actual_same_instance_manual_export_negative_contract
- actual_export::actual_manual_override_and_original_owner_contract

Affected target nonignored run: 35 passed, 0 failed, 7 ignored, receipt2394ms. This checks affected CPU contracts and compilation, not the ignored N/V/O actual bodies. Accepted pure13/endpoint-loss1/full/normal-positive/UIactual evidence was not rerun or upgraded to new whole-source success.

## Fixed artifacts

06-cargo-executables.json retains original Cargo records; 06-fixed6.json pins independent EXE/PDB copies and distinct producer/fixed FileIDs. Normal CLI is bin/player-cli/player-cli.exe SHA256 7695C7B68C66778C230BBB7B55E57287F15ED6EF11C0548111D9E20717A2F51B (32482816 bytes). Integration is bin/native_skip_cli/native_skip_cli-db61470599f87131.exe SHA256 B2869D15D12F4323030698BB3DEB0A7465D78991632E750C672B2E7CC1D10421 (6120960 bytes). Generated default player and all three PDBs are preserved because normal producer aliases also changed. UI C00FD4AA native-feature CLI and validation CLI were not reused.

## Restoration and approval-review event

02-preimage16.json captures fresh S preimages. Existing9 restored with SHA/mtime/FileID equality; original-absent7 moved out of S into gate/displaced-source, and four newly empty directories moved into gate/displaced-empty-dirs. 10-restored16.json confirms original existence/content/metadata. Root source/index writes0.

The first combined restoration command was rejected by automatic approval review at CreateProcess, before any mutation. No detailed policy reason was returned. Original tool transcript is preserved in this chat; 09-auto-review-rejection.txt records the visible error and boundary. This is a tool-action rejection, not a product/test failure. No deletion command was retried. The safer accepted sequence first retained fixed artifacts/producer identity, moved aliases at four exact absolute literal paths into gate custody, then copied completed independent originals; source restoration used verified S-contained fullpaths and reversible moves rather than deletion. No cleanup is pending for S or original aliases; displaced files remain preserved evidence.

Original alias4 content restored from previously completed independent preimages; content SHA reused, no fresh large rehash. New alias FileIDs are four explicit exceptions. New default normal producer4 retained in deps and independently fixed; cache changes classified, not restored over Cargo fingerprints. 11-cache11-returned.json is current cache11 baseline. carry-fixed265 adds only fixed6 to previously returned carry259. Displaced aliases are retained custody outputs, not extra accepted product artifacts.

Protected source62/protection203/rootpins6/carry259 had zero fresh size/mtime differences. These broad protection observations are metadata checks, not fresh content/FileID attestation. Rootpins6 small hashes and root13/required parents were separately checked. No unclassified metadata/cache change observed.

12-original-receipts6.json: terminaltrue/exit0 for all six owned commands. 12-EOF12.json records readable completed stdout/stderr EOF12; 12-PID12.json shows recorded numeric PID12 absent. Writer0, no outstanding original command owner. No claim of enumerating every compiler subprocess identity beyond the Job receipts.

## Remaining work

PM must accept this CPU return before separate N then V/O actual allocation. Run each once, stop first failure, use fixed default CLI/integration above and original inputs/deadline/owner contract. No source fixes or retries occurred in this slot. CLI technical acceptance is separate from AI quality: real product GUI and real samples plus explicit user judgement remain necessary; no user quality approval is claimed.

Skills applied: modoo.today requirement-to-testable-stories d28d17c26477c64a2121fcf097467e688df2372b10b086b545649c5c3e0d93cb; code-change-review-verification 9f73865f96ec0cefcf3a4023b294400258473640df3b1d2add03dcca8192d64b and required verdict-table 9d9619a829ae24d73645ef5f7c8126c8257caeddac80800632866257008ad2a9; test-fixture-and-mock-build a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100. Applied measurable gate conditions, compilation/registration/runtime separation and original fixture ownership; original bodies were actually read in this session, as documented in HANDOFF. No foreign-language development policy was added; PowerShell performed source/resource/artifact orchestration only. InventorySvc remains the user-confirmed disk-incident cause; this verification makes no alternate cause claim.
