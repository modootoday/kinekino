---
id: sot:kinekino/scope
title: 확정 요구와 해석
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 확정 요구와 해석

2026-10-21 상태 정정: [초기 추론 엔진 정본](../../infrastructure/inference/sot-014-inference-engine-research.md)의 Qwen3-0.6B Q4_K_M CPU 두 엔진 표본은 해당 시점의 실행이다. 이후 제한된 ASR/OCR·비전 등의 실행과 출력이 [모델 상태](../../../verification/ai/model-status.md)에 보존됐으므로 이를 전부 추론 전으로 읽지 않는다. 직접 대조한 asr-ko-base/ocr-ko-tight 원 측정 JSON은 exit0이고 실제 stdout도 존재한다. 이는 전체 기능·언어·held-out 품질·실행 중 dtype·제품 SDK 지원의 완료가 아니다. 과거 EXE/source 보존 한계는 [별도 감사](../../../verification/ai/provenance-audit.md)를 따른다.

[cached VAD 정상/취소](../../../verification/ai/cached-vad-resource-adapter-2026-10-21.md)와 [release 입력 actual23](../../../verification/ai/vad-release-profile-r2-cpu-2026-10-21.md)도 범위가 다르다. 같은 모델의 재실행/순수 계약 시험을 새 역할이나 품질 승인으로 더하지 않는다. ASR·화자분리·OP/ED·영상·에셋·검색·문구의 전체 제품 요구와 품질 인수는 유지한다.

## 최초 요구와 설계 맥락

사용자는 AI 모델을 Hugging Face 기반으로 지정했다. 모델 검색/메타데이터/버전·아티팩트 유통의 기준을 HF Hub로 한다. ‘HF 기반’이 클라우드 추론이나 사용자 미디어 업로드를 뜻하지는 않는다. 설계 기본은 Windows에서 필요한 모델만 내려받아 로컬 실행하는 방식이다. UI·도메인·유스케이스는 Rust로 유지하고, 추론 엔진은 별도 인프라 어댑터로 둔다.

음성인식으로 자막을 생성하는 ASR, 텍스트를 교정/요약하는 sLM, 영상 글자를 읽는 OCR, 피사체 분리/추적, 깊이 추정은 서로 다른 작업이다. 텍스트 전용 sLM에 오디오를 직접 넣어 정확한 자막·시간을 얻는다고 설계하지 않는다. 작은 모델이라는 이름만으로 메모리/속도/한국어·일본어 품질을 보장하지 않는다.

최초 설계 작성 작업에서는 모델 다운로드·실행·벤치마크·ONNX 변환을 수행하지 않았다. 그 당시의 미실행 서술을 위 후속 실제 관측에 소급 적용하지 않는다. 실험 실행·비활성 source 계약과 현재 제품의 지원 등록·runtime/model admission은 별개다.
