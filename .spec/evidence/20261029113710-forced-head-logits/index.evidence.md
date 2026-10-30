---
id: evidence:kinekino/20261029113710
title: 강제 정렬 timestamp classifier 전체 logits 관측
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 강제 정렬 timestamp classifier 전체 logits 관측

2026-10-18. 기존 길이0 실패 입력 JA1634·JA1516을 관측 도구0.2로 각각 실제 실행했다. **이전719의 선택 class·raw/fixed 시간·최종 item/f64 bit가 두 입력 모두 동일했다.** 신규 신경망 실행2회이며 exit3/needs_review를 그대로 유지했다. 시간 보정이나 분절을 고쳐 성공으로 만들지 않았다. 총 실제 정렬은12회, 구조3통과/9실패, 품질 승인0이다. 중복 입력 관측12회이지 독립 정확도 표본12개가 아니다.

## 관측 경계와 검증

후속 [고정 분절 A/B5회](forced-segmentation-ab-actual.md)에서 새default2건은 이 문서의 전체head관측과일치했고 원본분절에서도기존zero가남았다. 현재실제정렬총17회·구조3통과/14실패이며 위12회는이head단계시점의기록이다.

기존 [String/Regex 입력 대조](forced-tokenizer-contract.md)는 실제3prompt 전체 token ID가 같아 신경망 A/B를 생략했다. 이번에는 다른 후보를 섞지 않고 동일 tokenizer·분절·모델·F32·CPU4thread·batch1을 유지했다. 기존 forward/argmax/parse/최종 결과 구성 이후의 관측 분기에 timestamp 위치의 전체 logits 복사를 추가했다. non-timestamp 위치의 전체 tensor를 저장한 것은 아니다.

관측 전용 fork의 numerical body에서 관측 분기를 제거한 본문이 baseline과 동일하다는 회귀가 통과했다. 실제 실행 후 root 별도 정렬 방식으로 전체 row의 float32 little-endian SHA256·argmax·runner-up·margin·tie 수를 재계산했다. JA1634는62×5000, JA1516은56×5000으로 총118개 row가 모두 통과했다. 선택 logit·runner-up·차이의 float32 bit와 원시 row를 함께 보존했다. 실제 final bit 동등성까지 확인했지만 공식 Python processor/mel/kernel 수치 parity를 입증한 것은 아니다.

## 길이0 구간의 점수 차이

margin은 실제 선택 class logit에서 가장 높은 다른 class logit을 뺀 값이다. 보정된 확률이나 신뢰도가 아니다. 아래10개 timestamp에서 maximum_tie_count는 모두1이었다.

| 입력·word index | 원문 | raw ms 시작→끝 | fixed ms 시작→끝 | 시작 margin | 끝 margin |
|---|---|---|---|---:|---:|
| JA1634·2 | は | 2240→2240 | 2240→2240 | 3.454912 | 2.226093 |
| JA1634·5 | 0 | 2720→2800 | 2800→2800 | 0.050071 | 0.086460 |
| JA1634·7 | 0 | 3040→3040 | 3040→3040 | 0.117729 | 0.985298 |
| JA1516·9 | を | 7840→7840 | 7840→7840 | 0.223484 | 3.277039 |
| JA1516·10 | もっ | 8000→8000 | 8000→8000 | 0.089039 | 2.700279 |

이 관측에서 정확한 logit 동점은 없었다. margin 크기는 서로 다르고 일부 끝 경계는 시작 경계보다 차이가 컸다. 따라서 모든 길이0을 같은 동점 선택 문제로 묶지 않는다. raw equal4/repair collapse1의 기존 귀속은 유지된다. BF16·다른 kernel·공식 분절과 비교하지 않았으므로 정밀도 민감성이나 모델 자체 원인을 확정하지 않는다. runner-up을 골라 시간을 바꾸거나 임의 epsilon을 더하지 않는다.

## 실행과 증거

| 항목 | JA1634 | JA1516 |
|---|---:|---:|
| label | forced-head-ja1634-r1 | forced-head-ja1516-r1 |
| admission CPU % | 18.3415 | 29.1992 |
| admission free RAM bytes | 36671770624 | 34552541184 |
| wall seconds | 26.5947431 | 16.8052651 |
| sampled peak private bytes | 4036915200 | 4090966016 |
| worker exit / timeout | 3 / false | 3 / false |

