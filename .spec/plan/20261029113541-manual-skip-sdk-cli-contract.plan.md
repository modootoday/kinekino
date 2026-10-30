---
id: plan:kinekino/20261029113541
title: "Manual skip SDK/CLI connection contract, proposed"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Manual skip SDK/CLI connection contract, proposed

최신 source17 후보의 수명·읽기 구분: inspect는 저장된 ephemeral 상태를 읽어 미디어 교체 후 revision 확인/명시 close를 가능하게 하며, active engine 일치나 native 관측 증거로 쓰지 않는다. 잘못된 ID/host/저장 binding은 거절하고 실제 mutation/evaluate/skip은 current binding을 확인한다. schema/capability에도 stored-state scope를 표시한다. source17 C5E8E4…63AE6는 작성·직접 포맷 검사만 완료했고 실제 컴파일·endpoint 실행은 없다. core r3 15pass와 구분한다.

2026-10-20 source-reading preparation only. Source11 r1 was independently reviewed and a cancellation-reporting repair is being prepared; no API/CLI implementation, Cargo or host run is authorized by this document.

## Existing CLI and host facts

`src/bin/player-cli/runner.rs::run` processes one Request and exits. Without --endpoint, execute_local creates a new PlayerSdk for that invocation; it does not retain a SkipSession across fresh CLI processes. --endpoint instead forwards that one request to an explicitly selected current-user local host, with no fallback/start/retry. Local request/response roots cannot be combined with endpoint routing.

Existing --serve-native and --serve-playback own playback engine/session resources, inject one long-lived SDK into LocalHost, print a transport-ready endpoint and stay alive until stdin EOF. Native ready does not mean media loaded. `infrastructure/local_host/executor.rs` retains ApplicationClientHandle, serializes SDK calls and rejects concurrent busy requests instead of replaying after a timeout. LocalHost drop cancels I/O/joins transport, but an already dispatched synchronous SDK call may finish after transport owner drop. No stronger cancellation or synchronous worker-join guarantee is inferred.

Current hosts do not inject any skip-session owner; segments.* remain unsupported. The source-only core SkipSession is mutable ephemeral state, and its inspect snapshot is neither a durable document nor a complete restorable runtime pass (attempt/suppression guards are not serialized). Round-tripping that snapshot in separate CLI requests would lose guards and is not a valid persistence design.

## Proposed ownership: same endpoint, same host-owned session

Prefer extending the existing explicitly launched playback/native host's SDK injection with one skip-session owner, using the existing engine/session owner and transport. Do not add an implicit daemon, GUI automation or a second engine. SDK &self execute needs a small synchronized owner around core &mut SkipSession; serialize expected-revision check, state mutation and original seek admission as one ownership operation, fail explicitly on busy/poison/closing. Never unlock/recreate state between those phases. This is a proposed adapter, not a claim that LocalHost alone supplies such session storage.

An explicit proposed segments.open binds a host-generated opaque skip_session_id to current engine_instance_id + resolution_artifact_id, actual host instance/generation and a new core session. This is session creation, not a media detector or durable media fingerprint. Subsequent requests carry skip_session_id plus expected binding; the host rejects unknown/closed/stale/foreign-host sessions. Do not key only by engine integer, which may repeat after host restart. An inspect on another endpoint does not look up/create/restore the old session.

State lifetime: multiple fresh CLI processes can edit/inspect the same ephemeral state only while they target the same live endpoint and host-owned skip_session_id. Host/session close, playback replacement or restart invalidates it; no restart resumption, automatic source reconstruction, preference save/reopen or cross-episode copy. capabilities/inspect explicitly report durable=false, detection_supported=false, host_tick_supported=false. EOF cleanup may not retract a synchronous already-admitted seek; admission/observation remains truthful.

The stock EpisodePlaybackSessions skip adapter reports Seekability::Unknown and observation_authorized=false. Thus edit/verify/policy/inspect may become supportable with a host owner, while actual skip and unattended automatic execution remain blocked until independently observed seekability/current observation and a real tick integration are available. A host-ready event, nonzero duration, mock fixture or queued seek cannot promote these capabilities. No caller-supplied JSON observation or seekable=true is accepted.

## Typed methods and schema/status proposal

