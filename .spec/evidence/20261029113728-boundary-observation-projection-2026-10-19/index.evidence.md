---
id: evidence:kinekino/20261029113728
title: 내부 관측 raw3 → Observation1 연결 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 내부 관측 raw3 → Observation1 연결 검증

2026-10-19. 소유 범위는 `tools/boundary-observation/**`와 이 문서다. 제품 src/engine-lab/model-spike/vendor/UI·모델·기존 원문을 수정하지 않았고 추론·다운로드·Pages·VM·DPI를 실행하지 않았다. 새 source/actual R3 수집은 Root의 별도 업무다.

## 기준과 구현

- [raw 매핑 계약](../../player/infrastructure/inference/research-followups/gemma270-raw-observation-mapping.md), [미등록 startup 조사](../../player/infrastructure/inference/research-followups/gemma270-unregistered-sequence-zero.md), [ADR](../../../adrs/player/inference/adr-startup-trace-lifecycle.md).
- [FEA source manifest](../../../evidence/ai/engine-context-cap/gemma270-startup-lifecycle-source-r3/manifest.json) SHA `fea352797c0a162f22407fdfc6323cd52ff2967306ca7c5ec420e4f326c5bd87`을 OS SHA로 재확인하고 after19/20/22/23 및 관련 after 소스를 읽었다. base commit의 sampler 정의도 별도 읽기 전용 참조했다. 이를 전체 컴파일러 입력 증명으로 표현하지 않는다.
- [crate README/입력 계약](../../../../tools/boundary-observation/README.md). domain의 명시 raw3/외부 pin/변환 receipt, application의 lifecycle·row·forward·sampling·cohort·외부 join 정책, infrastructure의 duplicate-key 거부 decoder/SHA/파일 adapter를 분리했다. application은 ArtifactDecoder port를 받아 infrastructure 구현에 의존하지 않는다.

기존 raw2 decoder/accounting/예제와 Observation1 pure verify를 유지했다. raw3→namespace별 전체 accounting→startup sidecar→외부 receipt hash/identity/control/HTTP join→cohort3 projection→기존 application::verify를 호출한다. 성공과 실패 모두 raw SHA와 원본 ordinal/JSON pointer를 연결하고 raw payload를 보존한다. schema 숫자만 바꾸지 않는다.

startup의 max1/seednull/top_k1/logprobs0/temperatureNone와 cohort2/seed42/top_kNone/logprobs1/temperature0을 분리한다. 최초 합성 startup의 nullmodifier/logprobs1 가정은 base sampler 원문 확인 후 교정했다. 이전 합성 통과를 실제 관측으로 보고하지 않았다. 숫자0·3토큰·hello 문자열로 origin을 선택하지 않는다. 합성 fixture는 startup request91/sequence67/4tokens/다른문자열로 이 점을 확인한다.

## 검증 구분

합성 API·실제 Rust CLI의 공급값 검증 통과는 **actual schema3 capture·비간섭·품질·full compiler proof가 아니다**. 결과의 capture_authenticated/noninterference_verified/complete_compiler_inputs_proven은 false다. shared_contract_verified만 supplied-value 검증 결과로 쓴다.

실제 보존 R1 schema1 raw `574b4a87ab54aeeddfe94b3d4dc43d9dec2f162bca6952efd151ce6a297e266c`, R2 schema2 raw `46db1ce4ff620b9abdd7e18b67e319124a94cbabfc2ed4df73944dc02b35fbde`를 새 startup CLI에서 각각 exit1로 거부했다. 실패 보고서에는 각각 schema1/2, complete=false, invalid_events=2, dropped_events=0이 원문 그대로 남고 raw-input 복사 SHA도 일치했다. 기존 raw2 check_journal도 R2를 incomplete accounting으로 exit1 거부했다. 실제 실패를 정상 startup으로 다시 분류하거나 invalid를 차감하지 않았다.

초기 실행 원문/EXE/실패 sidecar는 `tools/boundary-observation/artifacts/2026-10-19-v3-conversion-r1/`이다. 당시 startup checker EXE SHA `4e0432f166ea4f6cdc104585db7f5f01c625890d60e62a1cb44899aea4b9e46e`, converter SHA `a80595f71f238b73fdb6d631c39bc89aa91b4cc33d9ae37b29b871f2b1d82d71`다. 이는 schema1/2 거부 checkpoint이며 후속 startup profile 교정/새 bin의 실행 증거로 섞지 않는다. 최종 gate/CLI hash는 다음 checkpoint로 기록한다.

## 실제 외부 근거와 남은 상태

후속 actual R3 raw SHA `70a9d07a36ae39aa65f305680a49a6b35ec9cab8932138e3ea1e3c4f35450511`를 고정 r2 startup bin으로 검증했다. exit0/startup_sidecar_verified=true이고 cohort/shared/capture는 false다. 근거는 `tools/boundary-observation/artifacts/2026-10-19-actual-r3/`의 stdout, 원문 복사와 sidecar다. full CLI도 호출했으나 실제 수집에 독립 TrustedPins/input mapping/RunReceipt가 없어 artifact-open 입구에서 exit1 거부됐다. **full join 본체를 실행하거나 runtimecontrols를 대조한 결과가 아니다.** 없는 pins/flavor/resolved/종료값을 발명하지 않고 DTO를 발행하지 않았다. 실제 수집·pin 원문은 연구 세션이 별도로 봉인한다.

[runtime controls 조사](../../player/infrastructure/inference/research-followups/gemma270-runtime-control-evidence.md)에 따라 실제 prefix/speculative/flavor getter 근거가 없으면 full join을 차단한다. argv 부재를 speculative=false로 만들지 않는다. 환경의 thread4는 configured 폭이며 OS thread cap이 아니다. 명시 env 변경·제거와 상속 env를 구분한다. engine child의 owned kill status와 outer 측정 exit를 구분하고 없던0을 넣지 않는다. stop 전후 raw SHA 안정성은 fsync 완료 ack가 아니다.

최종 r2 checkpoint는 `tools/boundary-observation/artifacts/2026-10-19-v3-conversion-r2/`에 보존했다. cargo fmt --check, unit 24건과 실제 Rust CLI integration 2건(총 26건), clippy --all-targets -- -D warnings, bins/examples build가 통과했다. startup bin SHA는 `77e82383ac6688547b24230e0f2d40c3bef2c2109673b3e70371619a2511464f`, converter bin SHA는 `600c1c7be76968897f1b43c4bbb56657c4187d2f7907f543f7a8d0e0c1bceeb3`다.

현재 새 R3 실제 journal의 actual3 변환/shared 통과는 없다. startup-only preflight가 통과하더라도 cohort_projection_verified/shared_contract_verified는 false다. Root의 우선순위 변경에 따라 이 구현 checkpoint를 닫고 실제 R3·runtime 근거·capture 비간섭·모델 품질은 AI 연구 후속으로 인계한다. 해당 미검증 항목은 제품의 사용자 수정·명시적 채택 구현을 막는 품질 승인 조건으로 승격하지 않는다.
