---
id: evidence:kinekino/20261029114248
title: 회차 전환 준비본 관측 — 2026-10-19
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 회차 전환 준비본 관측 — 2026-10-19

실패 출력 보존을 보강했다. 초기화 실패는 cleanup 성공이어도 수집한 Mpv를 진단 소유자에 반환하고 open/replace 공통 handler에서 기존 private catalog에 record한다. cleanup 미확인 또는 진단 저장 실패면 Active에 보존해 같은 instance의 명시 close/host shutdown으로 재처리한다. 공개 오류는 저장 성패와 안전한 artifact/code만 반환한다. 미등록 후보이며 저장 실패 주입·재조회·실행은 미검증이다.

부분 초기화 오류에서 미시작 stream 기본값을 complete로 오인하지 않도록 process_output/diagnostics 비교 후보를 추가했다. 공개 receipt 구조는 유지하고 private wrapper에 collector 시작/EOF 관측을 추가한다. 새 기본/빈 EOF 구별1개와 complete 조건1개를 포함해 준비 검사는12개다(기존 bounded-prefix 검사는 시작/EOF assertion만 보강). 아직 미실행이다. 정상 r3는 두 collector 생성 뒤 종료를 확인한 기존 checkpoint이며, 이 부분 초기화 경로의 증거로 소급하지 않는다.

owned_startup에 cfg(test) 전용 단계/cleanup 실패 주입과 Rust 테스트2개를 준비했다. 현재 unit-test executable의 동일 테스트를 명시 child probe로 실행하고(2초 자가 종료 fallback), stdout worker 실패 시 기존 stderr reader/child 정리 확인과 cleanup 미확인 recovery 보존→명시 shutdown/wait 재시도를 검사하도록 작성했다. OS 도구/셸 fixture가 아니다. 제품 요청에서 fault를 선택할 수 없으며 아직 테스트/child probe를 실행하지 않았다. 준비 테스트는 기존8+2=10개이고 통과 수치가 아니다.

actual_next helper를 ActualTransitionLeg(direction, expected_target_descriptor_sha)로 묶어 Next/Previous에 같은 assertion을 적용하도록 정리했다. 방향별 wire command·resolve key·로그/PNG prefix를 분리해 증거 덮어쓰기를 피한다. 두 방향 모두 같은 old 유지/대상 identity/영상·시간·전체 PNG/중복/소비 handle/old ID 검사를 사용한다. 실제 상위 driver 연결과 양방향 실행은 아직 없다.

submit 뒤 반환된 job record 검증이 실패해도 durable job 접수는 이미 일어났을 수 있다. resolve_job 초안은 이 오류에 job ID/attempt/created와 handle 미생성·엔진 미교체·job cleanup 미확인을 보존하도록 수정했다. 접수 전 실패처럼 처리하거나 취소 완료로 주장하지 않는다. 실제 손상 record 부정 인수는 미실행이다.

resolve_job의 completed output 검사를 resolved_artifact 함수로 분리하고 요청 descriptor/종류 불일치·출력 hash/bytes 불일치·누락/불완전 RawArtifact를 거부하는 Rust 테스트3개를 추가했다. 기존5개와 합쳐 준비8개이며 모두 미실행이다. 이 helper는 영수증 내부 결합만 검사하고 실제 raw 재검증은 이후 기존 EpisodeResolutionResults reader가 수행한다. 원문 검증이나 playback 성공을 테스트 fixture만으로 주장하지 않는다.

actual_next helper에는 진행 중 두 번째 begin의 RevisionConflict와 전후 완전한 jobs 목록 ID 동일성, resolve 중 old instance/artifact/무오류 유지, commit 뒤 재적용·소비 handle 조회의 NotFound 및 새 instance 보존 검사를 추가했다. jobs 목록은 cursor가null인지 확인해 일부 페이지로 전체 무접수를 주장하지 않는다. 아직 미실행이며 실제 인수 성공/테스트 통과 수치로 집계하지 않는다.

