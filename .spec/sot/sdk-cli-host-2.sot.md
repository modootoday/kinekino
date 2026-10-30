---
id: sot:kinekino/sdk-cli-host-2
title: SDK/CLI host 주입과 명령 표면 경계
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# SDK/CLI host 주입과 명령 표면 경계

2026-10-27 source375 private overlay 후속: 정상 file CLI는 LocalSpeakerFiles를 주입해 subtitles.speakers/edit_speakers를 지원한다. 기본 SDK/별도 review host의 미주입 상태는 Unsupported다. 기본 file CLI 9/97, 명시 model-root 구성 12/97 발견이며 전체 제품 인수가 아니다. [수동 계약](../analysis/manual-speaker-annotations.md).

2026-10-26 source 읽기 기준. 설계 소유 SDK, 통합 PM. 이 문서는 기존 sdk-contract의 조건부 지원을 구체화한다. 새 실행/제품 기능 추가가 아니며 canonical source 조사와 source375 private overlay의 미검증을 구별한다. source375 채택 시 차이 재대조가 필요하다. 명령별 wire schema의 원 권위는 실제 handler/schema다. 이 표는 public API를 변경하지 않는다.

## host 조립

| 조립 | 실제 caller / 주입 | 허용 범위와 주의 |
|---|---|---|
| 기본 SDK | api/mod.rs PlayerSdk::default/new | discovery 외 기능은 실제 주입 port에 종속. default unavailable을 제품 전체 미구현으로 해석하지 않음 |
| 기본 file CLI | src/bin/player-cli/runner.rs execute_local | SubtitleFiles/FfmpegFrameCapture discover, Windows snapshot adapter. decoder 발견은 실제 미디어 성공이 아님. PNG snapshot 전용 분기는 decoder를 discover하지 않음 |
| local roots | runner execute_local의 catalog/job/proposal/project/review root | 요청별 root를 열어 같은 SDK에 주입. Windows path guard 필요. --endpoint와 local roots 동시 사용 거절. roots가 있다는 이유로 live host/worker를 자동 시작하지 않음 |
| jobs host | src/bin/player-cli/job_host.rs | explicit worker + JobsApi. 제공 operation 목록은 해당 host capability. 기본 자막 convert/export_range와 별도 VAD opt-in; install/model 권한은 caller flag로 생성 안 됨 |
| provider host | provider_host.rs RetainingProviderJobsHost | JobsApi+catalog, catalog/episode browser 작업. transport ready와 browser/task/media ready 별개, site_verified=false 유지 |
| native host | native_host.rs + trusted native config | 명시 catalog artifact/mpv path+SHA/session 기반 native. 원 engine identity가 필요하며 process spawn≠loaded/decode |
| combined playback | playback_host.rs RetainingPlaybackWorkspaceHost | jobs/catalog/shared episode session/skip host/transition 주입. 원 host 소유권 하에서 resolve→observe→명시 commit. 자동 재시도·host 재시작 복원 없음 |
| endpoint client | local_host transport + runner execute_endpoint | 한 요청을 현재 사용자의 명시 host로 전달. 그 host capability가 권위. 전송 실패는 not_sent 또는 execution_unknown으로 구별, local fallback/자동 retry 없음 |
| GUI typed host | 동일 ApplicationClientHandle + sdk_execution + product plugin | SDK 정책 복제 금지. window/file picker는 별도 환경 capability. GUI 존재로 CLI 미지원 기능을 대신하지 않음 |

## 명령군과 노출 조건

각 행은 COMMANDS 및 mod.rs dispatch의 분류다. 포트가 없거나 실제 executor가 등록되지 않았으면 Unsupported다. 소스의 이름 존재/다른 host의 성공은 지원 증거가 아니다.

