---
id: evidence:kinekino/20261029114159
title: 동일 full mel·expanded IDs의 Rust timestamp head 관측 준비
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 동일 full mel·expanded IDs의 Rust timestamp head 관측 준비

후속 root 실제 실행은 [세 입력780000 logits·156클래스 대조](../ai/forced-head-logits-actual.md)와 [원본 시간 보정·최종 F64 대조](../ai/forced-timestamp-functions-actual.md)에 있다. 아래 모델 실행0회는 준비 세션의 범위로 보존한다. 중앙 검사기는 r1-final의115 compiler inputs·패치7개·고정 입력6개·config·EXE1을 검증한다.

2026-10-18. [공식 head 연구 계약](../../player/infrastructure/inference/research-followups/forced-head-pytorch-readiness.md)과 [encoder 실제114tensor 비교](../ai/forced-encoder-stage-actual.md)를 끝까지 읽고 별도 입력·관측 경로를 준비했다. **이 세션은 모델/weight load와 신경망 forward를 실행하지 않았다.** root가 별도로 진행한 공식 full-model load·KO forward를 준비 세션의 결과로 합산하거나 그 raw class를 테스트 정답으로 하드코딩하지 않았다. zero-duration과 모델 품질은 미해결이다.

## 분리와 실제 API

새 adapter는 `tools/forced-head-reference-observation`, 새 fork는 `runtime/ai-spike/source/lumos-head-reference-7984967`다. Rust bootstrap `tools/head-reference-bootstrap.rs`가 frozen encoder-r2 계보에서 Git/target/symlink 없이 새 경로로 복사하며 기존 목적지를 거부한다. b069 segmentation과 encoder-r2 source/EXE·증거, root tools/model-spike 및 root 문서는 변경하지 않았다. 제품 SDK capability/CLI 지원도 승격하지 않았다.

기존 public forced aligner는 PCM에서 prepare_batch를 실행하므로 외부 full mel/IDs를 받지 못했다. 새 `align_fixed_mel_observed(PreparedInputs, words)` 진입점과 공유 `align_prepared_internal`을 추가했다. 기존 align_internal도 원래 prepare_batch 이후 같은 helper로 들어간다. 준비 이후 encoder·audio merge·text/head forward·axis2 argmax·동일p timestamp 선택·class×80ms·parse_timestamp/repair·round3 계산문은 그대로다. helper의 결과 Vec capacity만 기존 audio.len()과 동등한 prepared.len()을 사용한다. 기존 expansion 함수는 visibility만 public으로 열었고 body는 변경하지 않았다.

Thinker/text/attention/rotary source는 b069와 그대로이며, encoder는 이미 검증된 r2의 passive 관측 계보를 유지한다. 새로운 head 실행은 encoder 일반 forward를 호출하므로 별도 encoder stage dump를 또 실행하지 않는다. 모델 loader도 기존 Qwen3ForcedAligner constructor를 사용한다. 임의 재작성 forward/generate·cache 갱신·새 argmax·runner-up 선택·mask/position·repair·zero clamp는 추가하지 않았다.

## 동결한 같은 입력 계약

원문/원음과 default words/prompt는 root의 고정 segmentation 결과 SHA `b4b09dde514f23903698ec9341b1aefcc6c21e1c779095e95ab447356fbd6089` 및 기존 원본 파일로 대조했다. 실제 local tokenizer 원본3개(vocab/merges/tokenizer_config)의 고정 SHA를 확인한 뒤 기존 expansion 함수를 같은 full NumPy mel의 길이에 적용했다. 준비에는 weight load/encoder/head 실행이 없다.

**새 same-mel 준비 입력을 동결한 것**이다. JP는 생성한 expanded IDs가 과거 SHA-pinned default head 관측의 actual IDs와 exact 일치하는 것도 확인했다. 하지만 과거 head는 PCM에서 Rust mel을 만든 실행이므로 새로운 same-mel head의 bitwise baseline으로 혼합하지 않는다. KO 과거에는 raw IDs가 없으므로 historical reference=null이며 새 비추론 생성으로 명시했다. 원본 whole processor/tokenizer parity를 승인한 것도 아니다. 첫 neural 비교의 양쪽은 아래 고정 expanded IDs를 직접 사용하며 재확장하지 않는다.

