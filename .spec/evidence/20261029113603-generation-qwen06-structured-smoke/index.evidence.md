---
id: evidence:kinekino/20261029113603
title: Qwen 0.6B 고정 profile 구조화 실제 실행
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Qwen 0.6B 고정 profile 구조화 실제 실행

2026-10-18. 고정 profile을 Rust LocalTextServer에 연결하고 실제 미디어의 기존 ASR draft·한국어 자막7개로 correction·cards·highlights를 각1회 실행했다. 기존 모델 추가 실험이며 품질 승인은false다.

## 계약과 관측

고정 domain profile만 받는 structured-text-selected 명령을 구현했다. 첫 GGUF 선택·알 수 없는 profile fallback은 없다. smoke는 constrained3개, paired는 기존3반복·교차순서18개다. paired18은 이번에 재실행하지 않았다. 입력5파일·모델·engine SHA를 시작 전에 확인했다.

모델은 unsloth/Qwen3-0.6B-GGUF revision50968a4468ef4233ed78cd7c3de230dd1d61a56b, Q4_K_M396705472B다. 모델 SHA ac2d97712095a558e31573f62f466a3f9d93990898b0ec79d7c974c1780d524a, engine SHA59ee8ff009a06cee8c39890cf6fd90d74f4617e46e88ac9b46a28a11538a9d73를 재확인했다. context2048·max256·temperature0·4threads·strict task schema·thinkingfalse다. request seed42를 보냈으나 CPU seed 적용은 증명하지 않았다.

| task | HTTP·구조 | 내용 한계 |
|---|---|---|
| correction | 200·strict pass·단어보존true | 오인식 draft 보존이며 음성 정답 교정은 아니다 |
| cards | 200·strict pass | 어두운 과거, 무고한 피, 바보로 다니다 등 부자연스러운 문장과 반복 |
| highlights | 200·strict pass | 전체7cue 선택, 범위 유용성 미승인 |

배치50.3019265초, 표본 peak private1270865920B·working1578708992B, exit0·timeoutfalse다. 시작을 포함한 소유 프로세스 트리100ms 관측이며 모델 단독 메모리·속도 순위가 아니다. 시작CPU10.707%·빈RAM36071628800B, admission60%미만·빈RAM12GiB이상·BelowNormal·단일worker였고 사용자 앱은 종료하지 않았다. 3건을 기존2B18건·constrained9/9와 동등 표본으로 비교하지 않는다.

독립 evaluator는 고정 prompt·task schema·request control·재parse·strict 판정·단어보존을 재계산해 저장값과 일치를 확인했다. 종료 후 모델/engine SHA와 server PID57232 부재도 확인했다. evaluator 자체 추론은0회다. 강제 부모 종료 Job cleanup·원본 tokenizer/template/rendered prompt parity·사람gold는 미검증이다.

## 보존과 후속

- [실행manifest](../../../evidence/ai/model-spikes-2026-10-18/results/text-structured-qwen06-smoke-r1/manifest.json), SHA8a31fe724074ebf0a84ea2266c98f4932d47b14229aeff9f29f2ceff59b8f844
- [측정](../../../evidence/ai/model-spikes-2026-10-18/results/text-structured-qwen06-smoke-r1.json), SHAf2f268da9e1c982390e96b16bff356c44961fc4946b284e608b184f6396f4408
- [독립 평가·전체 출력](../../../evidence/ai/model-spikes-2026-10-18/results/text-structured-qwen06-smoke-r1/smoke-evaluation.json)
- [실행build](../../../evidence/ai/model-spike-builds/437be00044d9f7ecce8a8dd5803e80ffa897d45eff74449979ffe962c90bcdca/build-manifest.json): compiled inputs177+EXE1
- [평가build](../../../evidence/ai/model-spike-builds/7d43794e37da559eca5bf6ac37c1d50456357e93aacc55f817020454ec6a144b/build-manifest.json): compiled inputs178+EXE1

두 단계53tests·fmt check·clippy all-targets -D warnings·build 통과. [과거inspector](generation-model-profile-inspection.md)의 미연결 상태를 이 후속 실행으로 갱신한다. Gemma/Llama load/run은 미실시다. [생성 준비](../../player/infrastructure/inference/research-followups/lightweight-generation-spike-readiness.md)·[품질 계획](../../../plans/player/ai-quality-gates.md)에 따라 접근·라이선스·변환계보를 확인하고 다른 family를 비교한다.
