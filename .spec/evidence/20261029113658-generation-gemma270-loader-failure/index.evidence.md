---
id: evidence:kinekino/20261029113658
title: Gemma270 고정 context 준비 실패
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Gemma270 고정 context 준비 실패

후속 [새 CLI](generation-gemma270-cli-build.md)에서 [실제 자막3요청](generation-gemma270-structured-smoke.md)과 [실제 경계 API](generation-gemma270-context-boundary-actual.md)를 수행했다. 아래 실패는 기존59ee 엔진의 동결 실행이며 덮어쓰지 않는다. 현재 품질 승인과 수정 EXE의 경계 재검증은 미완료다.

2026-10-19. [고정 GGUF 취득·헤더 검사](generation-gguf-header-actual.md) 뒤 기존 동일 smoke 계약의 Gemma 실행을 시도했지만 서버 준비 단계에서 실패했다. **신경망 요청 완료0개이며 실제 추론 완료 모델 수에 합산하지 않는다.**

실제 명령은 명시 profile `gemma270-q4`, suite `smoke`, context2048/max256/temperature0/4threads/offline/CPU다. 고정 mistral.rs0.9.4 EXE SHA `59ee8ff009a06cee8c39890cf6fd90d74f4617e46e88ac9b46a28a11538a9d73`가 `max_model_len=2048 is not supported by this multimodal loader`로 종료했고 Rust 어댑터는 server exited 오류를 보존했다. 전체 manifest와3개 HTTP 요청은 생성되지 않았다. 바깥 측정 명령의 종료 성공을 child 추론 성공으로 해석하지 않는다.

원본 source commit `4400935451da5e2dc7379a3f92fbbada66557f6c`의 [GGUF load_native_gemma3_text](https://github.com/EricLBuehler/mistral.rs/blob/4400935451da5e2dc7379a3f92fbbada66557f6c/mistralrs-core/src/pipeline/gguf.rs)는 projector 없는 Gemma3 text도 PreparedMultimodalSource·MultimodalLoader로 연결한다. GGUF metadata의 context32768과 실제 runtime cap2048은 서로 다른 값이다. 이 실패는 아직 tensor 연산이나 모델 의미 품질의 실패 증거가 아니다. source와 배포 EXE의 전체 빌드 입력 동등성을 인증한 것은 아니다.

context 인자를 제거하거나 metadata/config를 바꿔 다른 계약으로 재시도하지 않았다. 연구 세션의 [정확한 trait 거부 경로·Rust runtime-cap 수정안](../../player/infrastructure/inference/research-followups/gemma270-runtime-context-cap.md)은 Gemma3Loader의 runtime cap 처리 누락을 확인했고 원본 context32768·RoPE와 별도 runtime2048을 구분한다. 실제 엔진 패치·빌드·Gemma load/forward는 후속 작업이며 source 제안을 실행 성공으로 취급하지 않는다.

- [측정](../../../evidence/ai/model-spikes-2026-10-18/results/text-structured-gemma270-smoke-r1.json), SHA `064714791e64875fcf0b6fa0464626f237cd469c1429a41ce675ec6fbb967d01`
- [engine stderr](../../../evidence/ai/model-spikes-2026-10-18/results/text-structured-gemma270-smoke-r1/server.stderr), [worker stderr](../../../evidence/ai/model-spikes-2026-10-18/results/text-structured-gemma270-smoke-r1.stderr)
- [실행 build197입력+EXE1](../../../evidence/ai/model-spike-builds/a33b20bbd156f827278a8a00dc181cab2561d769337999d0993f2a4902d87046/build-manifest.json)

child exit1/timeout=false, wall5.7493807초, 표본 peak private310870016B·working312655872B다. 실패 시도 전체 값이며 모델 적재 후 메모리나 추론 성능이 아니다. 소유 server는 종료됐고 다른 앱을 종료하지 않았다. 모델 품질·제품 지원·재배포 승인은false다.
