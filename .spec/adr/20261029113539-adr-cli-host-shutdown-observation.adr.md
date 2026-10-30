---
id: adr:kinekino/20261029113539
title: CLI host 종료 관측 예산과 비정상 프로세스 경계
status: active
created: 2026-10-29
updated: 2026-10-29
---
# CLI host 종료 관측 예산과 비정상 프로세스 경계

상태: PM 정책 accepted, 2026-10-21. source-only 구현 후보이며 실제 CLI subprocess 인수는 미완료다.

## 관측 사실

LocalHost 명시 종료의 양 worker join 계약은 제품 commit324c3e0b41ae30aad8349bd161f80a14d45e0144와 고유17/양strict/fmt로 인수됐다. 기존 CLI4 Drop은 이 새 명시 종료를 소비하지 않았다. dependency의 기존 shutdown은 동기·소비형이고 별도 worker join을 포함하므로 LocalHost join이 전체 종료 완료를 보장하지 않는다.

## 채택 정책

공통 transport 관측 예산은 단일 `HOST_SHUTDOWN_OBSERVATION_BUDGET = 30초`다. 이 값은 측정된 최대 실행시간이 아닌 PM이 선택한 CLI 종료 정책이다. 공통 `advance`는 원 owner를 반환하고, CLI 관측 driver가 짧은 sleep/poll로 양 join을 기다린다. 대기 thread나 전역 owner leak을 새로 만들지 않는다.

양 정상 join 뒤에는 각 host call site에서 기존 dependency.shutdown을 동기 호출한다. 그 단계는 30초 예산 밖이며 전체 CLI 종료 bounded 보장은 하지 않는다. ready 출력/입력 대기와 종료 진단 I/O 역시 이 transport 예산의 전체 상한 보장 대상이 아니다.

관측 timeout/worker panic에서는 원 Pending/dependency/첫 오류를 runner 최상위 stack에 유지한 채 진단 후 명시 nonzero process::exit한다. 원 Drop·join·SDK 취소가 성공했다고 표시하지 않는다. 정상 stopped/host_lifetime_ended=true 출력0이다. 첫 ApiError가 있으면 원 exit mapping과 상세를 유지하고 추가 join/dependency 진단은 분리한다. 원 오류가 없으면 기존 EngineFailed exit mapping을 사용한다. 출력 실패는 stderr best-effort로 처리하며 성공 fallback을 만들지 않는다.

process::exit는 비정상 프로세스 경계이며 stack Drop을 실행하지 않는다. 의존 worker/자식 프로세스의 정상 종료를 보장하지 않는다. 이 정책은 CLI4에 한하며 GUI main/audio 종료 정책을 바꾸지 않는다. 원 owner를 버리고 정상 u8로 반환하는 방식이나 동기 dependency shutdown을 공통driver 안에 숨기는 방식은 채택하지 않는다.

## 미검증과 인수 조건

Read/Write 주입 EOF·readywrite/flush/read 오류, owner Drop0, 첫 error 보존, timeout/panic 순수 outcome 회귀를 작성한다. 단위 시험에서 직접 process::exit를 호출하지 않는다. 실제 최상위 abnormal exit·진단 실패·30초·EOF를 포함한 subprocess 인수는 중앙 별도 슬롯과 원 PID/receipt/cleanup 근거가 필수다. source-only 포맷 성공을 해당 인수로 승격하지 않는다. 구현 source pin과 독립 읽기·Cargo gate는 별도 인수한다.

단일 stdout DTO 정본은 [CLI 종료 wire](../../sots/player/application/local-host-cli-shutdown-wire.md)다. modoo.today plan/plan-authoring 및 code-change-review-verification 본문·참조를 적용해 관측/채택 정책/미검증을 분리했다.
