---
id: evidence:kinekino/20261029113602
title: Gemma270 context 경계 입력 후보 준비
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Gemma270 context 경계 입력 후보 준비

2026-10-19. Rust 도구로 고정 HF Gemma270 profile의 경계 시험 후보165개를 생성하고 컴파일 입력·실행 파일과 함께 보존했다. **실제 tokenizer count 호출·모델 forward는0회다.** 반복 문자 수를 토큰 수로 취급하지 않으며 실제 chat template 적용 후 정확한2047/2048/2049 후보를 선택해야 한다. [실제 호출 계약 조사](../../player/infrastructure/inference/research-followups/gemma270-context-boundary-runner.md)와 [엔진 준비 상태](generation-gemma270-compiler-compatibility.md)를 따른다.

## 고정 입력과 arm

prefix는 영문 패턴 계속하기 지시이고 공백+x를1900..2064회 반복한165개 UTF8 문자열이다. 각 후보의 user bytes SHA와 count/generation 요청 JSON을 보존했다. 이 범위에서 원하는 정확한 token count를 찾지 못하면 fixture 실패로 남기며 길이를 근사하거나 request를 자동 truncation하지 않는다. tokenizer-only count는165회 이내로 제한하고 단조성이나 반복 단위당1token을 가정하지 않는다.

| arm | 계약 |
|---|---|
| count | POST /v1/messages/count_tokens, model default, 한 user message, enable_thinking=false, truncate_sequence=false |
| boundary generation | POST /v1/chat/completions, 동일 message와 thinking/truncation, max_tokens2·temperature0·seed42·stream=false·ignore_eos=true |
| 품질 smoke | 기존3task와 max_tokens256을 유지. boundary의 EOS override나2token budget을 복사하지 않음 |

고정 원본 source에서 OpenAI ChatCompletionRequest의 ignore_eos는 bool이며 chat_completion이 SamplingParams로 전달하고 Sequence가 EOS 분기를 판단한다. 따라서 경계 arm에만 true를 명시해 EOS 조기 종료가 모델 길이 정지 검증을 가리지 않도록 한다. 이는 source 기반 실행 조건이며 실제 EOS/stop/생성 횟수를 관측한 결과가 아니다. 명시 stop token/string 등 다른 정지 원인도 실제 응답·내부 trace에서 구분해야 한다.

2049 prompt는 admission 거절/forward0,2048 prompt는 추가 생성 없이 거절 또는0생성,2047 prompt/max2는 전체 sequence<=2048을 요구한다. 현재 source의 is_done-before-add_token 가설은 실제 결과 전 확정하지 않는다. count 성공을 생성 admission 통과로 해석하지 않는다. rendered 원문 UTF8·U32LE IDs 및 detailed stop/forward capture도 별도로 필요하다.

## 보존과 도구 검사

[입력 manifest](../../../evidence/ai/generation-boundary-fixtures/gemma270-boundary-candidates-r1/manifest.json) SHA256은 `79c4c2f5a17460b19a9fbdc8fc262f47a4c97fa5d2f2d45fb0daf52914803a3a`다. candidates.json1465187B의 SHA256은 `9b9bf121a175c522a61beaf506a6170ea8d324c35da7d3fcf55c2b32b2878efb`다. observed_input_tokens는 모든 행에서 null이며 native32768/runtime2048은 검증할 계약으로 기록했다.

`prepare-generation-boundary gemma270-boundary-candidates-r1`은 파일 생성만 수행하며 모델 취득·로드·추론을 하지 않는다. 도구는 fmt check·61tests·strict clippy·locked offline build를 통과했다. [EXE와 컴파일 입력199개](../../../evidence/ai/model-spike-builds/1bd148d1bb9ad79cfcd62c953045e65ebe9e51a48d51be08e8bbd9faeb4e6221/build-manifest.json)를 실행 전에 선보존했다. 이 준비를 실제 모델 실행 수·품질 승인 수에 더하지 않는다. 후속 count/forward 관측은 준비 JSON을 덮어쓰지 않고 새 artifact에 저장한다.
