---
id: sot:kinekino/forcedaligner-timestamp-head
title: 공식 ForcedAligner 텍스트 결합·timestamp head 참조 준비
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 공식 ForcedAligner 텍스트 결합·timestamp head 참조 준비

2026-10-18. 원본 source와 보존된 실측만 읽은 독립 조사다. **full-model weight load·text/head forward·새 정렬은 실행하지 않았다.** 다음 실제 실행은 root가 담당한다. 기존 모델·환경·frozen 증거·도구는 수정하지 않았다.

## 출발 관측

[기존 분절 A/B 실제 결과](../../../../verification/ai/forced-segmentation-ab-actual.md)는 총 정렬 호출17회, 구조3통과/14실패, 품질0이다. 원본 nagisa 단위에서도 JA1516 raw equal2개, JA1634 raw equal2개/repair collapse1개가 남았다. 독립 정확도17표본으로 해석하지 않는다.

그 뒤 root는 원본 Qwen encoder를 CPU/F32/eager/eval/no_grad, strict397 tensor로 실제 실행했다. `runtime/ai-spike/acquisitions/encoder-official-forward-r1/encoder-forward-result.json` SHA256 `79faf4fe4efeee0fe8c5cb5cef17266d3aec55e49366bc741e429fd67d6811ab`, `runtime/ai-spike/results/encoder-stage-comparison-r1-encoder-comparison.json` SHA256 `799b119a285b66a8a34d0db03f865dc3164859ab7159a7fdde3fdab8b46d5bfc`를 읽었다. Rust stage r2와38 common tensor×3=114개가 고정 atol/rtol 각1e-4를 통과했다. 최종 maxabs는 KO5.3048e-6/JA1516 1.3828e-5/JA1634 1.0371e-5다. **encoder 통과는 text/head 일치나 zero-duration 해결이 아니다.** 이전 [encoder 준비 문서](forced-encoder-pytorch-readiness.md)의 실행 전 상태와 이 후속 실측을 구분한다.

## 고정 source·환경·입력

