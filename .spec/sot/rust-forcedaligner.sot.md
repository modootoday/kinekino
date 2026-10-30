---
id: sot:kinekino/rust-forcedaligner
title: Rust ForcedAligner의 실제 구현과 후속 실험 후보
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Rust ForcedAligner의 실제 구현과 후속 실험 후보

2026-10-18 별도 조사 목표. GitHub 고정 source·Cargo·HF/registry metadata만 읽었다. source clone·가중치/사전 모델 다운로드·빌드·추론은 수행하지 않았다. tools 코드와 기존 실측/증거는 변경하지 않았다. 앞선 [Rust ASR 조사](rust-asr-runtime.md)의 `검토 후보에서 align head 미확인`은 alan/airvzxf/voxora/mutranscriber 범위였다. **추가 후보에는 실제 Rust ForcedAligner 구현이 있다. 생태계 전체 부재로 확대할 수 없다.** 아래 source 구현 확인은 로컬 Windows 실행·정렬 품질 통과를 의미하지 않는다.

## 고정 후보와 우선순위

| 후보·고정 commit | 실제 align 구현 | 후속 선택 |
|---|---|---|
| [lumosimmo/qwen3-asr-rs](https://github.com/lumosimmo/qwen3-asr-rs/tree/7984967e8701cc90bb17d06453901036e36ca578) | Candle classify head·timestamp mask/argmax·tick→ms→sec·LIS repair·JP/KO processor·golden fixture | **JP/KO 목표 우선 후보**. KO 먼저, JP는 공식 nagisa와 분절 차이를 명시해 평가. 배포 고지·사전 권리와 Windows 빌드는 선행 게이트 |
| [openvalle/valle-asr](https://github.com/openvalle/valle-asr/tree/ddfa3042bc1674d5a33385ad15b45a0e32a46bc9), crate0.2.0 | Candle classify_positions·tick/repair·standalone align·모델 교체/해제·30초 상한 | **영어 등 통제 실험 차선 후보**. JP/KO 단어 정렬은 명시적으로 거절하므로 JP/KO 대체품이 아님 |
| [eclipse005/qwen-aligner-rs](https://github.com/eclipse005/qwen-aligner-rs/tree/bb8c73ba670d65b3f5fddc5f5b8791bc07d4392a) | 자체 CPU/CUDA encoder/decoder/classify·timestamp 복원·nagisa JP/한국어 LR | 구현은 확인. 소프트웨어 LICENSE 부재·nagisa 내장 모델 권리·추가 모델 반입 게이트 때문에 우선 실행 보류 |
| [huanglizhuo/QwenASR](https://github.com/huanglizhuo/QwenASR/tree/37c1e35be333945dc9b5771a6a33fe4b2064533f) | CPU classify_num 기반 forced_align·argmax·LIS·ms 결과 | Windows에 직접 적용 불가한 Unix 의존 source 확인. JP/KO는 문자 분절이므로 공식 단어 정렬과 다름. 이번 Windows 우선 후보 제외 |

성능 README·주석의 reference 일치/속도/VRAM 값은 upstream 주장이다. 이번 세션은 이를 재현하거나 이 호스트의 성능으로 전환하지 않았다. 각 후보는 같은 Qwen checkpoint를 사용할 수 있으나 언어 분절·전처리·수치 경로·시간 보정이 달라 출력 단위부터 대조해야 한다.

## 1. lumosimmo: JP/KO를 위한 실제 Candle 정렬 경로

고정 [library Cargo](https://github.com/lumosimmo/qwen3-asr-rs/blob/7984967e8701cc90bb17d06453901036e36ca578/qwen3_asr/Cargo.toml)는 package `qwen3_asr`0.1.0, workspace edition2024/resolver3, default features=[], Candle0.9, tokenizers0.21이다. `forced-aligner`가 Lindera/IPADIC embed 기능을 켜고 `audio-loading`이 Symphonia0.5/rubato/reqwest 등을 추가한다. CPU는 default feature가 없으며 accelerate/Metal/CUDA/MKL/flash-attn은 선택 사항이다. CPU 경로는 Candle tensor 연산이며 libtorch/ORT wrapper가 아니다. 네이티브 regex/선택 backend 의존성까지 순수 Rust라고 보장하지 않는다.

구현 근거:

- [thinker.rs](https://github.com/lumosimmo/qwen3-asr-rs/blob/7984967e8701cc90bb17d06453901036e36ca578/qwen3_asr/src/model/thinker.rs)는 forced_aligner model_type에서 `classify_num`을 요구하고 `thinker.lm_head`의 출력 크기로 사용한다. tied embeddings로 정렬 head를 대체하지 않는다.
- [forced_aligner/model.rs](https://github.com/lumosimmo/qwen3-asr-rs/blob/7984967e8701cc90bb17d06453901036e36ca578/qwen3_asr/src/forced_aligner/model.rs)는 config의 timestamp_token_id/timestamp_segment_time을 읽고 full forward→logits.argmax(2)→timestamp 위치 추출→class_id×tick(ms)→processor repair→초 단위 round3 결과를 만든다. 단순 ASR token 시간 추정이나 균등 시간 배분이 아니다.
- 실제 API는 `Qwen3ForcedAligner::from_pretrained(model_id_or_path, &Device, &LoadOptions)`와 `align(audio: &[AudioInput], text: &[String], language: &[String])`다. 모델 종류를 검증한다. 로컬 경로로 호출하고 HF ID 자동 다운로드를 피한다. ASR 모델과 같은 객체라고 가정하지 않는다.
- [processor.rs](https://github.com/lumosimmo/qwen3-asr-rs/blob/7984967e8701cc90bb17d06453901036e36ca578/qwen3_asr/src/forced_aligner/processor.rs)는 한국어 embedded dictionary와 soynlp LTokenizer 동작 이식을 구현한다. 일본어는 Lindera+IPADIC 및 merge terms/NFKC·fallback을 사용한다. **공식 nagisa와 동등한 분절이라고 단정할 수 없다.** token boundary가 달라지면 같은 timestamp class를 비교해도 같은 단어 경계 실험이 아니다.
- `golden_forced_aligner_processor`, `golden_forced_aligner_masked_output_ids`, `golden_forced_aligner_japanese_nagisa_diff`, `golden_transcribe_timestamps`, `smoke_forced_aligner` test source와 JP/KO golden JSON이 있다. 존재만 확인했으며 실행·fixture 생성 근거 전체 감사는 하지 않았다. 모델을 받는 integration test를 일반 test처럼 실행하지 않는다.

후속 CPU library build 명령(이 세션에서는 실행 금지·미실행): 고정 source checkout을 별도 `runtime/ai-spike/source/lumos-qwen3`에 준비한 뒤 실행 담당만 수행한다. 이 source에는 Cargo.lock이 있다.

```text
git clone --no-checkout https://github.com/lumosimmo/qwen3-asr-rs runtime/ai-spike/source/lumos-qwen3
git -C runtime/ai-spike/source/lumos-qwen3 checkout --detach 7984967e8701cc90bb17d06453901036e36ca578
cargo build --manifest-path runtime/ai-spike/source/lumos-qwen3/Cargo.toml --locked --release -p qwen3_asr --no-default-features --features forced-aligner,audio-loading --lib --jobs 2
```

확인 사항: forced-aligner의 embed-IPADIC dependency build는 사전 자산을 포함할 수 있다. 프로젝트의 새 모델/사전 반입 권리·네트워크·파일 manifest 게이트를 먼저 통과시킨다. `git clone`이 모든 후보에서 작은 소스만 받는다고 가정하지 않는다. 이번 연구는 clone하지 않았다. Windows 빌드 성공은 미확인이고 edition2024에 맞는 toolchain/dependency native compiler 검토가 필요하다.

library만 빌드하면 실행 CLI가 생기지 않는다. 알려진 정답 transcript를 `align`에 전달하는 작은 Rust spike adapter는 **미구현 제안**이다. 기존 player-model-spike에 align role이 있다는 뜻이 아니다. 기존 실제 CLI [args.rs](https://github.com/lumosimmo/qwen3-asr-rs/blob/7984967e8701cc90bb17d06453901036e36ca578/qwen3_asr_cli/src/args.rs)의 `transcribe --model <LOCAL_ASR> --audio <WAV> --language Japanese --return-timestamps --forced-aligner <LOCAL_ALIGNER> --max-batch-size 1`은 ASR+정렬 경로이며, 정답 transcript만의 독립 align 시험과 다르다. CPU/dtype global option은 `--device cpu --dtype f32`다. CLI는 audio-loading/forced-aligner/timing을 library 의존에서 켠다. 이 인자들은 소스로 확인했지만 실행하지 않았다.

권리: workspace Cargo의 선언은 MIT OR Apache-2.0이나 조회한 root에는 LICENSE/NOTICE 파일이 없다. Cargo SPDX 선언과 원문·저작권 고지 보존은 구분한다. `korean_dict_jieba.dict` 주석은 Qwen upstream 자산을 Apache로 지목하지만 정확한 원본 byte/hash·적용 고지는 별도 확인한다. IPADIC/Lindera 사전의 별도 고지와 일본어 merge terms 유래도 출시 전 inventory가 필요하다. 원본 모델의 Apache 카드가 이 모든 부속 파일의 권리를 대신하지 않는다.

## 2. valle-asr: 권리·로컬 입력 계약이 분명한 영어 통제군

고정 [Cargo.toml](https://github.com/openvalle/valle-asr/blob/ddfa3042bc1674d5a33385ad15b45a0e32a46bc9/Cargo.toml)은 version0.2.0, edition2024, MSRV1.99, default qwen3/download/cli, Candle0.11.0(default-features=false), memmap2, safetensors, RustFFT, tokenizers0.23.2(fancy-regex)를 선언한다. 로컬 `rustc --version` 읽기 결과는1.99.0이었지만 빌드하지 않았다. docs.rs의 대상은 Linux이므로 Windows 보장이 아니다. registry0.2.0 checksum은 `743b789b36e97ee131d15191d16f2086e9d288c56bf739c590d97dc427885c2e`; Git checkout과 registry byte 동등성은 미검증이다.

[network.rs](https://github.com/openvalle/valle-asr/blob/ddfa3042bc1674d5a33385ad15b45a0e32a46bc9/src/models/qwen3/network.rs)는 `classify_num` 출력 head와 `classify_positions`를 구현한다. [Qwen3::align/align_with_cancellation](https://github.com/openvalle/valle-asr/blob/ddfa3042bc1674d5a33385ad15b45a0e32a46bc9/src/models/qwen3/mod.rs)는 30초 이하 known transcript를 받아 timestamp 위치 class argmax→tick(ms)→repair·duration 경계 적용 결과를 만든다. ASR→aligner 전환 전에 resident network를 해제하는 구조도 있다. 이에 따른 실제 peak/unload는 미실측이다.

중요 한계: `align_chunk`는 Japanese/Korean 단어 정렬 미구현을 **명시적으로 오류로 반환**한다. 일반 ASR language map의 ja/ko 지원과 혼동하지 않는다. Chinese/Cantonese Han 문자와 다른 지원 언어의 단어 경로는 있지만 JP/KO 목표 해결책은 아니다. 영어 정답+기존 짧은 미디어로 모델/시간축 통제군을 만드는 후보로만 선정한다.

고정 source checkout을 `runtime/ai-spike/source/valle-asr`에 준비한 후 library만 검토하는 실제 build 명령(미실행):

```text
cargo build --manifest-path runtime/ai-spike/source/valle-asr/Cargo.toml --locked --release --no-default-features --features qwen3 --lib --jobs 2
```

source와 registry는 API 상태가 다를 수 있어 위 명령은 고정 Git source 기준이다. known-text align CLI를 새로 발명하지 않는다. `Qwen3::load(id, local_asr_dir, Some(local_aligner_dir))`로 별도 Rust adapter를 만들고 공개 `align`을 호출하는 작업은 후속 구현이다. 이 load 계약에는 ASR 경로/config/tokenizer도 필요하므로 aligner만 받으면 모든 API가 준비된다고 쓰지 않는다. `models.json`에는 원본 ASR/aligner SHA·파일 hash가 있으며 우리 기존 HF 계약과 일치하는 가중치 항목을 확인했다. 모델 다운로드 command는 존재하더라도 이번 범위에서는 사용하지 않는다.

권리: [LICENSE](https://github.com/openvalle/valle-asr/blob/ddfa3042bc1674d5a33385ad15b45a0e32a46bc9/LICENSE)는 Apache-2.0이고 [THIRD_PARTY.md](https://github.com/openvalle/valle-asr/blob/ddfa3042bc1674d5a33385ad15b45a0e32a46bc9/THIRD_PARTY.md)는 Qwen의 timestamp-repair 번역·alan 구조/fixture 참조·antirez/CrispASR 참조와 고정 SHA, third_party LICENSE 사본을 구분한다. source 설명상 alan runtime wrapper는 아니며 새 Candle 구현이지만 **alan과 완전히 무관한 독립 계보라고도 할 수 없다**. 복사 audio fixture 권리는 upstream 선언과 별도 자산 검토를 해야 한다. root 독립 미디어를 사용한다.

## 3. eclipse: 독립 CPU 엔진과 공식 분절에 가까운 경로, 권리 선행

고정 [Cargo](https://github.com/eclipse005/qwen-aligner-rs/blob/bb8c73ba670d65b3f5fddc5f5b8791bc07d4392a/Cargo.toml)는 package qwen-forced-aligner-rs0.1.0, bin qwen-aligner, edition2021, default cuda+cpu다. CPU는 gemm0.18/rayon, CUDA는 cudarc0.19.7/cuBLAS/NVRTC(CUDA12.8 feature)다. Candle/ORT/libtorch 의존 없이 직접 CPU f32 tensor와 CUDA engine을 구현한다. CUDA는 native NVIDIA engine/library binding이므로 전체 실행이 pure Rust라는 표현을 그대로 제품 설명에 쓰지 않는다.

[cpu_engine.rs](https://github.com/eclipse005/qwen-aligner-rs/blob/bb8c73ba670d65b3f5fddc5f5b8791bc07d4392a/src/cpu_engine.rs)의 text decoder는 `thinker.lm_head.weight`를 읽고 [1,sequence,classify_num] logits를 생성한다. [inference.rs](https://github.com/eclipse005/qwen-aligner-rs/blob/bb8c73ba670d65b3f5fddc5f5b8791bc07d4392a/src/inference.rs)는 timestamp 위치의 class별 first argmax→class_id×timestamp_segment_time(ms)→LIS fix→초 변환을 수행하고 raw/fixed ms도 반환한다. [timestamp.rs](https://github.com/eclipse005/qwen-aligner-rs/blob/bb8c73ba670d65b3f5fddc5f5b8791bc07d4392a/src/timestamp.rs)의 과거 inverted pair 사전 clamp 제거·signed 거리 복원 주석은 upstream 버그 수정 설명이며 우리 재현은 아니다. zero duration은 repair 후에도 존재 가능하다.

JP는 `nagisa_rs::Tagger::embedded()`를 매번 model load에서 준비하고 KO는 embedded Korean dictionary LR 분절을 쓴다. nagisa git dependency는 `cd7726e78ede8451c72cc0fabfd55d2c4ed62442`로 고정되어 있다. 해당 [nagisa-rs README](https://github.com/eclipse005/nagisa-rs/blob/cd7726e78ede8451c72cc0fabfd55d2c4ed62442/README.md)는 모델/사전의 바이너리 내장을 설명한다. **Qwen weights를 안 받아도 소스 dependency fetch/build가 nagisa weights를 반입할 수 있다.** 이 연구에서는 dependency clone/fetch/build를 하지 않았다. byte-identical 분절·약25MB 증가는 upstream 주장이고 사전·학습모델 권리 확인이 선행한다.

권리 문제: qwen-aligner와 nagisa-rs의 조사 root에는 LICENSE/NOTICE가 없고 Cargo에도 license 필드가 없었다. README의 Qwen/기타 원본 저작권 안내가 구현 자체의 포괄 라이선스를 대신하지 않는다. 공개 저장소라는 이유로 코드·내장 모델의 제품 반입/재배포 권한을 지어내지 않는다. 이 게이트가 해소될 때까지 실행 shortlist의 우선순위에서 보류한다.

실제 [main.rs](https://github.com/eclipse005/qwen-aligner-rs/blob/bb8c73ba670d65b3f5fddc5f5b8791bc07d4392a/src/main.rs)는 **`align` subcommand와 text 파일 경로**를 요구한다. README의 과거 `--text "문장"` 직접 문자열 예제를 그대로 쓰지 않는다. 권리·사전 반입 게이트 해소 후 다음 담당이 쓸 수 있는 실제 build/CLI 형식은 아래와 같다(전부 미실행):

```text
cargo build --manifest-path runtime/ai-spike/source/eclipse-qwen-aligner/Cargo.toml --locked --release --no-default-features --features cpu --bin qwen-aligner --jobs 2
qwen-aligner.exe align --model <LOCAL_ALIGNER_DIR> --audio <WAV_PATH> --text <TRANSCRIPT_TXT_PATH> --language Japanese --output <NEW_JSON_PATH> --device cpu
```

`Qwen3ForcedAligner::load(&Path, ModelOptions)`와 `align(AlignRequest)`가 실제 API다. README에 보이는 짧은 `Aligner::load` 이름을 현행 API로 쓰지 않는다. `.cargo/config.toml`에는 USTC sparse registry replacement가 있다. 고정 source checkout이어도 dependency registry 경로를 검토해야 하고 사용자 전역 Cargo 설정을 바꾸지 않는다. Windows CPU 가능성은 cfg cpu와 플랫폼 중립 Rust/gemm/memmap2 구조 근거뿐이며 실제 Windows build는 미확인이다.

## 4. QwenASR: 정렬은 있지만 Windows·JP/KO 계약이 다름

고정 [align.rs](https://github.com/huanglizhuo/QwenASR/blob/37c1e35be333945dc9b5771a6a33fe4b2064533f/crates/qwen-asr/src/align.rs)는 classify_num>0 모델을 요구하는 `forced_align(&mut QwenCtx, ...)`, class argmax와 LIS interpolation, start_ms/end_ms를 구현한다. JP/KO/Chinese/Cantonese는 각 Unicode 문자를 한 항목으로 쪼갠다. word-level이라는 출력 표제만 보고 공식 nagisa/soynlp 단위라고 표시하지 않는다. 대소문자를 포함한 language 문자열도 source 분기와 맞춰야 한다.

핵심 Windows 차단은 [safetensors.rs](https://github.com/huanglizhuo/QwenASR/blob/37c1e35be333945dc9b5771a6a33fe4b2064533f/crates/qwen-asr/src/safetensors.rs)의 `std::os::unix::io::RawFd`, libc mmap/munmap/pread 직접 사용이다. CPU-only/pure Rust라는 소개가 Windows source compatibility를 뜻하지 않는다. 이번 조사에서는 Windows port를 구현하지 않았다. [Cargo](https://github.com/huanglizhuo/QwenASR/blob/37c1e35be333945dc9b5771a6a33fe4b2064533f/crates/qwen-asr/Cargo.toml)는 libc 의존과 default blas/vdsp, Android aarch64/not-blas 조건의 int8-prefill·int8-encoder feature를 별도 선언한다. 이 INT8 경로가 x64 Windows aligner 양자화 지원이라는 뜻은 아니다. [MIT LICENSE](https://github.com/huanglizhuo/QwenASR/blob/37c1e35be333945dc9b5771a6a33fe4b2064533f/LICENSE)는 확인했다. alan dependency를 감싸는 구조는 아니지만 원본 모델·참고 코드 계보 전부의 독립성을 감사한 것은 아니다.

## 공통 모델·정밀도·검증 계약

모델은 기존 [고정 파일 계약](execution-readiness.md)의 공식 `Qwen/Qwen3-ForcedAligner-0.6B` SHA `c7cbfc2048c462b0d63a45797104fc9db3ad62b7`를 사용한다. `model.safetensors`1,835,544,544 bytes, LFS SHA256 `47831d0e82f96b20e9034dba01a075ee06436654719f6a68289e49f1b65ce0e7`; config/processor/tokenizer_config/vocab/merges/chat_template/generation config를 고정한다. 추가 tokenizer.json 생성 또는 loader fallback은 엔진별 source에 맞춰 확인하며 새 HF파일이 있다고 발명하지 않는다. 코드 라이선스·Qwen 가중치 카드·KO/JP 사전·nagisa 모델·학습데이터는 서로 다른 inventory 항목이다.

조사한 lumos/eclipse/valle 정렬 경로는 dense safetensors·CPU F32 또는 원본 BF16/F16 로드/승격 경로다. 실제 GGUF/Q4 정렬 head·encoder/decoder 실행을 확인하지 못했다. QwenASR Android INT8 feature와 Alkd ASR GGUF를 정렬 모델에 일반화하지 않는다. CPU peak는 모델 파일 크기보다 클 수 있고 JP 사전·내장 모델·full-sequence attention/classifier·원본+변환 dtype·ASR 동시 resident를 포함해 측정해야 한다. 첫 시험은 root single worker에서10~20초·batch1·CPU로 제한하는 제안이며 임의 메모리 상한 보장은 없다.

후속 단계:

1. lumos의 배포 고지/IPADIC·KO 사전 반입 조건과 Windows CPU library build를 먼저 검토한다. 별도 known-text adapter로 한국어 정답 원문·기존 일본어 transcript ID3개를 입력한다. JP는 nagisa reference와 분절 표를 먼저 비교하고 차이를 숨기지 않는다.
2. valle는 영어 정답+기존 짧은 음성의 통제군으로 동일 원본 가중치·원문·timebase를 비교한다. JP/KO 오류 거절도 검증하며 해당 언어 timestamp를 통과로 표시하지 않는다.
3. eclipse는 구현/내장 nagisa 모델 권한이 명확해진 뒤 공식 분절에 가까운 비교 후보로 승격한다. 자동 dependency fetch로 weights 반입을 우회하지 않는다. QwenASR는 별도 Unix 환경 또는 Windows port 과제로 남기며 VM을 켜지 않는다.
4. raw class ID·raw/fixed 시간·수정 사유·source offset·duration/단조성·zero/inverted/unmatched를 보존한다. 사전/분절 차이를 모델 정확도 차이와 분리한다. 사람의 경계 정답 없이 timestamp 오차를 계산하지 않는다. LIS repair가 의미 정답을 증명하지 않으며 zero duration을 자동 저장하지 않는다.

이 조사는 실제 Rust 정렬 구현의 존재와 구체적 준비 경로를 확인했다. 가중치 다운로드·빌드·추론·정렬 품질 검증·제품 통합 완료는 주장하지 않는다.