| 명령 | 소유 handler / 필요한 조건 | 제한·추가 인수 |
|---|---|---|
| version, capabilities, schema | queries/schema + 기능별 합성 | 지원표는 해당 instance의 모든 기능별 합성 결과로 판정; base supported()만 읽지 않음 |
| catalog.search, episodes.list | api/catalog + CatalogApi | direct HTTP/archive/provider 저장 결과 구별. category/page와 실제 text/genre/sort 지원 구별 |
| catalog.read_raw, episodes.inspect_resolution | catalog archive/read port | fixed result identity·chunk 한도, raw URL 자동 실행 없음 |
| episodes.resolve | jobs의 operation/host 경로 | 명령 선언만으로 direct SDK 실행 지원을 추정하지 않음. provider/combined jobs 등록 확인 필요 |
| playback.open, engine, status, close | native_session/shared episode sessions 또는 별도 status port | engine_instance_id/원 resolution, generation·종료 소유권 필요 |
| playback.play, pause, seek, set_volume, set_muted, set_rate, select_track | playback_control 또는 native session | capability별 지원. accepted/unobserved≠실제 적용. seek engine time≠source frame PTS |
| playback.audio_handoff | api/audio_handoff + control audio_handoff_available | 빈 payload, 실제 관측만 표시; 소리/청취 품질 승인 아님 |
| playback.previous, next, transition.observe, transition.commit, transition.release | api/episode_transition + EpisodeTransitions | 원 verified list와 owned transition. commit 없이 resolve 완료를 재생 완료로 표시 안 됨 |
| segments.open, inspect, evaluate, edit, accept_skip, set_skip_policy, skip, override, close | skip_segments/host + SkipHostApi | sessions/seek_submission은 별도 capability. 일반 guarded scope를 임의 해제하지 않음 |
| segments.skip_export, export_list, export_peek, export_consume | skip host export capability | issuer/session/원 ACK·consume, 일반 skip capability와 별도 |
| segments.detect | 선언된 요구/operation | 자동 detect/music 제품 executor의 등록·실측은 별도. 수동 skip 성공으로 대체 금지 |
| subtitles.convert, export_range | SubtitleStorage/usecase | UTF8 SRT/VTT·명시 손실동의·신규 파일, ASS/OCR/ASR와 별개 |
| subtitles.speakers, edit_speakers | LocalSpeakerFiles 명시 주입 | source375 수동 JSON 조회·새 revision 편집만 지원. 모델/미디어/품질 입장 아님 |
| subtitles.edit_cues | ProposalStore 명시 주입·cue snapshot correction | 전체 교정 초안 교체·필수 expected_revision·기존 acceptance 해제. 시각/문서 교정·in-flight 취소 비지원. [계약](../analysis/cue-text-editing.md) |
| subtitles.transcribe, diarize, align_speakers | 각 task 계약과 해당 executor/port | 현재 COMMANDS 이름만으로 지원 아님. jobs operation과 별도 typed usecase를 기능별로 조사/등록 |
| production.capture_frame | local FrameCapturePort 또는 engine-bound native capture | local decoder와 원 native session path 구별; frame content/PTS 관측 검증 |
| production.export_snapshot | api/snapshot_export + SnapshotPort | fixed local PNG SHA/명시 pixel crop/새 출력, network·arbitrary decoder 안 됨 |
| production.export_card_review_png | proposals/card_export + ProposalStore + CardImagePort | accepted OutputReviewTarget 기준. 전용 CLI Windows 분기와 feature 기반 capability 발견의 조립 차이를 인수해야 함 |
| proposals.export_subtitles | proposals/subtitle_export + ProposalStore+SubtitleStorage | accepted target, loss 확인, 새 SRT/VTT. 시간 생성/Accept/프로젝트 적용 없음 |
| production.download_media | download JobExecutor/OwnedAttemptExecutor + trusted preparation | 직접 execute가 Unsupported라도 owned lifecycle 경로는 별개. 기본 host 등록과 caller 연결 전 지원 주장 금지 |
| production.render_short, export_cards, segment_subject, build_assets, package_projection | 기능별 production/analysis 계약 | 전체 편집·batch·render executor의 실제 등록/출력 검증이 필요. card-review PNG로 전체 export_cards 완료 대체 금지 |
| models.list, inspect | ModelCatalogPort·명시 --model-root read-only file adapter | fixed revision/manifest·파일별 missing/present_unverified/size_mismatch, runtime/권리/품질 승인 아님. [계약](../analysis/model-catalog-reads.md) |
| models.verify | ModelCatalogPort.verification_available·exact catalog SHA/immutable revision/byte budget | 원 파일 동시 guard·checkpoint hash의 시점 무결성, 불일치 NeedsInput. [계약](../analysis/model-file-verification.md). 실행/권리/품질 승격 없음 |
| models.install, remove | 모델 manifest/admission·lifecycle 후속 | 현재 unsupported 유지. task/revision/license/precision/engine·자원·설치중/참조중 정책과 실제 인수 필요 |
| jobs.get, list, watch | JobsApi read_available | local --job-root readonly 가능. watch는 bounded event page, 자동 무한 stream 아님 |
| jobs.submit, cancel, retry | JobsApi execution_available/원 worker | operation capability·expected attempt/revision·동일 source snapshot 필수 |
| projects.create, inspect, save_revision | ProjectStore (+ live capture는 ProposalStore) | 원 immutable capture/locator 보존, 저장은 media open/적용 아님 |
| projects.reconnect_source, export | 선언된 요구 + 기능별 계약 | 구현 handler/지원 조건의 별도 확인 전 unavailable 유지. 전체 요구는 추적표에 유지 |
| reviews.inspect, read_text_asset, record_decision, decisions | ReviewStore/ReviewSession | 검증 package/assets/session guard, sample-only immutable 판정. 모델 전체 품질/자동 적용 없음 |
| proposals.create, list, inspect, edit, accept, import_historical | ProposalStore + optional list/historical reader | list/historical는 별도 capability. needs_repair 보존, edit/accept와 모델 품질 승인 분리 |
| proposals.edit_capture, accept_capture | capture_mutation + ProjectStore/ProposalStore guarded session | base COMMANDS와 별도 합성되는 표면. origin 없는 legacy는 읽기전용, continuation은 권한 token 아님 |
| proposals.inspect_output_review_target, record_output_decision, output_decisions | output_review session/read port | 현재 target 검증과 저장 이력 current_evidence=not_checked 분리. 사용자 실제 품질 승인 근거는 별도 |

