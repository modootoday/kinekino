---
id: sot:kinekino/qwen3-forcedaligner
title: Qwen3 ForcedAligner 원문 계약 감사
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Qwen3 ForcedAligner 원문 계약 감사

2026-10-18. 이 문서는 기존 10회 관측을 공식 계약과 대조한 **읽기 전용 감사**다. 새 신경망·processor 실행, 다운로드·원문 파일 저장·설치·빌드·기존 소스 변경은 0회다. 앞선 JER 준비 목표와 독립적이다.

결론: **Rust BPE 전처리의 정규식 지정 불일치는 소스로 확정했다. 그러나 기존 일본어 0길이의 원인으로 확정하지 않았다.** 최소 수정 후보는 `SplitPattern::Regex` 지정 한 곳이다. 먼저 동일 prompt의 token IDs가 실제로 달라지는지 확인해야 한다. raw head 동일값 4건과 repair collapse 1건은 기존 실제 증거로 확정된 서로 다른 경로다. 이 분류를 tokenizer 버그 하나의 영향으로 소급하지 않는다.

## 기준과 증거 범위

- 모델: `Qwen/Qwen3-ForcedAligner-0.6B`, revision `c7cbfc2048c462b0d63a45797104fc9db3ad62b7`. 이번 HF API 조회에서도 같은 main SHA를 반환했다. 가중치 SHA256 `47831d0e82f96b20e9034dba01a075ee06436654719f6a68289e49f1b65ce0e7`. 다른 `-hf` 모델로 바꾸지 않는다.
- 최신 공식 Qwen repo main 조회 SHA: `7c6daf77a2421100f5fb066495372c00129d39ff`, commit date 2026-06-26. 아래 링크는 이 SHA에 고정한다. 웹 search의 날짜/issue snippet은 원인 근거로 쓰지 않았다.
- Rust engine: lumos `7984967e8701cc90bb17d06453901036e36ca578`. 실제 관측 source는 [719 bundle](../../../../../evidence/player/forced-aligner-observation-builds/0.1.0-719dc709/build-manifest.json)의 `engine-working-source`를 우선 읽었다. runtime working copy만 보고 과거 빌드 소스라고 간주하지 않았다.
- [기존 실행 SOT](../../../../verification/ai/forced-alignment.md), JA1516/JA1634 raw 감사, 두 KO shift 비교, 숫자 term 비교를 읽었다. source/EXE 전체 무결성 검사를 새로 수행한 것은 아니다.
- a2/dca/d88의 `engine-source.tar`를 각각 읽은 SHA256은 모두 `22ae59e1c784c025200701e2891c9ef0d6bfc248b0facdad3e3f138669904600`이다. archive의 `qwen3_asr/src/processor/tokenizer.rs`를 stdout으로 읽어 아래 호출을 확인했다. 세 adapter Cargo.lock 및 719 engine Cargo.lock은 tokenizers **0.21.4**, registry checksum `a620b996116a59e184c2fa2dfd8251ea34a36d0a514758c6f966386bd2e03476`를 고정한다. archive를 추출·수정하지 않았다.

| 빌드 | 이번 감사의 사용 범위 | 실제 실행 귀속 |
| --- | --- | --- |
| a2e4568b | observed-build, lock, engine archive | baseline 4개와 KO shift 2개 |
| dca7db31 | manifest, lock, engine archive | JA1634 기본/7000 대조 2개 |
| d88a6e56 | manifest, lock, archive, dca→d88 patch | 최종 term Unicode 검증판; 새 신경망 실행 증거 없음 |
| 719dc709 | manifest, frozen engine/adapter source, raw 감사 | JA1634/JA1516 raw 관측 2개 |

이 10회는 입력·조건이 중복되는 관측이며 독립 정확도 표본 10개가 아니다. 구조 3통과/7실패와 사람의 시간 정확도 승인 0을 유지한다. d88의 Unicode term 검증 변경을 dca의 실제 추론 결과에 소급하지 않는다.

