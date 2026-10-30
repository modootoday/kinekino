---
id: sot:kinekino/worker-2
title: 로컬 검토 미디어 실행 worker
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 로컬 검토 미디어 실행 worker

2026-10-27 중앙 PM. review source를 episode HLS 타입이나 GUI LocalVideo로 위장하지 않는다. 내부 headless CPU 실행부터 공통 SDK control/정상 CLI에 연결하며 실제 GUI rendering과 품질 인수는 별도다.

## 책임과 소유권

- ReviewPlaybackWorker의 trusted Rust prepare closure는 Send인 설정만 worker로 전달하고, 원 ReviewSession/GuardedAdmittedReviewMedia는 worker 안에서 생성한다. 비 Send 세션에 unsafe Send를 추가하지 않는다.
- application의 concrete 입장 owner를 내부에서 source/path guard와 ReviewJobReservation으로 분해하고, 같은 PreparedFamilyJob을 spawn_prepared로 원 child owner에 이동한다. 고정 tool·review 독점 lock·source session은 같은 worker에 남는다.
- argv는 로컬 ISO BMFF용 mov/file 전용이며 config/scripts/ytdl/autoload·외부 자막/오디오 자동 검색을 끈다. headless null video/audio와 software decode·명시 CPU thread를 사용한다. 원본 path/argv/tool SHA/Job budget을 create-new INPUT.json에 기록하고 파일 guard를 종료까지 보유한다.
- 공개 handle Drop은 close 요청만 전달한다. worker는 실제 child exit와 원 Job Empty를 확인할 때까지 살아 있고 source/tool/lock을 보유한다. IPC worker도 실제 종료 후 join한다. timeout/실패를 물리 종료로 대체하지 않으며 미확인 상태에서는 owner를 유지한다.
- 부분 시작 실패는 원 LaunchFailure의 stage/error/termination error와 original process handle을 유지한다. 미할당 suspended child는 원 child exit와 별도 원 Job accounting Empty를 모두 조회한다. 이 조회를 membership나 원 Unknown receipt의 성공으로 바꾸지 않는다.

## 관측과 종료

전체·cleanup 예산은 child 생성 전 단일 Deadline으로 고정하고 이후 요청은 그 예산을 단축한다. startup timeout·소스 invalidation·raw 출력 한도·IPC 중단·사용자 close·자연 종료는 closing으로 전환한다. 연결돼 있으면 기존 fixed IPC quit을 보내고 기존 Supervisor가 EOF/force/실제 종료를 관측한다. 단순 request enqueue·quit 전송·Terminate 성공은 종료 증거가 아니다.

snapshot은 process-created/PID/creation time/instance, 원 NativePlayback 관측과 closing/physical-closed/family/IPC/forced/first-error를 구분한다. control은 기존 SDK typed PlaybackControl을 소비하며 새로운 raw command 입력을 노출하지 않는다. 실행 중 clone된 handle도 closing 이후 새 명령을 받지 않도록 connected를 내린다.

실제 시험은 정상 CLI→SDK→동일 instance seek enqueue 후 독립 clock 관측을 대조한다. 100ms 이내 clock landing을 frame-exact/tick proof 또는 Verified timeline으로 확대하지 않는다. status SDK의 완전한 source-bound 계약, range/Verified 저장·CLI, 실제9/GUI 연결 및 전체 인수는 남는다.

## 보류와 검증 근거

후속 r3 실제 시험에서 시험 LocalHost에 연결한 정상 CLI seek17/25.5초와 같은 instance의17.0/25.5초 clock 및 exit0/비강제/Job Empty/IPC 종료를 확인했다. public 제품 local-open/status/close가 완성됐다는 뜻은 아니다. 정확한 현재 source2172핀·실패 epoch·producer와 원 증거는 아래 실행 검증을 따른다.

수동 GUI·WARP·추론·다운로드·push 보류 유지. producer heavy1/CPU2 및 C cache 문맥을 따른다. [예약](../../verification/application/review-runtime-reservation-20261027.md)과 [실행 검증](../../verification/application/review-local-worker-20261027.md)을 함께 읽는다.
