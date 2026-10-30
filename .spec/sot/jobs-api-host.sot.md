---
id: sot:kinekino/jobs-api-host
title: 영속 jobs API와 실행 host
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 영속 jobs API와 실행 host

상태: 2026-10-19 root 공통 SDK/CLI에 연결하고 전체185 pass/9 ignored·strict Clippy·고정 CLI/host 인수를 완료했다. [실제 지원·고정 SHA·원문·한계](../../../verification/application/jobs-2026-10-19.md)를 따른다. 기본 backend의 jobs6은 unsupported, 명시 readonly root는3개 조회, 명시 worker host는6개 명령과 SRT/VTT task만 지원한다. [공통 상태](../jobs/state-transitions.md), [분석 경계](../analysis/job-boundaries.md), [제작 경계](../production/job-boundaries.md), [수명 결정](../../../../adrs/jobs/adr-001-explicit-execution-host.md)을 따른다.

## 독립 책임과 수명

`packages/media-jobs`가 GUI·root SDK에 의존하지 않는 job context를 소유한다. domain은 JobId·revision·attempt·상태·취소/완료 경합을, application은 executor/storage/event/identity 포트와 유스케이스를 소유한다. context infrastructure의 자막 executor adapter가 독립 media-subtitles 유스케이스를 재사용한다. root는 API Request dispatch·typed client·명시 CLI 조립·기존 path guard/identity 주입만 담당한다. package에서 kinekino_player/ApplicationClient/GUI를 역참조하지 않는다. project/proposal ID와 job ID를 혼용하지 않는다.

GUI 또는 명시 headless CLI host가 동일 JobService/JobStore와 worker 수명을 소유한다. 숨은 host 자동 기동·상주 서비스를 만들지 않는다. oneshot CLI의 `--job-root`는 get/list/영속 event read를 제공할 수 있지만 executor 수명이 없으면 submit/retry는 unsupported다. 명시 endpoint CLI는 같은 실행 host에 요청하며 GUI 자동화에 의존하지 않는다. headless host도 GUI 없이 같은 application 유스케이스를 실행한다. 실행 host는 한 root에 하나이며 소유 lease 없이 저장된 작업을 중단 처리하지 않는다.

## 첫 실제 작업과 입력

첫 executor는 기존 media-subtitles의 SRT/VTT 변환 유스케이스를 재사용한다. payload·손실 확인·speaker policy·원본 보존·신규 출력 정책을 복제하지 않는다. FFmpeg frame task는 후속이며, 취소 가능한 adapter와 결과 검증 전에는 job executor로 지원하지 않는다. 존재하지 않는 AI executor도 unsupported다.

submit은 알려진 operation과 typed adapter가 검증한 설정, 실제 UTF8 본문의 고정 snapshot/SHA를 받아 영속 admission 후 JobId를 반환한다. source blob과 journal record는 별도로 크기를 제한한다. caller가 지정한 기대 source SHA 불일치는 source_expired이며 ID를 예약하지 않는다. worker는 원래 파일을 다시 열어 snapshot SHA와 대조하고 publication까지 ordinary-file guard를 유지한다. 원래 파일이 바뀌거나 없으면 고정 snapshot을 다른 파일로 대체하지 않고 NeedsInput으로 기록한다. snapshot 보존과 현재 경로 파일의 가용성을 구분한다.

## 상태와 취소

execution_timeout_ms는 worker가 실행을 시작한 시점의 단조 시계 예산이며 queue 대기 시간·IPC transport timeout과 구분한다. domain은1ms..24시간의 저장 문법만 정의하고 실제 executor는 작업별 더 작은 한계를 검증한다. 첫 SRT/VTT adapter는 최대120초다. publication 예약 전과 실제 공통 publisher의 hard-link 확정 전에 deadline을 검사한다. source read가 반환된 뒤에도 deadline을 재검사하며 시간 초과를 취소 ack나 완료로 바꾸지 않는다.

admission 검증 뒤 Queued를 영속화하고 worker가 Running으로 전환한다. 실제 처리 단계만 게시하며 가짜 퍼센트·ETA를 만들지 않는다. 출력 publication/검증 뒤에만 Completed와 결과 receipt를 기록한다. 손실 확인 부족은 NeedsInput, 실행 실패는 Failed다.

cancel은 요청과 worker ack를 구분한다. 요청은 Cancelling이며 그 자체가 Cancelled 성공이 아니다. worker가 publication 전 취소를 확인했을 때만 Cancelled를 기록한다. cancellation 요청과 publication 진입은 같은 gate에서 선형화한다. publication이 이긴 경우 완료·검증 receipt를 유지하고 출력 파일을 삭제하지 않는다. 미완성/모호한 publication은 성공·취소로 가장하지 않고 복구 필요 상태와 원문을 남긴다.

