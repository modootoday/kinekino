---
id: evidence:kinekino/20261029113711
title: Qwen ASR 혼합 INT4 ONNX 실행 준비
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Qwen ASR 혼합 INT4 ONNX 실행 준비

2026-10-18 준비 단계의 기록이다. 경량 양자화 후보 조사·Windows 타입 검사·작은 계약 파일 취득 당시 실제 모델 로드·추론은0회였다. 후속 가중치 취득과 실제 결과는 [혼합 INT4 실제 검증](qwen-onnx-actual.md)을 따른다. 기존 [Qwen F32 실제5회](qwen-asr.md)와 구별하며 같은 모델 묶음의 실행 경로로 기록한다.

## 실행 경로 확인

| 항목 | 고정 식별 또는 관측 |
|---|---|
| Rust 엔진 fork | `andrewleech/transcribe-rs`의 `feat/qwen3-batch`, commit `d0dbccabe88dec21d4c1e24f97265316eb958bc1` |
| 기본 브랜치 | 이번 조사 시 `d97ae65debaa9bacd5d56aa7d308d13df43c97ec`; 이 main의 Qwen 지원으로 간주하지 않음 |
| ONNX 변환 도구 | `andrewleech/qwen3-asr-onnx`, 현재 source commit `4aee2ae0c355c7bc33bd5c09befec9e9eabd4b50` |
| HF 모델 | `andrewleech/qwen3-asr-0.6b-onnx`, revision `4fc24a1402e74db89c4d2ef256875e71680128c4` |
| 고정 metadata SHA256 | `dd0cded9ea7d8ffee09357514556bc3f37bbda08b640cb0c60d60fdb6a4ee142` |
| Rust 라이브러리 | transcribe-rs0.3.11, ORT2.0.0-rc.12, MIT 선언 |
| 모델 카드 | Apache2.0 선언·gated=false. 별도 배포 고지와 원본 checkpoint revision 계보는 미해소 |

기본 cjpais repo 또는 fork main의 문서만으로 Qwen 실행 API가 있다고 판단하지 않았다. 고정 feature branch의 `src/onnx/qwen3/{model,engine,prompt,tokenizer,mel}.rs`를 읽었다. Rust가 mel·prompt·decode를 처리하고 ONNX Runtime native 엔진을 호출하는 경로이며 순수 Rust tensor 엔진으로 부르지 않는다. 기존 Candle F32 loader에 이 파일을 넣지 않는다.

## Windows 검사와 소스 보존

처음 `--locked --offline --no-default-features --features qwen3 --lib` 검사는 optional `async-openai`의 로컬 index 부재로 실패했다. Qwen 추론 실패가 아니다. 그 뒤 고정 lock을 유지한 온라인 check로 필요한 crate만 확보해 성공했고, 같은 offline check의 exit0 로그를 보존했다. `ORT_SKIP_DOWNLOAD=1`을 사용했으며 native runtime 취득·최종 EXE link·실제 모델 실행을 검사한 것이 아니다. checkout은 detached 고정 commit이며 `git status --porcelain`은 비어 있었다.

[검사 로그·원본 source tar](../../../evidence/ai/qwen-onnx-preparation-2026-10-18/)를 보존했다. tar는 고정 commit의 `git archive`이며 SHA256 `43cb7701a3b47871223034bcadc32d4d21786695b2c9f5930859f1a4088d9405`다. target·모델을 archive에 넣지 않았고 upstream tests의 LFS pointer를 실제 음성 파일로 간주하지 않는다.

## 취득한 작은 계약 파일

root의 Rust `player-model-spike`에 후보 key만 추가했다. fmt·31tests·Clippy all-targets Dwarnings·locked offline build 통과 후 [86파일 frozen build](../../../evidence/ai/model-spike-builds/586d62129baf96388b3f478462bffc65ca09afd1aebb1f22146f7f135e994188/)를 먼저 보존했다. EXE SHA256은 디렉터리명과 같다.

`catalog-fixed`로 모델 revision을 고정하고 `fetch-record qwen-onnx-contract-r1`을 실행했다. 사전 CPU8.7890625%, 여유 RAM36,877,557,760bytes로 자원 gate를 통과했다. 기존 player 때문에 일반 preflight ready=false였으나 streaming acquisition의 독립 자원 기준을 적용했다. 사용자 앱·VM·DPI·전원·캐시는 변경하지 않았다.

