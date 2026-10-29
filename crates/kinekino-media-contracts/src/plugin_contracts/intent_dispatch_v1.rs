//! Bounded presentation mailbox contract, independent of SDK persistence policy.
//! Exact public UTF-8 identity; pins alone do not authenticate callback callers.
pub const BYTES: &[u8] = b"proposal-intent-dispatch/v1\n\
queue=private-review-owner-and-scope-bound; not-an-SDK-job-queue\n\
callback=typed-text-or-explicit-save-accept-refresh-enqueue-only\n\
queued-ack=admission-not-SDK-completion; no-registry-reentrancy\n\
drain=after-dispatch-return; text=change-text; action=controller-command\n\
node-lease=opaque-host-lifetime; foreign-retired-closed=rejected-and-rechecked\n\
renderer-owns=node-event-disabled-signature-guard\n\
capacity=1..64; queued-text-byte-budget=1..1048576; UTF8-no-truncation\n\
coalesce=consecutive-same-node-and-field-text-only; never-across-action\n\
duplicate-pending-action=reject; stale-text-or-view-error=discard-following-actions\n\
SDK-error=preserve-canonical-error-and-cancel-remainder\n\
processed-count=intents-not-success-or-quality; no-retry-autosave-autoaccept\n\
unregister-shutdown=close-queue-and-discard-pending-input\n";