정상 종료는 소유 worker에 취소를 요청하고 ack를 기다린다. 재시작 host가 exclusive ownership을 얻으면 남은 Queued/Running/Cancelling/Verifying 작업을 Interrupted로 기록한다. 자동 재실행·자동 retry는 하지 않는다. retry는 명시 요청·expected job revision을 요구하고 같은 logical JobId의 attempt를 증가시킨다. Completed는 다시 실행하지 않으며 기존 output을 덮어쓰지 않는다.

## 조회·event·bounded watch

첫 파일 adapter는 journal당 최대128개 logical JobId·작업당16 attempts·16384개 event metadata를 제한한다. 유효 상태 전이는 attempt당 최대5개 record라 admission/attempt 제한이 정상 terminal 기록 공간을 남긴다. 자동 이력 삭제·retention은 없다. get은 최신 해당 record와 journal tail, watch는 선택한 event page를 hash/문법 검증한다. 전체 header sequence의 누락은 fail closed하며, 모든 과거 record 본문을 매 조회마다 다시 읽었다고 주장하지 않는다. 선택된 과거 record 손상도 성공 응답으로 대체하지 않는다. list cursor는 고정 through_sequence의 생성 순서/상태를 유지하고 source 본문은 응답하지 않는다. page는 최대1MiB이며 limit200보다 byte bound가 먼저 적용될 수 있다.

get은 한 JobId의 현재 저장 상태, list는 안정 cursor와 limit 기본50/최대200을 반환한다. view에는 input SHA/operation/attempt/상태/실제 단계/결과·오류를 담고 source 본문 blob은 자동 포함하지 않는다. 저장 record와 응답은 transport의 request1MiB/response4MiB보다 작게 제한한다.

watch는 이 첫 단계에서 영속 event sequence를 읽는 bounded 조회다. `after_sequence`와 limit으로 정렬 event batch·next cursor·현재 latest sequence·gap 여부를 반환한다. 스트리밍/무제한 long-poll을 가장하지 않는다. 재연결은 마지막 확정 cursor부터 읽는다. 보존 이력보다 오래된 cursor는 gap과 재조회 필요를 명시하며, journal 손상·중간 sequence 누락은 정상 gap으로 숨기지 않고 fail closed한다. 아직 retention이 없다면 gap=false를 보장할 수 있는 보존 범위만 게시한다.

동기 ApplicationClient/LocalHost bounded dispatcher에서 watch가 event 도착을 기다리며 slot을 점유하지 않는다. durable event page는 실제 상태 전이 sequence를 반환하며 list 별칭이 아니다. GUI는 기존 비동기 worker port로 page를 요청하고 UI thread에서 callback을 기다리거나 unsafe Send로 우회하지 않는다. CLI 지속 출력이 추가되면 별도 명시 NDJSON/poll 모드로 구현하며 현재 단일 response 계약과 구분한다. 재연결 중 동시 cancel이 실제로 처리되는지도 인수에서 확인한다.

## 기존 wire와 인수

submit은 RequestContext.idempotency_key를 필수로 받는다. root·operation·정규화 task params·기대 source SHA·execution budget의 요청 fingerprint를 영속 key hash에 결합한다. 같은 key/같은 정규화 요청은 최초 snapshot을 유지한 현재 JobId/attempt를 반환하며 원래 파일을 다시 읽지 않는다. 다른 요청은 기존 revision_conflict(5)와 details.conflict_kind=idempotency_conflict로 거부한다. 같은 key는 새 inference/변환 실행 지시가 아니다. cancel은 payload.expected_attempt로 새 retry attempt를 잘못 취소하지 않으며 내부 revision CAS로 publication과 경합한다. retry는 context.expected_revision을 요구하고 immutable input을 바꾸지 않는다.

원 모델·엔진의 실제 입출력 인수는 [AI CLI 계약 인계](../../infrastructure/inference/research-followups/model-engine-cli-contract-handoff.md)를 따른다. 현재 executor는 SRT/VTT뿐이며 frame·모델 inference는 unsupported다. 기존9 raw/typed import와 실제 새 inference를 구분하고, model/profile/engine pins·exact request/raw response·typed result·process exit/cancel provenance는 코어 준비 뒤 별도 실제 CLI 인수로 연결한다.

기존60개 command ID는 유지한다. jobs6의 schema/capabilities는 실제 read/execution backend에 따라 명령별 상태를 반환한다. 기본 SDK는 unsupported를 유지한다. projects/proposals 저장 wire·receipt 최신성 false를 변경하지 않는다. idempotency 지원 범위와 충돌 오류·취소 ack·cursor 구조는 typed client와 동일하게 노출하고 API 계약 인계 후 root 조립에 반영한다. [구현 인수 계획](../../../../plans/player/job-persistence-acceptance.md)의 실제 Rust/CLI/host 검증 전까지 새 지원 성공을 주장하지 않는다.
