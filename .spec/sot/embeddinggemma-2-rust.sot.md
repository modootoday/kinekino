---
id: sot:kinekino/embeddinggemma-2-rust
title: EmbeddingGemma 2 Rust 실행 경계 조사
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# EmbeddingGemma 2 Rust 실행 경계 조사

2026-10-19 공식 [Google 발표](https://blog.google/innovation-and-ai/technology/developers-tools/embeddinggemma-2/)와 [HF 원본 card](https://huggingface.co/google/embeddinggemma-2)를 재확인했다. 정확한 이름은 EmbeddingGemma 2이며 생성 LLM 후보와 구분한다. 후속 [동일 입력 FP32/q4 실제 대조](../../../../verification/ai/embeddinggemma2-quantized.md)는 text 경로만 검증한 결과다. 멀티모달 지원 선언을 이미지·오디오·영상 경로의 실측 완료로 확대하지 않는다. 생성 모델의 Gemma·Llama·Qwen 경량 후보 비교는 [별도 family 연구](lightweight-model-families.md)를 따른다.

2026-10-18. 기존 [경량 후보](lightweight-model-families.md)를 보충하는 source/metadata 조사다. 모델·engine 취득/설치/빌드/추론은 하지 않았다. 원문은 네트워크에서 읽었지만 source 파일·가중치·ONNX graph를 로컬에 저장하지 않았다. 기존 frozen 증거와 root의 정렬/ASR/음악 실행은 변경·재검증하지 않았다.

**선정 제안:** 첫 Windows CPU text-only spike는 기존 Rust `ort` 사용 경험을 바탕으로 **EmbeddingGemma2 전용 5입력 어댑터**를 분리하는 경로가 가장 좁다. 확인한 Candle/mistral.rs에는 동등한 전용 CPU Rust 구현을 찾지 못했다. 이는 Rust 전체 생태계의 부재 선언이 아니다. 외부 ORT는 Rust의 명시적 어댑터로 연결하며 Python 개발 코드나 새 server를 도입하지 않는다. 실제 graph metadata·연산자·원본 수치 parity는 root의 실행 게이트로 남는다.

## 원본 모델과 정본 계약

조회한 `google/embeddinggemma-2` main은 **`914f7f89142e33e77833254d9c9b90c3cef7303b`**, gated=false, Apache-2.0 metadata다. source 모델은 단일 `model.safetensors` 1,488,915,288 B, LFS SHA256 `197a32965d4b1105faf060417baa899e193fb73cd401f42ec9295234d5553d79`다. 이 파일 전체를 text-only 초기 실험에 반드시 내려받아야 한다는 의미는 아니다.

`architectures=[EmbeddingGemma2Model]`, `model_type=embedding_gemma2`, text는 `embedding_gemma2_text`다. 이전 EmbeddingGemma 300M의 `Gemma3TextModel`, Gemma2 generation, Gemini Embedding 2 API와 다르다. Gemma4 기반이라는 이유만으로 generation loader를 재사용하지 않는다.

주요 text config는 24 layers, hidden512, output projection768, PLE hidden-per-layer512, 262144 vocab, sliding512/full attention 교대 및 layer05/11/17/23의 별도 head dimension/KV head 설정이다. bidirectional encoder와 512→768 head가 필요하다. 일반 causal Gemma4의 마지막 token→vocabulary logits는 같은 기능이 아니다.

공식 구성 파일에서 확인한 계약:

| 책임 | 계약 |
| --- | --- |
| 검색 query | `task: search result \| query: {text}` |
| 문서 | `title: none \| text: {text}`; 실제 title이 있으면 그 title을 사용 |
| tokenizer | 원본 tokenizer.json과 tokenizer_config, Gemma special tokens, right padding. 모델 최대 길이의 거대 sentinel을 실제 허용 context로 쓰지 않음 |
| pooling | `1_Pooling/config.json`: 768, mean, **include_prompt=true**. prompt와 special-token 처리 정책을 임의로 빼지 않음 |
| normalize | mask-aware mean 후 float32 L2. 그래프가 이미 sentence_embedding을 반환하면 token pooling을 다시 하지 않음 |
| MRL | 첫768차원 기준을 보존하고 prefix512/256/128을 취한 뒤 다시 L2. query/document 차원과 pipeline identity를 일치시킴 |
| 선택 적재 | 공식 Python model constructor는 vision/audio config=None이면 tower/projector를 만들지 않음. 이 사실은 어떤 Rust loader든 자동 지원한다는 뜻이 아님 |
| context | 카드의 8192 토큰과 config.max_position_embeddings=262144를 구분. 최소 spike는 batch1/최대256 tokens로 별도 제한하고 truncation을 기록 |

근거: [고정 모델 카드](https://huggingface.co/google/embeddinggemma-2/blob/914f7f89142e33e77833254d9c9b90c3cef7303b/README.md), [config](https://huggingface.co/google/embeddinggemma-2/blob/914f7f89142e33e77833254d9c9b90c3cef7303b/config.json), [prompt catalog](https://huggingface.co/google/embeddinggemma-2/blob/914f7f89142e33e77833254d9c9b90c3cef7303b/config_sentence_transformers.json), [pooling](https://huggingface.co/google/embeddinggemma-2/blob/914f7f89142e33e77833254d9c9b90c3cef7303b/1_Pooling/config.json), [Transformers 설명](https://huggingface.co/docs/transformers/v5.19.0/en/model_doc/embedding_gemma2). v5.20.1 tag가 가리키는 source commit은 `0ea877911a11499053df677befd4b35b172ff502`; [model constructor/head](https://github.com/huggingface/transformers/blob/0ea877911a11499053df677befd4b35b172ff502/src/transformers/models/embedding_gemma2/modeling_embedding_gemma2.py)를 읽었다. 실제 Python oracle 실행은 없다.

## Rust 및 명시적 외부 엔진 후보

| 후보/고정 source | 확인 | 초기 선택 |
| --- | --- | --- |
| Candle `12bb5d1f6350f4ea9827d9086f213feec1f0af6a` | models module/tree에는 Gemma4/quantized Gemma3 등이 있으나 전용 EmbeddingGemma2 경로는 확인하지 못함. Gemma4 text는 causal mask, 마지막 token, lm_head vocab logits 경로 | 바로 실행 후보 아님. full/sliding bidirectional·PLE·projection/pooling의 구현 차이를 단순 config alias로 숨기지 않음 |
| mistral.rs `28ed6daca455b4ae063896014d1ea698b912c539` | embedding loader의 architecture match는 Gemma3TextModel과 Qwen3ForCausalLM. EmbeddingGemma2Model은 unsupported 오류 | 기존 Gemma4/ISQ/embedding 지원을 이 모델의 지원으로 승격하지 않음 |
| rightkit-embed **0.5.0** | 공개 Rust source에 embedding_gemma2 분기, tokenizers encode_batch(add_special_tokens=true), ORT CPU, prefix/L2/MRL 구현 존재 | 참고 구현. 현재 onnx-community graph의 drop-in wrapper로 채택하지 않음 |
| llama.cpp `5aeac37a35b44fde4bdffaec9159f2fcb253b622` | gemma-embedding2 architecture 등록·factory·별도 graph·converter 실제 존재. symmetric SWA, noncausal, PLE, output projection을 구현 | 외부 native 대안은 존재. Rust 바인딩 revision/빌드·수치 검증을 추가로 해야 함 |
| llama-cpp-rs `224a1a3412b1744d5e17a8817e161cddb7c580aa`, crate0.1.161 | vendored llama.cpp gitlink=`d81235049384534c167caea52b85a694f6103d14`; 해당 tree에는 이전 gemma-embedding.cpp만 있고 embedding2 구현 없음 | 최신 Rust wrapper라는 이유로 최신 llama.cpp 모델 지원을 가정하지 않음. submodule 교체·ABI/bindings 재검토 없는 즉시 실행은 부적합 |
| taconite-embeddinggemma2 0.2.0 | 실제 Rust forward/processor가 있으나 AMD XDNA NPU의 IRON/XRT 또는 amdxdna ioctl kernel bundle 사용. docs target Linux, NPU 전용 GEMM/MHA | Windows CPU 대안 아님. 라이선스도 Apache-2.0 AND BSD-3-Clause AND MIT-CMU로 별도 검토 대상 |

근거: [Candle text](https://github.com/huggingface/candle/blob/12bb5d1f6350f4ea9827d9086f213feec1f0af6a/candle-transformers/src/models/gemma4/text.rs), [mistral loader](https://github.com/EricLBuehler/mistral.rs/blob/28ed6daca455b4ae063896014d1ea698b912c539/mistralrs-core/src/pipeline/loaders/embedding_loaders.rs), [rightkit published source](https://docs.rs/rightkit-embed/0.5.0/src/rightkit_embed/embedder.rs.html), [llama graph](https://github.com/ggml-org/llama.cpp/blob/5aeac37a35b44fde4bdffaec9159f2fcb253b622/src/models/gemma-embedding2.cpp), [converter](https://github.com/ggml-org/llama.cpp/blob/5aeac37a35b44fde4bdffaec9159f2fcb253b622/conversion/gemma.py), [binding tree](https://github.com/utilityai/llama-cpp-rs/tree/224a1a3412b1744d5e17a8817e161cddb7c580aa/llama-cpp-sys-2), [NPU crate](https://docs.rs/taconite-embeddinggemma2/0.2.0/taconite_embeddinggemma2/).

rightkit-embed published crate checksum은 `397f3b3467791d3c11cb1bd0aa2c9575216a062060c716b0a1fad908c2842684`, `.cargo_vcs_info.json`은 `64beb174fef9fb0c1574f28a2dfe85fabcbebe36`/`crates/rightkit-embed`를 가리킨다.

위 crate의 repository metadata는 Orthic-Labs/rightkit지만 GitHub API 조회는404였다. 따라서 공개 published source를 근거로 삼았으며 Git commit의 저장소 내용 동등성은 검증하지 못했다. Rust 최소 버전 선언1.98.0, 라이선스 MIT OR Apache-2.0, rightkit-onnx0.2.2 및 tokenizers0.22.2 의존을 확인했다. 별도 SQLite/vector-store까지 제품에 도입할 이유는 없다.

rightkit source는 `input_ids`/`attention_mask`와 선택적 token_type_ids만 공급하고 `sentence_embedding`을 읽는다. **onnx-community의 현행 모델은 modality feature 입력 3개가 추가로 필요**하다. 같은 `model_q4.onnx` 파일명과 model_type만으로 graph 계약이 같다고 보면 안 된다. 이 crate가 쓰는 text-only 별도 export의 전체 변환 source와 원본 revision 연결은 이 조사에서 확보하지 못했다. CPU 실행과 DirectML 측정은 author 보고이며 우리 host의 실측이 아니다. 특히 author의 DirectML parity 실패 보고를 보고도 finite output만으로 가속을 승인하지 않는다.

## ONNX: 텍스트만 실제로 적재할 수 있는 경계

조회한 `onnx-community/embeddinggemma-2-ONNX` revision은 **`daa72c51243991dfcaf9f9137d2c573d8f7790c0`**. text graph `model*`, vision_encoder*, audio_encoder*가 별도 파일이다. 텍스트 경로는 vision/audio session을 만들지 않고 text graph와 external data·tokenizer만 적재하는 구조다.

[HF Transformers.js 공식 model source](https://github.com/huggingface/transformers.js/blob/511bb61a7c46b73e33f1af3fb308fbd1afe98c02/packages/transformers/src/models/embedding_gemma2/modeling_embedding_gemma2.js), commit **`511bb61a7c46b73e33f1af3fb308fbd1afe98c02`**에서 실제 forward가 공급하는 입력을 확인했다. JS를 제품 실행기로 사용하자는 제안이 아니라 Rust로 옮길 source 계약 근거다.

| text-only 입력 | source가 보여주는 값 |
| --- | --- |
| input_ids | tokenizer가 만든 [B,T] token IDs; Rust ORT에서 실제 tensor dtype 확인 후 공급 |
| attention_mask | [B,T] padding mask; prefix와 special tokens 포함 정책 고정 |
| image_features | float32 **[0,512]** |
| video_features | float32 **[0,512]** |
| audio_features | float32 **[0,512]** |

모달리티 placeholder 개수와 해당 feature row 개수가 같아야 한다. text-only에서는 세 placeholder가 없어야 하며, literal image/audio marker를 넣어 벡터가 나왔다고 media embedding으로 표시하지 않는다. width는 text hidden512이고 sentence output768과 다르다.

source 설명상 출력은 token `last_hidden_state`와 이미 mean-pooled/L2인 `sentence_embedding`이다. root가 실제 graph를 취득한 뒤 Session metadata로 입력 이름·dtype·rank·output을 확인해야 한다. 이 연구는 binary ONNX를 읽지 않았으므로 opset·모든 node/operator·external-data 내부 경로를 확정하지 않았다. 예상 5입력과 달라지면 새 graph를 임의 fallback으로 수용하지 말고 차이를 기록한다. existing E5 adapter의 token_type_ids/384차원 mean 계약을 그대로 통과시키면 안 된다.

### 조회한 text artifact pin

다음 값은 고정 HF API LFS metadata이며 파일을 내려받아 SHA를 재계산한 결과가 아니다. 둘로 나뉜 graph/data를 함께 고정한다.

| 파일 | bytes | LFS SHA256 |
| --- | ---: | --- |
| onnx/model.onnx |421421|`bc47de15f81208a5c99e5ab10f746d5e33b51ea228b7dc0bef9c133a94f1c1c3`|
| onnx/model.onnx_data |1084170240|`9fd452bfc6916e5a92f080f8dada35598ca4512b21c65897d4c5974d5f9630a2`|
| onnx/model_quantized.onnx |495165|`d06edd601f851c633a2519304cbeb8dc6170d7ceb61b436625c17fb9b6e74953`|
| onnx/model_quantized.onnx_data |313724928|`278a7ff1248c3618e4bd11a607fc54f7bdc7778854230f3956d3f86bd9db4f3b`|
| onnx/model_q4.onnx |490742|`f9eeba97acddf139b8ee2ddf04bc30dceafa88de93fadf74d7644e0d61a477a9`|
| onnx/model_q4.onnx_data |174028800|`c3975f2d1ab7a1878ae31a7d7a9b7804a827aff3800b60dfceafce21cac3df49`|
| tokenizer.json |32170510|`4d777ef5bdc1aa36227abdfb77c3e49e7b9c892d16e1b6bda41c393504828be4`|

card에는 base_model=google/embeddinggemma-2 및 Apache-2.0이 있지만, 읽은 범위에서 exporter commit/opset/quantizer version/보정 조건과 원본 가중치 revision의 완전한 대응은 확인하지 못했다.

이 계보 누락을 원본 동등성으로 채우지 않는다. root는 취득할 때 원문 card/config·transformers_version=5.18.0.dev0 선언·graph/operator inventory·실제 SHA를 보존하고 변환물끼리의 비교와 원본 oracle parity를 구분한다. Optimum-ONNX HEAD `46533fb66570021f52a39d77112b703762fdaada`의 model_configs에서 전용 등록을 이번 텍스트 검색으로 찾지 못했다. 따라서 일반적인 Optimum 지원만으로 이 export의 재현 명령을 발명하지 않는다.

## 정밀도·양자화·Windows CPU

FP32 text graph를 첫 수치 기준으로 삼고 q8/q4는 별도 pipeline identity로 비교한다. FP16/BF16 파일은 CPU의 빠른 실행이나 낮은 peak를 보장하지 않는다. ONNX q4, GGUF Q4, mistral ISQ, MRL128d는 각각 다른 처리이며 교환할 수 없다.

[ORT 양자화 설명](https://onnxruntime.ai/docs/performance/model-optimizations/quantization.html)과 [v1.24.4 CPU MatMulNBits registration](https://github.com/microsoft/onnxruntime/blob/v1.24.4/onnxruntime/contrib_ops/cpu/quantization/matmul_nbits.cc)에서 float/MLFloat16 CPU kernel 구현은 확인했다. 이것이 선택한 DLL에서 이 graph의 모든 연산자가 동작한다는 증거는 아니다. 기존 root tool은 `ort=2.0.0-rc.14`/api-31이며 임의로1.24.4로 낮추지 않는다. 실제 선택한 DLL의 version/hash·API·opset·contrib kernels를 root가 대조한다.

ONNX card의 q4/q8 cosine·속도·크기는 변환자 측정이며 한국어/일본어 자막의 순위 보존이나 이 Windows CPU의 peak를 보장하지 않는다. DirectML/GPU/NPU는 초기 경계에서 제외한다. 길이0 feature tensor와 external data 지원도 실제 Session load/run으로 검증한다.

### GGUF의 별도 계약

`ggml-org/embeddinggemma-2-GGUF` revision **`bfcd298762cc34d0357ece5ebdd31791a3a374d8`**의 `.src_sha`는 `PRIMARY=914f7f89142e33e77833254d9c9b90c3cef7303b`다. card는 자동 변환을 ggml-org/convert와 연결한다. 다만 정확한 converter 실행 commit과 수치 parity까지 확정하지 않는다.

- text BF16 557950176 B / SHA256 `68bae29d62fb8c7d23e98d21fd4662753ddd636e6b62b8a70dfe059a9844f216`.
- text Q8_0 309855456 B / `2188ac1deca4b77dffefd603c2776a9d76d9d74ec01841392982ebb840b09135`.
- vision/audio용 mmproj는 별도982074784 B(BF16) 또는554821024 B(Q8_0). text-only에서 이 추가 파일을 적재하지 않는다.

llama.cpp converter는 EmbeddingGemma2Model을 등록하고 bidirectional sliding window 폭 변환·noncausal·output embedding length를 설정한다. 따라서 단순 GGUF 파일 존재보다 강한 지원 근거다. 현행 Rust binding 내장 commit이 오래된 점은 남는다. 최신 native engine을 Rust FFI에 고정하고 Windows CPU에서 검증하는 추가 작업이 필요하며 기존 mistralrs.exe에 전달하는 것만으로 해결되지 않는다. 멀티모달 server endpoint의 issue/PR 제목을 실제 media→vector 연결 완료의 근거로 쓰지 않는다.

## root에 인계하는 최소 spike 계약

1. **책임을 분리한 Rust adapter**에 original/converted revision, 모든 입력 파일 SHA, runtime DLL/EXE/build source를 고정한다. text-only ORT CPU, batch1, 최대256 tokens, intra4/inter1, 단일 worker로 제한한다. text graph/data·tokenizer 외 vision/audio 가중치를 취득하거나 Session을 만들지 않는다. 기존 E5/MiniLM 계약을 바꾸지 않는다.
2. 준비 단계에서 Session metadata와 external data를 읽어 위5입력/예상 출력을 대조한다. metadata 차이와 미지원 operator를 raw failure로 보존하고 BERT/causal Gemma loader로 fallback하지 않는다. 모델 head를 생략하거나512dim을768dim으로 취급하지 않는다.
3. 기존 동일 자막·무관 query·query/document prefix를 사용하고 BOS/EOS 등을 포함한 token IDs·mask·실제 truncation·prompt 문자열 hash를 보존한다. native768 shape/finite/norm과 검색 rank를 관측한다. 필요하면 token 출력의 mean을 독립 재계산해 sentence_embedding과 비교하되 이중 pooling하지 않는다.
4. FP32가 성립한 다음 **같은 입력의 q8 또는 q4 한 조건만** 별도 실행해 비교한다. cosine, top-k/rank 변화, 실패, peak private/wall을 기록한다. MRL은 같은768 출력의 후단 절단·재정규화 비교로 분리한다. 원본 PyTorch oracle 미검증 상태를 남기고 구조 성공을 모델 채택/품질 승인으로 표시하지 않는다.

이것은 구현 경계 제안이며 실행 완료 절차가 아니다. root가 단일 worker를 소유하고 연구 담당은 새 process를 시작하지 않는다. HTTP server나 Python bootstrap을 도입할 필요가 없다.

## 이후 vision/audio 확장과 권리

media는 단독 encoder의 pooling 벡터만 쓰는 것이 아니라 같은 text backbone에512폭 soft tokens와 대응 placeholder를 공급해 최종768로 보낸다. ONNX encoder 파일 분리는 확장 근거지만 Rust processor 완성 증거가 아니다. vision은 Gemma4 patch16/pool3/resize/rescale·position IDs·기본soft280을 고정한다. audio는 공식 processor의16k/128bins/frame320/FFT512/hop160/mel floor0.001/40ms per token을 쓴다. 기존 Whisper/Qwen의FFT400/mel floor를 재사용하지 않는다. video fps1/max32/uniform/frame budget140과 단일 image280도 구분한다. encoder 출력 수와 marker 수를 실측한 후 cross-modal 품질을 평가한다.

원본 model card Apache-2.0과 [Google 원문 Apache 안내](https://ai.google.dev/gemma/apache_2)를 확인했다. 이번 HF tree에 독립 LICENSE/NOTICE가 없는 점과 license metadata/링크 보존을 구분한다. ONNX/GGUF 변환물·ORT MIT·Rust wrapper·tokenizer·native llama.cpp 고지와 변환자 코드/부속물은 별도 inventory다. 공개 접근만으로 전체 변환 계보나 최종 배포 패키지 검증 완료를 주장하지 않는다.

이번 검증은 primary source/metadata와 기존 Rust source 읽기뿐이다. 코드 변경이 없으므로 cargo fmt/test/clippy는 실행하지 않는다. Windows build, 실제 graph 전체 node 검사, 추론·품질·peak·multimodal parity는 미검증으로 root에 인계한다.
