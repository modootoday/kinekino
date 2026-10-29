//! Document lifetime assembly contract, independent of SDK use-case contracts.
//! Exact public UTF-8 identity; no legacy document hash or signature alias.
pub const BYTES: &[u8] = b"proposal-document-session/v1\n\
parent-owns=single-registry-and-SDK-registration\n\
session-owns=review-controller,intent-queue,UI-registration,callback-owner,exports\n\
callback-reference=serializable-reference-not-authority\n\
dispatch=after-renderer-node-event-disabled-guard; drain=after-dispatch-return\n\
reprojection=validate-scope-owner-signature-fresh-queue-before-staging\n\
renderer-apply-failure=discard-new-stage-preserve-previous-owner-and-draft\n\
renderer-apply-success=promote-new-binding-before-retiring-previous-binding\n\
cleanup-pending=explicit-retry; retired-binding-bound=1; no-generation-rollback\n\
close=signal-queue-closed,revoke-callback-authority,unregister-UI-adapter\n\
failed-unregister=retain-owner-for-explicit-retry; drop-is-not-cleanup-proof\n\
completed-SDK-mutation=no-rollback-or-replay; node-guard-remains-renderer-owned\n";