## context는 기능별 계약

공통 RequestContext에 필드가 존재해도 모든 명령이 지원하는 것은 아니다. 현재 직접 source 확인한 규칙은 다음과 같다. 나머지는 각 handler의 validate를 그대로 보존하고 후속 변경 시 schema/client/CLI와 함께 갱신한다.

| 명령군 | expected_revision | idempotency | timeout |
|---|---|---|---|
| jobs.submit | 거절 | context key 필수 | context 거절, task execution_timeout_ms 별도 |
| jobs.retry | 양수 context revision 필수 | 거절 | context 거절 |
| jobs.cancel | context 거절, payload expected_attempt | 거절 | context 거절 |
| jobs.get/list/watch | 거절 | 거절 | context 거절 |
| projects.save_revision | 양수 필수 | 거절 | 거절 |
| projects.create/inspect | 거절 | 거절 | 거절 |
| proposals.edit/accept | context 필수 | 거절 | 거절 |
| proposal create/inspect/import 등 | command별 허용 목록 외 거절 | 거절 | 거절 |
| capture edit/accept | context 거절, payload continuation CAS | 거절 | 거절 |
| output record | 현재 output revision 필수 | context 거절, payload decision_id exact retry | context 거절 |
| output inspect/history | 거절 | 거절 | 거절 |
| reviews.* | 거절 | 거절 | 거절 |
| catalog.*, episodes list/inspect | 거절 | 거절 | context 거절; HTTP timeout 별도 |
| native session/audio handoff | 거절 | 거절 | context 거절; transport/engine 관측기한 별도 |
| card-review/subtitle accepted export | 거절 | 거절 | context 거절 |

제안 기능이 idempotency를 요구하면 기존 명령의 뜻을 바꾸지 말고 별도 typed 계약을 추가한다. CLI --timeout-ms는 ‘강제 취소 보장’이 아니며 handler 거절이나 transport 결과를 보존한다.

## 인수 gate

SB01 host별 capability/schema/client/dispatch 일치, SB02 unavailable port의 no side-effect, SB03 local-root/endpoint 혼용 거절, SB04 각 context 허용/거절과 exact retry, SB05 accepted와 관측 구분, SB06 오류·deadline·EOF·소유 정리, SB07 동일 Request의 CLI/typed host 결과 동등성. 입력 source/host/feature별 실제 --list/시험 receipt가 있어야 실행 통과다. 현재는 source 검토만이며 새 정상 bins/190 또는 사용자 GUI 인수는 미실행이다.

연결: [SDK 계약](sdk-contract.md), [수명·저장](../execution-storage-boundaries.md), [추적표](../../../../plans/player/design-completion/requirements.md).