현재 제품 기준은 [옵션 r3 실제 인수](native-options-cli-2026-10-19.md)다. 다른 회차 전환은 아직 제품에 등록되지 않았다. 아래는 `runtime/coordination/native-transition/`에 있는 Rust 준비본의 읽기·포맷 관측이며 타입 검사·테스트·실제 실행 증거가 아니다. 파일 해시는 같은 디렉토리의 `source-sha256.txt`에 있다.

| 준비 파일 | 등록할 책임/위치 | 현재 내용 |
| --- | --- | --- |
| adjacency | application/playback_session/transition | 검증 목록/현재 결과→기존 EpisodeQueue→인접 대상→검증 source 생성 |
| transition_module | 같은 application 하위 mod.rs | 내부5모듈과 공개 facade/DTO/port만 재수출; 제품은 runtime 경로를 include하지 않음 |
| service | 같은 application 하위의 교체 port | old instance/artifact 원자 대조와 이전 종료/새 process receipt |
| resolve_job | 같은 application 하위의 작업 조정 | 기존 JobsApi 제출/조회, ID/attempt/parameters/영수증/출력 해시·크기 대조 |
| slot | 같은 application 하위의 소유 수명 | pending1개, checked handle, 1회 commit, 명시 release, 영구 end |
| coordinator | 같은 application 하위의 facade | 제한된 begin/observe/commit/release DTO, 접수에서 handle+job ID 함께 반환 |
| native_replacement | infrastructure/mpv/session_engine child | 한 owner 잠금으로 old 대조/종료→launch→게시, EOF·정리 실패 소유 상태 |
| owned_startup | infrastructure/mpv/owned_startup.rs | spawn 직후 Mpv 소유, 단계별 구성 실패의 cleanup 결과와 recovery 객체를 typed 반환하는 초안 |
| owned_launch | infrastructure/mpv/owned_launch.rs | 고정 argv/검증 source를 typed owned startup으로 전달; raw Command는 외부 payload가 아님 |
| session_engine | 기존 infrastructure/mpv/session_engine.rs 비교 후보 | Active::from_mpv 공통 조립, open 초기화 오류 recovery를 Active에 보존 |
| process_output | 기존 infrastructure/mpv/process_output.rs 비교 후보 | collector_started/eof_observed 관측으로 미시작 빈 buffer와 EOF 빈 stream 구별 |
| diagnostics | 기존 infrastructure/mpv/diagnostics.rs 비교 후보 | worker 종료·양 collector 시작/EOF·무손실을 모두 요구해 complete 계산 |
| combined_host | infrastructure의 공유 조립 | 기존 native/provider/slot 소유, trusted native 검증 우선, 모든 shutdown 평가 |
| api | application/api | previous/next 및 transition 조회/적용/해제 typed 요청 어댑터 |
| api_mod | 기존 application/api/mod.rs의 등록 후보 | Option 초기화·generic builder4곳 전달·with_episode_transitions·execute/schema/capabilities 연결 전체 후보 |
| schema | application/api | 기존 native envelope/open/close 및 JobState 재사용, 지원 경계 |
| client | application/api/client child | 기존 review_call decoder를 재사용하는 typed client 확장 |
| cli_host | bin/player-cli의 입력·출력/조립 | 명시 serve-playback와5옵션, 공유 SDK/host 조립, EOF·입력 오류 뒤 명시 shutdown |
| actual_next | tests/jobs/native_episode child | 별도 두 회차의 target descriptor/새 영상·시간·전체 PNG/old ID 거부 인수 준비 |
| actual_driver | 기존 tests/jobs/native_episode.rs의 비교 후보 | combined mode·명시 fresh jobs/profile·목록 raw/result 재게시·Next/Previous 두 leg·최신 instance의 options/4controls/EOF 연결 |

20개 모두 미등록·미컴파일·미실행이다. 작성/수리 때 파일별 rustfmt와 --check를 실행해0을 확인했다. module/API/session engine root·actual driver 후보는 이동 전 대상명으로 선언하므로 skip_children=true로 포맷만 확인했다. 기존 source 생성 가시성을 넓히지 않고 application/playback_session 하위 등록을 전제로 한다. 새 worker/transport/GUI 자동화는 만들지 않았다. with_episode_transitions builder는 api_mod 후보에만 있고 제품 SDK에는 없다. runner mode 분기·global COMMANDS3개·client child 재수출·부모 mod 선언도 실제 등록 단계에 연결해야 한다. API/actual 기준 파일 해시는 각 baseline-sha256.txt에 보존했다. 전체 기존 파일 후보는 등록 때 현재 소스와 비교해 필요한 변경만 반영한다.

