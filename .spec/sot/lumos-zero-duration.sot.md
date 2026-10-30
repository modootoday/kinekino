---
id: sot:kinekino/lumos-zero-duration
title: "lumos 일본어 zero-duration: 관측과 원인 후보의 범위"
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# lumos 일본어 zero-duration: 관측과 원인 후보의 범위

2026-10-18 별도 원인 연구. root가 생성한 KO/JA1516/JA1634 출력 JSON과 고정 source/golden·공식 Qwen processor를 읽었다. 모델/사전 다운로드·빌드·추론·코드/기존 증거 변경은 하지 않았다. root JA1578 실행에는 관여하지 않았으며 아래 결론에 미포함이다. source 읽기만으로 개별 발화의 실제 경계 정확도를 승인하지 않는다.

## 관측 정본과 상태

engine은 lumos `7984967e8701cc90bb17d06453901036e36ca578`, 모델은 공식 ForcedAligner `c7cbfc2048c462b0d63a45797104fc9db3ad62b7`, CPU F32, known original transcript 입력이다. 다음은 **root 실행 결과를 읽은 관측**이며 이 연구 세션이 재실행/재계산한 결과가 아니다.

| root 출력 | 구조 관측 | 품질 상태 |
|---|---|---|
| [KO r1](../../../../../../runtime/ai-spike/results/forced-align-ko-r1-output.json) |19items, issues 없음, `structure_valid_quality_unverified` | 사람 경계 정답 미검증. 단어 정확도 승인 아님 |
| [JA1516 r1](../../../../../../runtime/ai-spike/results/forced-align-ja1516-r1-output.json) |28items, zero2개, `needs_review_invalid_alignment_structure` | needs_review 유지 |
| [JA1634 r1](../../../../../../runtime/ai-spike/results/forced-align-ja1634-r1-output.json) |31items, zero3개, 같은 needs_review | needs_review 유지 |

문제 index는0부터다. 아래 초 단위는 public API가 반환한 repair 후 값이며 raw class/tick이 아니다.

| 입력 | index | text | start→end(seconds) | 주변 관측 |
|---|---:|---|---|---|
| JA1516 |9|を|7.84→7.84|앞의 礼節은7.36→7.76 |
| JA1516 |10|もっ|8.00→8.00|뒤의 て는8.00→8.16 |
| JA1634 |2|は|2.24→2.24|앞의 に는2.08→2.24 |
| JA1634 |5|0|2.80→2.80|숫자 prompt가7/0/0/0으로 분리됨 |
| JA1634 |7|0|3.04→3.04|바로 앞의0은2.80→3.04 |

raw_class_ids/raw_timestamp_ticks/pre_repair_ms/fixed_ticks는 JSON에서 null이다. observation에 generated_prompt·segmentation_words·public_api_result는 있다. output의 제한 설명은 원시 head 미노출, repaired seconds 역산 금지, 추가 clamp/repair 없음, 공식 tokenizer/mel sequence parity 미검증을 명시한다. root의 exit3/needs_review 보존은 유효한 구조 실패 기록이며 성공으로 바꾸지 않는다. µs 필드도 public seconds의 단위 변환이지 추가 시간 관측이 아니다.

## 고정 source의 실제 처리 순서

소스 정본:

