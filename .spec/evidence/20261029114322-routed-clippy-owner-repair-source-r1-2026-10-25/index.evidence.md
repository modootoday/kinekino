---
id: evidence:kinekino/20261029114322
title: SDK strict-clippy owner repair5
status: active
created: 2026-10-29
updated: 2026-10-29
---
# SDK strict-clippy owner repair5

Source5 C2F8B3FB09CE98D3F2CD4A38F463B37F53A6208256F9F796E50BDEB281D53047, predecessors5 B30E2139A999D85FE2C1F82AAF758FF79F280ACC9567AAED3CC608ED8D33F80D, exact5.diff5B94172A1C25D22A95780C9413D6FAD7A6688B5CA972C9D23E3939F9BD6D9C20. Exact CFE1F4FA final251 predecessors; prior packages remain frozen. Private rustfmt apply/check5 pass, compile/test/clippy execution0 for successor. Actual original CFE1 fmt/type pass then clippy15 fail; this packet addresses five SDK-owned diagnostics only, not unseen test/dependency lints. UI10 has separate source6 4DE98A39.

ready.rs: remove remaining unused RoutedCheckpointObservation re-export from ready module. Concrete pub(crate) observation type and diagnostic_snapshot definition/return/value fields remain unchanged in private child module; original UI infers the type. No getter/body/borrow/Arc/error/authority change.

engine_cleanup.rs: collapse nested route/revoke-error conditions into ordered let-chain after original cleanup branch. Revoke executes only for retained original route when cleanup is absent, exactly as before; original state.remember/diagnostic Arc and parent-loss admission order unchanged.

geometry/rejection_observation.rs: collapse successful serialization/length check into ordered let-chain. Existing record-validation/process-once gate, payload, finite bits, bounds<8192 including LF, original Geometry error and best-effort I/O unchanged. No new diagnostic meaning/success.

late_routed/installation.rs: private install_original returns Box<FailureParts> instead of inline152B error. All five original failure branches box the exact original error/prepared owner. Public install signature already returns Box<LateInstallFailure> and is unchanged; its sole caller moves the same original fields into that existing carrier, preserving ownership/order/retention, without clone/default/reset/drop. No live admission or registry policy changes. FailureParts is private to this file with one private call, no external signature expansion.

registration.rs: one narrow #[expect(clippy::result_large_err, reason=...)] applies only to register_routed_media. The actual external NativeSurfaceRegistrar::register_reserved callback requires Result<C, ChildCreationFailure<C>> (packages/meari/.../window/surface_slots.rs219..226). C is the original cleanup-bearing PlaybackSurfaceChild, so callback Err344B retains original partial child/error. Boxing that Err would violate the exact external lifecycle callback type; boxing/forwarding C would change the registered NativeChild contract and cleanup type. Preserve original inline child and partial cleanup owners rather than redesign authority/lifetime for stack-size lint. No global allow, threshold override, data reduction, dummy read or ignored functional error. Rust/clippy must verify the expectation is fulfilled; this is not a passed lint result. PM explicitly permitted this narrow ownership-justified option.

Suggested impact: existing failure4+raw tamper1+prior SDK bootstrap/legacy5; add application::skip_segments::host::late_routed::tests::{foreign_and_replayed_install_preserve_original_port_then_delegate_existing_policy,cancelled_pending_install_stays_closed_and_returns_original_prepared_owner} for newly boxed private failure; reuse original diagnostic scalar fixture for let-chain. UI may add its original auto3/native ACK3 for changed projection/catalog branches. No new tests or public API. Source freezing does not authorize additional CPU/OS without central allocation.

Applied already-read modoo.today code-change-review-verification body9f73865f96ec0cefcf3a4023b294400258473640df3b1d2add03dcca8192d64b + required verdict-table9d9619a829ae24d73645ef5f7c8126c8257caeddac80800632866257008ad2a9 and test-fixture-and-mock-build a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100. Application: exact original lint evidence/contract and caller review, narrow boxing of opaque private failure, explicit lifecycle exception reason, original same-owner failure fixtures, authored versus executed distinction. Current369 custody immutable/source overlay/Cargo/native/userAI9/stage/commit/push0.
