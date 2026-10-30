---
id: plan:kinekino/20261029113630
title: "Download executor: source admission and ownership"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Download executor: source admission and ownership

2026-10-23. Initial SDK source investigation followed by separately allocated fixture gate. Actual application10 + Windows filesystem4 + shared identity guard2 passed; final fmt/strict passed and execution slot returned. See [actual gate SOT](../../../sots/verification/production/download-transfer-2026-10-23.md). This verifies transfer/storage foundations, not provider transport or CLI download. No network/browser/media download ran. Configured-worker SDK6 was separately accepted as root5ef988b6.

## Observed contracts

- `src/application/api/capabilities.rs` declares `production.download_media`; declaration does not establish supported execution.
- `src/application/episode_resolve/results.rs`: `EpisodeResolutionResults::read` returns historical verified bytes and explicitly does not assert a live browser session or an unexpired URL.
- `src/application/browser_provider/activation.rs`: `MediaRequestObservation` is a requested resource, not a response or admitted source.
- `src/application/browser_network/request_headers.rs`: request correlation rejects duplicate/redirect ambiguity; evidence contains Referer and observation time, not a downloadable session or save permission.
- `src/application/playback_session/guarded.rs`: opaque context belongs to original seek read admission. It cannot issue download authority.
- `packages/media-jobs/src/application/ports.rs`: reuse execution cancellation, deadline, verification and publication CAS. Historical source snapshots cannot restore provider credentials.
- `packages/media-file-publication/src/lib.rs`: existing no-overwrite hard-link publication consumes a complete byte slice. A streaming owned `.part` lifecycle requires a separate adapter; do not buffer an entire video through this API.

## Proposed isolated source ownership

Prepared under `runtime/development/media-download-r1`, preserving root Cargo, audio/VAD host, job host registration, shared index and existing SDK6. PM approved application/download, infrastructure/download and corresponding fixtures; final-source8 A02376A4 includes the separate r2 staged-output repair. Eight paths were explicitly mounted in S for gate16, root admission remains PM-owned. No operation support or capability registration follows from the fixture gate.

The source issuer must retain the original provider session and observed-media identity. The user's download request already supplies intent; do not invent an additional rights credential system. Distinguish actual provider authentication requirements. Caller-supplied URL JSON and historical resolution records cannot recreate a live session. Persist only a nonsensitive media/track identity and intent; re-resolve on expiry/restart and verify identity before resuming.

The first transfer use case must stream through an admitted lease into owned temporary output, enforce cancellation/deadline/size and response identity, validate the resulting media, then reserve publication and publish without replacing an existing file. Failure before publication cleans only owned partial output. A publication winner remains published if a later cancellation loses the CAS. A verifier stub or arbitrary GET cannot establish actual media support.

HLS VOD variant/audio/init/order/discontinuity handling, authenticated sessions, bounded retry, Range/object-validator resume, recovery and batch partial results remain required follow-up scope. No capability or CLI support claim until actual session adapter integration and CLI acceptance.

## Applied skill and evidence limits

Read `modootoday/software-qa/test-fixture-and-mock-build/SKILL.md` from modoo.today skills, SHA256 `a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100`. Applied actual contract-derived statuses and response shapes, explicit missing prerequisites, isolated mutable fixtures, cleanup/reacquisition checks. Synthetic transfer fixtures cannot prove provider authentication, real timing, media validity, durable job CAS or CLI support. Actual runner timings are preserved, with no benchmark claim. Scoped fmt/type/strict/actual16 gate is complete; provider/CLI acceptance remains pending.

See [download requirements](../../../sots/player/application/production/downloads.md) and [job boundaries](../../../sots/player/application/production/job-boundaries.md).

## Owned-probe checkpoint and remaining product acceptance

The source-only preparation described below subsequently passed the separately assigned **actual23** gate. See [owned probe actual SOT](../../../sots/verification/production/download-owned-probe-2026-10-23.md). Final staged source12 SHA256 `A836BA7E9C7EF47820DDFA127824D537505BEA7B415D04A801A54CF9D2E63EAC`, before12 `2F8B2C03350D93879BBF5A36E9C14B276884357C6FD973972419F9165229E01E`; r5/r6/r7/r8 and all first failures remain separate. This is not root adoption or product CLI/provider acceptance. Source-freeze statements below describe the earlier preparation phase.

2026-10-23 root `cce969547f2011e13d30badf2c5f6cc23a0a376f` reinspection. PM has taken the separate source-only checkpoint for independent review; its eleven authored/overlay files and three borrowed files are pinned by `runtime/development/media-download-provider-r1/probe-checkpoint-source.json`, SHA256 `178977B6DECEC23F1D34849AD2B1463AB1E86CE3B772A3F6CC3980AA8E9B50AC`. Existing four preimages match current root; seven new paths are absent from root. No compiler/process/network observations exist for this checkpoint. Preserve its bytes until review finding or actual compiler failure authorizes a separate revision. Its dependency diff is zero. HTTP/browser overlays and the proposed reqwest direct dependency are outside this compile closure.

