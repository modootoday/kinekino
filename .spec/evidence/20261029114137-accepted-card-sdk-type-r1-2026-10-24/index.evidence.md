---
id: evidence:kinekino/20261029114137
title: Card locked gate first failure — renderer test lint
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Card locked gate first failure — renderer test lint

Approved private source27 83F47C8E/effective61 91139FE5/renderer r3 F62852CC and candidate lock B2E3D540 were freshly supplied as62 files to existing SDK C warmS/T. Parent root a6ca29c/f85, canonical lock8F2 remained unchanged. Protection203 uses only the PM-attributed post-a6ca index baseline correction from the preceding lock lease; carry190 F9125486 and read/assets48 matched. No new lock resolution, target or asset copy.

The first actual command, `cargo clippy --offline --locked -j 1 -p media-card-render --all-targets -- -D warnings`, returned101/7226ms under conservative2/mask3/BelowNormal/descendant Job. Original error: packages/media-card-render/src/tests.rs:54 `.chunks_exact(4)` triggers `clippy::chunks_exact_to_as_chunks` with -D warnings; compiler suggests `.as_chunks::<4>().0.iter()`. The error is in frozen Media-owned test source. SDK did not repair it, add allow, retry or run later commands.

Renderer default no-run/fixed/list/nonignored24, native strict/no-run/list/nonignored24, root default/feature strict, feature CLI no-run/list9 and normal CLI build all remain unexecuted. No tests, ignored native buffer, actual PNG or root full suite ran. This lint finding does not establish or refute runtime PNG quality. No new fixed EXE/PDB was produced for carry; existing carry190 is retained unchanged. Clippy's partial checks are not a test/type-gate pass.

A preceding orchestration command had a PowerShell ParserError (extra closing parenthesis) before execution, source writes or Cargo. 00-setup-parser-error.json preserves that setup failure separately; corrected setup then admitted exact62. This was not a repeated Cargo command or source repair.

S61+lock62 SHA/mtime/FileID restored, including40 originally absent files. Only explicit files were removed inside verified absolute S boundary; empty directory scaffolds remain. Cache11 alias/producer SHA/stat/FI, rootpins6 SHA/stat/FI, protection203/carry190 stat/FI and private source/candidate62 SHA deltas0. Regular exclusive EOF2, recorded launcher/Cargo PIDs2 absent, writer0/pending0. No all-cache or exhaustive descendant claim. Meari D pending5/normal restorationfalse remains outside this lease.

Evidence root runtime/verification/accepted-card-sdk-type-r1: original03 stderr/receipt, 04-S-restored62.json, 06-final-audit.json, recorded process/EOF records and unchanged carry-fixed190.json. PM owns the next Media revision and fresh composite/CPU assignment. Root Cargo/lock/index/ignore and source27/renderer r3 remain unchanged; stage/commit/push0.

Applied already-read modoo fixture a393f15f and review9f73865f/required verdict9d9619a8: preserve first actual cause, exact source ownership and failed-versus-unexecuted boundaries. No new non-Rust development logic; PowerShell limited to file/resource/receipt orchestration.
