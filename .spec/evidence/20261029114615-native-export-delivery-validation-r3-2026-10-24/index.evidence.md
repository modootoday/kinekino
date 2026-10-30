---
id: evidence:kinekino/20261029114615
title: Validation export r3 — first format failure
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Validation export r3 — first format failure

PM allocated CPU-only validation for private source9 A2B7452A01D770AA540284CF42C6951B509C536381CC5B148F10069A4C5AE9DB on root371ed79414698e49f6ee7444ec00d5e84a0cd1cc. Evidence: `runtime/verification/native-export-delivery-validation-r3-cpu-r1/`.

Fresh admission matched promoted62 B540883F, lockB2E3D540, canonical before9(existing7/new2), cache11, protection203, rootpins6 and latest carry231 B7C86EC0. Existing normal producer4 and alias/worker independent raw6 were verified; underscore `player_cli.pdb` was explicitly included. Carry reused its preceding content hashes with fresh bytes/mtime/FileID, without whole-carry rehash. Only exact source9 was supplied to S; no whole SDK refresh.

Original first `cargo fmt --all -- --check` receipt is terminal/exit1/5845ms, launcher36300/child49400, conservative2/CPU2/mask3/BelowNormal/descendant Job. Stderr: `failed to resolve mod validation: .../src/bin/player-cli/validation.rs does not exist`. **Strict, type/no-run, listing, pure tests and actual playback all0**. No automatic repair/retry followed.

Source explanation: `src/bin/player-cli.rs` imports native_host with `#[path = "player-cli/native_host.rs"]`; private r3 declares bare `mod validation`, while its actual new leaf is `src/bin/player-cli/native_host/validation.rs`. This is a module-location composition failure at the format entry stage, not evidence of a native seek/ACK or product-policy failure. Leaf-only rustfmt with skip_children had not exercised this module resolution. The original r3/raw receipt and source remain frozen.

Restoration returned original7 SHA/bytes/mtime/FileID and original absences2. Promoted62 remains the resident baseline; alias4/worker2/normal producers were not written. Final audit and original process/EOF observations are in `05-restored9.json`, `06-final-audit.json`, `06-EOF2.json`, `06-recorded-PIDs2.json`, `06-lifetime.json`, and `slot-return.json`. Slot return is operational closure of this failed gate, not a passing fmt verdict. Prior PDB/preflight and isolated Meari cleanup exceptions remain separate.

PM separately authorized a later private r4 with only `#[path = "native_host/validation.rs"]` on the module declaration after this gate's restoration/return. No successor compile/run is included here; future format entry must verify it under its own receipt. General seek_submission=false and all current policies/assertions/budgets stay unchanged.

Applied already-read modoo.today `modootoday/software-qa/code-change-review-verification` body9f73865f96ec0cefcf3a4023b294400258473640df3b1d2add03dcca8192d64b and required verdict-table9d9619a829ae24d73645ef5f7c8126c8257caeddac80800632866257008ad2a9: retained the first failed executable path, traced its actual caller/module declaration, distinguished leaf format/source composition/type/actual verdicts, and did not widen a setup failure into product failure. No new non-Rust development logic, download, release-negative build, GUI or actual media run.