| 파일 | bytes | SHA256 |
|---|---:|---|
| README.md | 3,379 | d138aa17d5cd131ffcd038504cdb07d3fe83032e59b264f55e9998f8e1a7ca69 |
| config.json | 1,250 | df31c4689abe9d782366fffd2454b546291d0205d082b3fd01b99fb76a45b11f |
| preprocessor_config.json | 298 | 9c7c558a05f326fe4365e5e59e486383ff127dfd93ce1cd5e6e23f883cd02281 |
| tokenizer_config.json | 12,488 | 784c92a0a81fbbc440afb89842f8c16c8da28dfb9ff381fcea585b84f4eed7d1 |
| decoder_init.int4.onnx | 355,390 | 6d54633bbb9e6b1ecf372f218d04adb3bf6f2bfc442ce55c988526d532896faa |
| decoder_step.int4.onnx | 354,888 | 7eaa4f31d5eb6ae2937ff45222c32f692f2d5ac29b46cbae9046da332a9f317d |
| tokenizer.json | 11,429,377 | bd2a97b55c8f7f9c328c73ee9b9178771037e9f566dfca8e238a063d41cbac92 |

LFS3개 파일은 local hash와 HF LFS hash를 대조했다. 나머지4개는 local SHA를 기록했으며 LFS hash 대조로 표시하지 않는다. 원본 request/preflight/각file/result는 runtime acquisition에 보존하고 공통 report로 증거 snapshot에도 복사한다.

## 大きなファイルと精度の境界

아래는 서버 metadata와 카드로 확인했으며 아직 취득하지 않았다.

| 必要ファイル | bytes | HF LFS SHA256 |
|---|---:|---|
| encoder.int4.onnx | 745,762,694 | 3c027f880f677615de85e1f6934906e1d5d77624b724096d33accef99c753eed |
| decoder_weights.int4.data | 962,460,672 | d68cd1c0695a7ba42651d06b1bc1158e2e58af5f9adbb6dba874fbbb8a4f22cf |
| embed_tokens.bin | 311,164,928 | e80150119fa5f7e56e85aed64c3a02d5c78eb7a37cfdcb973d0987316f15bee2 |

encoder.int4.onnx와 encoder.onnx의 size·LFS hash가 같으며 카드도 FP32라고 명시한다. decoder만 MatMulNBits INT4 RTN block64 accuracy_level4다. 임베딩 파일은FP16이지만 Rust loader가F32로 확장한다. 전체가4bit라고 표시하거나 기존1,876,091,704byte safetensors보다 작은 다운로드라고 가정하지 않는다. native runtime·ORT 최적화·KV cache와 임베딩 확장까지 실제 private/working set에 반영해야 한다.

## 실제 실행 전 남은 조치

1. 원본과 별도인 frozen Rust fork에서 encoder·decoder intra4/inter1·순차 실행을 고정한다. 현재 encoder 기본 pool과 decoder_threads=0은 정책을 만족하지 않는다.
2. INT4 세 graph와 data·embedding을 모두 사전 hash 대조한다. 파일 누락 시 원본의 FP32 fallback을 adapter가 차단하고 변형을 보고한다.
3. 공식 tokenizer로 Korean/Japanese/English 언어 prompt ID를 대조한다. 현재 fork의 encode는 BPE merge가 아닌 greedy longest-match이며 짧은 영문 이름의 일치를 아직 실물 확인하지 않았다.
4. prefix 후처리 전 raw 출력·EOS/max-token 상태를 보존하고, native runtime·EXE·source·input·model manifest를 고정한다. 최대128token·30초 미만 입력·CPU4·단일 worker·timeout600초를 적용한다.
5. 기존 KO/JA3개 음성으로 CER를 대조한다. 엔진·export·mel·prompt가 달라 기존 Candle와의 차이를 양자화만의 효과라고 단정하지 않는다. 같은 ONNX FP32 경로의 paired 비교는 별도 후속이다.

## 공식 자료

- [고정 Rust source](https://github.com/andrewleech/transcribe-rs/tree/d0dbccabe88dec21d4c1e24f97265316eb958bc1)
- [고정 모델 카드와 파일](https://huggingface.co/andrewleech/qwen3-asr-0.6b-onnx/tree/4fc24a1402e74db89c4d2ef256875e71680128c4)
- [변환 도구](https://github.com/andrewleech/qwen3-asr-onnx/tree/4aee2ae0c355c7bc33bd5c09befec9e9eabd4b50)
- [Sherpa Qwen INT8 대안](https://k2-fsa.github.io/sherpa/onnx/qwen3-asr/pretrained.html): 공식 Rust binding은 별도 native C++ adapter 경로다. 이번에는 다운로드·실행하지 않았다.
