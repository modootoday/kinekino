---
id: sot:kinekino/rust-2
title: 최신 추론 엔진 비교와 Rust 실측 환경
status: active
created: 2026-10-29
updated: 2026-10-29
tags: [player]
verify: manual
---
# 최신 추론 엔진 비교와 Rust 실측 환경

## 기준

사용자 지시: 빠르게 변하는 최신 엔진을 조사하고 llama.cpp를 기본 정답으로 전제하지 않는다. HF 모델·경량 양자화·현 i9-9900KF/64GB/RTX2070S8GB 호스트(목표 노트북과 동급)·Windows·모든 자체 개발 코드 Rust·기존 ChatGPT/Terminal 유지가 기준이다. 모델 기능과 엔진 기능을 분리하고 업체/작성자 벤치마크와 이 호스트 실측을 구별한다.

## 공식 근거 비교 — 2026-10-18 조회

| 엔진 | 공식 확인 내용 | 이 프로젝트의 판단/미검증 |
|---|---|---|
| [mistral.rs](https://github.com/EricLBuehler/mistral.rs), [v0.9.4](https://github.com/EricLBuehler/mistral.rs/releases/tag/v0.9.4) | Rust SDK·HF/GGUF·ISQ/UQFF, 벤치 CLI. 공식 Windows 사전 빌드는 CPU. 릴리스 API v0.9.4/2026-09-24 | Rust 텍스트 추론 우선 비교 후보. CPU portable을 작업 폴더에 준비·릴리스 SHA256 검증. Windows CUDA를 사전 빌드 CPU 성공으로 주장하지 않음 |
| [mistral.rs v0.9 CPU 보고서](https://github.com/EricLBuehler/mistral.rs/blob/master/releases/v0.9.0/report.md) | GB10 ARM/Xeon에서 context·양자화별 비교. 일부 decode 이점과 x86 prefill 열세를 함께 공개 | 작성자 보고서이며 이 호스트/모델·Q4_K_M에서 승리한다는 증거는 아님. UQFF 균일 q4k와 혼합 GGUF Q4_K_M도 동일 비트가 아님 |
| [HF Candle](https://github.com/huggingface/candle) | Rust 경량 프레임워크·CPU/CUDA·Whisper/분할/양자화 등 예제 | ASR/비전의 Rust 직접 통합 후보. 모델별 구현·전처리·연산자/양자화 지원을 확인해야 하며 모든 HF 모델을 자동 실행하는 범용 로더로 간주하지 않음 |
| [ONNX Runtime GenAI](https://onnxruntime.ai/docs/genai/), [소스](https://github.com/microsoft/onnxruntime-genai) | 생성 API는 preview 표기, C API와 모델 구성/EP 경로 제공 | Rust에서 C ABI/검증된 바인딩으로 연결하는 후보. HF→ONNX/GenAI config·캐시·토크나이저·모델 architecture가 맞아야 함. 일반 ORT가 있다고 GenAI도 설치된 것은 아님 |
| [WinML/DirectML](https://onnxruntime.ai/docs/execution-providers/DirectML-ExecutionProvider.html) | DirectML은 유지보수 단계이며 신규 Windows 배포 기능은 WinML로 이동 | 새 Windows ONNX 구성은 WinML/현 EP ABI부터 검토. 기존 DirectML을 최신 우선 경로로 고정하지 않음 |
| [TensorRT RTX EP](https://onnxruntime.ai/docs/execution-providers/TensorRTRTX-ExecutionProvider.html) | RTX30/Ampere 이상 요구. 기존 built-in EP는 deprecated, 별도 ABI plugin 권장 | 현 RTX2070S는 대상에서 제외. ‘RTX GPU 보유’만으로 이 경로를 설치하지 않음. CUDA EP와 별개의 조건 |
| [OpenVINO 2026.4](https://docs.openvino.ai/2026/get-started/install-openvino.html), [GenAI](https://docs.openvino.ai/2026/openvino-workflow-generative.html) | Windows 배포·작은 footprint·int4/int8 compression·KV 최적화·GGUF 경로 문서 | Intel CPU 최적화 비교 후보. NVIDIA GPU 가속 경로라고 가정하지 않음. Rust에서 사용할 ABI/기능·변환비·배포 크기 확인 필요 |
| [MLC LLM](https://llm.mlc.ai/docs/install/mlc_llm.html) | Windows CPU+Vulkan·nightly 패키지·TVM/JIT/컴파일 의존 문서 | RTX의 Vulkan 후보이나 Rust 배포 연결과 모델 컴파일·캐시·toolchain 비용이 추가. Python 개발 코드를 프로젝트에 추가하는 방식은 선택하지 않음 |
| [llama.cpp](https://github.com/ggml-org/llama.cpp) | HF/GGUF·다중 backend·Windows CPU portable benchmark | 비교 기준선. 최신 stable 포인터 v0.7.0에는 nightly-tag만 있어 실제 CPU 아티팩트 b11716을 별도 확인/고정. 최종 엔진 선정 아님 |

OpenVINO/MLC/Candle/ORT의 엔진 전체를 설치/실측 완료한 것은 아니다. 현재 환경에 있던 PyTorch2.7.1+cpu·transformers4.53.2·ORT1.22.1은 읽기 전용 조사했으며 새 기준 환경으로 그대로 채택하지 않았다. Python으로 준비했던 자체 벤치 코드는 Rust 통일 지시 후 제거했다. 가중치·설정·오디오 fixture는 데이터로만 유지한다. Python 추론은 실행하지 않았다.

## CPU Q4 실제 측정 결과

연속 재생 창과 소유 수집기를 종료한 뒤 Rust preflight를 통과하고 엔진을 순차 실행했다. ChatGPT/Terminal은 유지했다. 같은 Qwen3-0.6B Q4_K_M 파일, 입력128/생성64·반복3 기준이다. 아래는 짧은 synthetic CPU 표본이며 실제 편집 품질이나 엔진 전체 우열을 뜻하지 않는다.

| 엔진 | 관측값 | 프로세스 트리 표본 최대 private / working set | 실행 시간 / runtime 크기 |
|---|---|---|---|
| mistral.rs v0.9.4 CPU | TTFT128 521.32ms, prefill247.6±21.8t/s, depth128 decode36.6±0.6t/s | 1,251,987,456 / 1,538,170,880B | 12.779초 / 98,571,776B |
| llama.cpp b11716 CPU, thread4 | depth0 pp424.99 / tg55.26t/s, depth128 pp382.98 / tg52.79t/s | 453,804,032 / 726,495,232B | 10.744초 / 50,248,989B |

[mistral 실행·해시·전후 자원](../../../../evidence/ai/mistral-cpu-q4-2026-10-18.json), [raw 출력](../../../../evidence/ai/mistral-cpu-q4-2026-10-18.txt), [llama 실행·해시·전후 자원](../../../../evidence/ai/llama-cpu-q4-2026-10-18.json), [raw 출력](../../../../evidence/ai/llama-cpu-q4-2026-10-18.txt). TTFT/pp 정의와 warmup·컨텍스트 할당이 달라 직접 배율 비교하지 않는다. 이번 CPU 표본은 새로운 엔진이 반드시 더 빠르고 가볍다는 가정을 지지하지 않는다. 최종 엔진은 아직 미선정이다.

첫 mistral 실행에서 외부 tokenizer의 vocabulary151669와 GGUF151936 불일치가 발생했다. 강제 tokenizer override를 제거하고 GGUF 내장 tokenizer를 사용해 정상 완료했다. [실패 기록](../../../../evidence/ai/mistral-tokenizer-failure-2026-10-18.json). Rust 도구의 HF 캐시 blob 경로에는 확장자가 없는 문제도 수정해 선언된 GGUF 경로를 유지하면서 실제 경로의 캐시 범위·해시를 검사한다. ASR/OCR/비전·GPU·한일 언어 정확도·재생 경쟁·노트북 발열은 미측정이다.

## 실제 준비한 환경

`runtime/ai-benchmark` 안에 portable CPU 두 엔진을 준비했다. 시스템 PATH·드라이버·전원·기존 Python 패키지·VM 설정을 변경하지 않았다. 다운로드 패키지 SHA256:

- mistral.rs v0.9.4: `1548778deb036d518051c13055922422d69030b958e1a85c970756e8b32679d0`.
- llama.cpp b11716 CPU: `79b143075ab87fb5aff6f8f8ca80500a984f4120b5199fa9f9b70b284f17428e`.
- [Unsloth Qwen3-0.6B Q4_K_M](https://huggingface.co/unsloth/Qwen3-0.6B-GGUF), revision `50968a4468ef4233ed78cd7c3de230dd1d61a56b`, 396705472B, SHA256 `ac2d97712095a558e31573f62f466a3f9d93990898b0ec79d7c974c1780d524a`. 원본 Qwen과 다른 변환 배포자임을 기록한다. 품질 동일성을 아직 확인하지 않았다.

Rust 도구 [tools/validation](../../../../../tools/validation/Cargo.toml)은 domain/playback에 연속성 정책, application에 preflight/benchmark/soak 유스케이스, infrastructure에 엔진 argv·프로세스 측정·파일 기록을 분리한다. 앱 소스와 독립된 작은 실행 도구이며 새 검증 로직은 Rust다. 모델·실행 파일은 고정 경로/해시로 확인하고 셸 명령·웹 입력으로 실행하지 않는다.

## 자원 절약 절차와 측정 계약

소유 재생 시험을 완료·정상 종료→소유 수집기 종료→다운로드 완료→read-only preflight→엔진 하나씩→종료/모델 해제. 기존 ChatGPT·Terminal·사용자 앱·WSL·서비스는 종료하지 않는다. preflight는 보호 프로세스 PID, 가용 RAM/CPU, 재생 시험 잔존 여부를 기록한다. RAM6GiB 미만·CPU80% 이상·재생 앱 잔존이면 추론을 대기시킨다.

worker는 BelowNormal, Rayon/OMP/MKL 4, 가능한 엔진 thread 옵션4, 오프라인이다. 100ms마다 소유 프로세스 트리의 private/working set을 수집한다. 표본 peak는 순간 peak를 놓칠 수 있고 memory-mapped 모델은 private만으로 평가할 수 없어 working set도 기록한다. OS 캐시를 강제 비우지 않으며 ‘cold process’와 ‘cold disk’를 구분한다.

동일 Q4 파일을 사용하되 CLI benchmark의 warmup·prefill/decode 정의가 달라 결과를 곧바로 승수 비교하지 않는다. 입력128/생성64·반복3을 우선 맞추고 JSON/raw stdout/stderr·exe/model hash·runtime 디렉토리 bytes·전후 자원을 남긴다. 모델 정확도, 장문·한일 언어, ASR/OCR/분리·깊이, GPU/재생 경쟁은 별도 게이트다. 최종 엔진은 성공한 실제 조합·품질·메모리·배포 비용으로 선정한다.