| case | S / words / timestamp 수 | T / L | frozen JSON SHA256 |
|---|---|---|---|
| ko1883 | 249 / 19 / 38 | 1248 / 162 | `d45326ac1ffbbeb7cfebdb09d92031f1a6be4ee33ceae5d3f396fb2ae9e66c0c` |
| ja1516 | 321 / 28 / 56 | 1674 / 218 | `08e6829e469ee699c96d124a01878514f0dbb0816ed96d2f102d775fc27c3d5b` |
| ja1634 | 257 / 31 / 62 | 1212 / 158 | `30f413370e93fc3b9c0cc9edff6d445cc8de573124f17049645caef17bcb2157` |

파일 경로는 `runtime/ai-spike/acquisitions/sdk-head-reference-inputs-r1/<case>.json`이다. schema_version1/input_contract=`same_official_mel_default_words_v1`, deny_unknown_fields다. 핵심 필드:

- `expanded_input_ids`: flat logical IDs. Rust input Tensor U32 `[1,S]`; Torch는 같은 값을 Int64 `[1,S]`로 만든다. `expanded_ids_u32le_sha256`은 logical IDs의 U32 LE bytes SHA다.
- `attention_mask`: flat all1 `[1,S]`; `feature_attention_mask`: flat all1 `[1,T]`. Torch는 둘 다 Int64, Rust attention은 U32 Tensor·feature mask는 u8 Vec를 기존 stack_features에서 길이 계산에 사용한다.
- `audio_feature_lengths=[T]`, `encoder_output_lengths=[L]`. 서로 바꾸면 거부한다. 실제 encoder audio pad151676 수는 정확히 L이며 모든 ID는152064 미만이다.
- `timestamp_positions`: 실제 expanded IDs에서151705가 있는 동일 index p, 정확히2W개. p-1 shift나 tokenizer vocab ID와5000-class ID의 혼동은 허용하지 않는다.
- `mel_shape/mel_sha256`, `transcript/transcript_sha256`, `audio_sha256`, `words`, 원시 `prompt/prompt_sha256`, `raw_prompt_token_ids`, 고정 tokenizer file identities, historical reference가 전체 보존된다. raw prompt에는 audio pad가1개만 있고 기존 함수로 한 번 확장한다.

ID U32 LE SHA는 KO `99047af536c8b8fc50ce28c28867d47f34165feef2f808b541a3403d9ef9273c`, JA1516 `b4c2a9ac20ec22d50ff018b71425f5765410017c3cb394fb62dc749431cf5e7a`, JA1634 `e8e580c71a919fb8efc490e72129cd0a188ad1c006562a967a33982f7290e6de`다. prompt 문자열만 같다는 이유로 IDs 동일을 가정하지 않았다.

full mel 원본은 기존 `.spec/artifacts/spikes/2026-10-18/acquisitions/mel-numpy-functions-r1/<case>-official-numpy-f32.bin`이다. shape/SHA는 이전 encoder 계약과 동일하며 KO `a11be61a84c23db6c9e0fe261b14b99142b76760bc21c2d5cd1e9a1bb58b4949`, JA1516 `5c760eb80f244f9ea99db57387de1fe024435c16f63e4e5eed359b9995fb92d8`, JA1634 `74510774b41a55ff689ae43a273b2e42629b87dd0aac444cbdf722dc84acca59`를 직접 넣는다. PCM→mel을 다시 계산하거나 encoder output을 외부 값으로 교체하지 않는다.

## 실제 실행 경계와 출력

observe는 임의 model/input/words/URL argument를 받지 않는다. 고정 case→고정 JSON 경로와 compiled SHA를 읽는다. 모델 입장 전에 serialized cohort SHA, 원본 full mel/config SHA, f32 values의 모든 bit, masks/positions/ID 범위·counts를 확인한다. 고정 model/config/tokenizer/weights 파일 guard를 유지하고 revision `c7cbfc2048c462b0d63a45797104fc9db3ad62b7`, config `d616c65d46c4b90bdc651b0a0963ea932732241140f337f9bb6b0335a9c8ef09`, single weight1835544544B/SHA `47831d0e82f96b20e9034dba01a075ee06436654719f6a68289e49f1b65ce0e7`를 검사한다. CPU/F32/flashfalse/Rayon4 요청이며 원래 cache 없는 teacher forward다.

