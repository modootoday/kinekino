---
id: sot:kinekino/embeddinggemma-2
title: EmbeddingGemma 2 후보와 검증 기준
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# EmbeddingGemma 2 후보와 검증 기준

2026-10-19 공식 모델 카드 재확인. `google/embeddinggemma-2`는 공개된 후보이며 생성 LLM과 구분한다. 아래 수치는 공급자 명세이며 이 호스트의 실측 결과가 아니다.

| 구분 | 공식 명세 | 프로젝트 용도 |
| --- | --- | --- |
| EmbeddingGemma 2 | 전체 740M, 텍스트 270M, 영상 인코더 170M, 오디오 인코더 300M | 자막·장면·오디오 의미 검색 후보 |
| 선택 로딩 | 텍스트 270M, 텍스트+이미지 440M, 텍스트+오디오 570M | 필요한 인코더만 메모리에 적재 |
| 출력 | 768차원, 512·256·128차원 축소 지원 | 인덱스 저장량과 검색 품질 비교 |
| 문맥 | 8,192 토큰 | 장면·자막 구간별 입력 제한 |
| 라이선스 | Apache 2.0 표기 | 실제 채택 revision의 LICENSE 재검사 |
| 생성 LLM | Gemma·Llama·Qwen 경량 모델군 | 자막 교정·요약·카드뉴스 문안·구조화 출력 별도 평가 |

## 기존 실측과의 연결

현재 Rust 스파이크는 `onnx-community/embeddinggemma-2-ONNX@daa72c51243991dfcaf9f9137d2c573d8f7790c0`를 고정한다. [텍스트 FP32 실측](../../../../verification/ai/embeddinggemma2-fp32.md)과 [양자화 실측](../../../../verification/ai/embeddinggemma2-quantized.md)은 이미 있으며, 이 후보를 새 미실행 모델로 중복 집계하지 않는다. 변환 체크포인트의 텍스트 실측을 Google 원본 구현과의 수치 일치나 멀티모달 품질 통과로 확대하지 않는다.

## 채택 조건

Rust 어댑터에서 실제 지원하는 연산·토크나이저·풀링·전처리를 먼저 확인한다. 모델 revision, 각 아티팩트 SHA256, 라이선스, 정밀도 및 양자화 방식은 실행 전에 고정한다. 공식 예시가 Python이라는 이유로 프로젝트 개발 언어 규칙을 완화하지 않는다.

텍스트 FP32 기준선과 경량 양자화의 검색 정확도, 무관 질의 거절, 메모리 최고치, 지연을 동일한 실제 미디어 입력으로 비교한다. 텍스트 전용 성공을 이미지·영상·오디오 검증 완료로 확대하지 않는다. 기존 스파이크의 모델 ID·revision과 일치하는지 확인하기 전에는 이전 결과를 이 후보의 증거로 귀속하지 않는다.

임베딩 유사도만으로 오프닝·엔딩 경계, 화자 분리, 음성 인식, 피사체 마스크 또는 3D 에셋 생성의 품질을 보장하지 않는다. 각 기능의 별도 검증 기준을 유지한다. 이 문서 작성에 따른 새 다운로드·추론은 수행하지 않았다.

## 근거

- [Google 공식 Hugging Face 모델 카드](https://huggingface.co/google/embeddinggemma-2)
- [Google 공식 추론 안내](https://ai.google.dev/gemma/docs/embeddinggemma/inference-embeddinggemma-with-sentence-transformers)
