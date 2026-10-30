---
id: sot:kinekino/review
title: 원본 review 재생 세션 유스케이스
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 원본 review 재생 세션 유스케이스

2026-10-27 중앙 PM. 소유는 공통 application의 세션 정책과 infrastructure의 고정 로컬 runtime adapter로 분리한다. [원 worker proof 수명](review-scoped-proof-worker.md)과 [저장 Verified 계약](api/review-source-draft-contract.md)을 소비한다. SDK 명령 등록·CLI transport·GUI 완료를 선언하는 문서가 아니다.

## 요청과 세션 권위

Selection은 case·association_id·positive association_revision만 받는다. 요청에서 미디어 경로·도구·argv·mapping 요약·모델을 받지 않는다. SessionRef는 trusted assembly가 host 수명마다 새로 공급하는 host_scope와 ordinal·정확한 Selection으로 구성한다. host scope 재사용은 금지한다. 참조 DTO를 역직렬화해도 세션을 만들지 않으며 원 service의 보유 owner와 완전히 일치해야 한다.

open은 case membership lease를 함수 전체에 보유하고, same case/key의 기존 기록을 control checkpoint보다 먼저 조회한다. 같은 selection은 같은 SessionRef와 최신 관측을 반환하고 다른 selection은 RevisionConflict다. 이 live 관측을 저장 transaction의 immutable receipt replay와 혼동하지 않는다. 신규 요청은 control을 확인하고 모든 이전 owner의 물리 종료를 검사한 뒤 하나만 생성한다. 이 service의 이력은 명시 capacity1~64이며 자동 삭제·key 만료·새 host에서 replay를 약속하지 않는다. 가득 차면 신규 open은 ResourceExhausted이고 기존 key 조회는 보존한다.

상태·닫기도 case lease와 exact host/ordinal/selection을 요구한다. close_requested는 종료 요청 수락이며 physical_closed가 아니다. 정상 닫기 재시도는 중복 request_close를 보내지 않는다. 새 slot은 owner_finished·physical_closed가 모두 참이고, child가 생성됐다면 family_empty·ipc_exited도 참일 때만 허용한다. 관측 실패·외부 owner binding 오류는 원 owner를 버리거나 새 프로세스로 대체하지 않는다. post-start 오류는 owner_retained/session/query_same_open_key/execution_unknown을 반환한다. service Drop은 닫기만 요청하고 runtime은 독립 원 thread가 실제 종료까지 보유한다.

## Host 종료 계약

trusted host의 begin_close는 open/control과 같은 reservation lock에서 새 입장을 영구 차단하고 모든 원 owner에 close를 요청한다. close 요청 실패도 다른 owner의 종료 요청을 막지 않으며 첫 오류와 owner_retained를 반환한다. 호출자는 그 첫 오류를 후속 관측 성공으로 지우지 않는다. 기존 key replay/status/close는 case 권한 아래 보존하고 새 key·control은 거절한다.

observe_shutdown은 begin_close 뒤에만 허용한다. 사라진 case/media 권한을 다시 요구하지 않고 원 owner를 직접 관측한다. owner_finished·physical_closed와 child가 있었을 때 family_empty·ipc_exited 모두 충족한 owner만 physically_finished로 센다. 모든 owner가 충족할 때만 complete다. 관측 오류는 완료가 아니며 원 owner를 유지한다. 이 API는 trusted Rust host용이며 SDK 요청으로 host 전체 종료를 노출하지 않는다. transport 종료·deadline 만료·Drop만으로 물리 정리 성공을 계상하지 않는다.

공통 assembly::review_playback::start_transport는 공급된 PlayerSdk의 기능을 유지하면서 동일한 service를 주입한다. startup 실패는 원 service와 오류를 함께 반환하며 caller가 명시적으로 정리한다. ReviewTransportHost::begin_shutdown은 service 입장을 닫은 뒤 transport 입장을 닫고 PendingReviewTransport에 원 pipe/SDK join과 service를 보유한다. 고정 관측 deadline을 연장하지 않으며 deadline 이후에도 원 pending을 보유·관측할 수 있지만 첫 Timeout을 지우지 않는다. complete는 물리 관측이고 succeeded는 complete이면서 첫 오류가 없을 때만 참이다. Drop은 best effort이며 성공 receipt가 아니다. session reservation을 기다리는 관측은 hard preemption을 보장하지 않는다.

## 고정 로컬 adapter 조립

