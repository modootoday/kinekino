---
id: sot:kinekino/rust-qwen3-asr
title: Rust Qwen3 ASR 실행 경로 재조사
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Rust Qwen3 ASR 실행 경로 재조사

2026-10-18. 새 조사 목표로 고정 source/Cargo/CLI/HF metadata만 읽었다. 소스/모델 설치·모델 다운로드·빌드·추론은 하지 않았다. 이 문서가 앞선 조사에서 남긴 `Rust 경로 미확보` 표현을 최신 범위로 한정한다. **Qwen3 ASR의 Rust 구현은 존재한다. Windows CPU 빌드/실제 일본어·한국어 품질은 미실측이며 ForcedAligner 지원과 별개다.** 기존 정렬 문서는 수정 범위 밖이므로 이 문서와 plan008을 최신 인계로 참조한다.

## 선택: 직접 엔진을 먼저, 같은 엔진의 어댑터를 차선으로

| 후보·고정 소스 | 소스 확인 결과 | 다음 실행 판정 |
|---|---|---|
| 1. [alan890104/qwen3-asr-rs](https://github.com/alan890104/qwen3-asr-rs/tree/c5ef09646af6278d2ba8b8ceaf543ffb32d1a5dc), package qwen3-asr0.2.2 | Candle0.9.2 tensor, Rust mel/encoder/decoder·local safetensors loader·CPU F32 변환·language prompt·batch/streaming API | 최소 외부 계층으로 CPU ASR spike 우선. tokenizer.json 생성만 작은 Rust 준비 adapter 필요. GGUF 지원 없음 |
| 2. [voxora adapter](https://github.com/airvzxf/voxora/tree/126de16bcdce6b4c0f52781ac25b9c1741f81c40), published voxora-qwen3asr0.6.3 | 같은 qwen3-asr0.2 의존, default-features=false. CPU 기본. 로컬 tokenizer 준비 public API·언어/옵션 검증 | source가 확인된 로컬 tokenizer helper를 재사용하거나 공통 계약 비교용. 독립 엔진 품질 대조군은 아님 |

순수 Rust 모델 계산 경로이며 ONNX Runtime 또는 libtorch wrapper가 아니다. 다만 tokenizers의 `onig`와 향후 CUDA/Metal backend에는 native build/link 의존성이 있을 수 있어 전체 의존성이 Rust만이라고 부르지 않는다. 별도 C/C++ 추론 엔진을 설치할 필요가 없다는 점과 Windows 빌드 성공은 다른 주장이다.

## alan 소스와 Windows CPU 빌드 계약

고정 [Cargo.toml](https://github.com/alan890104/qwen3-asr-rs/blob/c5ef09646af6278d2ba8b8ceaf543ffb32d1a5dc/Cargo.toml)은 edition2021, rust-version1.75를 선언한다. default=`metal`, optional cuda/hub, candle-core/nn0.9.2, tokenizers0.22(onig), hound3, rubato0.15, rustfft6이다. **Windows CPU는 `--no-default-features` 필수**다. Rust1.75 선언만으로 현재 lock 전체 MSRV가1.75에서 검증됐다고 보지 않는다. root Cargo.lock은 존재하며 조사 tree에는 `.github/workflows`가 없어 Windows CI 통과 근거는 확보하지 못했다.

아래는 다음 실행 담당이 별도 격리 source checkout을 준비한 뒤 수행할 정확한 build 명령이다. 이 조사에서 수행하지 않았다. 기존 checkout이 있으면 그 디렉토리를 덮어쓰지 말고 별도 경로를 선택한다. source root의 Cargo.lock을 보존한다.

```text
git clone --no-checkout https://github.com/alan890104/qwen3-asr-rs runtime/ai-spike/source/qwen3-asr-rs
git -C runtime/ai-spike/source/qwen3-asr-rs checkout --detach c5ef09646af6278d2ba8b8ceaf543ffb32d1a5dc
cargo build --manifest-path runtime/ai-spike/source/qwen3-asr-rs/Cargo.toml --locked --release --no-default-features --bin benchmark --jobs 2
```

`benchmark`는 실제 `src/bin/benchmark.rs`에 존재하는 bin이다. `--no-default-features` 상태에는 Hub 다운로드 feature도 없다. 기존 target dir 환경변수에 따라 실행 파일 위치가 달라질 수 있으므로 빌드 출력에서 확인한다. 소스 기본 출력 위치일 때 다음 CLI는 실재한다:

```text
runtime/ai-spike/source/qwen3-asr-rs/target/release/benchmark.exe --model-dir runtime/ai-spike/models/qwen3-asr-06b-local --audio-dir runtime/ai-spike/assets/qwen3-asr-wav-fixture --runs 1 --label qwen3-asr-06b-cpu
```

두 입력 디렉토리는 **준비가 필요한 제안 경로**이며 현재 자동 생성돼 있다고 가정하지 않는다. audio-dir에는 고정 hash의 기존16kHz mono PCM WAV만 넣고 빈 디렉토리 성공·출력 개수0을 통과시키지 않는다. args parser는 잘못된 인자를 조용히 무시하고 일부 인자 누락은 index panic 가능성이 있으므로 문서 밖 flag를 덧붙이지 않는다. `--language`, `--gguf`, `--timestamps`는 이 bin의 flag가 아니다. 언어 강제는 아래 API adapter에서 수행한다.

`examples/transcribe.rs`도 존재하지만 required-feature=`hub`이며 MODEL_ID/CACHE_DIR 환경변수와 `from_pretrained`를 사용한다. `src/hub.rs`는 resolve/**main**을 고정하고 `.complete` 없는 디렉토리를 삭제·재다운로드한다. revision/hash/기존 증거 보존 규칙에 맞지 않아 다음 spike에서는 호출하지 않는다. `--features hub`만 추가한 Windows 빌드는 default Metal도 켜지므로 사용하지 않는다.

## 정확한 HF 원본 파일과 tokenizer 준비

공식 [Qwen/Qwen3-ASR-0.6B](https://huggingface.co/Qwen/Qwen3-ASR-0.6B/tree/5eb144179a02acc5e5ba31e748d22b0cf3e303b0), SHA `5eb144179a02acc5e5ba31e748d22b0cf3e303b0`의 API blobs를 읽었다. 다음은 서버 metadata이며 실제 다운로드/로컬 hash 검증이 아니다.

| 파일 | byte | 역할 |
|---|---:|---|
| model.safetensors | 1,876,091,704 | encoder+decoder 원본 가중치; LFS SHA256 `79d6cbd4c98c7bbffe9db2edac07f56cd6637d0d5944b27f6c2b8353840323ea` |
| config.json | 6,193 | loader 모델 구조 입력 |
| vocab.json | 2,776,833 | tokenizer 합성 원본 |
| merges.txt | 1,671,853 | tokenizer 합성 원본 |
| tokenizer_config.json | 12,487 | added_tokens_decoder 포함 tokenizer 합성 원본 |
| preprocessor_config.json | 330 | 전처리 parity 기준; alan loader가 자동 적용하지 않음 |
| chat_template.json | 1,161 | prompt parity 기준; alan은 hardcoded prompt 경로 |
| generation_config.json | 142 | reference generation 기준; 실제 options와 비교 |
| README.md | 57,456 | 공식 모델 카드와 Apache-2.0 선언 보존 |

원본 HF tree에는 `tokenizer.json`이 없다. 따라서 safetensors+config를 받았다고 `AsrInference::load` 준비가 끝나는 것이 아니다. loader 필수 파일은 config.json, model.safetensors(또는 지원 index/shards), **tokenizer.json**이다. 추가 준비는 Rust로 vocab/merges/tokenizer_config에서 tokenizer.json을 합성하고 입력3파일 hash·helper source SHA·출력 hash를 기록한다. alan helper는 hub module 안의 private 함수이므로 외부에서 직접 호출할 수 없다. 새 프로젝트 adapter로 그대로 공개한 것처럼 호출 예제를 지어내지 않는다.

차선 구현인 voxora의 실제 public API `QwenAsrEngine::prepare_model_dir(&Path)`는 `hf` feature 하에서 로컬3파일로 tokenizer.json을 만들며, 이후 `load_with_device(..., Device::Cpu)`가 가능하다. 해당 helper의 NFC/정규식 split/ByteLevel/BPE·추가 토큰 순서가 공식 tokenizer와 일치하는지 일본어·한국어·특수 토큰의 encode/decode parity를 확인해야 한다. tokenizer를 새 모델 가중치라고 취급하지 않고 파생 파일로 추적한다.

model card의 Apache-2.0 표기와 Qwen 공식 [코드 LICENSE](https://github.com/QwenLM/Qwen3-ASR/blob/7c6daf77a2421100f5fb066495372c00129d39ff/LICENSE)를 분리 보존한다. HF 이 SHA에는 별도 LICENSE/NOTICE 파일이 없으므로 가중치 원문 확보 완료라고 쓰지 않는다. alan [MIT LICENSE](https://github.com/alan890104/qwen3-asr-rs/blob/c5ef09646af6278d2ba8b8ceaf543ffb32d1a5dc/LICENSE)와 모델 Apache·tokenizer 부속 권리·학습데이터는 서로 다른 항목이다.

## 전처리·언어·정밀도·메모리와 품질 게이트

[inference.rs](https://github.com/alan890104/qwen3-asr-rs/blob/c5ef09646af6278d2ba8b8ceaf543ffb32d1a5dc/src/inference.rs)와 [mel.rs](https://github.com/alan890104/qwen3-asr-rs/blob/c5ef09646af6278d2ba8b8ceaf543ffb32d1a5dc/src/mel.rs)를 읽었다.16kHz, FFT400, hop160, config mel bins(공식128), Hann/Slaney filterbank, hop배수 zero-pad·center reflection·마지막 STFT frame 제외·log10/clamp(max−8)·(x+4)/4 경로다. hound WAV 입력은 채널 평균 후 필요하면 rubato resample한다. 기존16kHz mono fixture에서는 resample을 피하고 sample count/원본 duration을 보존한다. mel parity·짧은 입력 reflect edge·pad 유발 반복을 별도 시험한다. kaldi-fbank의 기존 parity 관측을 이 mel 구현의 검증으로 재사용하지 않는다.

`TranscribeOptions::default().with_language("Japanese")`/`"Korean"`와 `with_max_new_tokens(...)`가 실제 API다. `ja`/`ko`가 아닌 전체 언어명으로 prompt를 만든다. forced mode 결과 language는 검출명 대신 `forced`로 기록될 수 있으므로 입력 강제 언어도 manifest에 남긴다. 기본 max_new_tokens512; 첫 spike는16.74초 이하3표본·단일 worker·CPU로 제한하는 제안이다. 무음·반복·허위 언어·입력시간 초과 decode·빈 출력·timeout을 분모에 포함한다.

encoder/decoder는 모두 `LinearW(candle_nn::Linear)`의 dense tensor 경로다. GGUF/QTensor/양자화 matmul loader가 없다. CPU에서는 BF16/F16 tensor를 F32로 바꾸므로1.876GB 파일이 실행 메모리 상한이 아니다. 변환 중 원본+F32·audio activation·KV cache·tokenizer·allocator와 unload 후 잔존을 측정한다. CPU4/interop1은 root adapter가 지원 여부를 확인해 적용하며 이 crate에 존재하지 않는 `--threads` flag를 쓰지 않는다. [CPU integration test](https://github.com/alan890104/qwen3-asr-rs/blob/c5ef09646af6278d2ba8b8ceaf543ffb32d1a5dc/tests/cpu_inference.rs)는 실제 모델이 필요한 ignored test이며 통상 cargo test 통과가 inference 성공 근거는 아니다.

## fork·wrapper·다른 crate 판정

- [airvzxf fork](https://github.com/airvzxf/qwen3-asr-rs/tree/3b262c3fa5a48da693865240959348f00751a51a)의 기본 branch는 bench/compare-rust-vs-torch다. Cargo/linear/transcribe 경로는 같은0.2.2·default Metal·dense safetensors 경로이며 추가 HTTP/stdin 예제가 있다. 이를 새 양자화 엔진이나 Windows 검증으로 승격하지 않는다. 같은 MIT 고지를 유지한다.
- [voxora-qwen3asr0.6.3 package Cargo](https://docs.rs/crate/voxora-qwen3asr/0.6.3/source/Cargo.toml)는 edition2024/MSRV1.88, Apache-2.0, cpu default, optional hf/metal/cuda, qwen3-asr ^0.2 default-features=false다. registry checksum `4746343359d38824bdca172817e381ce6b4289a17140707fe6369975dff2d589`. fixed GitHub source의 [engine.rs](https://github.com/airvzxf/voxora/blob/126de16bcdce6b4c0f52781ac25b9c1741f81c40/voxora-qwen3asr/src/engine.rs)는 word_timestamps=false, streaming=false를 명시한다. upstream streaming API가 있어도 wrapper 기능과 같지 않다. full-name20개 언어 검증은 Japanese/Korean을 포함하지만 official 모델의 더 넓은 언어 지원 전체를 노출하는 것은 아니다. registry package와 GitHub commit이 byte-identical임은 미검증이다. 선택 시 아래 Git source build에 고정하고 registry 대조는 별도 한다.
- [mutranscriber0.2.0](https://docs.rs/crate/mutranscriber/0.2.0/source/Cargo.toml)은 MIT OR Apache-2.0·default GStreamer·candle0.9 기반 자체 audio_encoder/qwen3_decoder/mel 모듈이다. registry checksum `e484195f1511000d69684bf999cedaf83bf3ff4e3258b18d5f04ec24375f4ecc`. [transcriber.rs](https://docs.rs/crate/mutranscriber/0.2.0/source/src/transcriber.rs)는 no-default-features에서 파일 transcribe가 GstreamerNotEnabled를 반환하고16kHz f32 slice용 transcribe_audio는 별도다. [model.rs](https://docs.rs/crate/mutranscriber/0.2.0/source/src/model.rs)는 safetensors/BPE load 경로다. 검토한 module/API에 GGUF/강제 정렬 구현을 확인하지 못했다. repository metadata는 null이므로 commit 계보는 아직 못 고정했으며 package checksum으로만 고정한다. 최소 Windows spike에서는 native media stack 확대와 별도 tokenizer 검증 부담 때문에 우선순위에서 제외한다.

voxora 차선의 실제 library build 명령(다음 담당용, 미실행): 고정 Git checkout을 `runtime/ai-spike/source/voxora`에 준비한 뒤 다음을 수행한다. Git source의 Cargo.lock을 보존하며 Rust1.88 이상과 Windows native 의존성 빌드를 먼저 확인한다.

```text
git clone --no-checkout https://github.com/airvzxf/voxora runtime/ai-spike/source/voxora
git -C runtime/ai-spike/source/voxora checkout --detach 126de16bcdce6b4c0f52781ac25b9c1741f81c40
cargo build --manifest-path runtime/ai-spike/source/voxora/Cargo.toml --locked --release -p voxora-qwen3asr --no-default-features --features cpu,hf --lib --jobs 2
```

이 명령은 library만 빌드하며 WAV를 전사하는 새 CLI가 아니다. `prepare_model_dir`와 `load_with_device`를 호출하는 작은 Rust adapter는 후속 구현이다. 기존 player-model-spike의 infer role에 Qwen이 이미 등록됐다고 주장하지 않는다. docs.rs build 실패 표시는 확인됐지만 해당 Linux docs build와 Windows CPU build 결과는 별개다.

## Alkd GGUF: 파일은 실재하나 현재 선택 엔진과 불일치

고정 [Alkd/qwen3-asr-gguf](https://huggingface.co/Alkd/qwen3-asr-gguf/tree/06a21d592b9cfea5e8dbeea1cab79d67861ae5b8)의 카드·blobs를 읽었다. 아래는 다운로드 필요 목록이 아니라 **보류 중인 대안 파일 inventory**다.

| 파일 | byte | LFS SHA256 |
|---|---:|---|
| qwen3_asr_0.6b_q4_0.gguf | 546,036,256 | `540b85ffa6c6181d1bfa964e54e31b43aacf1659a197bf35e1d64599da12f5e1` |
| qwen3_asr_0.6b_q4_k.gguf | 604,699,168 | `7245b8ec39cc103e12fee692cda5944d636bea5b7a0b84e5415c8217ef01d036` |
| qwen3_asr_0.6b_q8_0.gguf | 1,012,824,608 | `f64a088123e9cff226cfbf7e0d1f26a1674561900e108e14a4d7acd73aba43b5` |
| qwen3_asr_1.7b_q4_k.gguf | 1,340,432,352 | `426c8cb67cf12b227be5e2796a1240055823b7f01de3b00bea6f5ade62a81775` |
| qwen3_asr_1.7b_q8_0.gguf | 2,512,740,320 | `ccb17fabe7e2f9b5363dfb5a77ece8d836038f0c601157df220d4dbd3f1a8928` |

카드는 Apache-2.0과 Q4_K 차원256 미정렬 tensor의 Q8_0 fallback(0.6B encoder dimension896)을 선언한다. 이는 **변환자 주장**으로, tensor별 quant dtype를 직접 읽어 검증하지 않았다. 원본 Qwen checkpoint revision·quantizer commit·변환 recipe·tokenizer/config 포함 여부가 고정돼 있지 않고 HF tree는 GGUF5개+README+.gitattributes뿐이다. 별도 LICENSE/NOTICE도 없다. `2/2` 광고 정확도는 한국어·일본어 품질 근거가 아니다.

카드의 `cargo run --release -- --gguf ...`는 선택한 alan source에서 유효하지 않다. [공식 저장소 issue5](https://github.com/alan890104/qwen3-asr-rs/issues/5)도 이 불일치를 제보했고 조사 시 open이다. 사용자가 시도한 임시 GGUF 추가가 느렸다는 제보는 재현 benchmark가 아니므로 성능 결론으로 쓰지 않는다. 양자화 encoder/decoder를 실제로 지원하는 loader commit과 tensor명·config/tokenizer 계약을 확보하기 전 다운로드하지 않는다. 크기가 작다는 이유로 현재 safetensors runner에 전달하지 않는다.

## ForcedAligner와 upstream Candle 상태

alan/airvzxf의 module tree와 inference/config/result surface, voxora의 capability/engine, mutranscriber model/config/decoder/transcriber를 조사했다. 선택 runtime의 결과는 text/language/raw output이며 **ForcedAligner의 classify head·timestamp logits→시간 복원·word span API를 확인하지 못했다.** ASR audio encoder/decoder 존재가 aligner 지원은 아니다. 현재의 정렬 차단은 이 조사 후보 범위에서 유지되며 전 세계 Rust 구현 부재로 일반화하지 않는다. [기존 aligner 파일/config 계약](execution-readiness.md)을 별도로 따른다.

[Candle issue3345](https://github.com/huggingface/candle/issues/3345)는 구현 링크를 찾는 출발점이며 지원 판정은 고정 소스에 근거했다. [PR3509](https://github.com/huggingface/candle/pull/3509)는 조사 API상 open/merged=false, head `9ff601ccf151a1e4511b223cb18e670ab4596cb5`였다. 변경 목록은 qwen3_asr audio_encoder/model/rope/cache와 ASR example/tokenizer이고 aligner head 파일을 확인하지 못했다. PR 요약의 timestamp 관련 표현만으로 ForcedAligner를 지원한다고 쓰지 않는다. Candle upstream 미병합 상태와 별도 crate 구현 존재를 구분한다.

## 다음 실행 담당의 순서

1. source checkout/lock/feature build만 먼저 확인한다. 실패하면 Windows compiler/onig·dependency 오류와 Cargo.lock을 남긴다. 이 조사에서 build pass를 주장하지 않는다.
2. 고정 원본5개 loader/tokenizer 입력과 parity용 config·카드를 project Rust HF downloader로 manifest화하고 source/license를 연결한다. Hub main 자동 다운로드는 쓰지 않는다. 이번 연구 세션은 다운로드하지 않았다.
3. Rust tokenizer 준비 adapter를 작성하거나 voxora helper를 검토해 재사용한다. 모델·원문 텍스트 token parity와16k mono feature parity부터 확인한다. 새 adapter 명령은 구현 후에만 실행 문서에 추가한다.
4. root 단일 worker에서 기존 일본어3표본+한국어 원음의 text/CER·반복·언어 강제·원본 duration·private peak/unload를 측정한다. batch 자동 실행과 정렬 자막 저장을 분리한다. 양자화 비교·streaming·GPU·강제 정렬은 이 성공으로 완료하지 않는다.

이 결과는 `Rust ASR 경로 없음`이라는 탐색 차단을 해소한다. 남은 구체적 준비는 Windows build·tokenizer 합성/parity·고정 원본 파일 확보·실제 품질 측정이며 ForcedAligner는 별도다.
