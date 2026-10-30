---
id: evidence:kinekino/20261029113559
title: Gemma270 startup lifecycle 실제 관측
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Gemma270 startup lifecycle 실제 관측

2026-10-19. PM이 [고정 R3 엔진·worker](generation-gemma270-startup-worker-pin.md)로 `gemma270-boundary-startup-lifecycle-r1`을 한 번 실행했다. 연구 세션은 추가 추론 없이 종료된 원문을 Rust 도구로 복사·전후 해시 대조하고 별도 관측 추출을 보존했다.

## 원문과 식별

[snapshot manifest](../../../evidence/ai/generation-boundary-actual/gemma270-boundary-startup-lifecycle-r1/snapshot-manifest.json) SHA256은 `24450fcd64038deb71d72c27b8550f52ccbbe7cd9886e016b5355bdb4904b209`다. 원문 run15파일·외부 측정1·outer stdout/stderr2와 파생 observation1을 구별해19파일을 보존했다. 원래 run manifest를 snapshot manifest로 덮지 않았다. [원시 journal](../../../evidence/ai/generation-boundary-actual/gemma270-boundary-startup-lifecycle-r1/run/internal-trace/journal.json) SHA256은 `70a9d07a36ae39aa65f305680a49a6b35ec9cab8932138e3ea1e3c4f35450511`이며 PM 보고와 일치한다. [파생 추출](../../../evidence/ai/generation-boundary-actual/gemma270-boundary-startup-lifecycle-r1/observation.json)은 원문을 대체하지 않는다.

engine SHA는 `b7ece1736cf49d85523fe514cd09fb65bb1e3912c8f634677e1264196462a7f8`, worker SHA는 `f763a4cf9800c6492b91b73219cc7c785d5e4050a9797b96be4cb004f7f9ee41`다. source·fixture는 인계 정본의 FEA352·85009c8e pin을 사용한다. R2 실패 원문과 엔진은 그대로 보존한다.

## 직접 확인한 회계

schema3 raw는 complete=true, startup_accounted=true이며 invalid_events/dropped_events/flush_failures는 각각0, first_storage_failure=null이다. 총72event와 row_batches2를 기록했다. 이들은 recorder의 관측값이며 공통 SDK validator 통과를 의미하지 않는다.

startup request0/sequence0은 cohort와 별도로 admitted·closed·drained=true, responses1, completion_done=true다. `startup_lifecycle` origin에 forward_enter/return 각1, model_metadata1, rope_positions18, lifecycle_response1과 lifecycle_drained1이 있다. startup의 입력 shape은 `[1,3]`이고 원래 hello dummy를 계측한다. 완료된 cohort request는2/4/6이다. cohort에는 request/rendered/admission/case_closed 각3, forward_enter/return 각1, rope_positions18이 기록됐다. startup과 cohort 합계 forward는2이며 이 확인으로 off/on 비간섭이나 layer position의 의미 검증을 대신하지 않는다.

| 입력 token | count HTTP/token | generation HTTP | 저장된 동작 |
|---|---|---|---|
| 2047 | 200 / 2047 | 200 | accepted, sequence1, forward shape `[1,2047]`, usage prompt2047/completion1/total2048 |
| 2048 | 200 / 2048 | 400 | 생성 여유 없음, admission 거절, sequence 없음 |
| 2049 | 200 / 2049 | 400 | runtime capacity2048 초과, admission 거절, sequence 없음 |

## 종료와 판정 한계

[종료 원문](../../../evidence/ai/generation-boundary-actual/gemma270-boundary-startup-lifecycle-r1/run/server-shutdown.json)은 owned JobClose 성공 후 wait, server PID90032, 실제 exit0을 기록했다. 원래 worker manifest의 raw 전후 안정성도 true다. [외부 측정](../../../evidence/ai/generation-boundary-actual/gemma270-boundary-startup-lifecycle-r1/measurement.json)은 exit0, timed_out=false, 65.8439968초를 기록했다. PM 관측 peak private1,331,810,304 bytes/working set1,564,905,472 bytes이며 load를 포함한 표본이다. performance_comparison_eligible=false를 유지하며 성능 우열로 사용하지 않는다.

full_boundary_gate_passed/shared_observation_contract_verified/noninterference_verified는 모두false다. startup/cohort recorder의 완결 관측을 근거로 resolved controls·flush ACK·공통 DTO strict join·terminal witness·off/on 비간섭·모델 품질을 승인하지 않는다. SDK 소유 세션이 이 보존본으로 decoder/sidecar/join 부족 근거를 별도로 확인한다. 추가 추론은 이 보존 작업에 포함하지 않았다.

## 보존·무결성 도구 검증

Rust `preserve-startup-actual`은 새 snapshot 경로만 허용하며 source와 copy의 bytes·SHA를 대조했다. 이후 `check-trace-diagnostics-pinned`에 이 고정 snapshot19파일과 파생 관측 재추출 일치 검사를 추가했다. 최종 fmt check·11tests·all-targets strict Clippy·locked offline build와 실제 통합 check 모두exit0이다. [검증 원문](../../../evidence/verification/gemma-startup-pin-2026-10-19-r3/)의 spec-maintenance-sealed 접두 로그가 마지막 결과다. 최종 검사 EXE는4,989,952 bytes/SHA256 `8d56790738fa2ef6eb8db1d0969ffb534b1095ba842aa808dc2d150f5581ba74`다. 최초 snapshot SHA 상수의 전사 오타는 fail-closed로 검출돼 정정했고 실패 로그를 남겼으며 보존 원문은 변경하지 않았다.
