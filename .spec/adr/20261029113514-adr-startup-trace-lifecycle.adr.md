---
id: adr:kinekino/20261029113514
title: 초기화 요청의 추적 생명주기와 소유권
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 초기화 요청의 추적 생명주기와 소유권

상태: 내부 구현 결정 채택, 전체 시험·새 EXE·실제 인수 진행 중. 모델 품질 승인 결정이 아니다.

## 맥락

[R2 실제 진단](../../../sots/verification/ai/generation-gemma270-internal-trace-diagnostics-actual.md)은 정상 요청 앞의 미등록 sequence0에서 invalid2를 기록했다. [소스 조사](../../../sots/player/infrastructure/inference/research-followups/gemma270-unregistered-sequence-zero.md)는 기존 MistralRs::new 초기화 Completion 요청과 연결했다. 기존 요청·forward·sampling·응답 drain을 유지하면서 관측 누락을 수정해야 한다. 숫자0·입력3토큰·문자열hello는 내부 요청의 소유권 증거가 아니다.

## 선택과 대안

| 선택지 | 이점 | 비용과 판단 |
| --- | --- | --- |
| 특정 ID·입력 모양 예외 | 작은 변경 | 실제 호출을 숨기고 외부 요청의 패턴 모방을 허용하므로 배제 |
| 추적 때 초기화 실행 생략 | invalid를 없애기 쉬움 | 원래 연산·상태를 바꾸므로 배제 |
| 모든 NormalRequest에 origin 필드 추가 | 명시적 운반 | 다수 생성자·분산/복제 경로와 공용 타입 변경이 필요함. 확장 시 재검토 |
| 내부 생성지점 permit + 응답 채널 소유권 | 원래 요청 타입·실행 횟수를 유지하며 동일 객체 경로를 연결 | 단일 엔진·단일 초기화라는 현재 진단 범위에 한정하여 채택 |

기존 내부 요청 생성 직후 crate 전용 permit을 발급한다. 실제 request ID와 응답 채널의 동일성을 add_request 진입에서 함께 확인하고 한 번 소비한다. 공개 HTTP에 origin 필드를 추가하지 않는다. permit은 tokio WeakSender로 보관한다. 강한 Sender를 저장하면 원래 response drain의 채널 종료를 막을 수 있으므로 사용하지 않는다. request ID만 같고 채널이 다르면 내부 요청으로 등록하지 않는다.

## 관측 계약과 책임

순수 lifecycle/permit, 채널·응답 callback, journal 저장을 Rust 모듈로 분리한다. startup과 cohort의 namespace·완료 집계를 구별한다. startup은 실제 토큰화·forward·metadata·18개 레이어·sampling·응답 종류·drain·종료 기록이 필요하다. received_any만으로 성공 처리하지 않는다. 미등록 비cohort 요청과 종료된 startup의 추가 row/forward는 계속 거부한다. original invalid counter를 감산하지 않는다.

raw schema3은 startup_lifecycle/startup_accounted와 event request_origin을 명시한다. complete는 정상 요청3개 외에 초기화 ledger도 요구한다. schema2 decoder가 이 추가 필드를 무시하게 바꾸지 않는다. 이후 adapter는 startup sidecar를 검증한 뒤 cohort3개만 공통 DTO로 projection해야 한다.

## 범위와 후속

현재 opt-in 진단은 STATE의 단일 엔진과 실제 초기화가 있는 Gemma CPU CLI 범위다. 초기화를 생략하는 runtime/category는 이 profile의 complete를 얻지 못한다. 일반 제품 경로는 trace가 꺼져 있고 초기화 연산은 그대로다. 관측 비용·스케줄링·캐시/RNG의 비간섭은 별도 실제 비교로 검증해야 한다. 이 결정을 비간섭 통과나 전체 capture 진위 증명으로 표현하지 않는다.

다중 엔진·초기화 반복·origin을 복제해 전달해야 하는 경로가 추가되면 전용 carrier와 EngineSession/LifecycleID를 재검토한다. [구현·게이트 현황](../../../sots/verification/ai/generation-gemma270-startup-lifecycle-implementation.md)을 기준으로 실제 완료 범위를 기록한다.
