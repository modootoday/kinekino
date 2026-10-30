---
id: plan:kinekino/20261029113643
title: Live entry native retirement gate
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Live entry native retirement gate

PM assigned UI host integration plus SDK/native port implementation after the original267r3 GUI Slot(NotVideo). Predecessor source720AA4F17C15955A2A7FC3ECBCC15CF72FA3DD024DFAFE43EBB09929E033A137. Current tested source remains frozen. No new Cargo/GUI allocation.

SDK owns typed native retirement port/opaque identity-ticket, original frame consumer, native lease cleanup and fresh Viewing registration. UI owns ViewerHost fields/install boundary, Draining and return-to-Viewing gating, original request/error/ticket preservation and focused host regressions. No shared LocalHost/SDK shutdown, new inference, guard waiver or fake frame.

| Host state | Required operation and admission |
| --- | --- |
| Original callback Draining | Existing consume_reads/prepare_route_release drains original callback tickets/attachment first |
| Native retirement begin | Request retirement of the exact originally bound native lease once; retain its typed ticket/owner |
| Native retirement pending | Poll the same ticket with bounded original continuation/retry; no Studio Starting/current inspection/mount/frame activation |
| Native retirement failure | Preserve first typed error and original carrier; do not replace/drop owner or claim release. Keep inputs closed and cleanup possible |
| Native retirement complete | Only original cleanup receipt can allow Studio Starting, inspection, mount, retained proof and actual new frame observation |
| Return to Viewing | Prepare fresh checked registration/generation with SDK port before activating the returned Viewing route; original old lease/ticket cannot authorize a new surface |
| Close while pending | Existing original cleanup owner remains responsible; UI request cannot erase uncertain/pending native cleanup |

Implementation uses the SDK-confirmed concrete RouteMediaPort<PluginScope>/opaque ticket contract rather than a duplicate UI policy trait. UI implementation is handed off as runtime/development/product-live-native-retire-gate-r1/source6.json SHA256 CEC577294E02BF61FB109F045C1BDAF8F51766885F084020D5FDF513827F049C, followed by product-live-native-retire-rearm-guard-r1/source2.json SHA256 8ECEBD3464B447FACF8DC51BD6FADDF6446893AEF973FC1137DFF095C4BF4C5C. The latter explicitly rejects a new retirement while fresh Viewing registration remains pending. Both original manifests remain immutable. SDK/native adapter final source pin and joint central checks are pending; UI rustfmt/check passed, Cargo/type/test/GUI have not been run. The separate Viewing everyday UX directory is incomplete and excluded.

Focused regressions must preserve original same-SDK/current inspection/ACK/release and all existing strict asserts. Add pending→complete ordering, one original retirement ticket, foreign/stale ticket rejection, original first error/close ownership and fresh return registration cases. Model tests do not prove OS child disappearance or actual retained frame; one later central GUI verifies those boundaries after CPU checks.

Evidence and exact error: ../../sots/verification/presentation/normal-gui-projection267-r3-awaiting-frame-2026-10-25.md. The old video node is absent from the Studio UIA but physical child bounds are not independently observed. Source shows callback-only release and active old consumer video revalidation; SDK and Meari confirmed existing original release_child/poll cleanup can retire the child without closing the main window. NotVideo remains an appropriate rejection.
