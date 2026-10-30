---
id: evidence:kinekino/20261029113644
title: 원본 일본어 분절을 고정한 실제 정렬 A/B
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원본 일본어 분절을 고정한 실제 정렬 A/B

2026-10-18. [고정 cohort와 대조 순서](forced-segmentation-cohort.md)에 따라 같은 frozen Rust 어댑터로5회 정렬을 실제 실행했다. 새 기본2건은 기존 head 관측과 일치했고 JA1516의 같은 단위 control도 실제 모델 입력·전체 raw head·logits·최종 결과가 같았다. **원본 nagisa 분절로 바꿔도 JA1516의 길이0 두 건과 JA1634의 세 건이 남았다.** 이를 해결한 것으로 표시하거나 시간을 임의 보정하지 않았다.

총 실제 정렬 호출은 기존12회에서17회, 구조3통과/14실패가 됐다. 이번5회 모두 exit3/needs_review·timeout=false이며 독립 정확도 표본5개가 늘어난 것은 아니다. 품질 승인0이다.

## 통제와 관측

worker는 [최종 r2](../application/forced-segmentation-adapter-2026-10-18.md)의 EXE `b069f93674fdd38ce74e7b343681b16aa501c1775f3dbeb289501a78ab3e6bb2`다. 보존 EXE와 byte 일치하는 사본을 `runtime/ai-spike/bin/segmentation-r2`에서 실행했다. 모델 revision `c7cbfc2048c462b0d63a45797104fc9db3ad62b7`, 원본 audio/transcript, CPU/F32/batch1/threads4를 유지했다. root admission은 고정 build manifest·EXE·cohort·audio/text pair·선택 words SHA·정확한 flags·새 label/output을 확인했다.

| label의 segmentation-ab- 이후 | 단위 | 길이0 | wall초 | 표본 peak private bytes |
|---|---:|---:|---:|---:|
| ja1516-default-r1 | 28 | 2 | 21.8948325 | 4091359232 |
| ja1516-control-r1 | 28 | 2 | 13.3877201 | 4090753024 |
| ja1516-original-r1 | 28 | 2 | 13.5396685 | 4091187200 |
| ja1634-default-r1 | 31 | 3 | 22.6279458 | 4028891136 |
| ja1634-original-r1 | 32 | 3 | 13.7348208 | 4027023360 |

순차 단일 worker·BelowNormal·offline·600초 root-child timeout/kill·100ms owned-tree 표본이며 Windows Job60초 참조 함수 실행기와 구별한다. 로드/해시/관측 복사/직렬화를 포함하고 OS cache를 비우지 않았다. 다른 사용자 앱·VM·DPI를 변경하지 않았으며 performance_comparison_eligible=false다. 최초/후속 wall 차이를 분절 방식의 속도 효과로 해석하지 않는다. working set·preflight/resource 전후·실제 args도 각 측정 JSON에 있다.

## 먼저 통과한 대조

[JA1516 기본 감사](../../../evidence/ai/model-spikes-2026-10-18/results/segmentation-ab-ja1516-default-r1-historical-evaluation.json)와 [JA1634 기본 감사](../../../evidence/ai/model-spikes-2026-10-18/results/segmentation-ab-ja1634-default-r1-historical-evaluation.json)는 기존 frozen head 출력의 모든 raw/head/classifier row·public item·prompt/tokenizer 필드를 비교해 일치했다. 새 actual model-input IDs는 추가 관측이며 과거 없던 필드를 소급 생성하지 않았다.

[JA1516 같은 단위 control](../../../evidence/ai/model-spikes-2026-10-18/results/segmentation-ab-ja1516-control-r1-comparison.json)은 같은 EXE/compiled inputs/model/audio/text/CPU4를 확인하고 generated_prompt·segmentation_words·raw_head 전체·public_api_result·tokenizer가 모두 같았다. provenance/file metadata와 timing만 대조 범위 밖이다. JA1634의 같은 단위 control은 비추론 검증만 있고 추가 neural control을 실행한 것은 아니다.

## 원본 분절의 결과와 잔여 원인

[JA1516 A/B](../../../evidence/ai/model-spikes-2026-10-18/results/segmentation-ab-ja1516-original-r1-comparison.json)와 [JA1634 A/B](../../../evidence/ai/model-spikes-2026-10-18/results/segmentation-ab-ja1634-original-r1-comparison.json)는 controls_equal=true, tokenizer 동일, 단위·prompt·raw_head·public 결과는 달랐다. 전체 출력이 같다는 의미가 아니며 단위가 달라 index별 전체 시간을 단순 zip하지 않았다.

| 원본 분절의 남은 단위 | raw ms 시작→끝 | 최종 초 시작→끝 | 관측 |
|---|---|---|---|
| JA1516 index9 `を` | 7840→7840 | 7.84→7.84 | raw equal |
| JA1516 index10 `もっ` | 8000→8000 | 8→8 | raw equal |
| JA1634 index2 `は` | 2240→2240 | 2.24→2.24 | raw equal |
| JA1634 index5 `0` | 2720→2800 | 2.8→2.8 | repair collapse |
| JA1634 index7 `0` | 3040→3040 | 3.04→3.04 | raw equal |

