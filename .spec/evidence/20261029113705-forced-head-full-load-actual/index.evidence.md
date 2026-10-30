---
id: evidence:kinekino/20261029113705
title: 공식 ForcedAligner 전체 모델 strict 로드 실제 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공식 ForcedAligner 전체 모델 strict 로드 실제 검증

2026-10-18. [공식 encoder114공통stage 대조](forced-encoder-stage-actual.md) 다음 게이트로 원본 전체 Qwen3ASRForConditionalGeneration을 실제 구성하고 checkpoint 전체를 strict=True로 로드했다. **전체 로드는 통과했으며 head forward·logits·argmax 비교는 아직0회**다. 모델 품질·zero-duration 해결·제품 SDK 지원 승인은 아니다.

## 고정과 입력 경계

[원본 head 참조 준비](../../player/infrastructure/inference/research-followups/forced-head-pytorch-readiness.md)의 Qwen source7c6daf77a2421100f5fb066495372c00129d39ff와 기존 사설 import 환경을 사용했다. 모델 revision은c7cbfc2048c462b0d63a45797104fc9db3ad62b7, weights1835544544B/SHA47831d0e82f96b20e9034dba01a075ee06436654719f6a68289e49f1b65ce0e7, config SHAd616c65d46c4b90bdc651b0a0963ea932732241140f337f9bb6b0335a9c8ef09다.

정규 고정 weight 파일을 열어 SHA를 검사하고 read sharing만 허용하는 Windows handle을 유지해 작업 중 다른 쓰기·삭제를 허용하지 않았다. 원본 class·safe_open·공개 load_state_dict를 Rust PyO3에서 직접 호출했으며 전체 wrapper에는 prefix를 제거하지 않았다. 원본 Python source·환경 파일을 수정하거나 wrapper/stub/eval을 작성하지 않았다.

domain은 classify_num5000·vocab152064·hidden1024·text28layer·tie_word_embeddings=false를 검사한다. 실제 checkpoint missing/unexpected key가 하나라도 있으면 실패한다. 누락 weight를 random 초기값으로 실행하거나 head와 embedding을 묶지 않는다. 이번 실행에는 아직 mel·prompt·IDs를 입력하지 않았다.

## 실제 관측

| 항목 | 결과 |
|---|---|
| checkpoint tensor | 708개 strict 로드, missing/unexpected0 |
| 저장 정밀도 | 708개 모두 torch.bfloat16 |
| 실제 실행 parameter | 708개·917728896원소 모두 CPU F32·finite |
| embedding | 152064×1024 |
| classifier | 5000×1024, embedding과 data_ptr 다름 |
| attention | audio24+text28=52층 모두 실제 config eager 확인 |
| 모델 상태 | eval, 수치thread4·interop1 |
| backend | MKLDNN enabled=true, Torch build 설정 원문 보존 |
| head forward | 0회, 이 단계는 load·검사만 수행 |

모델명0.6B를 전체 상주 parameter 수로 해석하지 않는다. 이 checkpoint 전체는917728896원소이고 encoder만316427904원소였던 이전 단계와 범위가 다르다. BF16 파일 크기·F32 parameter bytes·process peak도 별도다.

## 자원·종료·기존 경로 회귀

사전CPU10.2713%·빈RAM36674449408B·ready=true에서 단일 worker를 admission했다. 사설 kill-on-close Job·BelowNormal·300초 deadline·1MiB/stream을 적용했고 사용자 ChatGPT/Terminal 및 다른 앱은 종료하지 않았다.

전체107.9043835초, 모델 구성/strict load/parameter 검사62.8133673초, exit0·timeoutfalse였다.20ms root-child 표본5234개·query failure0에서 private7435804672B·working6248787968B를 관측했다. 환경/source 재검증·import·전체 F32 초기화·BF16 state·복사·finite 검사·출력을 포함한다. 추론 peak나 제품 native backend 메모리·노트북 전력 측정으로 일반화하지 않는다. 원본 nagisa의 SyntaxWarning은 숨기지 않고 stream에 보존했다.

같은 최종 EXE의 기본 JER6개 회귀는 exit0·3.6508436초였고 stdout SHA8b516b1ce73187162497b011789a0ee9e4b05b814b06485e7dc766e541b80fba로 과거 출력과 완전히 같다. 전체 dscore CLI나 모델 품질 승인으로 확대하지 않는다.

## 재현 증거와 후속

- [전체 load metadata·checkpoint key/shape·parameter/attention 목록](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/head-original-load-r1/head-model-loaded.json), SHA2ca85a2606843688332c18800d873cf7a7b5e9e8aec822936ce222ab6d7760e3
- [감독·메모리·종료 관측](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/head-original-load-r1/observation.json), [원본 stream](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/head-original-load-r1/worker.stderr)
- [실행 EXE288e52ed7fe54a16ada092652604ac6670705536a9844233166950d6fea56a96의 선보존65입력+EXE1](../../../evidence/player/jer-function-reference-builds/288e52ed7fe54a16ada092652604ac6670705536a9844233166950d6fea56a96/build-manifest.json), manifest SHA3115365f9811c8971cfeb0c80130ee67d84e5d38b9e7888de1a3fccedbc554ca
- [기존 기본 경로 회귀](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/head-load-jer-regression-r1/observation.json)·[전체 출력](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/head-load-jer-regression-r1/worker.stdout)

참조 도구5tests·fmt·strict clippy·locked offline build를 통과했다. classifier 축/tied flag를 바꾼 config를 거부하는 비추론 회귀를 추가했다. root 모델 도구58tests·fmt·clippy·build도 통과했다. 이 실행은 모델 load만1회이며 alignment17회·encoder6회에 합산하지 않는다.

다음은 SDK 세션이 준비하는 고정 same-mel cohort의 words·prompt·expanded IDs·mask·T/L·timestamp 위치를 검증하고 KO 한 건을 원본 thinker(**inputs).logits로 실행하는 것이다. feature_attention_mask를 명시하고 audio pad 재확장·p-1 shift·generate를 하지 않는다. KO 계약/수치 게이트 뒤 JP2개로 확대한다. full logits의 고정atol/rtol과 raw argmax exact는 별도 판정이며 encoder 통과·load 통과를 head 일치로 대체하지 않는다.
