---
id: adr:kinekino/20261029113533
title: "ADR010: 실측을 기반으로 AI 후보를 단계적으로 확대"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# ADR010: 실측을 기반으로 AI 후보를 단계적으로 확대

상태: 실험 경로 적용, 제품 기본 모델·배포 확정은 보류. 2026-10-18.

## 근거

사용자는 노트북 수준에서 동작하는 경량 HF 모델·양자화·Rust 우선 도구를 요구했다. 같은 호스트의 실제 입력 반복 비교에서 Whisper tiny Q8은 FP32보다 느렸고 Depth V2 Q8은 최대 표본 메모리가 더 컸다. 작은0.6B/0.8B 텍스트 모델은 구조/내용 품질을 통과하지 못했다. 파일 크기만으로 속도·실행 메모리·품질을 확정하지 않는다.

## 결정

Rust application/domain 계약과 엔진 어댑터를 유지하고 role별 실제 미디어 입출력·의미·자원·권리 게이트를 적용한다. 네이티브 ORT/FFmpeg와 Rust Candle/mistral.rs를 구현 언어와 구분한다. 구조화 출력은 스키마와 근거·내용을 각각 평가하며 모델 응답을 셸로 실행하지 않는다.

다음 텍스트 실험으로 같은 Qwen3.5 계열의2B Q4_K_M을 추가했다. [HF 변환 저장소](https://huggingface.co/unsloth/Qwen3.5-2B-GGUF)와 [공식 원본](https://huggingface.co/Qwen/Qwen3.5-2B)은 Apache2.0 표기를 확인했다. 변환 repo revision `f6d5376be1edb4d416d56da11e5397a961aca8ae`, 파일1,280,835,840 bytes, LFS SHA256 `aaf42c8b7c3cab2bf3d69c355048d4a0ee9973d48f16c731c0520ee914699223`를 고정한다. 2B가 더 최신이거나 항상 더 우수하다고 가정하지 않고0.8B 실패와 같은 프롬프트로 먼저 비교한다. 모델 구조는 VLM이지만 텍스트 경로만 실행하며 projector/영상 인식을 주장하지 않는다.

기존 파일 다운로드 상한1GiB는 유지하고 위 repo/revision/파일/hash의 정확한 조합에만1.5GB 상한을 허용했다. 변경 revision이나 다른 파일에는 확대 상한을 주지 않는다. 파일 전체를 RAM에 올리지 않고 스트리밍 저장·해시 검증한다. 이 상한은 실행 RAM/VRAM 보장이 아니다.

## 결과와 남은 판단

강제 정렬도 공식 `Qwen/Qwen3-ForcedAligner-0.6B` revision `c7cbfc2048c462b0d63a45797104fc9db3ad62b7`의 `model.safetensors`(1,835,544,544 bytes, LFS SHA256 `47831d0e82f96b20e9034dba01a075ee06436654719f6a68289e49f1b65ce0e7`) 조합에만2GB 취득 상한을 적용했다. 고정 lumos Windows CPU library 및 독립 adapter 첫 빌드 성공 후 취득했고 해시를 검증했다. 정렬 head가 있는 별도 checkpoint이며 ASR 가중치를 대체해 사용하지 않는다. GGUF 정렬 경로·재생 동시 resident·시간 정확도는 이 취득 성공으로 입증되지 않는다.

Qwen ASR 비교를 위해 공식 `Qwen/Qwen3-ASR-0.6B` revision `5eb144179a02acc5e5ba31e748d22b0cf3e303b0`의 `model.safetensors`(1,876,091,704 bytes, LFS SHA256 `79d6cbd4c98c7bbffe9db2edac07f56cd6637d0d5944b27f6c2b8353840323ea`) 조합에만 2GB 다운로드 상한을 추가했다. 고정 Rust 엔진의 Windows CPU release 빌드 성공을 확인한 뒤 취득을 시작했다. 이 엔진은 GGUF를 지원하지 않고 CPU BF16→F32 변환이 있으므로 경량 양자화 채택으로 표현하지 않는다. Whisper 비교 기준과 실제 메모리·품질 관측을 위한 실험 예외다. tokenizer ID 보존과 공식 전체 전처리/토큰 시퀀스 동등성은 별개 게이트다.

실제 결과는 [텍스트 검증](../../sots/verification/ai/text.md)과 [후속 계획](../../plans/player/plan-008-ai-followup-research.md)에 연결한다. 후보의 다운로드/실행 성공은 출시 선택이 아니다. 원본 checkpoint SHA·변환 도구 버전/NOTICE·다표본 한국어 품질·구조화 디코딩·GPU/재생 경쟁·패키징은 남은 게이트다. 세부 성능은 raw measurement와 실패 분모를 따른다.
