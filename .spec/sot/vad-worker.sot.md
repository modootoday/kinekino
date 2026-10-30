---
id: sot:kinekino/vad-worker
title: 전용 VAD worker의 관측 결과 경계
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 전용 VAD worker의 관측 결과 경계

2026-10-21 중앙 배정 source-only r4 계약 후보. 구현은 runtime/development/vad-worker-r4/이며 기본 operation은 비활성이다. [현재 목표](../../../../plans/player/ai/media-current-goal.md)와 [static worker 결속 ADR](../../../../adrs/player/adr-vad-static-worker-release-binding.md)을 따른다.

후속 실제 검증: 원 r4 source40 그대로 [default CPU64](../../../verification/ai/vad-worker-default-r4-cpu-2026-10-21.md)를 수행해 P0 22/protocol28/worker14 모두 통과했다. 정확3 default all-target strict와 새3 BIN의 실제 목록/동일 BIN exact64를 확인했고 원72receipt 종료·보호·슬롯 반환을 마쳤다. 아래 계약은 그 default/synthetic 범위에서 검증됐으며 native adapter·loader/main/원 AI process cleanup·제품 지원·품질로 확대하지 않는다.

일반 공통 P0 analyze는 기존 post-engine cancellation/error 검사를 유지한다. worker의 observe_worker_analysis는 같은 내부 artifact/audio/score 검증을 사용하되 WorkerAnalysisObservation이라는 별도 data 타입을 반환한다. caller JSON이 정책을 선택하지 않고 cancel flag를 숨기지 않는다. 이 타입에는 SDK scope·marker·permit·원 process owner·VerifiedJobOutput이 없다.

| 관측 경계 | worker 후보의 결과 | 의미하지 않는 것 |
|---|---|---|
| 준비/읽기/엔진 전 취소 | Cancelled 오류 | Partial0·완료·ACK |
| 엔진이 Partial을 반환하고 원cancel=true·processed>0·범위/scores유효·cutoff유효 | checked Partial 관측 | 원 child 종료·EOF·family cleanup |
| Complete 후 원 cancel | 기존 Cancelled 오류 | Complete를 Partial로 합성한 관측 |
| 엔진 오류·invalid Partial·rejected sink | 해당 오류 | 취소 성공 |
| Partial 후 원 cutoff 만료 | Timeout 오류 | 갱신된 deadline·취소 ACK |

원 cancel 없는 엔진 Partial은 partial 데이터로 남지만 parent cooperative marker의 근거가 아니다. parent는 기존 requested/sent signal·원 attempt/process identity·actual wire/context·terminal/EOF/family cleanup을 별도로 검증한다. 공통 result의 Inferred도 parent settlement/CAS/publication 전에는 Job 완료 candidate가 아니다.

r4는 이 구분의 P0 fixture9와 runner→공통 wire fixture2를 추가했다. 최초 direct Rustfmt34/source-only 상태를 보존하고 후속 default64에서 실제 타입/계약을 검증했다. 실제 main/부모 자원/신뢰 release/loader·native/품질은 여전히 미완료다. modoo code-change-review-verification을 원 호출 경로와 실행 근거를 분리하는 데 적용했으며 전체 AI/미디어 범위를 축소하지 않는다.
