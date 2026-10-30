---
id: sot:kinekino/gemma-3-270m-gguf-runtime-context2048
title: Gemma 3 270M GGUF runtime context2048 거부와 패치 준비
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Gemma 3 270M GGUF runtime context2048 거부와 패치 준비

2026-10-19. root의 실제 준비 실패와 고정 source를 읽은 독립 조사다. **이 세션은 engine/model 취득·빌드·load·추론·root 코드 변경을 하지 않았다.** 아래는 검증할 Rust 패치 제안이며 구현/통과 증거가 아니다. [기존 선정 계약](lightweight-generation-spike-readiness.md)의 context2048·동일 모델·수치 정책을 유지한다.

## 실제 관측

root 결과 폴더 `runtime/ai-spike/results/text-structured-gemma270-smoke-r1`에는 preflight, input-files, model-inspection, server-start 및 stdout/stderr가 있다. stderr69B는 `max_model_len=2048 is not supported by this multimodal loader`다. SHA256 `26bb691e8cd87827726c310b3eb3392a940c837d344d9fa051d93d06658c3871`. stdout은0B다. 읽은 폴더에는 completed 요청/생성 결과나 별도 최종 측정 JSON이 없다. 이를 의미 품질 실패 또는 생성0/3 정확도로 집계하지 않는다. 준비 단계 실패다.

server-start는 engine EXE SHA256 `59ee8ff009a06cee8c39890cf6fd90d74f4617e46e88ac9b46a28a11538a9d73`, localhost14346, Windows Job kill-on-close를 기록한다. `tools/model-spike/src/infrastructure/local_text_server.rs:80`의 `--max-model-len 2048`을 읽었다. 이 제한을 제거하거나 서버가32768로 뜨면 해결이 아니다.

root header 검사 `runtime/ai-spike/results/generation-gemma270-header-r1-gguf-header.json` SHA256 `88692e14b6c290d6273774d4178df476f3a3fb474f0f1e6f9d976a1f595cee70`은 GGUF v3/253115424B, 고정 모델SHA `b1baabd6b729e4041822220d3e648e00d99cac5df86b10dffb77bcccf0688e39`의 bytes/hash 검증을 기록한다. repo는 `unsloth/gemma-3-270m-it-GGUF@c90975dbd40c0c7b275fefaae758c3415c906238`다. 이번 연구에서 모델 전체 hash를 다시 계산하지 않았다.

관측 metadata는 architecture gemma3, native context32768, embedding640, layers18, FFN2048, Q4head/KV1head, head_dim256, sliding_window512, rope_base1000000, tokenizer vocab262144, BOS2/EOS106다. FFN2048과 runtime context2048은 서로 다른 축이다. embedded template SHA `af95fbef33b76a50e5f463ff9766f85ce84f5849d915c4bd6c1619d852ac3231`을 보존한다. 실제 rendered prompt/tokenizer parity와 모델 품질은 아직 승인되지 않았다.

## 고정 source의 정확한 경로