순차 단일 worker·BelowNormal·오프라인·100ms 표본이며 다른 사용자 앱을 종료하거나 캐시를 비우지 않았다. 출력 직렬화 및 관측 복사도 wall에 포함되어 성능 비교 eligible=false다. 첫 실행 후 host CPU 관측99.81%는 admission 표본18.34%와 별개다. admission은 지속 CPU hard cap이 아니다.

- worker EXE: `c7f22740bcec064413b98021a407246ed94f1139d98ddf95446795518caae460`.
- engine patch: `db134902cdeebfd3ea3ecf47574a50ed9c574e87dfd13b038b3a56aeaae20dbb`.
- [선보존 worker bundle](../../../evidence/player/forced-aligner-observation-builds/0.2.0-head-r1/build-manifest.json)은 adapter·관측 fork·테스트·컴파일 포함 사전 자산 등74개 입력을 빌드 시 해시와 현재 바이트로 대조했다. 실제 보고서의 compiled-input manifest SHA256도 동결 helper와 일치했다. Rust/native 모든 전이 의존성의 재현 빌드 증명은 아니다.
- worker compiled-input manifest SHA256: `cbfcfe522de549cce8a821d1ed69c83a67ee6bb347217af82b0ac05e834281a3`.
- root orchestrator: `180610b73b6816ca39141bd97f9328acd29ab7e616716b70fb9011d7d41213af`, 컴파일 입력98파일과 EXE1개를 합친99개 bundle을 선보존했다.
- 모델 revision·가중치·원음/원문은 [기존 정렬 정본](forced-alignment.md)과 같다. 원본719 bundle과 기존 출력은 덮어쓰지 않았다.
- 원시 파일은 `runtime/ai-spike/results/{label}-output.json`, stdout/stderr/measurement/preflight, 독립 검증은 `{label}-head-evaluation.json`이다. 각 검증은 observed/baseline/measurement/build manifest 해시를 기록했다. 평가기는 새 추론0회다.

worker snapshot README의 이전 버전 설명에 남은 “No logits” 문장은0.1 당시 범위다.0.2의 timestamp 전체 row 관측은 이 문서와 실제 출력이 정본이며 동결 파일을 소급 수정하지 않는다. working README는 버전별 범위를 명시하도록 정정했다.

## 검사와 다음 게이트

관측 도구 fmt check·test17개(include-ignored 포함)·clippy all-targets -D warnings·locked offline release build를 통과했다. MSVC LNK4098 native defaultlib 경고는 test/build에 남아 있으며 제거 완료로 표시하지 않는다. root 담당 도구 fmt/test37/clippy/locked offline build도 통과했다.

문서 검사는 새 동결 upstream README의 `SERVE_COMPATIBILITY_MATRIX.md` 누락으로 최초 실패했다. 기존719 원문과 SHA256 `6bd8c0276062851df9cd93ff99a6b8031a69530f53e2857719b20f57f402bc03`이 같음을 확인해 검사기의 정확한 경로·해시 분류에 새 동결 경로만 추가했다. 다른 경로·다른 해시·다른 대상은 계속 오류다. 이 회귀를 포함한 spec-maintenance test5개와 fmt/clippy/release build가 통과했다. [최초 검사 결과](../../../evidence/ai/alignment-head-integrity-2026-10-18/links-before-upstream-classification.json)를 보존하며 누락 문서를 만들거나 동결 README를 고치지 않았다. 보존 upstream 결함은 이제2곳이며 정상 링크로 계산하지 않는다.

후속 [mel 수학 계약 대조](forced-mel-math-contract.md)는 실제3발화의 전체 행렬을 독립 Rust DFT와 비교해 허용치 안이었다. 공식 PyTorch/kernel·encoder 수치 parity는 별개다. 다음은 공식 분절·실제 framework/encoder 대조, 사람 word 경계·cue 검토 저장, 필요 시 같은 입력 정밀도 대조다. 이번 관측은 오류 단계와 점수 차이를 좁혔으며 모델 오류를 해결하거나 제품 자동 정렬을 승인한 결과가 아니다. [잔여 게이트](../../../plans/player/ai-quality-gates.md)를 따른다.
