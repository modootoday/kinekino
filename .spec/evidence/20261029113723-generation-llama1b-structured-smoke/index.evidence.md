---
id: evidence:kinekino/20261029113723
title: Llama1B 고정 profile 구조화 실제 실행
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Llama1B 고정 profile 구조화 실제 실행

2026-10-19. [고정 GGUF 취득·헤더 검사](generation-gguf-header-actual.md) 후 Llama3.2 1B Instruct Q4_K_M으로 기존 실제 미디어 ASR draft·한국어 자막7cue의 교정·카드·하이라이트를 각1회 실행했다. 실제3요청 모두 HTTP200이지만 구조 통과는2/3이며 품질 승인은false다.

## 동일 입력 계약과 실제 결과

`bartowski/Llama-3.2-1B-Instruct-GGUF@067b946cf014b7c697f3654f621d577a3e3afd1c`, 807694464B, SHA `6f85a640a97cf2bf5b8e764087b1e83da0fdb51d7c9fab7d0fece9385611df83`를 선택했다. engine SHA59ee8ff009a06cee8c39890cf6fd90d74f4617e46e88ac9b46a28a11538a9d73, context2048/max256/temperature0,4threads/CPU/offline/prefixcache0/maxseq1이다. seed42·thinkingfalse를 요청했으나 실제 seed/모델별 thinking 적용은 인증하지 않는다.

기존 [Qwen0.6 smoke](generation-qwen06-structured-smoke.md)의5개 고정 입력 파일과 task schema를 유지했다. 기존 prompt에 포함된 `/no_think`도 그대로이며 Llama에 새로 추가한 토큰이 아니다. family별 template와 tokenization이 다른 변수는 남는다. 기존2B18요청이나 constrained9개와 동등 표본·성능 비교로 취급하지 않는다.

| task | HTTP | JSON·strict | 관측 내용 |
|---|---:|---|---|
| correction | 200 | 실패 | completion256·finish=length. uncertain_words 문장 반복 중 잘린 JSON이며 proposed는 첫 문장만 포함. 단어보존은 parse 실패로null |
| cards | 200 | 통과 | headline은 cue1 문장, summary는cue4 문장을 복사했지만 cue_ids는6·7. 구조상 유효 ID여도 내용 근거가 맞지 않음 |
| highlights | 200 | 통과 | cue3~6과7을 두 range로 선택. 영어 일반 설명만 있어 편집 유용성·한국어 표현 품질은 미승인 |

교정의 prompt tokens135/completion256, 카드312/36, 하이라이트304/44가 engine usage에 기록됐다. usage는 engine의 보고값이며 실제 rendered prompt/token ID 캡처나 원본 tokenizer parity가 아니다. 최종 부정확한 교정이나 카드 출력을 자동 적용하지 않는다. 카드의 근거 불일치는 저장된 cue 원문과 출력 직접 대조이며 모든 paraphrase의 사람 평가를 대신하지 않는다.

## 자원과 종료

root 자원 gate는 CPU9.08203125%·free RAM36346114048B에서 통과했다. batch27.29919초, server ready2.2757647초,100ms 표본 peak private2122715136B·working2767605760B, child exit0/timeout=false다. 로드·hash·tokenization·행별 저장 포함이며 Qwen/Gemma와 반복 성능 순위를 만들지 않는다. BelowNormal·단일 소유 server·private Windows Job을 사용했고 종료 후 PID101000 부재와 model/engine SHA를 독립 evaluator에서 재확인했다.

evaluator는 고정 model/input/prompt/schema/control을 확인하고 strict 판정을 재계산했다. 구조 실패를 평가 실행 실패로 숨기지 않으며 실제 의미 품질 승인은0개다. evaluator 자체 신경망 호출0회다. 강제 부모 종료 Job cleanup 시험·다른 작품/언어·원본 framework/변환 수치·원본/embedded template·사람 gold·최종 제품 통합은 미검증이다.

## 재현 증거

- [실행manifest·모든 요청/응답](../../../evidence/ai/model-spikes-2026-10-18/results/text-structured-llama1b-smoke-r1/manifest.json), SHA `5153622b21936b147a2b72d908c9932448673979df5c668320a3cdc5639f5d29`
- [측정](../../../evidence/ai/model-spikes-2026-10-18/results/text-structured-llama1b-smoke-r1.json), SHA `232c798d7b3eb2ebe2e22eb1083c338784fd8bb25231451faa2853644a72de87`
- [독립 평가](../../../evidence/ai/model-spikes-2026-10-18/results/text-structured-llama1b-smoke-r1/smoke-evaluation.json), SHA `83569208f01903d290c78791878786395924458aeaec16a40dabc1ea760b04a7`
- [선보존 실행/평가 build197입력+EXE1](../../../evidence/ai/model-spike-builds/a33b20bbd156f827278a8a00dc181cab2561d769337999d0993f2a4902d87046/build-manifest.json)

새 모델 family 한 묶음이 추가돼 실제 추론·출력 확인22묶음이며 품질 승인0개다. 같은 모델의3요청을3개 모델로 합산하지 않는다. Gemma의 준비 실패는 별도 실패 표에 두고 실제 추론 완료에 합산하지 않는다.