공통 serve_stdio는 ready JSON write/flush 뒤 stdin EOF 또는 오류까지 원 transport를 유지한다. ready 실패는 read를 건너뛴다. I/O 최초 오류와 cleanup 오류는 별도로 보존하고 성공 판정은 둘 다 없어야 한다. cleanup 관측10초 만료 때는 원 PendingReviewTransport를 반환한다. into_completion은 원 worker join/모든 session 물리 완료를 관측한 경우에만 소비하며 Drop·시간초과를 성공 receipt로 바꾸지 않는다. 실제 BrokenPipe에서도 원 미디어 정상 종료와 host 오류 종료는 동시에 성립할 수 있다. [검증](../../verification/application/review-stdio-20261027.md).

assemble_sdk는 하나의 LocalReviewSessionConfig에서 model review reader·source 저장/검증 API·playback service 세 adapter를 함께 조립한다. supplied PlayerSdk의 그 외 기능은 보존한다. source와 playback은 같은 root·trusted admission을 사용하며 서로 다른 JSON 설정을 읽지 않는다. 원 세 root를 먼저 보유·분리 확인한 뒤 writable user store를 초기화한다. readonly는 초기화하지 않는다. runtime/local launcher는 원 ordinary 경로를 사용한다. 기존 Rust-only admission 결정과 사용자 AI품질 판정 권위는 바꾸지 않는다.

assemble_sdk의 writable은 user-review source 저장소에 적용한다. model review의 기존 decision 계약을 전체 readonly 모드로 바꾸는 옵션이 아니며, 품질 승인 주체·명시 판정 규칙도 변경하지 않는다. 조립만으로 decision을 생성하거나 원본 package/object를 수정하지 않는다.

공통 assembly::review_playback::assemble_local(config,max_history)가 OS가 새로 생성한 scope와 하나의 service를 조립한다. caller는 이 service를 GUI/CLI 공통 PlayerSdk에 주입하고 host를 새로 만들 때 scope를 재사용하지 않는다. [실제 수명 재생성 근거](../../verification/application/review-host-assembly-20261027.md)는 동일 프로세스 내 새 host에서 ordinal 재사용·상호 참조 거절을 검증했다. 기본 제품 host 자동 설정과 별도 프로세스 재시작 전수 인수는 아직 남는다.

LocalReviewSessionConfig는 별도 review/user/runtime root·고정 tool SHA·명시 profile/Job budget·trusted timeline nomination·전체/정리/raw 한도를 필수 Rust 입력으로 받는다. Default/Deserialize는 제공하지 않는다. root guard를 보유하고 runtime root는 다른 두 root와 겹치지 않는다. 외부 요청으로 이 설정을 갱신하지 않는다.

start는 worker를 하나 만들고 준비 결과는 관측으로 전달한다. 따라서 최초 open의 owner 생성은 재생 준비 완료가 아니다. 원 worker는 readonly transaction의 exact association revision을 보유하고 Verified record와 새 retained proof의 정확 일치를 검증한다. Unverified·missing·wrong binding이면 child 생성 전에 실패한다. 첫 준비 오류의 원 ErrorCode와 진단 문자열은 failure 관측으로 보존하며 Unverified는 NeedsInput이다. 일반 기존 runtime 문자열 오류는 EngineFailed로 분류하고 이를 성공/종료로 바꾸지 않는다.

Verified 확인 뒤 고정 runtime의 독점 lock/Job을 먼저 예약하고 무거운 proof·원본 검증을 진행한다. trusted host는 모든 review caller에 같은 runtime root를 공급해야 하며 이 한도를 별도 root로 우회하지 않는다. 원 envelope·case·proof·media·tool·Job을 run 종료까지 보유한다. 기본 low-level spawn의 호환 경로를 제품 Unverified 지원으로 광고하지 않는다. nomination은 명시한 case/media/producer만 허용하며 다른 실제9 표본으로 일반화하지 않는다.

현재 adapter는 고정 headless CPU runtime으로 창/소리/시각 품질을 입증하지 않는다. paused engine 연결은 coverage 전체·frame-exact·AI 품질 승인이 아니다. [공개4명령 wire](api/review-playback-contract.md)와 source-bound 제어·fresh scope 공통 조립·[명시 transport 종료](../../verification/application/review-transport-20261027.md)는 실제 CLI까지 검증됐다. 제품 기본 host 설정·trusted admission 공급·별도 프로세스 재시작/실패/취소 전체 인수는 남는다. 무거운 작업1 중앙 배정·기존 소유권/WIP·수동 GUI/추론/다운로드/WARP/push 보류를 유지한다.