각 단위·raw/fixed/final 값은 해당 원문 prefix가 같은 위치에서 실제 관측했다. 기존 raw equal4/repair collapse1 분류가 이 두 실험군에서도 유지됐다. 분절 차이 자체는 확인됐지만 이 표본의 기존0구간을 해소하지 못했다. 분절을 모든 오류 원인에서 배제하거나 모델/정밀도 자체의 결함을 확정한 결과는 아니다. 공식 encoder/kernel·정밀도 대조와 사람 word 경계 정답이 남는다.

## 평가기 호환 문제 수정과 독립 감사

기존 평가기는 과거 API의 상위 raw_class_ids/pre_repair_ms를 unavailable/null로 가정했다. 새 어댑터는 실제 raw head에서 이 두 필드를 제공하므로 최초5개 평가는 `unavailable raw observations must remain null`에서 종료했다. 해당 stderr와 실패 당시 build를 보존했고 원시 결과·모델·시간을 바꾸거나 재추론하지 않았다.

평가기만 profile별로 수정했다. 기존 label의 null 계약은 유지하며, 새 고정 EXE의 segmentation label은 상위 class 배열과 raw_head.selected_class_ids, pre_repair_ms와 actual raw ms가 정확히 일치해야 한다. unsupported raw_timestamp_ticks/fixed_ticks는 계속null이어야 한다. 불일치/legacy 경계 거부 회귀를 추가해52tests·fmt check·clippy all-targets -D warnings·locked offline build를 통과했다.

수정한 독립 평가5건은 모두 inference_completed/source_and_text_hashes_match/raw_seconds_bits_preserved/actual_top_level_raw_mapping_verified=true였다. 단위28/28/28/31/32와 위0구간만 재확인했고 원음 범위·원시 시간 bit를 검증했다. 평가 결과는 같은 label의 `-evaluation.json`, 최초 실패 로그는 `.evaluation.stderr`, 수정 후 로그는 `.evaluation-r2.stderr`로 구별했다. evaluator 성공을 alignment 구조 통과로 바꾸지 않는다.

## 재현 근거와 다음 작업

실제 inference orchestrator는 `3f78c25ec412f0bbd69c4f2c5bbf10ea5a0c4ec672441aadb56f4576006fc4ea`, 컴파일 입력175개·EXE1개를 [선보존](../../../evidence/ai/model-spike-builds/3f78c25ec412f0bbd69c4f2c5bbf10ea5a0c4ec672441aadb56f4576006fc4ea/build-manifest.json)했다. 수정 evaluator는 `a17166f6242e7a15eb868750454039efe0bd7c5280bd19cae57d80195f3888a9`의 별도 [175입력/EXE1 묶음](../../../evidence/ai/model-spike-builds/a17166f6242e7a15eb868750454039efe0bd7c5280bd19cae57d80195f3888a9/build-manifest.json)이며 새 neural 실행0회다.

SDK helper `compare-runs.exe` SHA `260d177884deaed24c5bb1959b6497695e2936e3fdae0eed979a67f0bac4178c`로 control/A/B를 비교했다. 중앙 검사는 pinned worker manifest와128개 보존 입력·4개 diff patch·primary/helper EXE3개의 bytes/hash를 확인했다. 원시 출력·전체 logits·expanded model IDs·측정·preflight·독립 평가·실패 로그를 공통 보고서에 보존했다. 전체 native/transitive 재현 빌드나 품질 승인과 구별한다.

공통 report는1568 snapshots를 기록했다. [중앙 검사 보고서](../../../evidence/verification/segmentation-build-integrity-2026-10-18-r1/build-evidence.json) SHA256은 `77587dccd45c1cd01c61d615c8df32c074d2637169dc9b1c896a9a4419a8219a`다. 사용한 [검사기 EXE](../../../evidence/verification/segmentation-build-integrity-2026-10-18-r1/player-spec-maintenance.exe) SHA256 `5665a152167e7c7117c45640e7c472cb3c07779f0d1dc46240e7bdeb728aa8d4`와 [검사 시점 소스](../../../evidence/verification/segmentation-build-integrity-2026-10-18-r1/source/src/main.rs)·Cargo 파일도 보존했다. 소스 복사는 컴파일 attestation과 구별한다. 해당 검사에서 문서318개·정본 링크 오류0·보존 upstream 오류2를 확인했고 담당 검사기7tests·fmt·clippy·release build가 통과했다.

다음 우선순위는 공식 encoder/kernel 또는 정밀도 경로의 controlled 대조, 독립 word 경계 정답·검토 저장 계약이다. 분절 A/B를 같은 입력으로 반복하거나 임의 merge/zero 보정을 기본값으로 넣지 않는다. [잔여 품질 게이트](../../../plans/player/ai-quality-gates.md)를 따른다.
