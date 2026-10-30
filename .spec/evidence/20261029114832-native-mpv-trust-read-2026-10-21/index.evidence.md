---
id: evidence:kinekino/20261029114832
title: Trusted mpv isolated read diagnostic
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Trusted mpv isolated read diagnostic

2026-10-21 SDK gate `runtime/verification/native-mpv-trust-read-r1`. Root736de48eb7c4f87f8a75eede9fc72d4cdcd74c3f; UI baseline1414, independent diagnostic exact1 source81E4C6E4C78453F7190893D8F42B63A6815B3A257A291EB726C179042747D321. This is a read-cost observation, separate from [original actual-media readiness failure](native-export-actual-media-2026-10-21.md).

## Observed

Default/native all-target strict Clippy and workspace fmt --check passed. Integration compiled once (130657ms0). Final executable SHA38FAD2DEC20A50F2ADF2B584F70875D6D8E1CF95CE8A4E31E5FE95EAEA2A7441 was fixed-copied and used for both exact tests: actual_export::trust_read_diagnostic_rejects_relative_path_and_invalid_hash (1 passed), actual_export::diagnostic_trusted_mpv_open_read_only_elapsed (1 passed, explicitly ignored selected once). No other runtime tests selected; 34 filtered for each. No source repair or retry.

Original TrustedMpvExecutable::open, with original immutable path guards/full BCrypt digest and no policy/timeout/cache bypass, returned success after **315784µs**. Verified bytes **121051136**, expected and observed SHA **f8bf7e8fd8d6696e35441c55091498060e4594f724a132fd6c46579341bc0790**, error_code=null. Fresh create_new proof is `evidence/diagnostic.jsonl`, **374bytes**, with process_started=false/network_used=false/host_cause_established=false. Mpv was read, not executed; host/media/network operations0.

## Limits and next evidence

The prior actual1 ready error and null child exit/forced cleanup remain unchanged. This isolated read followed earlier checks and does not describe original cache state, scheduling, raw-record observation latency, catalog initialization, SDK assembly or local-host startup. It cannot establish or refute digest cost as the cause of the original ready timeout. Production path/timeout/target/URL changes are not justified by this one measurement.

If allocated, original host readiness phase timing and raw-record poll before/after timing are the next distinguishing evidence. They must preserve original validation, deadline check order, original owner, and failure result. No actual media retry or extra diagnostic is authorized by this document. Validation9 remains an independently frozen, uncompiled source overlay; this read gate does not verify endpoint-loss recovery.

## Ownership and protection

Original resource-runner EEA9DEE6…/conservative2, 2CPU/affinity3/BelowNormal/descendant Job; existing snapshot/native cache. Entry RAM28.88GiB/GPU5737MiB/C189.53GiB. Six original receipts terminal0; recorded12 PIDs absent, original exec handles recovered, no PID reopen/kill/unrelated app changes. Runner receipt has descendant Job policy and terminal status; it does not provide a new inner host family receipt because no host was launched.

Root1454/snapshot1414+exact1/bundles324/index/source/before/original SDK pin/mpv/binary original+copy differences0. Heavy/shared explicitly returned before documentation. `closeout.json` SHA **189A15BAE434D9F98319FCCCFC9BEE2ADCA91D0E7F7967930CE285C297EFEF8D**. An unrelated cargo process observed during compilation was not owned or changed; its existence is not causal evidence.

Applied modoo.today code-change-review-verification SKILL.md9f73865f… and verdict-table9d9619a8… to separate shown call path, direct read observation and unverified host-cause claim. Full SDK/provider/native export/AI objective remains active and incomplete.