| Proposed command | Request essentials | Typed response / support boundary |
| --- | --- | --- |
| segments.open | explicit current engine/resolution binding | opaque skip session + SkipSnapshot; requires host-owned session manager |
| segments.inspect | session id + expected binding | current SkipSnapshot, ephemeral lifetime/capabilities; no state creation |
| segments.edit | session/binding, segment id, kind, in_us/out_us | current SkipSnapshot; manual origin only, edit clears verification |
| segments.accept_skip | session/binding, segment id, annotation_revision | current SkipSnapshot; confirms marker, not seek/AI result |
| segments.set_skip_policy | session/binding, independent OP/ED Off/Manual/Auto | current SkipSnapshot; storing Auto does not claim execution availability |
| segments.evaluate | session/binding, segment id, manual/automatic intent | Decision plus actual current state revision/pass identity; may update runtime reverse-seek guard |
| segments.skip | session/binding, segment id, intent, cancelled | SkipResult plus post-action revision; no write/Cancelled status/Admitted ControlReceipt |
| segments.override | session/binding, explicit cancel/watch/manual-seek/new-pass action | updated revision and CancelStatus or SkipSnapshot, one shared cancel policy |
| segments.close | session/binding | invalidated skip session only; does not close playback or revoke queued controls |
| segments.detect | existing command | Unsupported; no model or fabricated detections |

Names/extra commands remain proposed and require exact SDK ownership allocation; none are registered by this document. Open is a missing necessary lifecycle command, not existing support. New-pass is an explicit user operation, never an automatic consequence of evaluating a backward position or retrying a request.

All wire payloads deny unknown fields. Integers are DecimalU64 decimal strings, ranges [in,out) in EnginePlayback time. Single expected-revision authority should be RequestContext.expected_revision for mutations, parsed checked and mapped into the existing core DTO's expected_revision; do not expose conflicting duplicate expected_revision in payload. Read/evaluate schema must explicitly distinguish any runtime guard transition from configuration revision. Reject unsupported idempotency_key/timeout options consistently in schema and runtime; transport's5s deadline is separate, not operation cancellation. No automatic retries on transport disconnect/execution_unknown.

Core request DTOs exist for edit/verify/policy/skip, but session-id routing/lifecycle/evaluate/override wire DTOs do not. Core SkipResult, CancelStatus and SkipSnapshot currently derive Serialize only. A genuine typed SDK client decoding them needs a separately reviewed minimal Deserialize change (and strict response schemas) or approved typed response DTOs; current source is not called a finished wire contract. Reuse existing ApplicationClient/Response and common SDK errors, never return generic Value while claiming typed method completion.

## Outcome and error mapping

NotSubmitted+Decision::Blocked is a successful evaluation with explicit reason/seek calls0, not seek success. Native unknown/freshness failures remain visible; a future convenience command may map a blocked action to NeedsInput/Unsupported only by a documented consistent adapter policy. Admitted means original Accepted/DeferredUntilHandoff and Completion::Unobserved. No response field may invent landed, completed, cancelled=true or auto-detected.

Malformed schema/type/range/id → InvalidArgument; stale session/config/annotation/engine binding → RevisionConflict or explicit session NotFound (do not silently create replacement); missing port → Unsupported; busy/closed/poisoned owner → explicit existing error, no replay. Admission errors retain the original ApiError and an attempted guard; clients must inspect the same live session for new revision/state after ambiguity before an explicit manual retry. Request_id is correlation, not an idempotency key.

R1 review found cancelled=true in skip() incorrectly returned CancelledBeforeAdmission after earlier admitted/deferred/error attempts. Repair delegates this branch to the existing cancel() policy and returns Cancelled {session_revision,status}; accepted/deferred/error follow-up must be TooLateOrUnobserved. For record of earlier queue state, no source-level cancellation flag retracts an already-admitted seek. Both skip(cancelled=true) and explicit override(cancel) must expose the same distinction.

## Later actual CLI acceptance, not a fake fresh-process state assumption

1. Inject a counted synthetic playback/skip owner into existing LocalHost in a Rust integration fixture (same transport, no mpv/model/GUI). Start it explicitly and hold owner alive. This proves only that fixture's core+SDK+transport.
2. Fresh CLI A uses endpoint open→session identity; fresh CLI B edits; C confirms; D sets policy; E inspects same owner and verifies revisions/manual origin/durablefalse. No local CLI flag pretends to save state to disk.
3. Under fake trusted port, explicit evaluate/skip asserts original seek count0/1, target/binding/unobserved receipt, override and backward/new-pass semantics. Stock native adapter negative case must still submit0 for unknown seekability/authorization. Real native positive seek remains unverified.
4. Host/session shutdown/restart or wrong endpoint/session/binding must reject old id; new host may issue a distinct id but may not restore old preferences/attempt guards. Transport timeout after transmission means execution_unknown and no retry; inspect same owner is recovery, not replay.
5. Pin actual fresh CLI binary, per-request JSON/response/exit and session snapshots. Require supported/unsupported flags and strict schemas to agree with each host's injected capabilities. No generated detector confidence or durable reopen pass count.

Source invariants remain r1 frozen until review repair is separately pinned. This contract prep edits only .spec. Skills: previously read modoo.today requirement-to-testable-stories applied explicit lifecycle dependencies and measurable session continuity/guard-loss negatives; source-vs-execution evidence separation maintained. No new non-Rust validation code or host process was introduced.
