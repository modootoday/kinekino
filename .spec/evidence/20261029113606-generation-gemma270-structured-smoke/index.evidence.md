---
id: evidence:kinekino/20261029113606
title: Gemma270 실제 자막 기반 구조화3요청
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Gemma270 실제 자막 기반 구조화3요청

2026-10-19. Qwen0.6/Llama1B와 같은5개 고정 입력·prompt·schema를 새 CPU 엔진에서 실행했다. HTTP3/3완료/strict JSON2/3이지만 **내용 품질은 미달이고 자동 적용하지 않는다.** 경계 arm과 별도로 max256/temp0/seed42/thinking=false를 유지했다.

| task | 구조/종료 | 실제 내용 |
|---|---|---|
| correction | JSON실패·length/256tokens | proposed가 thir, uncertain_words 반복 후 잘림. 파싱 실패로 단어보존 평가는 null |
| cards | strict통과·stop/66tokens | headline/summary는cue1·2 복사인데 cue_ids는1..7전부. 적절한 근거 선별 미달 |
| highlights | strict통과·stop/56tokens |1..7전체를 묶고 이유에 혼합 언어와 원문에 없는 가족 서사를 생성 |

허용ID/JSON을 내용 정확도로 승격하지 않는다. 모델 크기·양자화·변환·엔진 중 원인을 원본 framework/정밀도 대조 없이 단정하지 않는다. 기존2B18요청과 동등 표본이 아니며 rendered IDs/template parity는 미검증이다.

## 고정 조건과 자원

unsloth/gemma-3-270m-it-GGUF revision c90975dbd40c0c7b275fefaae758c3415c906238, SHA256 `b1baabd6b729e4041822220d3e648e00d99cac5df86b10dffb77bcccf0688e39`,253115424B다. 혼합정밀도/계보 한계는 [헤더](generation-gguf-header-actual.md)를 따른다. [e862 엔진](generation-gemma270-cli-build.md)의 runtime2048/prefixcache0/maxseq1/CPU4/paged off/offline으로 실행했다.

preflight CPU10.546875%/free35694100480B, BelowNormal,36.0653998초/peak private1231151104B/working1451917312B, exit0/timeout=false다. usage prompt/completion은189/256·434/66·424/56이다. 단일 correctness 관측으로 모델 간 공정한 반복 성능·노트북 적합성을 승인하지 않는다. server 종료와 엔진/모델 SHA를 재검증했다.

## 보존과 후속

[manifest](../../../evidence/ai/model-spikes-2026-10-18/results/text-structured-gemma270-smoke-cap-r1/manifest.json) SHA256은 `66b1657ad6663c6484edd22e98d6316ed2237829eb8df07644bf477ce6bc8c18`, [측정](../../../evidence/ai/model-spikes-2026-10-18/results/text-structured-gemma270-smoke-cap-r1.json)은 `7b812db2093adda97ebfbf3b50d019ba44b857bcf891048d1b6b873757858f2e`, [평가](../../../evidence/ai/model-spikes-2026-10-18/results/text-structured-gemma270-smoke-cap-r1/smoke-evaluation.json)는 `3beec6437c79cd7579d0bbd81d1785c33091dc89da2ba8cf1e8c0bfd452d0a4b`다. 평가기 실행은 새 추론0회다. 선보존201 compiler inputs/EXE1개의 b36 worker를 사용했다. report3623 snapshots에 이 결과와109count/3경계를 포함했다.

실제 실행·출력 확인은23개 역할/모델 묶음, 품질 승인0개다. 반복/선별 실패/언어 혼합을 실제 실패 예제로 보존하고 원본/변환·다른 작품/언어·독립 사람 검토·검토 저장을 검증한다. [context 수정 재검증](generation-gemma270-context-boundary-actual.md)은 이 품질 게이트와 별개다.