원본은 [Qwen3-ASR commit](https://github.com/QwenLM/Qwen3-ASR/tree/7c6daf77a2421100f5fb066495372c00129d39ff) `7c6daf77a2421100f5fb066495372c00129d39ff`, 로컬 `runtime/ai-spike/source/qwen-original-package-r1`이다. 모델은 `Qwen/Qwen3-ForcedAligner-0.6B@c7cbfc2048c462b0d63a45797104fc9db3ad62b7`, weight1835544544B/SHA256 `47831d0e82f96b20e9034dba01a075ee06436654719f6a68289e49f1b65ce0e7`다.

| 로컬 원본 파일 | SHA256 |
|---|---|
| qwen_asr/core/transformers_backend/modeling_qwen3_asr.py | 2fb5d98da1933748f5117ee05ce4e7150c9ead8154fb8e25f7af3968b853adc7 |
| qwen_asr/inference/qwen3_forced_aligner.py | fb586127507c5980d11c6d848b530e652afad3ae0707cba6eb0c90377b2a2111 |
| qwen_asr/core/transformers_backend/processing_qwen3_asr.py | a61f1fbfc06e3dfc63a2b8d15e3b29e98b032792c3fb7d0e01c04ca9c2181ffb |
| 고정 모델 config.json | d616c65d46c4b90bdc651b0a0963ea932732241140f337f9bb6b0335a9c8ef09 |

`runtime/ai-spike/source/encoder-reference-private-r1`의 기존 실제 import 환경을 읽기 재사용 후보로 삼는다. CPython3.13.3 DLL SHA `69fd86ea29370697c203f7e12830084f920f490766a8e3045af52c036a9ad529`, Torch2.9.1+cpu, Transformers4.57.6, NumPy2.3.2, nagisa0.2.11/Accelerate1.12.0 등 실제 import 결과를 root가 새 실행 manifest에 연결한다. 기존 환경을 변경하거나 추가 package가 필요하다고 가정해 설치하지 않는다. 원본 import는 nagisa·soynlp·librosa·soundfile closure 및 nagisa 초기화를 포함한다.

full official NumPy mel은 [기존 실제 mel 비교](../../../../verification/ai/forced-mel-numpy-actual.md)의 `.spec/artifacts/spikes/2026-10-18/acquisitions/mel-numpy-functions-r1` bin이다.

| case | T / encoder L | mel SHA256 |
|---|---|---|
| ko1883 | 1248 / 162 | a11be61a84c23db6c9e0fe261b14b99142b76760bc21c2d5cd1e9a1bb58b4949 |
| ja1516 | 1674 / 218 | 5c760eb80f244f9ea99db57387de1fe024435c16f63e4e5eed359b9995fb92d8 |
| ja1634 | 1212 / 158 | 74510774b41a55ff689ae43a273b2e42629b87dd0aac444cbdf722dc84acca59 |

모델·원음·transcript SHA는 기존 actual 결과에서 그대로 이어받고 mel·prompt·words·**expanded IDs**를 각각 별도 manifest 값으로 고정한다. prompt 문자열만 같다고 IDs가 같다고 선언하지 않는다.

## 실제 teacher forward 계약

[원본 align](https://github.com/QwenLM/Qwen3-ASR/blob/7c6daf77a2421100f5fb066495372c00129d39ff/qwen_asr/inference/qwen3_forced_aligner.py#L435)은 processor 입력을 `self.model.thinker(**inputs)`에 한 번 넣고 `.logits.argmax(dim=-1)`한다. transcript가 이미 주어진 단일 teacher-style forward이며 labels/loss를 계산하는 학습 실행은 아니다. **generate를 호출하지 않는다.** top model은 `.thinker`를 소유하며 해당 wrapper에 일반 forward 구현이 없다. `.generate()`는 autoregressive 새 token 선택·cache 갱신·후속 audio 제외를 하므로 같은 참조가 아니다. timestamp class ID는 tokenizer vocab token ID가 아니어서 decode·BOS/EOS·sampling으로 해석하지 않는다.

### 모델 생성과 weight load

Rust PyO3/C-API에서 원본 module을 import한다. `Qwen3ASRConfig.from_json_file(local_config)` → `Qwen3ASRForConditionalGeneration(config)`로 전체 객체를 만들고 공개 `load_state_dict(..., strict=True)`로 전체 checkpoint key를 **prefix 제거 없이** 매핑하는 방법이 가장 명시적이다. 원본 구조는 `thinker.audio_tower.*`, `thinker.model.*`, `thinker.lm_head.weight`다. 기존 encoder처럼 `thinker.audio_tower.`만 제거하는 처리는 full wrapper에 쓰지 않는다. 원본 `Qwen3ASRThinkerForConditionalGeneration(config.thinker_config)`만 만들 때는 `thinker.` 전체 prefix를 제거해야 한다. 이 둘을 섞지 않는다.

대안은 원본 concrete class의 `from_pretrained(local_dir, local_files_only=True, torch_dtype=torch.float32, attn_implementation="eager", device_map=None, output_loading_info=True)`다. HF missing/unexpected/mismatched/error 및 실제 모든 parameter를 검사하고 strict 대응 없이 warning만으로 통과시키지 않는다. 실제 checkpoint 전체 key 개수·shape/header는 이번 조사에서 로드하지 않았으므로 **encoder397에서 전체 개수를 추정하지 않는다**. config text tie_word_embeddings=false, classify_num=5000, embedding152064×1024, head5000×1024다. `_tied_weights_keys` 이름 목록만 보고 embedding과 classifier를 묶지 않는다. 누락 head를 생성값으로 실행하지 않는다.

기본 dtype F32를 생성 전에 고정하고 CPU/F32 `.eval()`, inference_mode 또는 no_grad, threads4/interop1을 실제 검증한다. config에 남은 BF16 표시와 실행 parameter dtype을 구분한다. `model.set_attn_implementation("eager")` 뒤 audio tower와 text model 및 각 attention의 실제 config를 확인한다. 자동 SDPA·device_map·quantization·compile은 이 기준 실행에 넣지 않는다.

### thinker 호출 kwargs

| 인자 | 단일 case 값 |
|---|---|
| input_ids | CPU torch.int64 [1,S], 고정된 **확장 후** IDs |
| input_features | CPU torch.float32 [1,128,T], 같은 bin; 다시 mel 생성하지 않음 |
| attention_mask | CPU torch.int64 [1,S], 실제 비padding 위치1; batch1 무padding이면 전체1 |
| feature_attention_mask | CPU torch.int64 [1,T], 이 입력은 전체1, 합 T |
| audio_feature_lengths | 생략 권장; 기록할 경우 [1] 값 **T**이며 L이 아님 |
| position_ids | None; 원본 get_rope_index가 실제 mask에서 계산 |
| past_key_values/cache_position | None; 새로운 전체 forward |
| use_cache | False; cache 없는 Rust full forward와 맞춤 |
| inputs_embeds/labels | None; 원본 embedding·audio merge를 실행하고 loss는 생략 |
| return_dict | True; `.logits` 접근 |

중요: 원본 `get_audio_features`는 feature_attention_mask가 없으면 전달된 audio_feature_lengths도 None으로 덮고 다시 `feature_attention_mask.sum`을 호출한다. 따라서 **길이만 전달하는 경로는 source상 실패**한다. mask를 제공하면 합계 T를 쓰므로 원본 코드 수정 없이 호출 가능하다. 이는 현재 요구 호출 계약의 발견이며 zero-duration 원인 확정이 아니다. 이전 준비 문서의 “mask 또는 lengths” 설명은 이 source 세부 제한을 반영하여 읽어야 한다.

## prompt·확장·logits 축

원본 `encode_timestamp`는 `<|audio_start|><|audio_pad|><|audio_end|>` 뒤에 각 word와 `<timestamp><timestamp>`를 결합한다. chat template/BOS/language instruction을 임의 추가하지 않는다. processor는 mel mask 합에서 L을 계산하고 audio pad를 L번 반복한 뒤 tokenizer를 호출한다. 직접 expanded IDs를 입력할 때는 이 확장을 **다시 수행하지 않는다**. 시작151669/끝151670/audio pad151676/timestamp151705, vocab_size152064다. timestamp token 수는2×word 수, audio pad 수는 L, 각 ID는 embedding 범위 안이어야 한다.

thinker는 embedding [1,S,1024]에서 input_ids==151676인 곳을 encoder [L,1024]로 `masked_scatter`한다. source의 get_placeholder_mask 설명과 달리 실제 함수에는 개수 일치 검사가 없으므로 root Rust admission이 정확한 개수를 검사해야 한다. 초과 feature가 조용히 무시되는 경우까지 거부한다. text model의 최종 [1,S,1024] → bias 없는 lm_head → logits **[1,S,5000]**이다. `<timestamp>` 위치 p의 logits는 **동일 위치 p**에서 선택한다. p-1 next-token shift를 하지 않는다. raw class c∈[0,4999], timestamp_ms=c×80이다. token151705와 class c의 축을 혼동하지 않는다.

argmax는 logits class 축 마지막 차원에서 수행한다. 원본 torch tie는 첫 최대 index를 고르므로 Rust 관측의 maximum_tie_count와 lowest-index 처리도 기록한다. 모든5000개 class를 비교하고 top1/top2/margin을 별도 기록한다. repair·round·최종 seconds는 이 raw 비교 뒤의 별도 계약이다.

## 기존 head 증거와 동일입력 한계

아래는 `runtime/ai-spike/results/{label}-output.json`의 `observation.raw_head.model_input_token_ids`, `timestamp_positions`, `timestamp_classifier_rows[].logits`를 직접 읽은 값이다. 각 row5000개 logits 및 F32 LE SHA·margin·tie가 있다.

| label | S / timestamp 수 / words | output SHA256 |
|---|---|---|
| segmentation-ab-ja1516-default-r1 | 321 / 56 / 28 | 5d3cc0ad5082137b1d6a37bc513f9dc14c0e36e824adce2672c67431c2147bd5 |
| segmentation-ab-ja1634-default-r1 | 257 / 62 / 31 | ca489fce4b24c30ccceb2916ff52a8d56da67a572fdb124b3535ded50e9f7b7a |
| segmentation-ab-ja1516-original-r1 | 323 / 56 / 28 | 6210ff5c311ba98a009d7dca560127c81bdb754f09db3acea257630c0b70a3c4 |
| segmentation-ab-ja1634-original-r1 | 259 / 64 / 32 | 32e6be1de7be864e795a7f079667fb2b640ff53f18dcfa825c0982639f3f33cc |

같은 words 수에서도 S가 달라진다. 원본/기본 단위·정규화·prompt를 혼합하거나 위치별 zip을 하지 않는다. JA1516 control은 같은 기본 입력의 동일 결과를 보인 통제군이며 새 neural 반복이 필요하다는 뜻이 아니다. 과거 forced-head-ja*는 별도 historical baseline이며 actual expanded IDs가 추가된 위 결과와 구분한다.

KO 최초 `forced-align-ko-r1-output.json` SHA `fcd8722d96fba8fae346ab616ed87140518d2b8505c5bb43effa428930486441`에는 prompt/words는 있지만 raw_head/expanded IDs/전체 logits가 없다. **새 Rust 관측 없이는 KO raw 일치를 판정할 수 없다.** JP 과거 결과도 full official NumPy mel을 주입한 실행이 아니므로 새 동일 mel 비교와 bitwise 동등 baseline으로 취급하지 않는다. root가 같은 mel·IDs로 실행한 Rust 관측을 새 frozen 증거로 마련해야 한다. 현재 기존 명령이 외부 mel/IDs를 수용한다고 주장하지 않는다.

tokenizer 원본 파일은 vocab2776833B/SHA `ca10d7e9fb3ed18575dd1e277a2579c16d108e32f27439684afa0e10b1440910`, merges1671853B/`8831e4f1a044471340f7c0a83d7bd71306a5b867e95fd870f74d0c5308a904d5`, tokenizer_config12666B/`3ab80063f8511deb9566e6ad438d17b7a6277fcffd52d92854112f19d36bd81c`다. 공식 tokenizer/processor parity가 아직 검증됐다는 뜻은 아니다. 첫 neural 비교는 frozen IDs를 직접 제공해 이 변수를 통제하고, 원본 processor 재생성 IDs의 exact 일치는 별도 비추론 게이트로 남긴다.

## text eager·mask·position source 비교

원본 `Qwen3ASRThinkerTextModel.forward`는 Transformers4.57.6 `create_causal_mask`를 호출한다. 기존 private site-packages의 `masking_utils.py` eager_mask는 causal/key padding boolean을 F32의0 및 `torch.finfo(dtype).min`으로 바꾼다. batch1/no-cache/all1에서는 key≤query가 허용되는 [1,1,S,S]다. Rust `model/attention.rs::make_causal_mask`도 tril×key mask에0/f32::MIN을 쓴다. encoder의 무mask eager와 **text의 causal mask를 혼동하지 않는다**.

원본은 q/k linear→각 head128 RMSNorm→rotary, KV8head를 Q16head로2배 반복→QKᵀ/√128+mask→F32 softmax→V→o_proj다. 실제 decoder layer가 사용하는 class는 `Qwen3ASRTextAttention`이며 비슷한 이름의 `Qwen3ASRThinkerTextAttention`만 보고 판단하지 않는다. Rust CPU 경로도 동일 순서의 연산을 사용한다. 구조 대응은 수치 일치 증거가 아니며 attention kernel/GEMM/RMSNorm 차이는 실제 tensor로 확인한다.

원본 get_rope_index는 attention_mask.float() cumsum-1, pad위치1, [3,1,S] 확장이다. 따라서 이 호출 경로의 position_ids는 source상 F32이며 Rust의 I64 위치와는 논리 정수값을 비교하고 실제 dtype을 따로 기록한다. cache_position은 torch.arange의 정수다. 이번 전부1 mask에서는 세축 모두0..S-1, rope_delta0, cache_position0..S-1이다. Rust get_rope_index도 좌측padding 가정 아래 같은 위치값이다. config는 theta1000000/head_dim128/mrope_section[24,20,20]/interleaved=true다. 원본 rotary는 F32 inv_freq·position matmul 후 interleaved3축, 복제한 cos/sin을 반환한다. Rust는 반차원3축 cos/sin을 만들고 적용 시 interleave하므로 **중간 tensor 형상 자체를 그대로 zip하지 않는다**. 실제 q/k에 적용된 결과 또는 정규화한 논리 cos/sin을 비교한다. 세축 position이 같은 이번 입력에서는 modality 재배열을 잘못해도 드러나지 않을 수 있어 이 cohort로 일반 multimodal rotary를 승인하지 않는다.

읽은 Rust source는 `runtime/ai-spike/source/lumos-segmentation-7984967/qwen3_asr/src/`다. `model/thinker.rs` SHA256 `f13ac5efacecc94c2f716029838faf0b9cfdfce29c44cdca11689a2bb9a28288`, `model/thinker_text.rs` `3ebb996e8a4e724141c9003e83fd18ae0f49151316796f1c6e21c952f9c50135`, `model/attention.rs` `b81edead070e6ee40b06497f33283dd2be4a0dce805a469a0421c9f95e4e853e`다. source hash 판독은 새 full-model executable의 compiled-input attestation을 대신하지 않는다.

## root 최소 실행·관측 게이트

1. **비추론 admission**: 기존 source/env/model manifest, config/head shape/tie flag, 실제 checkpoint key 완전성, PCM/transcript/mel/prompt/words/expanded IDs를 고정한다. mask/length/pad/timestamp positions/5000class를 확인한다. KO 누락 IDs와 동일 mel Rust 관측 계약을 먼저 마련한다. 기존 frozen 결과를 덮어쓰지 않는다.
2. KO 한 건, CPU/F32/batch1/threads4/inter-op1/eager/eval/no_grad/use_cache=false로 실제 thinker를 한 번 호출한다. logits [1,S,5000] 전체는 필요하면 보존하되 최소 timestamp [2W,5000] 전체를 little-endian row-major F32로 저장한다. shape·finite·SHA·raw argmax·tie/margin을 함께 고정한다. KO에서 계약/수치 실패면 JP로 확대하기 전에 첫 차이 지점을 조사한다.
3. KO 게이트 뒤 JP1516 기본(S321), JP1634 기본(S257)을 같은 조건으로 각1회 수행한다. 원본 nagisa expanded IDs는 그 뒤 별도 한 변수 A/B로만 다룬다. 이번 최소3건에 과거17회를 더해 새로운 독립 품질 표본으로 표현하지 않는다.
4. shape·모든입력ID·timestamp positions·class axis는 **exact**; 유한값100%; logits 모든 원소는 사전 제안 기준 `abs(actual-ref)<=1e-4+1e-4*abs(ref)`로 비교한다. 해당 게이트는 이번 source 조사에서 통과하지 않았다. root는 실행 전에 고정하고 실패 뒤 완화하지 않는다. raw class IDs는 모든 timestamp 위치 **exact** 일치를 별도 요구한다. logits 허용오차 통과지만 작은 margin으로 argmax가 바뀌면 raw-ID 게이트 실패다. 반대로 argmax만 같고 logits 차이가 크면 tensor 게이트 실패다.
5. Rust/PyO3 native 공개 forward/pre-hook으로 embed_tokens, thinker.model 입력의 **merge 후 embeddings**, rotary 출력, layer0..27 출력, model.norm, lm_head 입력/출력을 무변조 관측한다. decoder layer 반환은 원본 tensor이며 encoder layer tuple과 다르다. pre-hook with_kwargs로 실제 mask/position/cache도 기록한다. functional masked_scatter를 원본 재작성하지 않고 text-model pre-hook에서 결과를 읽는다. Q/K rotary 뒤는 functional 경계라 현재 공개 module hook만으로 직접 얻었다고 주장하지 않는다. 기존 hook 어댑터 확장/새 Rust 대응 stage dump는 root 작업으로 남는다.

## 자원·라이선스·미검증

이 단계는 encoder316427904 params만이 아니라 embedding/text28layers/head도 로드한다. BF16 weight 파일 약1.835GB와 전체 F32 상주·원본 state_dict·변환 복사·capture가 중첩될 수 있다. from_pretrained 저메모리 loading 또는 safetensors 단위 로드 선택은 root가 실제 peak와 strict 검증을 유지하며 정한다. 여기서 새 weight load는 하지 않아 전체 peak/로드 성공을 추정값으로 승인하지 않는다. attention score는 F32 기준 layer당16×S²×4B; S321이면 약6.6MB이며28층 전체 attention matrix를 hook으로 보존하면 추가비용이 크다. 최종 logits는 S×5000×4B이고 timestamp rows는2W×5000×4B다. 필요 stage를 사전 고정하고 hidden tensor를 detach/cpu/contiguous 후 즉시 직렬화해 객체 수명을 제한한다. no_grad는 activation capture 비용을 없애지 않는다.

원본 Qwen source/model Apache-2.0, PyTorch 및 전이 package 조건을 유지한다. 기존 환경의 soynlp GPLv3·soxr LGPL 및 native 번들 조건도 사라지지 않는다. 연구 재현 환경을 제품 배포 허가로 간주하지 않는다. 새 다운로드/설치/기존환경 변경은 이번 업무 밖이다.

full-model strict load/key수, 실제 dtype/eager 전파, 입력 processor parity, text/head tensor 수치, raw argmax 및 zero 원인은 미검증이다. source 대응만으로 오류를 확정하거나 mask/position/시간/단위를 수정하지 않는다. 이번 변경은 본 신규 문서와 research coordination뿐이므로 cargo 검사·모델 실행은 하지 않았다.