준비된 경계 테스트는5개이며 실행하지 않았다. adjacency3개는 목록 순서/중복/동일 제목/비숫자 slug/첫·끝/누락·stale·외부 URL을, slot2개는 EOF 뒤 제출 방지와 handle overflow를 다룬다. 실제 helper는 작업30초·native 진행30초와 기존 capture4초 게시기한을 전제로 한다. PNG는 hash/bytes/engine/artifact/dimensions 및 독립 next_frame+finish를 검사하도록 작성했다. 최종 EOF·기존 options/4controls 연결은 남는다.

오류 관측의 구별: launch_episode는 spawn 이후 worker 구성 실패도 같은 오류로 반환한다. 따라서 launch 오류의 new_process_created는null, launch 전 EOF는false, launch 후 EOF는true로 준비했다. replacement_accepted와 new_engine_owned는 별개며 새 정리 실패는 owned=true/cleanup_confirmed=false다. 생성null/owned=false는 초기화 중 child 정리를 확인했다는 뜻이 아니다.

기존 mpv/mod.rs의 spawn_process worker 구성 실패 경로에는 kill/wait/join 결과를 버리는 지점이 있다. 이는 읽기로 확인한 미검증 위험이고 실제 실패 재현은 없다. 정상 옵션 r3 종료 성공을 이 오류 경로에 소급하지 않는다. 소유권/정리 결과 보존과 fault injection이 남는다.

owned_startup 초안은 trusted launch argv만 받아 child spawn 직후 Mpv에 담고 stdout/stderr/IPC 구성 실패 때 기존 shutdown의 결과를 검사한다. 정리 실패는 Box<Mpv> recovery를 OwnedStartupFailure에 보존하며 URL/argv/OS 오류 문자열 없이 단계/instance/생성/cleanup 상태만 반환한다. owned_launch·session_engine 후보와 native_replacement 초안에 into_parts 후 recovery를 Active에 보존하는 연결을 작성했다. 초기화 실패 스냅샷은 명시 error이며 새 open을 거부하고 observe/동일 instance close로 정리를 재시도할 소유권을 유지한다. fault injection·타입 검사·실행은 아직 없다. 기존 제품은 아직 이 경로를 쓰지 않으므로 문제가 해결됐다고 표시하지 않는다. legacy spawn 경로는 GUI 공통 엔진 이관 때 별도로 검증하고 임의 삭제하지 않는다.

준비본 검토에서 target의 native 전송 정책 검사보다 old close가 먼저였던 순서를 수정했다. StoredEpisodeResolution이 허용하는 명시 Empty Referer도 native adapter에서는 Unsupported이므로 기존 network_plan::arguments를 old close 전에 호출한다. 정책을 복제하지 않고 실제 launch validator를 재사용한다. native 불가 target 때문에 old 엔진을 먼저 닫는 경로를 줄인 소스 수리이며 미컴파일·미실행이다. 등록 뒤 이 target의 거부와 old instance 유지 부정 검사가 필요하다.

resolve submission key는 기존 JobsApi 정책을 사용한다. 키 재사용으로 역사적 완료 작업이 반환될 수 있으므로 fresh resolve 인수는 새 작업/시도를 확인해야 한다. 전체 전환 idempotency/revision/완료 timeout, 자동 commit/retry/restart, release에 의한 취소 완료를 제공한다고 표시하지 않는다. commit receipt는 process 생성이며 별도 native 영상·시간 관측이 필요하다.

UI NeedsRepair 소비는 반환됐으며 현재 중앙 우선순위는 Meari Repeat→원본 Card/Highlight 소비다. SDK 등록·heavy는 별도 checkpoint로 조율한다. 제품 root는 옵션 r3 입력을 유지하며 준비본만 변경했다. [남은 구현·검증 계획](../../../plans/player/native-episode-transition.md)을 따른다.
