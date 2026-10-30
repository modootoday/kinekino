---
id: evidence:kinekino/20261029113553
title: Gemma270 runtime cap Rust 패치와 core 컴파일
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Gemma270 runtime cap Rust 패치와 core 컴파일

후속 [Rust1.99 Windows 호환 수정](generation-gemma270-compiler-compatibility.md)에서 원래 strict clippy·최종 소스 core check·전체1371tests가 통과했다. 아래 실패는 cap-only 단계 이력이다. CLI build·실제 경계·추론은 후속 정본을 따른다.

2026-10-19. [실제 Gemma 준비 실패](generation-gemma270-loader-failure.md)와 [원인·수정안 조사](../../player/infrastructure/inference/research-followups/gemma270-runtime-context-cap.md)를 기반으로 고정 mistral.rs 소스를 확보하고3개 책임 파일을 수정했다. CPU 전용 core check·fmt, Gemma 관련52tests, 전체 core1371tests 통과/5ignored를 확인했다. **strict clippy는 Rust1.99의 기존 async-trait 생성 attribute 문제로 실패했으며 새 CLI EXE·실제 cap 경계·모델 추론은 아직 통과가 아니다.**

## 고정 소스와 변경

원본은 `EricLBuehler/mistral.rs@4400935451da5e2dc7379a3f92fbbada66557f6c`다. Rust 개발 도구 `tools/engine-lab`이 자원 검사 뒤 사설 runtime checkout으로 shallow fetch했으며 HEAD·clean status·Cargo.lock SHA를 기록했다. preflight CPU22.94921875%·free RAM35414212608B였다. 사용자 앱·기존59ee 엔진·가중치·계정·VM·디스플레이는 변경하지 않았다.

| 책임 | 구현 |
|---|---|
| Gemma3TextConfig | private optional runtime 길이와 effective 길이 검증. native max_position_embeddings는 보존 |
| config runtime helper | None이면 원문 borrowed 경로. Some은1..=native 범위만 허용하고 내부 field만 주입한 메모리 JSON을 생성. unknown JSON은 보존하며 예약 필드 충돌·vision override는 거부 |
| Gemma3Loader | runtime_config override를 연결하고 model_config metadata에 effective 길이를 사용 |
| TextModel | 일반 full cache·자체 max_seq_len·ModelConfigMetadata에 effective 길이를 적용. global/local RoPE에는 원래 cfg/native 길이를 전달하며 sliding512는 유지 |

native/runtime 길이는 text model 생성 시 tracing 필드로 구분한다. 이 로그 추가만으로 실제 생성이나 cap enforcement를 확인한 것은 아니다. metadata만 바꾸거나 `--max-model-len`을 제거하지 않았고 GGUF·원본 config 파일·RoPE·quantized tensor binding은 수정하지 않았다.

## 컴파일과 시험 상태

실제 `cargo check --locked -p mistralrs-core --no-default-features -j 2`는 exit0,5분37초로 완료했다. native 의존성도 별도 빌드됐고 `.cargo/config.toml`의 target-cpu=native를 유지했다. 현재 호스트용 core 컴파일이며 다른 노트북/GPU/최종 CLI binary의 검증이 아니다. `cargo fmt --all --check`도 통과했다.

새 Rust 회귀3개는 config None byte 보존/unknown JSON·native 설정 보존, cap1/2048/native 허용·0/native+1/overflow/reserved/vision 거부, loader metadata2048·native32768 및 다른 미지원 loader 거부를 담당한다. 실제 crate의 `cargo test --locked -p mistralrs-core --lib gemma3 --no-default-features -j 2`는52passed/0failed/1324filtered였고 새3개를 포함했다. 최초 test-profile 컴파일27분03초 뒤 시험 자체0.05초였다. 최종 출력 보존을 위해 동일 생성 EXE의 Gemma52개를0.04초에 재실행했으며 새 모델 추론으로 계산하지 않는다.

후속 전체 `cargo test --locked -p mistralrs-core --lib --no-default-features -j 2 -- --test-threads=1`는 HF_HUB_OFFLINE=1로 실행했고1371passed/0failed/5ignored, 시험17.77초였다.5개는CPU throughput benchmark,별도Gemma4 tokenizer/GGUF, Muse-Glimmer main/mmproj, Qwen3.5 main/mmproj, 문서 재생성 시험이다. 외부 자료나 문서 재작성을 이번 통과에 포함하지 않는다. weight 없는 unit/fixture 회귀와 실제 Gemma270 model 생성·경계 시험은 별도다.

`cargo clippy --locked -p mistralrs-core --all-targets --no-default-features -j 2 -- -D warnings`는 mistralrs-mcp의 기존 async-trait0.1.89 macro가 생성한 무메시지 must_use attribute에 대한 Rust1.99 double_must_use10개로 실패했다. 테스트 빌드의 기존 Windows SIGINT_GRACE_WAIT 미사용 경고1개도 보존했다. lint를 allow/suppress하거나 모델 정책을 바꾸지 않았다. 변경 crate만 분리하는 no-deps 진단과 정확한 매크로 수정 경로는 조사 중이며 이를 원래 strict gate 통과로 대체하지 않는다.