- [lumos model.rs](https://github.com/lumosimmo/qwen3-asr-rs/blob/7984967e8701cc90bb17d06453901036e36ca578/qwen3_asr/src/forced_aligner/model.rs), 특히 timestamp position 선택·`class_id * timestamp_segment_time_ms`·`parse_timestamp`·`round3` 경로.
- [lumos processor.rs](https://github.com/lumosimmo/qwen3-asr-rs/blob/7984967e8701cc90bb17d06453901036e36ca578/qwen3_asr/src/forced_aligner/processor.rs)의 tokenize_japanese_with_terms/split_ascii_digit_runs/encode_timestamp/parse_timestamp/fix_timestamp.
- [공식 Qwen processor](https://github.com/QwenLM/Qwen3-ASR/blob/7c6daf77a2421100f5fb066495372c00129d39ff/qwen_asr/inference/qwen3_forced_aligner.py)의 동일 책임 함수들.

처리 순서는 언어 분절→각 word 뒤 timestamp marker2개→audio/tokenizer 준비→5000-class head logits→argmax→timestamp 위치 class만 추출→config tick80ms 곱→LIS 기반 repair→integer ms→초 단위 소수3자리 반환이다. 공식 구현도 marker mask/argmax·config tick 곱·repair·초 단위 rounding 구조를 사용한다. 이 구조 유사성은 tensor/분절/출력 parity 전체를 입증하지 않는다.

## 가설별 증거와 확정 불가 구간

### 80ms resolution: 같은 class 선택은 가능하지만 단독 원인 확정은 불가

config의 timestamp_segment_time=80과 model.rs의 class→ms 곱은 확인했다. 시작·끝 marker는 별도 class 예측을 하며, 코드에는 해당 word의 end가 start보다 반드시 큰 class여야 한다는 제약이 없다. 따라서 **같은 class를 예측하면 raw부터 zero-duration이 될 수 있다.** 짧은 조사/숫자 단위에 coarse grid와 여러 marker가 배치된다는 점은 그 가능성과 양립한다.

그러나80ms 격자 자체가 모든 짧은 token을 반드시0으로 만든다는 뜻은 아니다. 같은 숫자열에서도 어떤0은 양의 구간을 받았다. 현재 public repair 후 값만으로는 원래 두 class가 같았는지 확인할 수 없다. 반환 초/80ms로 나눠 raw class를 역산하지 않는다. repair interpolation은 grid 사이 값을 만들 수 있어 최종 seconds가 raw 격자를 그대로 대표한다고도 할 수 없다.

`round3`는 이미 integer ms로 된 시간값을 초로 바꾼 뒤3자리로 반올림한다. 이 source 단계는80ms 차이를0으로 압축하는 메커니즘이 아니다. 앞선 repair의 interpolation→integer truncation은 별도이며, 실제로 이번0을 만들었는지는 raw trace가 없어 모른다. binary float/µs 변환이 최초 원인이라는 근거도 현재 없다.

### LIS repair: zero를 보존하거나 만들 수 있으나 이번 발생 단계는 미확정

lumos와 공식 Qwen은 longest **non-decreasing** subsequence를 고른다. `data[j] <= data[i]`이므로 같은 timestamp도 정상 anchor가 될 수 있다. anomaly1~2개는 좌/우 정상 anchor의 시간으로 채우고 긴 anomaly는 보간하거나 한쪽 anchor로 채운다. pair 단위의 엄격한 `start < end` 보장이나 최소80ms 폭 강제는 없다. 따라서 raw equality를 그대로 남길 수 있고, raw inversion/비단조를 수정하면서 두 경계를 같은 anchor로 만들 수도 있다.

이것은 source에서 확인되는 정책 특성이며 이번5개 zero가 실제로 repair 때문에 생겼다는 증거는 아니다. 필요한 관측은 해당 marker의 pre-repair IDs/ms와 repair 변경 여부다. 같은 public seconds를 만들 수 있는 raw 경로가 여럿이므로 출력만으로 단계를 선택할 수 없다. 공식도 같은 비감소 정책을 쓰므로 'zero가 있으니 Rust repair 버그'라고 단정할 수도 없다.

두 source의 구현 표현에는 Rust saturating index/f32→i64와 Python signed integer/배열 처리 차이가 있다. 이번 item에 그 차이가 작용했는지는 미확정이다. 작은 source상 표현 차이를 곧바로 실제 오류로 승격하거나 전체 repair 함수를 교체하지 않는다. raw 입력이 확보되면 같은 sequence를 두 고정 알고리즘 계약으로 Rust에서 재생해 최초 차이를 확인하는 것이 후속이다.

### 일본어 분절: JA1634 숫자 분할과 직접 연결, zero 인과는 아직 가설

lumos의 고정 처리 순서에는 NFKC→Lindera/IPADIC(실패/빈 결과면 script fallback)→hiragana bridge/particle/prefix 보정→**split_ascii_digit_runs**→katakana compound split→user/default term merge가 있다. `split_ascii_digit_runs`는2자리 이상 ASCII 숫자 token을 각 문자로 나눈다. 현재 JA1634 prompt에는7/0/0/0 각각의 timestamp 쌍이 존재한다. 따라서 숫자 marker 증가와 이 source 변환은 직접 연결된다. 정확히 Lindera 전후 어느 token에서 comma가 제거됐는지까지는 중간 분절 trace가 없다.

공식 Japanese path는 nagisa.tagging(text).words 뒤 clean_token을 사용한다. 공식 source에는 lumos의 별도 ASCII digit-run split 단계가 없다. **공식 nagisa가 이번 7,000을 정확히 어떻게 분절하는지는 이 입력에 대해 관측하지 않았다.** 공식은 반드시7000 한 단어라고 추측하지 않는다. lumos가 공식 분절과 다른 알고리즘인 점과 이번 숫자를 여러 marker로 나눈 점은 확인 사실이며, 이로 인한 시간 경쟁이 zero 원인인지는 실험 가설이다.

JA1516의 もっ/て와 조사 を도 분절 이후 두 marker가 부여되는 입력 단위다. morphology 경로·post-split rule·fallback 중 어느 단계가 각 경계를 만들었는지는 현재 중간 trace로 구분되지 않는다. 본문 후반의 もって는 한 token인 관측도 있어 모든 같은 표기/어미가 동일하게 분절된다고 가정하지 않는다. KO에서 구조 실패가 없었다는 사실은 KO LR 분절 또는 언어 전체의 우수성 비교 근거가 아니다.

## golden이 보장하는 것과 보장하지 않는 것

고정 [golden_forced_aligner_processor.rs](https://github.com/lumosimmo/qwen3-asr-rs/blob/7984967e8701cc90bb17d06453901036e36ca578/qwen3_asr/tests/golden_forced_aligner_processor.rs)는 ZH/KO fixture의 word list·prompt·processor 입력을 비교하는 구조다. [masked output IDs test](https://github.com/lumosimmo/qwen3-asr-rs/blob/7984967e8701cc90bb17d06453901036e36ca578/qwen3_asr/tests/golden_forced_aligner_masked_output_ids.rs)는 지정 fixture의 timestamp marker 위치 argmax ID를 비교한다. 이는 현재 FLEURS JP1516/1634와 다른 입력이며 이 호스트에서 해당 tests를 실행한 것도 아니다.

[Japanese nagisa diff test](https://github.com/lumosimmo/qwen3-asr-rs/blob/7984967e8701cc90bb17d06453901036e36ca578/qwen3_asr/tests/golden_forced_aligner_japanese_nagisa_diff.rs)는 완전 일치를 요구하지 않는다. source상 token boundary recall≥0.75, precision≥0.90, F1≥0.80, timestamp coverage≥0.75·mean absolute drift≤600ms·max≤1200ms의 허용치가 있다. 이는 upstream approximation 회귀 기준이며 제품의 word accuracy 승인 기준으로 가져오지 않는다.

고정 [JP reference JSON](https://github.com/lumosimmo/qwen3-asr-rs/blob/7984967e8701cc90bb17d06453901036e36ca578/fixtures/golden/forced_aligner_ja_nagisa_reference.json)은 request와 fixtures/audio/asr_en_3s.wav 경로 등 별도 fixture를 참조한다. filename만으로 실제 음원 내용을 판단하지 않으며 이 fixture를 FLEURS 원음의 수작업 단어 경계 정답으로 사용하지 않는다. unit test의 monotonic 조건도 `<=`이지 모든 word의 양의 duration 보장이 아니다. tests/golden 존재는 이 host의 실행 통과·단어 정확도 증거가 아니다.

## 다음 실험: 관측성을 먼저 확보하고 한 요인씩

다음은 root 통합 담당의 기존 승인 범위에서 선택할 **후속 제안**이다. 이 연구 세션은 어떤 source도 바꾸거나 새 결과를 생성하지 않았다. 새 CLI가 이미 존재한다고 쓰지 않는다. 모델/원음/원문/hash·CPU dtype·processor baseline·source commit을 고정하고 초기 r1 증거를 덮어쓰지 않는다.

| 순서 | 단일 변경/질문 | 고정 code path·실제 API | 얻어야 할 증거 |
|---|---|---|---|
| A | 의미 동작을 바꾸지 않는 진단 관측 추가 | model.rs에서 ts_positions/pred_ids→timestamp_ms→parse_timestamp의 전후; processor.rs fix_timestamp 입출력 | item/문자 offset↔timestamp 위치↔raw class↔raw ms↔fixed ms↔repair_changed. zero가 raw부터인지 repair 이후인지 구분 |
| B | 추가 추론 없이 JP 분절 중간 단계 확인 | tokenize_japanese_with_terms의 각 단계, public encode_timestamp | Lindera 성공/fallback 여부, 숫자 split 전후·merge 전후 words/prompt. 기존 원문을 수정하지 않음 |
| C | JA1634 사용자 merge term **7000 한 항목**만 A/B | 실제 `ForcedAlignProcessor::from_japanese_user_terms` 또는 `with_japanese_user_dictionary_path`, `from_pretrained_with_processor` | 먼저 encode_timestamp에서7/0/0/0이 한 단위로 합쳐지는지 확인. 추론은 그 뒤 root worker1개. 비교는 word count/marker 배치가 달라진 실험임을 명시 |
| D | JA1516의 もって 한 항목 merge를 별도 비교 | 같은 public user-term API, 다른 추가어/규칙은 유지 | zero token을 합쳐 안 보이게 만드는 효과와 실제 경계 정확도 개선을 구분. 조사 を의 zero까지 해결됐다고 일반화하지 않음 |
| E | raw sequence가 같을 때 repair 정책 parity만 비교 | 공식 fix_timestamp 계약과 고정 Rust fix_timestamp | 원래 class/ms를 그대로 replay. 최초 달라지는 index·분기·truncation을 기록. 원음 재추론 없이 단계 분리 가능 |

A는 public API에 없는 raw trace를 노출하는 진단 fork/adapter 작업이 필요하다. 기존 source SHA가 그대로라고 숨기지 말고 baseline commit+patch hash+실행 파일 hash로 별도 라벨을 둔다. logits top2/margin 기록은 class 선택 불안정성 검토에 도움이 될 수 있으나 필요할 때만 추가하며 raw logit을 현재 관측한 것처럼 만들지 않는다.

C의 user-term API는 default terms에 사용자 항목을 합치는 현행 기능이다. baseline의 전체 merge 세트와 user term1개 차이를 보존한다. `7000` 실험은 공식 nagisa parity 시험이 아니라 **정해진 단위 분절을 바꾸는 원인 실험**이다. もって와7000을 한 번에 추가하거나 decoder/precision/repair를 동시에 바꾸지 않는다. source의 numeric split 단계를 바로 삭제하기보다 먼저 public override를 쓰면 변경 책임을 좁힐 수 있다.

금지할 결론/처리: tick80을 더 작게 바꾸면 모델의 학습 시간축 자체를 재해석하게 되므로 resolution 개선 실험이 아니다. zero를+1µs/+80ms로 늘리거나 이웃 시간에 clamp해서 구조 통과를 만드는 것은 원인 규명이 아니다. min-duration 도입은 별도 제품 정책/검토 산출물이며 raw 관측을 대체하지 않는다. 사용자 merge로0개가 되더라도 사람 경계 정답 없이는 품질 통과가 아니다.

현재 가장 강한 결론은 **5개 zero를 원시 예측·repair 중 한 단계로 귀속할 수 없으며, JA 숫자 분절은 확인 가능한 다음 단일 변수**라는 것이다. root의 needs_review와 원시 evidence를 유지하고 진단 관측→분절 단계→한 항목 merge 순서로 좁힌다.
