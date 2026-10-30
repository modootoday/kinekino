---
id: sot:kinekino/review-playback-sdk-cli
title: Review playback SDK/CLI 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Review playback SDK/CLI 계약

2026-10-27 중앙 PM. [세션 유스케이스](../review-playback-sessions.md)를 얇은 DTO/schema/dispatch로 소비한다. 기능 정책·원 owner·proof 검증은 application/infrastructure에 유지한다.

## 명령과 context

공개 명령은 reviews.playback.open/status/close/control이다. capability의 고유 명령 목록은97개이며 명령 발견 수를 기능 성공률로 계산하지 않는다. default SDK/CLI는 manager 미주입으로 네 명령과 schema status가 모두 unsupported다. trusted host가 Arc<ReviewPlaybackSessions>를 PlayerSdk.with_review_playback_sessions로 주입하면 세 수명 명령을 제공하며 control은 launcher.controls_available도 요구한다. tool/root/budget/nomination은 요청에서 받지 않는다.

control payload는 {session,engine_instance_id,control}이다. control은 Play/Pause/Seek/SetVolume/SetMuted의 공통 PlaybackControl 계약을 따른다. context.expected_revision/idempotency_key/timeout_ms는 금지하며 고정 worker 응답 한도2초를 사용한다. source/session/instance/원 proof를 발행 전에 확인하고 close 이후 새 제어를 거절한다. data:{session,receipt}의 accepted/unobserved는 실행 관측이 아니며 재시도 권한도 아니다. 발행 여부가 불확실한 오류는 execution_unknown으로 보존한다.

Play는 연결·paused 관측·retained coverage 내부 clock이 필요하다. Seek는 coverage 양 끝점을 포함한다. 관측 재생 clock이 범위를 벗어나면 같은 owner가 Pause를 발행하고500ms 내 관측되지 않으면 원 정리 경로를 수행한다. 이는 polling 관측 정책이며 frame/sample 단위 hard bound가 아니다. [실제 CLI 근거](../../../verification/application/review-session-control-cli-20261027.md).

open payload는 selection:{case,association_id,association_revision}이다. context.idempotency_key는 필수이며 expected_revision은 null만 허용한다. timeout_ms는 신규 owner 입장의 한도1~30000ms(기본30000)이고 worker의 고정 runtime deadline을 연장하지 않는다. 같은 case/key의 committed session은 deadline checkpoint보다 먼저 조회한다. 같은 키·다른 선택은 RevisionConflict다.

status/close payload는 session:{host_scope,ordinal,selection}이다. context.expected_revision/idempotency_key/timeout_ms는 null 또는 생략만 허용한다. close는 해당 session의 idempotent 종료 요청이며 별도 mutation key가 없다. DTO의 알 수 없는 필드·caller executable/path/mapping은 거절한다. 신규 owner 생성 뒤의 관측 오류는 owner_retained/session/query_same_open_key/execution_unknown을 보존한다.

## 응답·관측

수명 명령 data는 SessionView:{session,close_requested,observation}이다. observation은 process_created/connected/closing/physical_closed/family_empty/ipc_exited/owner_finished/engine_instance_id/has_error/failure와 position_us/duration_us/paused/volume_percent/muted를 반환한다. 미관측 값은 null이며 요청 수락으로 채우지 않는다. failure의 준비 단계 원 ErrorCode를 유지한다. message는 기술 진단이며 GUI의 제품 문구로 그대로 채택하지 않는다.

open의 ok는 원 owner가 보유됐다는 뜻이며 연결·재생 완료가 아니다. close_requested는 종료 요청 수락이고 physical_closed가 아니다. 상태를 조회해 실제 child/Job/IPC/owner 완료를 확인한다. Unverified 입력은 준비 단계 failure NeedsInput과 child 미생성으로 나타날 수 있으며 open 수락 응답을 재생 성공으로 계산하지 않는다. 종료 뒤 reviews.playback.status는 case 권한 아래 보존된 terminal session을 반환한다. 이는 generic playback.status의 live 관측 만료/NotFound 계약과 구분한다.

capabilities.review_playback은 manager availability/max_history/host 수명 replay/physical 종료 필수/frame_exact=false/quality_approved=false를 표시한다. schema는 요청과 관측의 실제 wire 구조를 구분한다. 연결된 paused headless engine을 source coverage 전체·frame-exact·사용자 품질 승인으로 표시하지 않는다.

## 검증·미완료

SOURCEFECF의 SDK 경계7시험·실제 normal CLI18process·기본96명령 discovery 및190/정상bins/전후입력동일을 [새 wire 근거](../../../verification/application/review-session-cli-20261027.md)에서 확인했다. 기존 typed 실제 시험은 [4312입력 근거](../../../verification/application/review-playback-sessions-20261027.md)로 보존하며 이번 wire 구현의 실제 CLI 성공으로 승계하지 않았다. 제품 기본 host의 fresh scope/구성 공급·command별 재생 제어·재시작/실패/취소 전수·전체 CLI 인수·GUI/AI는 전체 요구에 남는다. 수동 GUI·추론·다운로드·WARP·push 보류·heavy1/C cache·기존 WIP/소유권을 유지한다.