| Requirement | Current evidence | Required next acceptance |
| --- | --- | --- |
| Original sealed input and child ownership | Actual23 includes fresh child BIN, original stdin seek/EOF, OS32 guard denial, client drop/run unwind and same-owner close; six original child close receipts/raw12 | Actual supplier/runtime/real-tool failure paths beyond this synthetic owner checkpoint |
| Core publication and destination protection | Root foundation actual16 plus actual23 original core single CAS, retained-output hash and existing destination preservation | Cancellation before/after CAS through actual product download, failed persistence/close recovery and product CLI |
| Live source admission | Separate authored browser metadata lease and original-request bridge | Compile compatible old callers; actual browser original generation/request/header correlation and invalidation; no URL/history-based session reconstruction |
| HTTP transport | Separate authored public whole-response200 adapter; no root consumer | Concrete retained SourceOpener and actively driven host runtime; actual supplier response, invalidation/deadline/cancel during send/read, no redirect/retry admission reuse; physical runtime/resolver cleanup evidence kept separate from body drop |
| Probe/media validity | Actual23 synthetic ownership remains separate. Later fixed-tool information5/helper8 observed FFprobe6.1/GPLv3-or-later/Input fd and current private12s MP4 metadata; see [information gate](../../../sots/verification/production/ffprobe-owned-observation-2026-10-23.md). First owned-fd normal08 expired at the unchanged8s observation before probe evidence creation; no real-fd acceptance yet | Preserve old08/15s original attempt and diagnose preparation/control costs; then actual original leased-stdin media/packet/track/container, malformed rejection and exact source-cancellation semantics. Information/file-protocol metadata is not this acceptance |
| HLS VOD | Root has domain HLS parsing files; no download collector/remux executor found | Complete VOD selection, separate audio/init, segment order/discontinuity, bounded retained collection/remux and real output probe; explicitly classify live/encrypted/key-dependent inputs |
| Resume/recovery | No accepted Range download path | Range plus ETag/Last-Modified same-object checks, changed/expired source re-resolution and same media/track check, restart recovery without persisted signed URL/cookie, bounded retry/backoff and cancellation |
| Shared SDK and player-cli | Current root search finds download command declaration in capabilities only | Typed request/normalization and source selection; actual host registry/executor; real CLI success/error/cancel/save/restart, then GUI thin-client consumption |
| Download modes and batch results | Requirements retained in downloads SOT; no completed mode implementation asserted | Distinguish original save/remux/transcode without implicit degradation; source/quality/tracks/container/estimated size; portable collision-safe filenames and per-item partial outcomes |

The four expected main fixture names are `original_stdin_seek_eof_and_single_core_publication`, `dropped_client_retains_original_child_lease_until_cancel_close`, `run_unwind_retains_original_child_lease_until_same_owner_close`, and `rejected_candidate_preserves_existing_destination`. They are not an observed compiled test list. The existing destination case alone does not prove cancellation-after-CAS, restart recovery, real decoding or provider support. Record additional cases at their actual integration layer rather than repeating SDK functionality through GUI automation.

The completed gate consumed only the frozen minimum closure plus separately preserved failure-based revisions under its assigned heavy slot. Actual registration/execution was root20 plus selected owned-process3; the latter BIN registered5, not all executed. Detailed original input/gate proposal is in `runtime/development/media-download-checkpoint-r4/GATE.md`; actual evidence and remaining broad integration checks are in the SOT above. This gate cannot enable the product command by itself. Central deferral of model inference/download remains in force; no model download or provider transfer was started.

Applied the already-read `modootoday/software-qa/code-change-review-verification/SKILL.md`, SHA256 `9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B`, verdict-table reference SHA256 `9D9619A829AE24D73645EF5F7C8126C8257CAEDDAC80800632866257008AD2A9`: each row separates current source observations, authored assertions, prior actual evidence and missing acceptance. The fixture skill above supplies isolated case/teardown rules; neither skill licenses synthetic evidence as actual service/media support.

## Subsequent real-fd gate in progress

The synthetic actual23 source12 was subsequently adopted as bddeca87a129e89633833337ee86f6ce8934a6d5; its earlier source-only statements above are historical. New real-file source2 BB5 has actual registration23 (old20/newignored3), fmt-onlyr2 F5A703 and fixed BIN F1576C7C9759152A6F1A72C263E42D8E17DE1E8DB70A56413F7E785495D45F34. Root740b/library65cc/lockE469 remains the current assigned base. The first real normal08 failed the existing8s observer; original job reached Running then host-shutdown cancellation and Failed/Cancelled-code without publication. Original host Complete returned, no tool child was launched, and raw08/BIN/journal are preserved under runtime/verification/download-real-probe-r1. No runtime or policy timeout was enlarged.

SDK temporarily returned heavy with process/handle/session/cleanup0 while Meari uses its separate library gate. New diagnosticr3 D98134B0EE907F617D52ABBB1CE25CEC4EAC24ACA0C4C21FB67BC3786C528528 measures only the original control delegation and prepare progress; it is not applied or executed. The first next SDK slot runs this old-production diagnostic1. Typed BCrypt/checkpoint repair3 F869A0DD64F5A727F0D7FA907890F4429F1E51E39D81FA22B65C40EEA08F8795 and effective5 1FE50D536653A502CA454C5549EA185623957ABB13A694001BB873CEB4BFA75A are private source-only, not a proven cause or performance fix. Six new unit names and real3/old20 remain expected follow-up validation. Original8s/15s/64KiB checkpoint cadence and original controls/guards stay intact.

The HTTP opener77DA is also private source-only: it retains the same ProviderSession and original host port, but no concrete physical runtime/resolver ACK exists. In particular, HttpTransport's post-send validation can fail before returning a session, so the future original host must retain that earlier Response/request lifetime too. No permissive true/Drop ACK or product CLI support follows from the authored opener. HLS, recovery, modes/batch and all remaining SDK/CLI product requirements above are unchanged.
