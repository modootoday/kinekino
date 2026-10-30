---
id: sot:kinekino/legacy-lifecycle-gui-cli
title: legacy 재생 lifecycle와 GUI/CLI 인수 경계
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# legacy 재생 lifecycle와 GUI/CLI 인수 경계

2026-10-19 [공통 API 이관 조사](catalog-episodes-playback-api-handoff.md)의 실행 수명·인수 자료다. 현재 source를 읽은 사실과 다음 구현의 조건을 분리하며 새 재생·browser·프로세스를 실행하지 않았다.

## 현재 재사용할 책임

[PlaybackRoute](../../../../../../src/application/playback_route.rs)는 명시적 Web/native 선택과 동일 generation 자동 재시도 억제를 담당한다. [EpisodeTransition](../../../../../../src/application/episode_transition.rs)은 최신 선택·기존 paused/native 의도·page 준비·일회성 소비·취소·30초 만료를 관리한다. [NativePauseIntent](../../../../../../src/application/native_playback.rs)는 startup 기본 bool을 사용자 의도로 오인하지 않게 한다. 이 정책을 공통 controller에서 재사용하며 화면별로 복제하지 않는다.

현재 [transport UI](../../../../../../src/presentation/transport.rs)가 media 후보로 mpv를 직접 생성하고 web 복귀·seek/toggle을 조합한다. [workspace](../../../../../../src/presentation/workspace.rs)의 local open도 직접 player를 생성한다. [PlayerApp](../../../../../../src/presentation/player_app.rs)의 event pump·native 관측·전환 처리가 전체 lifecycle을 연결한다. 이 orchestration을 application으로 옮겨 GUI와 CLI가 같은 session/ticket 정책을 호출하게 해야 한다.

[mpv adapter](../../../../../../src/infrastructure/mpv/mod.rs)는 인접 mpv.exe 발견·고정 argv·별도 IPC pipe·소유 child 종료를 맡는다. source는 인자 구분자 뒤에 전달하며 shell이나 요청 제공 program/flags를 실행하지 않는다. 현재 실행 시 검사는 is_file 중심이고 과거 배포/해시 기록이 매 실행의 바이너리 인증을 대신하지 않는다. executable 발견·버전/해시·프로세스·OS surface는 infrastructure 책임으로 남긴다.

native source의 parent window handle을 공통 API 입력으로 직접 받지 않는다. application의 surface 식별을 조립된 platform surface로 매핑하고, 독립 CLI에 필요한 renderer/host adapter가 없으면 capability를 제한한다. CLI는 GUI 버튼·주입 클릭·창 좌표에 의존하지 않는다.

## 기존 공통 port의 정확한 의미

[PlaybackStatusPort](../../../../../../src/application/playback_status/mod.rs)는 실제 native 관측을 전달하며 미관측 pause/idle/time을null로 유지한다. published_age는 발행 나이이며 실제 sensor event 나이와 같다고 보장하지 않는다. position은 engine playback time이고 source PTS가 아니다.

[PlaybackControlPort](../../../../../../src/application/playback_control/mod.rs)의 play/pause/seek/volume receipt는 acceptance=accepted, completion=unobserved다. 큐 제출 성공을 실제 적용·화면 재생·정확한 seek 완료로 승격하지 않는다. [native binder](../../../../../../src/infrastructure/native_playback_control.rs)와 GUI의 shared client 경로는 재사용할 기반이며 open/close/backend/queue 전체 이관 완료는 아니다.

새 PlaybackSessionPort는 validated source lease·원래 회차와 engine instance·선택 route·전환 revision·기대 paused 상태·deadline을 결합한다. 생성/queued/opening/관측된 재생/실패/종료를 구별하고 늦은 callback이나 이전 engine 제어가 새 session에 적용되지 않게 한다. 이전/다음은 실제 목록/현재 회차/경계에서 계산하며 제목 숫자나 URL 변환을 추측하지 않는다.

## 구현 뒤 필요한 GUI/CLI 인수

| 인수 대상 | 같은 application 유스케이스로 입증할 조건 |
|---|---|
| discovery/schema | 실제 provider 주입에 따라 command 지원을 표시. 이름만 등록·빈 payload schema인 명령은 unsupported 유지 |
| catalog/list | 같은 query/work snapshot에 같은 ID/순서/원문 href. fresh empty와 미관측/parser drift/timeout을 구별 |
| resolve | 원본회차→허용 route→media 관측의 ticket/generation/provenance 결합. blob·외부 scheme·lookalike host·stale 결과 거부 |
| open/전환 | 같은 source lease·mode·paused 의도를 보존. native 미준비 중 웹 상태를 임의 성공으로 바꾸지 않음 |
| 빠른 이전/다음·취소 | 최신 선택만 반영, 카탈로그 이동 후 자동 재진입 없음, 경계 wrap 없음, 만료 뒤 늦은 결과 적용 없음 |
| control/status | 동일 engine_instance_id와 typed 값·오류. accepted와 applied 관측을 분리하고 null을0/false로 메우지 않음 |
| close/재시작 | 소유 child/IPC만 정리. session·lease 만료를 표시하고 다른 사용자/세션 프로세스를 이름으로 종료하지 않음 |
| 경로 동등성 | GUI와 endpoint CLI의 결과/오류·revision 일치. standalone provider 부재는 같은 실패 계약. GUI 자동화로 parity를 대체하지 않음 |

우선 모델·네트워크 없는 port fake/고정 HTML·IPC·manifest fixture로 규칙을 검증하고, 실제 사이트·native 수명·독립 CLI는 별도 후속 인수로 구분한다. 이번 읽기 전용 조사에서는 어느 새 실기도 실행하지 않았다.

## 오류와 미지원의 제안 경계

기존 공통 ErrorCode를 사용한다. 어댑터/기능 미제공은 unsupported, 잘못된 입력은 invalid_argument, 유효 대상 없음은 not_found, 사용자 입력·세션 준비 필요는 needs_input, stale lease는 source_expired, 실제 deadline은 timeout, 취소는 cancelled, 실제 backend 실패는 engine_failed로 구별할 것을 제안한다. source 취득 실패를 빈 목록·완료로 숨기지 않는다. retryable·details·remediation·revision 요구는 SDK 소유 계약에서 확정한다.

현재 없는 lifecycle payload/schema, source lease·backend capabilities·browser actor, resolver/queue 저장·관측 freshness·명령별 실제 적용 receipt·독립 provider runtime이 남았다. [SDK 정본](../../../application/api/sdk-contract.md)과 [기존 native 제어 검증](../../../../verification/application/native-playback-control-2026-10-19.md)을 기준으로 후속 인수를 분리한다. 이 자료가 API 지원·GUI 시각 일치·전체 재생 품질 통과를 뜻하지 않는다.
