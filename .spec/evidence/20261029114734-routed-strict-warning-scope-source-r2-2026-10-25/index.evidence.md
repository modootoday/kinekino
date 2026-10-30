---
id: evidence:kinekino/20261029114734
title: SDK strict-warning source preparation
status: active
created: 2026-10-29
updated: 2026-10-29
---
# SDK strict-warning source preparation

Frozen source8 F6625E8E37819F8A495DD89D7AD82D0104FC7BB1FEC47A4F9CF037A5DFB0754B; predecessors8 9BE326404D5DAAC76894F3968CFF13514CDDDD70FEBCF6B7C0E0E9B3094244EB; exact8.diff 4AE94827A233A1A53F1D542FF412BE6EED184BC119F35A86DC66DB1CE5C4961F; exact5 BA3FCB1D3FE9FBB03C65A093EA7429D0FDFA187051004684B1979AC69C5F77AE.

Baseline217 559E4C392210ECBE523825522A1341F9F86426BF734030D5F4A70EBB7F03F42D unchanged. Gate07-type.stdout contains21 warning records/15 distinct messages, not warning-free. This source8 consolidates prior C648 source2: jobs routed leaf unchanged from C648; native_playback/routed.rs extends its exact cfg(test) scope with issuer retention name. Compose source8 directly over217; do not overlay C648 separately on top and overwrite successor. No unique path added (eight replacements).

Changes by responsibility:
- Remove unused RoutedCheckpointObservation re-export at routed root only. Original concrete type, diagnostic_snapshot return and actual UI inferred consumption remain in ready module unchanged.
- Remove entirely unreferenced legacy infallible secondary_error/retained_revocation_error pair. Selected217 source and canonical src/tests have no call sites; these are not real test-only getters, so cfg(test) would leave dead libtest warnings. Original fallible diagnostic_snapshot already reads the same secondary owner without mutating/revoking and remains unchanged, including original Arc identity/borrow-failure behavior.
- Registration registrar/failure, export issuer, activated client/read owner and activation failure ready/secondary revocation-error fields receive explicit _retained_* names. Field type/order/visibility/initial value/strong ownership/Drop timing unchanged. All original initializers and existing bootstrap test references renamed consistently. Activated ready is actually used by production and keeps its existing name. No RAII owner removed, Option cleared, drop inserted, fake read, broad allow/expect or cloned replacement owner introduced. Original registration/activation errors retained even where UI has not yet implemented their display.
- C648's legacy routed assembly constructor/types and list/peek/consume convenience impl are actually test-only based on selected/canonical references; cfg(test) retained as previously reviewed. Production struct/route/issuer/skip owner remain. Existing standalone/concrete configured-client APIs unchanged. C648 removes two previously uncalled jobs scaffolds (run_categories and old nine-job validate_final); active original identity/EOF/family/FinalV2 3/5 validation unchanged.

Scope exclusions: ProductPlaybackSetupFailure original_transition_lifetime/original_error warnings are UI-owned; UI has been notified about actual missing setup-error/lifetime observation and must connect them in its own successor. SDK does not cfg/delete/fake-read those getters or touch UI consumer. Diagnostic217 private Geometry/raw observation2 leaves unchanged, protection354 unchanged, CPU slot idle/owned by PM. No runtime retry, Cargo, GUI, model, download, stage, commit or push in this authoring work.

Private exact8 rustfmt --edition2024 --config skip_children=true --check passed. No type/test/clippy execution; cannot claim strict warnings resolved until allocated composed checks. Reuse existing unchanged test bodies; exact5 consists of original three bootstrap late-activation/client/review-retention cases plus original ready-failure-preserves-owner and review-owner-after-route-replacement legacy cases. Bootstrap test leaf edits only field access names. No new test/registration/body/policy. Production and libtest/jobs no-run must prove cfg/import/name consistency, then exact5, then requested full fmt/test/strict clippy final gates as PM allocates. No predicted warning count is a verified result.

Applied already-read modoo.today code-change-review-verification body9f73865f96ec0cefcf3a4023b294400258473640df3b1d2add03dcca8192d64b + required verdict-table9d9619a829ae24d73645ef5f7c8126c8257caeddac80800632866257008ad2a9 and test-fixture-and-mock-build a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100; exact source references retained in SDK213 handoff and prior C648 HANDOFF. Application: actual compiler warnings/reference evidence first, retention semantics preserved instead of masking lifetimes, original same-client/failure/review fixtures, explicit unexecuted limits.