실제 관측 report는 `head-result.json`이다:

- input_cohort 전체와 hash, actual prepared IDs/attention Tensor values/feature mask/T, 실제 stacked full mel shape/SHA를 비교해 기록한다. 주입 입력을 사용하고도 기존 오디오 전처리 결과라고 표시하지 않는다.
- 실제 logits shape `[1,S,5000]`와 전체 S의 argmax class vector를 기록한다. timestamp row들은 **모든5000개 class**를 가진 `[2W,5000]` F32 LE logical row-major 파일로 보존한다. `000-timestamp_logits-0.f32le.bin`과 sidecar의 shape/bytes/SHA/nonfinite count가 있으며 top-k만 저장하지 않는다. 비timestamp 위치의 full logits matrix 자체는 저장하지 않는다.
- 실제 선택 class/position, raw f32 ms와 bit, 기존 public fix_timestamp replay의 fixed ms 및 changed indices, 반환 final text/seconds/f64 bit를 보존한다. captured final과 실제 반환값을 bit로 대조한다. raw selected class가 full argmax의 동일 위치 값인지도 확인한다.
- row audit는 실제 선택값·tie 수·최초 최대인지·finite 여부를 **관측만** 한다. tie에서 다른 class로 교체하거나 runner-up을 고르지 않는다. 비유한 logits도 binary에 남기고 수를 기록하며 clamp하지 않는다.

**미구현 관측**: 실제 내부 position_ids/rope/merged embeddings/text layer별 tensor, non-timestamp full logits matrix는 이 worker의 범위 밖이다. `actual_position_ids=null`과 명시적 note를 남긴다. 기존 get_rope_index source는 바꾸지 않았으며 batch1/all1의 논리 position 일치 source 계약과 실제 internal tensor 관측은 다르다. root의 공식 pre-hook tensor를 이 Rust worker에서도 이미 덤프했다고 주장하지 않는다. Rust loader는 기존 tensor 조회/shape 검사를 유지하지만 Torch strict708 key 감사를 독립 재현했다는 주장도 하지 않는다.

모델/kernel/쓰기 오류이면 nonzero이며 입력과 partial sidecar만 남을 수 있다. final report가 없는 부분 파일을 성공으로 취급하지 않는다. `model_forward_completed=true`는 worker가 완료됐을 때만 기록되고 alignment_quality_approved는 항상false다. full mel 경로의 Rust/Torch logits 허용오차와 raw IDs exact gate, zero-duration 원인은 root가 실제 실행 후 별도로 판정한다.

## 비추론 검사와 보존

Rust6tests가 통과했다. cohort3의 고정 SHA/S/2W/T/L/ID·mask 계약, T/L 바꿈/invalid ID/marker/unknown field/path 거부, actual argmax가 tie의 두 번째 class인 synthetic row에서도 선택을 바꾸지 않는 감사, NaN 보존, CLI prepare의 byte-exact 동결 재현과 model/input fallback flag 거부, 공유 helper의 준비 이후 전체 수치 body 불변, 기존 encoder 계산 불변을 확인했다. source regression은 추가 capture 부분만 제거하고 allocation capacity 이름만 환원하여 기존 encoder/merge/text/head/argmax/raw/repair/final 계산과 비교한다. expansion은 visibility만 다르며 thinker/text/attention/rotary source도 동일하다. copied upstream의 neural/zero-weight model tests나 actual observe는 실행하지 않았다.

```text
cargo fmt --manifest-path tools/forced-head-reference-observation/Cargo.toml --check
cargo test --locked --offline --release --manifest-path tools/forced-head-reference-observation/Cargo.toml --target-dir runtime/parallel-build/lumos-observation -j 2
cargo clippy --locked --offline --release --manifest-path tools/forced-head-reference-observation/Cargo.toml --target-dir runtime/parallel-build/lumos-observation -j 2 --all-targets -- -D warnings
cargo clippy --locked --offline --release --manifest-path tools/forced-head-reference-observation/Cargo.toml --target-dir runtime/parallel-build/lumos-observation -j 2 -p qwen3_asr_head_reference --lib -- -D warnings
cargo build --locked --offline --release --manifest-path tools/forced-head-reference-observation/Cargo.toml --target-dir runtime/parallel-build/lumos-observation -j 2 --bins
```