[Gemma52 replay·core1371 전체 원문 및 strict clippy 실패](../../../evidence/verification/gemma-cap-core-tests-2026-10-19-r1/mistral-gemma-cap-tests-full-r1.stdout)는 같은 evidence 폴더에 보존했다. 전체 unit 통과를 모델 품질이나 모든 workspace/CLI 기능 통과로 확대하지 않는다.

소유 engine-lab은 fmt check·cargo test·clippy all-targets -D warnings·locked offline build를 실행했다. r2 snapshot 도구 시점은 unit test0개였다. 후속으로 resource admission을 domain으로 분리하고 CPU60%/NaN/낮은 RAM 및 evidence label 탈출 거부2tests를 통과했다. 새 도구 EXE `c20e4825eb82f55716f0a2daca5ed0200d6af067fae2f88ab2524beaad25f935`는 [r3 snapshot](../../../evidence/ai/engine-context-cap/gemma270-runtime-cap-r3/manifest.json)에 연결했으며 upstream 회귀와 혼합 집계하지 않는다. CLI 검증 정책을 PowerShell로 구현하지 않았으며 Git은 명시적 Rust 어댑터로 실행한다. 도구의 Git 명령은 private checkout·고정 원격/commit·비대화형·BelowNormal·시간 제한에 국한한다.

## 보존 근거

- [고정 소스 취득 결과](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/mistral-gemma-cap-source-r1/result.json)의 runtime 원본은 `runtime/ai-spike/acquisitions/mistral-gemma-cap-source-r1/result.json`이며 spike report3482 snapshots에 수집했다. 원본 Cargo.lock SHA `6be556f2577d9441f5d16c7ae7fec82b53e7492e3830b279e2c8bd9aa2ad92d4`.
- [r2 원본/수정3파일·patch·Cargo/lock/빌드 설정·MIT LICENSE](../../../evidence/ai/engine-context-cap/gemma270-runtime-cap-r2/manifest.json), manifest SHA `20199f9a239f519155a72431b3259a2054d6aae9574783ffc0bd4b183fc2cd6f`, patch SHA `a40d4a886e278bf79fe5b875f8fb1e514b3a0a0d02dad33c7677760076314402`.
- [core check 원문 stderr](../../../evidence/verification/gemma-cap-core-check-2026-10-19-r1/mistral-gemma-cap-check-r1.stderr), SHA `90993d4c23aef289685ba9fa0216cdb9ce0b9593ada1cf7d7e8078ee4e6737d6`, 같은 폴더의 stdout·rustc-version.txt.
- 소스 취득 도구 EXE `12358f03d539d2f77a310c6a16352022251c0abd63572ecb78b587d553c7f0c8`와 r2 snapshot 도구 EXE `cf76afc8aece16dc11e3756feef39ae0575b8e6ff17453148165d58b85109a38`는 각각 `.spec/evidence/ai/engine-lab-builds/` 아래 컴파일 입력·EXE·manifest를 선보존했다. 현재 source를 과거 실행에 소급 귀속하지 않는다.

patch snapshot 시점의 manifest는 engine_compilation_verified=false로 동결돼 있다. 이후 core check 통과는 위 별도 실행 근거이며 전체 CLI 생성의 통과로 manifest를 재작성하지 않는다. 중앙 검사기에 engine-lab4빌드의35개 컴파일 입력·EXE4개 및 고정 r2 patch의 원본/수정3쌍·설정/라이선스4개를 연결해 실제 검사를 통과했다. 검사기8tests·fmt·strict clippy·release build도 통과했다. [중앙 검사 보고서](../../../evidence/verification/gemma-cap-source-integrity-2026-10-19-r1/build-evidence.json)의 scope는 보존 bytes이며 전체 transitive/native compiler attestation·모델 품질과 구별한다. 감사 폴더의 checker-source-at-audit는 당시 소스 복사이고 compiler attestation manifest가 아니다.

## 남은 순서

1. [호환 수정](generation-gemma270-compiler-compatibility.md)의 strict clippy/check/최종 전체1371tests는 통과했다. CLI offline 캐시 부재를 기록하고 고정 lock의 명시적 dependency 취득/build를 진행한다.
2. 새 CPU CLI를 별도 build profile·features/lock/compiler 설정과 함께 보존한다. 기존 설치 엔진을 덮어쓰지 않는다.
3. native32768/runtime2048·모델/일반캐시 metadata·rendered prompt를 실제 관측한다.2047/2048/2049 경계와 generation stop을 확인한다. prompt나256-token budget을 조용히 줄이지 않는다.
4. 같은3개 prompt/schema로 Gemma 실제 smoke를 수행하고 구조·근거·의미·자원을 평가한다. 제품 AI capability·모델 품질 승인0개는 이 구현으로 변경하지 않는다.
