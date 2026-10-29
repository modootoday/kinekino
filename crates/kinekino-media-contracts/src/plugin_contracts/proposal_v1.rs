//! Proposal host contract v1.
//! These canonical UTF-8 bytes define a new identity, not a legacy hash alias.
pub const BYTES: &[u8] = b"plugin-proposal-host/v1\n\
native-only; canonical SDK request/result/error types; registry owns scope and grants\n\
capabilities=proposals.create,proposals.inspect,proposals.edit,proposals.accept\n\
owned-async-profile-adds=proposals.import_historical\n\
create/inspect=no-expected-revision; edit/accept=require-expected-revision\n\
accept=explicit; target-source-revision=immutable-source-snapshot\n\
no-context-stripping; no-automatic-retry; no-output-repair; no-invented-timestamps\n\
raw/source/draft/acceptance=distinct; stale-response=origin-scope-only\n\
historical-import=verified-source-and-raw-binding; no-new-inference\n\
acceptance=immutable-receipt; automatic-application=false; quality-approval=false\n\
contract/provider/request/reply/error-identities=distinct; pins-are-not-authentication\n";

/// Explicit opt-in profile; the original Native4/owned Native5 bytes above stay unchanged.
pub const OUTPUT_REVIEW_PROFILE_BYTES: &[u8] = b"plugin-proposal-output-review-host/v1\n\
native-only; canonical SDK request/result/error types; registry owns scope and explicit capability grants\n\
capabilities=proposals.create,proposals.inspect,proposals.edit,proposals.accept,proposals.import_historical,proposals.inspect_output_review_target,proposals.record_output_decision,proposals.output_decisions\n\
create/inspect/import/output-review-inspect/output-history=no-expected-revision\n\
edit/accept=require-current-draft-revision; record-output-decision=require-current-output-revision\n\
record=proposal-id+decision-id+exact-accepted-target+criteria-identity+explicit-verdict+required-note+declared-reviewer+optional-claimed-time\n\
inspection=one-canonical-state+target+full-criteria-document+criteria-identity+session-evidence-observation\n\
history=immutable-records+current-evidence-not-checked; declared/claimed/adapter-observed-local-time=distinct\n\
technical-acceptance-is-not-quality-approval; output-decision-scope=accepted-output-only; automatic-application=false; counts-as-model-sample=false\n\
same-owned-worker-and-document-registry; no-extra-process; no-context-stripping; no-automatic-retry\n\
scope-generation-grants-cancel-terminal-owner-ack=registry-and-existing-execution-lifetime\n\
publication-may-exist=preserve-original-error-and-request; explicit-reinspect-reselect-same-id-body-retry-only\n\
contract/provider/request/reply/error-identities=distinct; pins-are-not-authentication\n";
