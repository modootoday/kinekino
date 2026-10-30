---
id: evidence:kinekino/20261029114157
title: Parent record observation and CLI help union verification
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Parent record observation and CLI help union verification

2026-10-21 allocated SDK gate `runtime/verification/native-ready-help-union7-r1`; root **e93a0b7b778e2b524d607acffd5737aa27a04305**. Union7 source **2440683C…12A9A870** plus accepted resource4 from AI13 **6EC80C84…DFEECEF**, disjoint exact11. Baseline shared1414 already includes adopted isolated-read diagnostic. Before7 matches root/snapshot; accepted resource4 hashes match root. Integrated11 SHA **1B4051CB4815C01EDAB0A41B8C878AA3650931EF774376AE4D933C26A38C6BB6**. Final snapshot1417 manifest SHA **9A6F03A9981051ACCC857488040BC03B60E8191733E42976A9FF762BE4AF8CF6**.

## Actual results

| Check | Result |
| --- | --- |
| Root default all-target strict Clippy | exit0,65097ms |
| Root native-proposal-window all-target strict Clippy | exit0,84667ms |
| owned-process all-target strict Clippy | exit0,7638ms |
| workspace fmt --all --check | exit0,14653ms |
| owned-process unit/state_machine no-run | exit0,9198ms; compiled artifacts, not passes |
| New metrics unit exact1 | 1passed/0failed |
| Entire pure state_machine | 9passed/0failed; existing8/new1 |
| Native SDK exact11 | 11passed/0failed on one fixed binary; no full integration/helper OS selection |
| Explicit normal CLI build | exit0,92338ms |
| Normal CLI --help read1 | exit0,64ms; stdout6294B/stderr0 |

**21 unique pure passes, including6 new**, not21+6. Native selection covers helper existing6/new2, ready contract new2, existing explicit-null stop contract1. Shared helper imported by jobs.rs also compiled under all-target checks, but runtime passes counted once in native_skip_cli. Metrics unit second resource test and nested support tests were not selected. No new source repair, original nonzero, retry, blanket lint allowance or assertion relaxation.

Normal help output jobs sentence includes both subtitles.convert and subtitles.export_range, removes “currently supports only subtitles.convert”, and retains no model inference. This corrects an existing-operation description; no executor support was added or conversion/range execution reverified by reading help.

## Final artifact lineage and literal features

- Owned unit **0A507ED9F65B9DA40A9DA3F17CC8639E019A0FE5492733F71D80B61BF420E5EF**,756736B; state_machine **527AE934819F0EE8EDF817C49FE155E611617C089DD125F6A66C6D474ED898D3**,951296B, default owned-process package configuration.
- Native SDK **DEF882AA42D6336D2D2DC3D49EEDC553AC832E45266F64F0D6DBBEC88B9D3D0F**,5826560B; no-run251502ms0; exact11 uses fixed original-matching copy.
- Normal CLI **F7D7D9AB2A3A935997AC663AF1BA36C6BC1743BE4777E4ADF077AFA0327C7310**,30148096B; explicit normal build, original+fixed copy hash match, same final integrated source as SDK. Help uses this copy.

Product native strict/SDK no-run/normal CLI build use literal Cargo feature **native-proposal-window**. “native-ui” is not a Cargo feature. Root default strict and owned package checks use no extra feature; native-observer-test-support/test-support/validation-export-delivery-loss are not activated. No build configuration was changed mid-run. Current default all-target compile checks do not prove default binary link/runtime behavior; enabled feature does not make these pure tests render UI.

## Ownership and limits

Original EEA9 runner conservative2/2CPU/affinity3/BelowNormal/descendant Job; offline/locked absolute manifest/-j1/thread1; existing snapshot/native cache. Entry available RAM30.06GiB/VRAM5718MiB/C189.09GiB. All21 runner receipts terminal0, recorded42 PIDs absent, original exec handles recovered/outer handles0. Root1455/snapshot1417/bundles382/index/each4 original+fixed binary differences0. Closeout SHA **7216C934E9FED107916A1F6CAD49BBA30BA42FD6EEE9F59F71801E1D4F8C631D**; heavy/shared explicitly returned before documentation. Runner terminal/Job policy is not a new inner host-family receipt: no host was launched.

Actual ready-only/host/media/network/replay/validation-feature/AI inference/GUI runs0. [Original actual-media readiness failure](native-export-actual-media-2026-10-21.md) remains failed; [isolated315784µs read](native-mpv-trust-read-2026-10-21.md) remains separate. Pure fake-owner outcomes do not establish real ready/EOF, native decode/ACK/landing or endpoint-loss recovery. Validation9 and archived replay2 remain independently frozen/uncompiled. Full provider/category/subtitle/production/AI objective is incomplete.

Next actual ready-only1 requires explicit allocation using accepted normal CLI and SDK pins, fresh Fixture evidence and same original bounds. Feature minimization is a later configuration decision, not a reason to invalidate/redo passing checks absent a new change. Modoo.today code-change-review-verification was applied to separate source boundary, actual pure/CLI observations and unexecuted host/media claims.
