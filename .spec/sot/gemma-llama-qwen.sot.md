---
id: sot:kinekino/gemma-llama-qwen
title: 경량 Gemma Llama Qwen 후보 검토
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 경량 Gemma Llama Qwen 후보 검토

2026-10-18 사용자 지시로 임베딩과 텍스트 생성 모델의 비교 범위를 확장한다. 아래는 공식 자료 확인 단계이며 새 모델 취득·추론·품질 승인 결과가 아니다.

## 역할별 후보

| 역할 | 후보 | 확인한 사실과 다음 게이트 |
|---|---|---|
| 텍스트·멀티모달 검색 | google/embeddinggemma-2 | 공식 카드의 전체 740M, 텍스트 270M 및 선택 가능한 비전 170M·오디오 300M, 768차원 출력. 텍스트 경로부터 Rust 실행·prefix·pooling·정규화·revision·양자화 parity를 확인한다. |
| 텍스트 생성 | google/gemma-4-E2B-it | 공식 HF 후보 존재 확인. E2B 명칭을 전체 상주 가중치 크기로 간주하지 않는다. 실제 전체 파일·메모리·라이선스·Rust 엔진 지원을 확인한 뒤 취득한다. |
| 텍스트 생성 | meta-llama/Llama-3.2-1B-Instruct | 공식 HF 후보 존재 확인. 한국어·일본어 실제 자막의 의미 보존, 접근 조건·라이선스·양자화 provenance를 별도 검토한다. |
| 텍스트 생성 | Qwen3 및 Qwen3.5 경량 계열 | 기존 0.6B·0.8B·2B Q4 실행 결과는 구조 개선과 의미 오류를 분리한다. 새 revision·정밀도·모델은 기존 실행 성공으로 승인하지 않는다. |

EmbeddingGemma 2는 생성 LLM과 역할이 다르다. 임베딩 차원 축소는 가중치 양자화와 별도 실험이다. 공식 카드의 소비자 기기 적합성 설명을 이 호스트의 실측 메모리·속도로 대신하지 않는다. 기존 MiniLM·E5의 동일 검색 입력과 무관 질의를 유지하고 사람 정답 및 독립 입력을 추가한다.

## 명칭과 비교 기준

2026-10-18 공식 카드를 재확인했다. 사용자 언급에 해당하는 로컬 공개 가중치 후보는 **EmbeddingGemma 2**다. 별도 제품인 [Gemini Embedding 2](https://ai.google.dev/gemini-api/docs/models/gemini-embedding-2)는 Gemini API 모델이며 로컬 HF 모델 후보와 혼합하지 않는다. EmbeddingGemma 2 카드에는 Apache 2.0이 선언되어 있다. 실제 취득 시 고정 revision의 라이선스 파일도 보존한다.

검색 실험은 텍스트 인코더만 적재하는 경로부터 시작하고, 이후 텍스트와 영상 프레임·오디오 간 검색으로 확장한다. 출력 768차원의 기준 결과와 512·256·128차원 절단 후 재정규화를 비교한다. 이는 벡터 저장량 실험이며 모델 상주 메모리 절약을 입증하지 않는다. 가중치 양자화는 별도 조건으로 입력·prefix·pooling·정규화를 고정해 비교한다. 공식 카드의 benchmark는 full precision 결과이므로 양자화 품질에 전용하지 않는다.

Gemma·Llama·Qwen 생성 모델은 자막 교정, 카드 문구, 하이라이트 설명의 동일 입력으로 비교한다. 임베딩 검색 점수와 생성 정확도를 같은 순위로 합치지 않는다. 현재 이 문서의 신규 Gemma·Llama 후보는 조사 단계이며 실행 완료 또는 제품 채택 상태가 아니다.

## 실행 전 조건

1. HF revision·라이선스·원본과 변환 파일 해시 및 변환 도구를 고정한다.
2. Rust 엔진의 실제 아키텍처·토크나이저·양자화 로더 지원을 확인한다. 단순 파일 형식 지원으로 대체하지 않는다.
3. 텍스트 생성은 기존 교정·카드·하이라이트 요청을 유지하고 사실성·누락·환각과 JSON 구조를 별도 평가한다.
4. 자원 게이트를 통과한 단일 worker로 실행한다. 기존 앱을 종료하지 않으며 CPU 기준 실측과 GPU 결과를 혼합하지 않는다.

## 공식 근거

- [EmbeddingGemma 2 HF 카드](https://huggingface.co/google/embeddinggemma-2)
- [Google DeepMind EmbeddingGemma](https://deepmind.google/models/gemma/embeddinggemma/)
- [Gemma 4 E2B HF](https://huggingface.co/google/gemma-4-E2B-it)
- [Llama 3.2 1B Instruct HF](https://huggingface.co/meta-llama/Llama-3.2-1B-Instruct)
- [Qwen3.5 0.8B HF](https://huggingface.co/Qwen/Qwen3.5-0.8B)

실행 현황은 [모델 검증 상태](../../../../verification/ai/model-status.md)를 따른다.
