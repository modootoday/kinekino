---
id: evidence:kinekino/20261029113633
title: Gemma270 초기화 생명주기 추적 구현
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Gemma270 초기화 생명주기 추적 구현

2026-10-19. [R2 실제 invalid2](generation-gemma270-internal-trace-diagnostics-actual.md)의 발생 경로를 수정했다. [결정과 제한](../../../adrs/player/inference/adr-startup-trace-lifecycle.md)은 별도 ADR로 관리한다. R1/R2의 소스·EXE·실패 기록과 실제3case는 보존했다.

## 구현

lib.rs의 기존 내부 요청 생성지점은 request/응답 채널에 연결된 permit을 발급한다. add_request는 ID와 같은 응답 채널인지 확인해 한 번 등록한다. 약한 채널 참조라 원래 drain을 막지 않는다. 같은 숫자 ID만으로 등록하지 않고 비cohort 요청을 조용히 무시하지 않는다. startup의 actual prompt IDs, forward entry/return, metadata/18layer positions, sampling, CompletionDone/오류, response drain/종료를 같은 namespace에 기록한다. 입력을 재토큰화하거나 원래 forward를 추가 실행하지 않는다.

cohort requests/completed는3개를 유지하고 startup 종료는 별도로 기록한다. 종료 전·후의 비정상 sequence 사용과 중복·누락·응답 실패는 완결로 처리하지 않는다. rawschema3/request_origin/startup_lifecycle/startup_accounted를 추가했다. 원래 counter는 유지한다. raw complete는 별도 full boundary verifier나 비간섭 검증이 아니다.

## 고정 입력과 gate 상태

[새 source manifest](../../../evidence/ai/engine-context-cap/gemma270-startup-lifecycle-source-r3/manifest.json) SHA256은 `fea352797c0a162f22407fdfc6323cd52ff2967306ca7c5ec420e4f326c5bd87`, patch SHA256은 `d61faa0e331fbab5a0f77b47d5cac728bbb0e2502289759111902b3d7b6b0ed7`다. 검토 경로24개(새파일5개)를 명시적으로 허용했고 base 설정4개도 보존했다. lib.rs와 lifecycle 두 모듈을 새 범위에 포함했으며 기존21파일 범위로 위장하지 않았다. 이 manifest의 engine_compilation_verified/model_inference_verified=false는 source 보존 시점의 선언이다.

최종 source의 core+CLI all-targets offline check는exit0(29.10s), workspace cargo fmt --all -- --check도exit0이다. 신규 모델 없는 회귀는 같은 ID/다른 채널 모방, 약한 참조의 채널 수명, forward/레이어/응답/drain 누락·중복, 초기화 전후 등록 경계, 오류 응답의 실패 판정을 포함한다. 전체 core 시험은1385통과·실패0·무시5(시험 실행10.54s)이며 [원문](../../../evidence/verification/gemma-startup-core-2026-10-19-r3/)을 보존했다. 최종 core+CLI strict Clippy와 CLI build도exit0이며 [원문·worker gate](../../../evidence/verification/gemma-startup-pin-2026-10-19-r3/)를 별도로 보존했다. 시험·컴파일 시간을 모델 추론 성능으로 해석하지 않는다.

새 CLI build는 위 고정 source의 전후 해시를 확인했다. 최종 EXE는99,084,288 bytes/SHA256 `b7ece1736cf49d85523fe514cd09fb65bb1e3912c8f634677e1264196462a7f8`로 별도 startup Stage에 보존했다. [엔진·최종 worker pin과 실행 명령](generation-gemma270-startup-worker-pin.md)에 snapshot/Revision·wrapper 회귀와 비추론 검증 결과를 기록했다. 새 실제 모델 실행은 이 체크포인트에서 수행하지 않았다. 이전 R2 engine97a3를 새 소스의 실행 파일로 귀속시키지 않는다.

## 남은 인수

[완료된 source check·workspace fmt·snapshot/build 도구 시험과 Clippy 원문](../../../evidence/verification/gemma-startup-source-check-2026-10-19-r3/)을 보존했다. 진행 중인 core test/최종 Clippy·CLI build의 로그를 이 완료 묶음에 섞지 않았다. [중앙 무결성 검사](../project/integrity-2026-10-19.md)는 고정 R2 실패와 R2/R3 소스 보존본을 검증했으며 현재 source의 전체 시험·새 실제 실행 통과를 뜻하지 않는다.

[외부 실행 설정 근거 조사](../../player/infrastructure/inference/research-followups/gemma270-runtime-control-evidence.md)에 따라 [실행·종료 receipt 수집기](text-server-receipts.md)를 구현했다. 명시적 argv/환경·JobClose+wait·raw전후SHA 수집 코드의 회귀는 통과했지만 새 엔진 실제 실행의 receipt는 아직 없다. outerworkerexit의 공통 RunReceipt 통합과 speculative·prefix/scheduler등 실제 적용값 수집도 남는다. 환경의thread폭4를OS전체프로세스threadcap4로표현하지않는다. 공통DTO strictjoin에필요한근거가없으면임의default로통과시키지않는다. 현재SDK소유세션이별도library의schema3decoder/startupsidecar/외부receiptjoin/DTO변환을개발중이며그완료를이sourcegate로대체하지않는다.

현재 source와 새 CLI EXE·별도 Revision/worker를 고정했다. 이후 PM 단일 worker가 실행한 [동일3case 실제 관측](generation-gemma270-startup-lifecycle-actual.md)에서 startup ledger와 cohort3개의 recorder 완결·raw invalid0·실제 종료 receipt를 확인했다. 앞 문단의 receipt 미수집은 pin 준비 체크포인트 상태다. schema3의 strict decoder·외부 HTTP/worker controls·startup sidecar→공통 DTO 변환·검증, terminal witness와 off/on 비간섭은 별도 인수다. R2의 invalid2를 소급 제거하거나 raw complete만으로 full gate를 통과시키지 않는다.