## 확정 불일치: Split의 리터럴/정규식 구분

719 frozen `qwen3_asr/src/processor/tokenizer.rs:206–210`은 Qwen2 패턴을 문자열로 선언한 뒤 `Split::new(pretokenize_regex, SplitDelimiterBehavior::Isolated, false)`를 호출한다. tokenizers 0.21.4의 [Split 구현](https://github.com/huggingface/tokenizers/blob/v0.21.4/tokenizers/src/pre_tokenizers/split.rs#L19)은 `&str`을 `SplitPattern::String`으로 바꾸고 `regex::escape`를 적용한다. 따라서 이 호출은 문자 클래스 등을 해석하는 정규식이 아니다. 동일 crate의 local registry source도 읽어 확인했다.

반면 공식 Transformers [Qwen2Converter](https://github.com/huggingface/transformers/blob/v4.57.3/src/transformers/convert_slow_tokenizer.py)는 동일 패턴에 `Regex(...)`를 명시한다. 이 고정 버전은 계약 비교용 source anchor이며, 과거 공식 Python 실행 환경이 4.57.3이었다는 뜻은 아니다. 공식 Qwen processor는 Qwen2/Qwen2TokenizerFast 경로를 선언한다.

Rust의 `ByteLevel::new(false, true, false)`에서 세 번째 인자는 `use_regex=false`다. 가운데 true를 regex 활성화라고 읽으면 안 된다. ByteLevel이 이 누락된 정규식을 대신 적용하지 않는다. NFC normalizer와 byte encoding을 사용한다는 사실만으로 Qwen2 sequence 동등성을 주장할 수 없다. vocab 전체 및 special token ID 검사는 이 문제를 검출하지 못한다.

**가능한 최소 변경**은 별도 fork에서 `Split::new(SplitPattern::Regex(pretokenize_regex.to_owned()), ...)`로 pattern 종류만 고치는 것이다. source 전체 교체, 사전 변경, 시간 보정 변경과 묶지 않는다. 이 감사는 패치를 적용하거나 테스트하지 않았다. 특히 깨끗한 일본어 글자열은 BPE merge 경계가 우연히 같을 수 있다. 이 호출의 잘못은 확정이지만 현재 JA1516/1634 prompt ID 차이, logits 차이, zero 감소는 모두 미검증이다.

## 전처리·입력·모델 계약 대조

| 구간 | 공식 계약 / 현재 Rust | 판정 및 한계 |
| --- | --- | --- |
| 알려진 원문·언어 | 공식 encode는 Japanese/Korean을 분기한다. adapter는 원문 hash를 보존하며 Japanese/Korean 이름을 전달한다 | 언어 오선택 증거 없음. 번역/ASR 문장으로 교체하지 않음 |
| 일본어 단위 | 공식 wrapper는 nagisa 결과를 L/N/아포스트로피 기준으로 clean한다. Rust는 NFKC→Lindera/IPADIC→hiragana bridge/particle/prefix 처리→숫자 분리→katakana 처리→사용자 merge를 수행한다 | **명백한 알고리즘 차이**. 이 transcript에서 공식 nagisa word list를 새로 실행하지 않았으므로 특정 단어의 정답 분절을 발명하지 않음 |
| 한국어 단위 | 공식 soynlp LTokenizer와 고정 Korean scores; Rust는 동일 의도를 수작업 구현하고 공백별 L/R 및 clean 적용 | 사전/동률 선택까지 공식 word list 실물 동등성 미검증. archive의 CRLF byte 차이를 곧바로 의미 차이로 보지 않음 |
| prompt/template | 양쪽 모두 audio start/pad/end 뒤 word와 timestamp 두 개를 반복한다 | 일반 ASR의 system/user chat template를 사용하지 않는 경로가 맞음. 새 chat template를 추가하는 것은 수정 후보가 아님 |
| BPE | 원본 vocab/merges/added tokens 및 NFC를 사용; adapter의 `official_sequence_parity_verified=false` | 위 Split 호출 불일치가 확정됨. special ID 일치는 전체 sequence parity가 아님 |
| PCM·범위 | 공식 입력은 mono 16k float32 범위 정규화. adapter는 mono PCM16만 받아 i16/32768→f32, upstream Waveform(sr=16000)으로 전달 | 현재 fixture는 범위 내 finite PCM으로 재샘플링 없음. 범위 밖 float/다른 sr에서 normalize/resample 순서가 다른 일반 경로는 이번 실패 원인 증거가 아님 |
| mel | 양쪽 128 bins, 400 FFT, hop160, Slaney, 0–8kHz, log floor1e-10, dynamic range8, (log+4)/4. Rust는 f64 FFT/합산 후 f32, Torch 경로는 f32 STFT/필터 곱 | 설계 상수 대조는 맞음. 실제 동일 파형 tensor 수치 parity는 미검증이며 bit-identical로 표시하지 않음 |
| frame/mask·audio pad | 공식 processor는 attention mask 길이로 encoder output length를 구하고 audio pad를 확장한다. Rust는 동일 100frame/13position 잔여 공식과 batch padding을 사용 | 단일 음원·단일 pad 조건에서 구조 일치. 전체 input_ids/mask/mel/실제 audio embedding 길이를 함께 dump한 증거는 없음 |
| position IDs/RoPE | 공식 mask cumsum−1, padding=1, 3축 동일값. Rust get_rope_index는 left-padding suffix 기준 같은 수열을 계산하고 broadcast한다 | 현재 batch1/all-ones에서 0..N−1. config theta1e6·section[24,20,20]·interleaved 읽기 경로 확인. layer별 수치 parity는 미검증 |
| decoder/head | 양쪽 audio features를 audio-pad embedding에 삽입하고 causal text decoder→bias 없는 classify_num=5000 head를 적용한다 | vocabulary generation/자기회귀 text decode를 timestamp class로 잘못 해석한 증거 없음. 전체 weight/tensor 수치 대조는 수행하지 않음 |

공식 근거: [aligner wrapper](https://github.com/QwenLM/Qwen3-ASR/blob/7c6daf77a2421100f5fb066495372c00129d39ff/qwen_asr/inference/qwen3_forced_aligner.py), [audio normalization](https://github.com/QwenLM/Qwen3-ASR/blob/7c6daf77a2421100f5fb066495372c00129d39ff/qwen_asr/inference/utils.py), [processor](https://github.com/QwenLM/Qwen3-ASR/blob/7c6daf77a2421100f5fb066495372c00129d39ff/qwen_asr/core/transformers_backend/processing_qwen3_asr.py), [model](https://github.com/QwenLM/Qwen3-ASR/blob/7c6daf77a2421100f5fb066495372c00129d39ff/qwen_asr/core/transformers_backend/modeling_qwen3_asr.py), [Whisper implementation anchor](https://github.com/huggingface/transformers/blob/v4.57.3/src/transformers/models/whisper/feature_extraction_whisper.py). Python framework/kernel 버전이 과거 실행과 고정 대조됐다는 주장은 하지 않는다.

adapter의 `validate_pcm`은 400–480000 samples(25ms–30s)를 허용한다. 이는 이 스파이크의 명시적 입력 한도이며 공식 모델의 긴 음성 지원 범위로 일반화하지 않는다. 실제 raw JA1516은267840 samples/16.74s, JA1634는193920/12.12s이며 모두 hop160의 정수배다. 여기서 임의 30초 crop이나 source offset이 더해졌다는 증거는 없다.

## 시간 토큰·후처리·수치 정밀도

양쪽은 logits의 같은 sequence 위치에서 argmax를 구한 뒤 **input token ID=151705인 위치**만 뽑는다. next-token shift나 텍스트 token decode를 하지 않는다. class×80은 **ms**이고, repair 뒤 integer ms/1000을 3자리 seconds로 만든다. Rust의 실제 class/ms/f32/final f64 감사와 같은 경로다. 80ms를 80초/80us로 잘못 환산한 증거가 없다. [고정 config](https://huggingface.co/Qwen/Qwen3-ForcedAligner-0.6B/blob/c7cbfc2048c462b0d63a45797104fc9db3ad62b7/config.json)도 151705/80/5000을 선언한다.

공식 fix는 non-decreasing LIS의 `<=`와 가장 먼저 찾은 최대 subsequence를 사용한다. 동일한 인접 값은 합법적인 입력으로 보존하며, 짧은 anomaly는 가까운 정상값으로 붙여 양수 span을 0으로 만들 수 있다. Rust의 해당 분기·동률 선택은 이 소스와 대응한다. 따라서 `end>start`를 보장하는 알고리즘으로 재해석하거나 epsilon을 덧붙이지 않는다.

원문은 numpy 값을 list로 옮긴 후 Python 수치로 보간한다. Rust는 긴 anomaly의 보간을 f32로 하고 마지막에 trunc한다. 이 **장구간 보간 정밀도 차이**는 확인했지만, 실제 5개 zero의 원인으로 보지 않는다. raw class≤4999의 class×80 정수는 f32에서 정확히 표현 가능한 범위다. raw부터 동일한 두 class를 최종 float rounding이 만들었다고 설명할 수 없다. 가까운 값 복사로 바뀐 JA1634 index10/12도 장구간 보간 오차와 구분한다.

공식 예제의 BF16 사용은 강제 dtype 계약이 아니다. 실제 Rust는 CPU F32, flash off다. config의 thinker dtype=bfloat16만으로 F32 경로가 잘못됐다고 판정하지 않는다. 반대로 F32 승격이 공식 BF16/kernel의 argmax와 같다고도 보장하지 않는다. 현재 top2 logits/margin이 없어 near-tie 민감도를 측정하지 못했다. quantization이나 dtype 변경을 이 수정과 함께 적용하지 않는다.

## 실제 관측이 이미 좁힌 범위

| 입력/항목 | raw ms → fixed ms | 귀속 가능한 범위 |
| --- | --- | --- |
| JA1634 index2 は | 2240→2240 그대로 | raw equal, repair 생성 아님 |
| JA1634 index5 0 | 2720→2800가 2800→2800 | positive raw의 repair collapse |
| JA1634 index7 0 | 3040→3040 그대로 | raw equal |
| JA1516 index9 を | 7840→7840 그대로 | raw equal |
| JA1516 index10 もっ | 8000→8000 그대로 | raw equal |

raw 관측판은 baseline a2의 단어/최종 f64 bit와 모두 일치했다. 이는 719 계측의 최종 동작 비변경 증거이고 공식 모델 수치 parity는 아니다. 실제 raw 전체 logits와 private LIS flags는 없다. [JA1634 감사](../../../../../evidence/ai/model-spikes-2026-10-18/results/forced-observed-ja1634-r1-raw-evaluation.json), [JA1516 감사](../../../../../evidence/ai/model-spikes-2026-10-18/results/forced-observed-ja1516-r1-raw-evaluation.json).

KO 1초 shift는 모든 38개 경계가 기대 대비 ±40ms, 1.04초 shift는 36개 일치/2개 −80ms였다. 격자 정수배에서도 차이가 남으므로 위상만으로 설명을 끝내지 않는다. 7000 merge는 숫자 zero 2개를 줄였지만 は zero는 남았다. 이는 출력 단위 변경의 실제 효과이며 공식 분절이나 사람이 확인한 경계 정확도 통과가 아니다.

## 단일 후속 요청: 정규식 pattern 종류만 고친 입력 계약 A/B

root에 요청할 한 업무는 **frozen719 대비 Regex pattern 한 곳만 변경한 별도 Rust fork의 입력/결과 A/B**다. 기존 bundle, default 사전/분절, 원음/원문, dtype, 80ms/repair, thread 수는 유지한다. 예상 결과를 맞추기 위한 merge term 추가·timestamp repair 변경·zero 강제 연장·삭제는 하지 않는다.

먼저 모델을 로드하지 않는 단계에서 JA1516·JA1634 및 KO baseline의 기존 generated_prompt를 그대로 사용해 A/B tokenizer serialization, pretokenized spans, 전체 input_ids·special timestamp 위치·expanded audio-pad 개수·mask를 기록한다. 한국어 숫자 혼합과 합성 `7000`/문자+숫자/공백/아포스트로피는 pattern 작동을 확인하는 진단일 뿐 품질 지표가 아니다. 모든 fixture에서 IDs가 같다면 그 변경으로 기존 zero가 해결된다고 주장하거나 의미 없는 신경망 반복을 요청하지 않는다.

실패 입력 중 실제 IDs가 달라지는 것이 확인된 **한 입력만** root의 기존 자원 정책 아래 별도 frozen candidate로 실제 실행한다. matching baseline719 raw 실행을 A로, 새 fork 실행을 B로 사용하고 EXE/source/config/weight/input hashes와 CPU F32/batch1/threads4를 기록한다. 비교는 timestamp 위치·selected classes·raw ms·fixed ms·최종 bit/zero 수다. 가능하면 같은 실행에서 head top2/margin을 관측하되 수치 body를 변경하지 않는다. 입력 차이가 없으면 이 neural 단계는 생략한다. 이것이 이 감사의 최소 요청이며 새 모델/분절기/양자화/여러 성능 실험으로 범위를 넓히지 않는다.

이 A/B로 확인할 수 있는 것은 정규식 호출 수정의 실제 영향까지다. 공식 nagisa/soynlp와 full processor tensor parity 및 사람 경계 정답 없이는 모델 원래 한계·분절 단위 영향·수치 kernel 차이를 완전히 분리하지 못한다. 원인 미확정 상태를 유지하는 것이 timestamp 수리로 구조 검사만 통과시키는 것보다 정확하다.

## 감사 앵커와 검증 한계

719 manifest SHA256: `276630252dd9061c790236756f88467a3ff0dff5df18b9f3b0a0dacf862136be`.

| 읽은 frozen source | SHA256 |
| --- | --- |
| processor/tokenizer.rs | `993480389256cd82e1d57ad156b832376f5c81c310e9cb866a1ece3d55b2b9f4` |
| forced_aligner/processor.rs | `83c9adc59117d0f3d9c712249e36c2f7a90b8aedef2ede6e3efce0e92fe6aa8f` |
| forced_aligner/model.rs | `cf2e02bc7e3b5e066e3573c025742718157000641826655f183191033cf0c640` |
| processor/feature_extractor.rs | `9687a313e61b4b557b0dcdca736a06fe6df6ffc8268da0b2bca72f3ca2d3bf8a` |
| model/thinker.rs | `f13ac5efacecc94c2f716029838faf0b9cfdfce29c44cdca11689a2bb9a28288` |

raw-evaluation SHA256: JA1516 `302c1752e5962d1a7276505fea1b882c538bb8f94d3dc6569b4eaa0550483601`, JA1634 `d5f2aefc1c525eec1a160c4f08789d4b23ef95704c52f50df94871fa7dbc1396`.

위 해시는 이번 읽은 파일의 byte identity다. 전체 snapshot 재검사나 컴파일 입력 attestation을 새로 완료한 것은 아니다. 이번 변경은 이 문서와 coordination 문서뿐이며 코드 변경이 없어 cargo fmt/test/clippy를 다시 실행하지 않았다. 소스 호출·고정 dependency 의미·공식 primary source를 대조한 정적 발견과 실제 모델 인과 검증을 구별한다.
