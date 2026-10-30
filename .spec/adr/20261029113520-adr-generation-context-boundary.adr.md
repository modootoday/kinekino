---
id: adr:kinekino/20261029113520
title: 생성 context cap은 전체 sequence를 제한한다
status: active
created: 2026-10-29
updated: 2026-10-29
tags: [player]
---
# 생성 context cap은 전체 sequence를 제한한다

이 프로젝트의 runtime cap은 prompt+생성 결과의 전체 sequence에 적용한다. 처리 가능한 마지막 position만 제한하는 의미와 혼합하지 않는다. native 길이·RoPE·weight·정밀도는 유지한다.

[실제 API](../../sots/verification/ai/generation-gemma270-context-boundary-actual.md)는2047+2/2048+1=2049를 보고했다. 가득 찬 text/multimodal prompt는 추가 생성 전 거절하고 explicit truncate=true인 경우만 기존 경로로 공간을 확보한다. embedding의 정확한 최대 입력 길이는 유지한다. 샘플 후보가 마지막 빈 slot을 채울 때 종료하며 max_tokens·EOS·stop 우선순위는 보존한다.

입력/원본API/원본EXE를 동결하고 unit·새EXE 동일3경계 재검증을 분리한다. emitted sequence 초과와 처리 position 초과는 별도다. count 성공·length 문자열·unit만으로 full gate를 통과시키지 않는다. rendered UTF8/IDs·상세 stop/forward를 추가 관측한다. 경계 max2/ignore_eos=true를 품질 max256에 복사하지 않는다. 실험 fork의 정책이며 제품 AI 지원·모델 품질 승인 또는 upstream 전체의 길이 정의를 단정하는 근거가 아니다.

후속 [수정판 실제 API](../../sots/verification/ai/generation-gemma270-context-boundary-fixed.md)에서2047+1=2048,가득 찬2048/초과2049의명시HTTP400을 확인했다. 실제API 범위의 회귀는 해소됐으며 내부trace/full gate는 남는다. 기본원본엔진과 수정판은 명시 revision 및 별도SHA로 구분하고 과거평가를 재귀속하지 않는다.