대상은 [mistral.rs commit4400935](https://github.com/EricLBuehler/mistral.rs/tree/4400935451da5e2dc7379a3f92fbbada66557f6c)다. v0.9.4 tag source와 설치 EXE의 모든 빌드 입력 일치까지 증명한 것은 아니지만, source의 오류 문자열과 실행 stderr가 일치한다.

1. [pipeline/gguf.rs:590](https://github.com/EricLBuehler/mistral.rs/blob/4400935451da5e2dc7379a3f92fbbada66557f6c/mistralrs-core/src/pipeline/gguf.rs#L590)는 projector 없는 gemma3를 `load_native_gemma3_text`로 보낸다. 같은 파일732행부터 이 함수가 `prepare_gemma3_text_config`·HalfSplit rope layout·Gemma3 bindings·GGUF weight source/tokenizer/template를 준비한다.
2. 이 함수는 `PreparedMultimodalSource`와 `MultimodalLoaderType::Gemma3`를 사용한다. 명칭에 multimodal이 있어도 `Gemma3Config::Text`는 실제 text-only 분기다. 다른 vision model로 잘못 식별됐다는 증거가 아니다. `GGUFSpecificConfig::multimodal_config`는 max_model_len을 그대로 전달한다(189행).
3. [pipeline/multimodal.rs:568](https://github.com/EricLBuehler/mistral.rs/blob/4400935451da5e2dc7379a3f92fbbada66557f6c/mistralrs-core/src/pipeline/multimodal.rs#L568)는 `self.inner.runtime_config(&config, self.config.max_model_len)?`를 호출한다.
4. [loaders/multimodal_loaders.rs:166–177](https://github.com/EricLBuehler/mistral.rs/blob/4400935451da5e2dc7379a3f92fbbada66557f6c/mistralrs-core/src/pipeline/loaders/multimodal_loaders.rs#L166)의 trait 기본 구현은 Some값이면 오류를 반환한다. **직접 거부 문자열은173행이다.** [Gemma3Loader impl4030행](https://github.com/EricLBuehler/mistral.rs/blob/4400935451da5e2dc7379a3f92fbbada66557f6c/mistralrs-core/src/pipeline/loaders/multimodal_loaders.rs#L4030)에 runtime_config override가 없어 이 기본 구현이 호출된다.

따라서 source상 직접 원인은 **Gemma3 loader의 runtime cap 지원 누락**이다. GGUF가2048을 수치적으로 지원하지 못한다는 판정이 아니다. GGUF archive/config/tokenizer 준비가 진행됐을 수 있으나 모델 객체 load/generation 성공을 뜻하지 않는다. 이 지점을 고쳐도 다음 load 실패 가능성은 미검증으로 남긴다.

### 변조하면 안 되는 native 계약

[gguf/gemma3_config.rs](https://github.com/EricLBuehler/mistral.rs/blob/4400935451da5e2dc7379a3f92fbbada66557f6c/mistralrs-core/src/gguf/gemma3_config.rs)는 native metadata에서 max_position_embeddings를 합성하고 validation에서 GGUF context_length와 exact 일치를 요구한다. 외부 config를2048로 바꾸거나 GGUF header를 수정하면 이 계약을 훼손한다.

[Gemma3TextConfig](https://github.com/EricLBuehler/mistral.rs/blob/4400935451da5e2dc7379a3f92fbbada66557f6c/mistralrs-core/src/vision_models/gemma3/config.rs)는 max_position_embeddings를 갖고, [text.rs](https://github.com/EricLBuehler/mistral.rs/blob/4400935451da5e2dc7379a3f92fbbada66557f6c/mistralrs-core/src/vision_models/gemma3/text.rs)의 global/local RoPE 생성과 normal cache(551행), TextModel.max_seq_len(564행), ModelConfigMetadata.max_seq_len(568행)에 사용한다. [layers.rs:2413](https://github.com/EricLBuehler/mistral.rs/blob/4400935451da5e2dc7379a3f92fbbada66557f6c/mistralrs-core/src/layers.rs#L2413)의 Gemma3RotaryEmbedding은 native max position으로 테이블을 만든다. rope_theta/factor/local base/sliding_window/pattern/query scalar/softcap/quantized tensor binding을 이번 cap 패치에서 바꾸지 않는다.

## 최소 권장 Rust 패치: native 길이와 runtime 한도를 분리

**3개 책임 파일에 국한한 text-only 패치**를 먼저 제안한다. 공용 trait 기본 거부는 유지한다. 실제 diff·compile·test는 root가 수행한다. 새 파일이나 field명은 아래 제안이며 현재 존재하는 API로 서술하지 않는다.

1. `vision_models/gemma3/config.rs`: Gemma3TextConfig에 private optional `runtime_max_model_len: Option<usize>`를 `#[serde(default, rename = "_mistralrs_runtime_max_model_len")]`로 추가한다. 기존 native `max_position_embeddings`는 그대로다. `effective_max_seq_len() -> Result<usize>`는 None이면 native, Some(cap)이면 **1<=cap<=native**일 때 cap을 반환하고 나머지는 오류로 한다. helper와 생성자의 검증을 공유해 우회된 deserialization에도 유효성을 확인한다.
2. 같은 책임의 `apply_runtime_max_model_len(config, cap)`은 원본 JSON을 `serde_json::Value`로 보존하면서 text-only variant를 확인하고 internal field 하나만 넣은 **메모리 내 파생 문자열**을 반환한다. native field 덮어쓰기나 디스크 쓰기는 없다. 외부 config에 reserved field가 이미 있으면 충돌을 거부한다. cap0/native초과/형식 오류/WithVision은 명시적 오류로 유지한다. 이 단계에서 vision 지원을 확대하지 않는다. cap=None은 기존 `Cow::Borrowed(config)` 경로를 유지하되 주입 reserved 값의 출처 검증을 우회하지 않도록 load admission과 helper의 경계를 root가 고정한다.
3. `pipeline/loaders/multimodal_loaders.rs`: Gemma3Loader에 runtime_config override를 추가해 Some이면 위 helper의 Cow::Owned, None이면 기존 borrowed config를 반환한다. text-only scope를 명시한다. 같은 loader의 `DeviceMappedModelLoader::model_config`(4397행의 max_seq_len)도 effective helper를 사용하도록 한다. 공통 loader가 원본 config로 tensor-parallel metadata를 물을 때는 native 값이며 실제 runtime config를 줄 때는 cap값이다. 이 둘을 혼동하지 말고 load 후 metadata를 검증한다.
4. `vision_models/gemma3/text.rs::TextModel::new`: effective 길이를 한 번 검증하여 **normal full cache max_seq_len·TextModel.max_seq_len·ModelConfigMetadata.max_seq_len**에 사용한다. global/local RoPE constructor에는 변경 없는 native cfg를 준다. sliding cache의512 설정은 그대로 유지한다. 이로써 runtime cap은 native RoPE 테이블 크기를 줄이지 않지만 원본 수치 정책과 분리된다. 테이블 메모리 최적화는 후속 범위다.

변경은 세 파일이며 위 목록3/4는 각각 loader 연결과 실제 text model 적용 책임이다. struct literal이 있는 Rust fixture/생성 지점은 optional field 추가에 맞춰 컴파일을 고쳐야 한다. 원본 JSON unknown field나 rope layout stamp를 typed serialize로 유실하지 않는다. helper의 파생 JSON SHA와 native JSON SHA를 각각 기록하여 “조용한 config 변경”이 되지 않도록 한다. source 파일/GGUF/config 바이트는 변경 없이 유지한다. runtime cap은 실행 설정이며 engine manifest/log에 **native32768/runtime2048**을 함께 표시한다.

더 짧은 우회인 trait에서 config 그대로 반환, max_model_len=None, max_position_embeddings2048 대체, metadata만2048로 보이게 하는 패치는 수용하지 않는다. cap이 실제 cache와 요청 제한에 연결돼야 한다. 원본 context를 학습길이나 RoPE 확장으로 해석하는 새 정책도 넣지 않는다.

### downstream cap 전달 근거와 검사점

`vision_models/gemma3/mod.rs::max_seq_len()`는 language_model의 값을 위임한다. `pipeline/multimodal.rs:1483`은 `model.max_seq_len()`을 읽어 `GeneralMetadata.max_seq_len`에 담는다. 따라서 text model의 값부터2048이어야 한다.

[engine/add_request.rs](https://github.com/EricLBuehler/mistral.rs/blob/4400935451da5e2dc7379a3f92fbbada66557f6c/mistralrs-core/src/engine/add_request.rs)는 prompt_tokens.len()>metadata.max_seq_len이면 truncate_sequence=false일 때 ValidationError를 반환한다. cap 범위의 prompt를 모델이 자동으로 줄이면 동일입력 조건을 잃는다. [sequence.rs:1717](https://github.com/EricLBuehler/mistral.rs/blob/4400935451da5e2dc7379a3f92fbbada66557f6c/mistralrs-core/src/sequence.rs#L1717)은 현재 tokens.len()>=max_model_len을 검사한다. 같은 함수에는 add_token이 검사 뒤 호출된다는 주석이 있으므로 이 조건만 보고 출력 token까지 포함한2048 상한을 보장하지 않는다. 호출자가 실제2048을 전달하는지와 마지막 token 추가/forward 순서를 root가 회귀로 검증해야 한다. 경계 실패면 별도 serving admission/stop 수정이 필요하며 loader3파일 패치만으로 완료하지 않는다.

이번 structured spike의 고정 max_tokens256은 줄이지 않는다. root의 사전 admission은 최종 rendered prompt token수+256이2048을 초과하면 명시적으로 실패시킬 수 있다. prompt를 자르거나256을 몰래 줄이는 대신 입력 조건 미충족으로 기록한다. 이것은 일반 engine의2048 경계 회귀를 대신하는 검증은 아니다.

## root 회귀·실행 게이트

### 모델 없이 Rust unit/계약 검증

- helper의 None 경로 byte 보존, cap2048/1/native 허용, 0/native+1/비정수/overflow/reserved 충돌/WithVision 거부. native max_position_embeddings32768, rope/scale/theta/window512/pattern/queryscalar/softcap/quantization/prefix 및 기타 unknown JSON이 그대로인지 비교한다.
- Gemma3Loader.runtime_config(Some2048)가 기본 trait 거부를 벗어나고 parsed native는32768/runtime은2048인지 확인한다. 다른 지원되지 않는 loader의 Some2048 거부와 Gemma3 vision 거부를 유지한다. 기존 GGUF native validation은 여전히 mismatch를 거부한다.
- config/helper와 loader model_config metadata 양쪽을 검사한다. 실제 text-model cache/max_seq_len/model metadata가 같은 effective 값을 쓰는지 Rust 회귀를 추가한다. weight 없는 테스트와 실제 model construction이 필요한 테스트를 구분한다.
- 변경 crate/워크스페이스 지침의 `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings` 및 root가 관리하는 locked/offline 빌드·compiled-input manifest를 수행한다. 본 세션은 수행하지 않았다.

### root 실제 엔진/모델 검증 순서

1. 별도 engine fork의 base commit·patch·Cargo.lock/features/컴파일 입력·새EXE SHA를 선보존한다. 기존59ee EXE/실패 폴더를 덮어쓰지 않는다. GGUF bytes/SHA와 embedded template/tokenizer를 재검증하고 동일 file로 실행한다. 원본 모델파일·externalconfig 변경0을 기록한다.
2. 같은 CLI에서 **--max-model-len2048 유지**, CPU/4threads/max-seqs1/paged-attn off/prefix-cache0/offline 등 기존 정책을 유지한다. 내부 native/runtime 길이·cache/model/general metadata2048을 관측하고 ready/health가 실제 성공하는지 확인한다. 단순 오류 문자열 소멸만으로 승인하지 않는다.
3. rendered prompt의 BOS/chattemplate/EOS와 최종 token IDs를 관측하고 요청 입력 SHA를 기존 input-files와 연결한다. 자동 truncation=false를 확인한다. 엄격 JSON/schema·temp0/requestseed42/max_tokens256 등 기존 정책을 바꾸지 않는다.
4. runtime 경계 회귀는 **최종 template 적용 후** token수2047/2048/2049를 구분한다. >2048 prompt는 모델 실행 전에 명시적으로 거부하고,2048 이하는 native32768로 넘어가 계속 생성하지 않으며 총 sequence의2048 stop을 확인한다. generation budget 포함 여부/finish_reason/offset의 off-by-one를 기록한다. 경계 확인용 요청은 품질 smoke3건과 따로 집계한다.
5. 위 게이트 후 기존 correction/cards/highlights 각1건 최소 smoke를 실행한다. 구조 성공과 의미 누락·환각·참조 정확도는 별도 판정한다.32768 native 길이로 비교 실행해 수치 동등성을 확인하는 실험은 별도 controlled 실행으로 root가 명시적으로 소유하고,2048 고정 제품 spike를 대신하지 않는다. 본 조사에서 수치 parity나 품질을 미리 승인하지 않는다.

## source provenance와 남은 미검증

고정 Git tree에서 읽은 size/Git blob SHA1:

| source | B | Git blob |
|---|---:|---|
| pipeline/gguf.rs |66641|6cddd5df1b28fee808f2d373cc265dfcf8f837c8|
| pipeline/multimodal.rs |125959|202ae9406d4643cce7da57288722df75dd942de5|
| pipeline/loaders/multimodal_loaders.rs |427857|6e3fdba975fd99e181080508e82b25ff263c962f|
| vision_models/gemma3/config.rs |2693|719c0f90e2e48a449360f18a6b2c32201d913b74|
| vision_models/gemma3/text.rs |39138|78a5e22eb87ab5616d1c66fc1486e367be11426f|
| gguf/gemma3_config.rs |26901|c8dddd48629cc6f24332cd5237ae291e6aece739|

이는 Git blob 식별자이며 배포물 SHA256이 아니다. 원문은 위 고정 commit의 raw/GitHub 페이지로 읽었으며 로컬 source checkout이나 실행기를 새로 취득하지 않았다. 패치 API명은 제안이다. 컴파일 가능성·Windows load·cap end-to-end·수치 parity·structured JSON/의미 품질은 모두 미검증이다. base engine 라이선스와 Gemma 모델 terms/변환 template 별도 표기를 유지하며 공개 GGUF라는 이유로 원본조건 해소를 주장하지 않는다.

이번 변경은 이 별도 연구 정본과 `runtime/coordination/research.md` 인계뿐이다. 모델/UI/SDK/Pages/VM/DPI/다른 앱과 기존 frozen 증거는 변경하지 않았다.
