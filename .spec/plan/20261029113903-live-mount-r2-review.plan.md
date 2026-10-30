---
id: plan:kinekino/20261029113903
title: Live mount R2 source12 읽기 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Live mount R2 source12 읽기 검토

후속 R1 strict02에서 shared predecessor의 tests.rs 미사용 import/EndpointClient::connect 부재/RouteId TryFrom 부재가 실제 확인됐다. 원547을 상속한 R2 6A0A에도 같은 fixture 정합 문제가 남는다. 따라서 아래 초기 정적 검토의 ‘발견0’은 타입 통과 근거가 아니며 현재 알려진 컴파일 오류는 R1 repair1의 별도 R2 rebase revision으로 반영해야 한다. 현재 R2 원본은 보존하고 합성/실행하지 않는다.

source12 SHA6A0A5864E175D5CDC35EC1F4AC493E55F6A646F3A3A588F4DEEFEC7E07C230D0의 읽기 전후 manifest/raw12 일치를 연구 담당이 확인했다. 원 R1 private predecessor와 채택 task3/scoped9를 구별했다. 확정 P1/P2 발견0이며 코드 수정·빌드·실행0이다.

원 mount receipt의 즉시 proof 발급과 실패 carrier, 첫 commit 이후 bind/arm 실패의 HostEffects receipt, AwaitingArm의 같은 proof 재사용·receipt 재생 금지, AwaitingFrame 입력 차단, typed task route·opaque frame/notice 검증·None 거절, 별도 Viewing 복귀 receipt/proof와 기존 close ledger 재사용을 호출 경로로 조사했다. Card strict/Jobs guard를 넓힌 근거는 발견하지 못했다.

작성14는 실제 checked Some→모든 task 활성화·편집·별도 복귀 proof의 양성을 입증하지 않는다. 실제 notifier 오류/재arm, postcommit proof 발급 실패 주입·전체 owner cleanup도 미실행이다. controller 시험을 Viewer 진입 인수로 확대하지 않는다. R1 실제 인수와 별도 R2 feature 컴파일·구체 fixture/OS 관측 배정이 선행이다.

기존 modoo code-change-review-verification 본문·필수 verdict 참조의 원 호출 경로/미검증 분리 기준을 적용한 읽기 조사이며 구현 책임은 UI 담당에 유지한다.