모두 통과했고 수정 fork rustfmt/standalone Rust bootstrap clippy-driver도 통과했다. 기존 isolated dictionary와 release target/cache를 재사용했다. 초기 encoder-only lock에 forced-aligner deps를 추가하려다 offline에서 uncached getrandom을 거부했고 실제 다운로드는 하지 않았다. frozen b069의 full lock으로 새 tool lock을 seed하여 기존 registry pin을 유지하고 새 local package만 추가했다. helper 이동 중 옛 audio 변수에 대한 allocation capacity 참조를 발견해 prepared.len()으로 고친 뒤 불변/전체 검사를 통과했다. 기존 native MSVC LNK4098 경고는 억제하지 않았고 새 Rust clippy/test 실패는 남지 않았다.

[최종 snapshot](../../../evidence/player/forced-head-reference-builds/r1-final/build-manifest.json)은115개 compiler/source/frozen input을 검증·복사하고 full mel3/config/head fixture3와 EXE를 선보존했다. b069 기준7개 source diff에는 상속한 encoder-r2 관측 변경과 새 head 진입/capture/visibility 변경이 함께 들어 있다. 양쪽 tree SHA와 정확 patch, 새 파일의 원본 부재도 기록했다. saved EXE --build-info가 보존 helper의 compiled SHA와 일치했다. weights는 복사·로드하지 않았다.

| identity | SHA256 |
|---|---|
| saved worker EXE | `805fccf7fa18594bba18d0617f586aa593e2854780d6bf6105c98cf638b73fa5` |
| build manifest | `a6857ec61db23e3a8f805f6db5b1d2b7c7ed9cba0dd64566cde47bd20f7588bf` |
| embedded compiled input | `0efe4359b07480f02c5f9330d8e5b0abc7ad6774a21f27ad995560ddda21f9b3` |
| Cargo.lock | `a74b3c4741e366cf7951829ee8f50fa945089ebca06235d82002c38e0d6f307d` |

## root 실제 실행 명령

root가 saved bytes를 검증하고 필요하면 같은 bytes를 허용된 private bin에 복사한 뒤 기존 단일-worker supervision으로 실행한다. 이 세션은 아래 observe를 실행하지 않았다.

```powershell
$headExe = 'D:/workspace/@myso-kr/com.kinekino.player/.spec/evidence/player/forced-head-reference-builds/r1-final/forced-head-reference-observation.exe'
& $headExe observe --case ko1883 --output 'D:/workspace/@myso-kr/com.kinekino.player/runtime/ai-spike/results/head-reference-rust-ko1883-r1'
# KO gate 이후에만 root가 선택하여 별도 새 directory로 실행
& $headExe observe --case ja1516 --output 'D:/workspace/@myso-kr/com.kinekino.player/runtime/ai-spike/results/head-reference-rust-ja1516-r1'
& $headExe observe --case ja1634 --output 'D:/workspace/@myso-kr/com.kinekino.player/runtime/ai-spike/results/head-reference-rust-ja1634-r1'
```

같은 fixture의 Torch input_ids/masks는 batch1로 reshape하고 IDs를 다시 확장하지 않는다. audio_feature_lengths를 전달하면 T를 사용하고 feature_attention_mask를 생략하지 않는다. 같은 timestamp p의 모든5000 logits와 actual raw class exact를 각각 비교하며 gate 실패 후 허용오차나 시간을 바꾸지 않는다. root가 보고한 공식 load·KO forward는 별도 원본 증거이며 여기서 Rust 수치/품질 통과를 선행하지 않는다. root 도구/문서 수정, 다운로드·모델 load·신경망 추론·GUI·VM·DPI·사용자 ChatGPT/Terminal 종료·제품 지원 승격은 하지 않았다.
