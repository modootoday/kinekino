---
id: sot:kinekino/retrieval-embedding
title: 경량 한국어·다국어 retrieval embedding 후속 연구
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 경량 한국어·다국어 retrieval embedding 후속 연구

조사일: 2026-10-18, HF metadata/source 조회 기준 14:12~14:16 KST. **다음 실제 스파이크는 `intfloat/multilingual-e5-small`의 공식 FP32 ONNX를 우선한다.** 검색 학습·필수 query/passage 접두사를 현재 MiniLM과 비교하면서, 기존 Rust ORT·tokenizers·384차원 masked mean/L2 계산을 가장 적게 변경할 수 있다. 이 선택은 한국어 품질 개선을 실증했다는 뜻이 아니다. 최신 후보로 EmbeddingGemma 2 텍스트 전용 ONNX를 2순위에 두고, EmbeddingGemma-300m은 별도 계약의 비교 후보로 둔다.

이 문서는 후속 조사/제안이며 제품 채택 규격이나 실행 결과가 아니다. 공식 카드·텍스트 config/source·HF API metadata만 읽었다. 가중치·ONNX graph·tokenizer binary를 다운로드하지 않았고 설치·빌드·추론·코드 수정도 하지 않았다. 아래 SHA256은 **HF LFS metadata의 기대값**이며 로컬 파일을 해시 검증한 값이 아니다. API는 `/api/models/{repo}?blobs=true`, revision은 응답의 전체 `sha`, 파일명/bytes/hash는 `siblings`를 읽었다. API metadata와 다운로드 후 실제 SHA256을 구별한다.

## 현재 관측을 유지할 기준

root의 [FP32 평가](../../../../../../runtime/ai-spike/results/embedding-fp32-observed-r1-semantic-evaluation.json), [AVX2 평가](../../../../../../runtime/ai-spike/results/embedding-avx2-observed-r1-semantic-evaluation.json), [쌍 비교](../../../../../../runtime/ai-spike/results/embedding-fp32-avx2-comparison-r1.json)를 읽었다. 고정 MiniLM revision은 `e8f8c211226b894fcb81acc59f3b34ba3efd5f42`이며, 동일 fixture SHA256은 `b20ea540f1784e21acc5d78b6eb355ca465d39687b271a33ac8facff7cc237b8`다.

실제 한국어 자막 26개와 임시 의역 query 5개·negative 1개·exact-copy control 1개로 총 33개를 embedding했다. positive 5개에서 provisional MRR은 FP32 0.55 / AVX2 0.5533333333333335, hit@3은 둘 다 0.8이다. '찾지 못했다' 관련 cue 20의 순위는 12/10, 고마움 의역은 3위였다. exact-copy와 negative는 positive 집계에서 제외됐다. assistant가 작성한 임시 관련성 집합이며 독립 인간 gold가 아니다. 이 수치로 일반 한국어·다국어 품질, no-match threshold, highlight 품질 또는 양자화 속도 우열을 승인하지 않는다. 원본 수치와 실패 사례를 후보 조사로 덮어쓰지 않는다.

현재 [embedding.rs](../../../../../../tools/model-spike/src/infrastructure/inference/embedding.rs)는 raw text를 `encode(text, true)`로 처리하고 최대 128 tokens, padding 없음, `int64 input_ids/attention_mask/token_type_ids`를 넣는다. 첫 output `[1,n,384]`를 가정한 뒤 mask-aware mean/L2를 적용한다. 후보별 접두사·입출력·projection 처리를 여기에 무조건 끼워 맞추면 안 된다. 현재 observed-run allowlist도 기존 MiniLM에 고정돼 있으므로 새 후보는 검토된 별도 계약이 필요하다. 이 문서에는 미구현 CLI를 실행 가능한 명령처럼 제시하지 않는다.

## 후보 비교와 우선순위

| 후보 | 공식 모델/학습 근거 | 토큰·최종 차원 | Rust 경로와 현재 adapter 차이 | CPU·권리·재현 관점의 판정 |
|---|---|---|---|---|
| **1. multilingual-e5-small** | Microsoft 연구진 E5; 12층 BERT 계열, multilingual retrieval contrastive 학습. 공식 카드에 한국어 Mr.TyDi 결과도 있음 | 최대 512, 384d | 공식 repo ONNX → Rust `ort`; XLM-R tokenizer JSON → Rust `tokenizers`; query/passage 접두사 + masked mean/L2. 실제 ONNX signature 확인 필요 | FP32 약 470 MB, 지금과 같은 차원/계산으로 비교 비용 작음. card MIT. 공식 양자화는 AVX512-VNNI용이며 AVX2 배포라고 부르지 않음 |
| **2. EmbeddingGemma 2, text-only** | Google의 2026-10 신형; 전체 740M 중 텍스트 270M, 100+언어. Gemma 4 기반 | 카드 8,192, 768d; MRL 512/256/128d | ONNX Community graph + Rust ORT 가능 경로. ids/mask 외 empty media features 필요; graph가 최종 sentence embedding을 제공. 384d 또는 Gemma 3로 취급 금지 | 텍스트 FP32 graph+data 약 1.085 GB, q4 약 175 MB. Apache-2.0. 신형 공식/변환 자료와 Rust로 옮길 수 있는 명시 I/O 경로는 있으나 로컬 kernel/signature/정확도는 미검증 |
| **3. EmbeddingGemma-300m** | Google 2025-09 모델, 300M 계열(전체 pipeline 약 308M), 100+언어 | 최대 2,048, 768d; MRL 512/256/128d | ONNX Community FP32 graph는 token output과 **최종 sentence output**을 제공하는 공식 HF ORT 예제 있음. 원래 pipeline은 mean 뒤 Dense 2개; 단순 mean/L2 대체 금지 | FP32 graph+data 약 1.235 GB, q4 약 197 MB. Gemma 독자 라이선스; 원본 raw config는 인증 제한으로 못 읽음. Gemma 2보다 우선할 근거 부족 |

학습/평가 근거는 [E5 기술 보고서](https://arxiv.org/abs/2402.05672), [E5 공식 카드](https://huggingface.co/intfloat/multilingual-e5-small), [Google EmbeddingGemma 2 카드](https://huggingface.co/google/embeddinggemma-2), [HF EmbeddingGemma 소개](https://huggingface.co/blog/embeddinggemma)다. 공개 benchmark는 query/corpus·평가 버전이 달라 이번 5-query MRR과 수평 비교하지 않는다. E5의 한국어 지원이나 Google의 multilingual 성능 표는 우리 자막 검색 품질 증명이 아니다.

최근 대안도 선별했다. [Microsoft Harrier-270m](https://huggingface.co/microsoft/harrier-oss-v1-270m)은 MIT 표기와 원본 revision `31de22b673913c7d658c0f03f792d77c2dcf8ebd`를 확인했지만 해당 공식 repo 파일목록에 ONNX가 없어 이 짧은 ORT 후속의 우선안에서 제외했다. [Jina v5-text-nano](https://huggingface.co/jinaai/jina-embeddings-v5-text-nano)는 2026-02 출시·239M·last-token/task adapter 계약이지만 CC-BY-NC-4.0이며, mean pooling 후보와 같게 취급하지 않는다. 두 모델의 모든 변환 저장소/Rust 구현을 부재로 판정한 것은 아니다. 이 조사에서 확인한 안정적인 경로와 제품 제약을 기준으로 세 후보에 집중했다.

## 고정 revision과 실제 배포 파일

원본과 변환물은 다른 저장소다. 다음 revision은 이번 조회 시점의 **선택 가능한 동결 identity**이며 과거 변환자가 사용한 정확한 원본 SHA를 소급 입증하지 않는다. 모든 bytes는 십진 byte 수다. graph와 `.onnx_data`를 반드시 한 쌍으로 취급한다.

| 역할 | 저장소·전체 revision | 관측한 실제 파일 |
|---|---|---|
| E5 원본+공식 ONNX | [intfloat/multilingual-e5-small](https://huggingface.co/intfloat/multilingual-e5-small/tree/614241f622f53c4eeff9890bdc4f31cfecc418b3) `614241f622f53c4eeff9890bdc4f31cfecc418b3` | `model.safetensors`; `onnx/model.onnx`; `onnx/model_qint8_avx512_vnni.onnx`; `onnx/model_O4.onnx`; `tokenizer.json`; `tokenizer_config.json`; `1_Pooling/config.json` |
| Gemma-300m 원본 | [google/embeddinggemma-300m](https://huggingface.co/google/embeddinggemma-300m/tree/57c266a740f537b4dc058e1b0cda161fd15afa75) `57c266a740f537b4dc058e1b0cda161fd15afa75` | `model.safetensors`; `2_Dense/model.safetensors`; `3_Dense/model.safetensors`; tokenizer/config/module files. metadata 공개, 일부 raw 문서는 인증 제한 |
| Gemma-300m ONNX 변환 | [onnx-community/embeddinggemma-300m-ONNX](https://huggingface.co/onnx-community/embeddinggemma-300m-ONNX/tree/5090578d9565bb06545b4552f76e6bc2c93e4a66) `5090578d9565bb06545b4552f76e6bc2c93e4a66` | `onnx/model.onnx` + `model.onnx_data`; `model_quantized.onnx` + data; `model_q4.onnx` + data; tokenizer/config |
| Gemma 2 원본 | [google/embeddinggemma-2](https://huggingface.co/google/embeddinggemma-2/tree/914f7f89142e33e77833254d9c9b90c3cef7303b) `914f7f89142e33e77833254d9c9b90c3cef7303b` | `model.safetensors`; `modules.json`; `1_Pooling/config.json`; `2_Normalize/config.json`; tokenizer/config |
| Gemma 2 ONNX 변환 | [onnx-community/embeddinggemma-2-ONNX](https://huggingface.co/onnx-community/embeddinggemma-2-ONNX/tree/daa72c51243991dfcaf9f9137d2c573d8f7790c0) `daa72c51243991dfcaf9f9137d2c573d8f7790c0` | text `onnx/model.onnx` + data; `model_quantized.onnx` + data; `model_q4.onnx` + data. 별도 `audio_encoder*` / `vision_encoder*`는 이번 text-only 대상 아님 |

### E5 원본·실행·양자화 identity

| 파일 | bytes | 기대 SHA256 |
|---|---:|---|
| `model.safetensors` | 470641600 | `1a55775f53449dac10a2bcbc312469fac40b96d53198c407081a831f81c98477` |
| `onnx/model.onnx` **첫 스파이크 대상** | 470268510 | `ca456c06b3a9505ddfd9131408916dd79290368331e7d76bb621f1cba6bc8665` |
| `onnx/model_qint8_avx512_vnni.onnx` | 118346824 | `dd476dd0c2514e9b9be83aeb3853fac0763e0bdf4a71645407587d77c48a2d88` |
| `onnx/model_O4.onnx` | 235052531 | `4654c156f3e4171abc9c716cdb771bf9116455d15ac1aab364aeeede0e3205b0` |
| `tokenizer.json` (`onnx/tokenizer.json`도 동일) | 17082730 | `0b44a9d7b51c3c62626640cda0e2c2f70fdacdc25bbbd68038369d14ebdf4c39` |
| `onnx/sentencepiece.bpe.model` | 5069051 | `cfc8146abe2a0488e9e2a0c56de7952f7c11ab059eca145a0a727afce0db2865` |

[공식 ONNX PR #4](https://huggingface.co/intfloat/multilingual-e5-small/discussions/4)는 owner 측 export bot의 ONNX 추가와 merge를 보여 준다. HF history상 merge commit은 `ffdcc22a9a5c973ef0470385cef91e1ecb461d9f`다. [양자화 PR #12](https://huggingface.co/intfloat/multilingual-e5-small/discussions/12)는 Sentence Transformers `export_dynamic_quantized_onnx_model`의 dynamic QOperator, QUInt8 activation / per-channel QInt8 weight 설정을 명시하며 merge commit은 `6a0d452a575215f80b8f66276dd4ee5d504942c6`다. export 환경 전체·당시 원본 파일 byte 대응까지 재현 검증한 것은 아니다.

`model_O4.onnx`는 [Optimum O4 규격](https://huggingface.co/docs/optimum-onnx/onnxruntime/usage_guides/optimization)의 mixed FP16/GPU 최적화 계열이다. CPU 기본 후보로 선정하지 않는다. 확인한 공식 tree에 E5 `model_qint8_avx2.onnx`는 없다. VNNI 파일명을 AVX2로 바꾸거나 현재 MiniLM AVX2 관측을 E5 실행 결과로 옮기지 않는다. CPU ISA·ORT kernel compatibility는 후속 실제 session 검사 대상이다.

### Gemma 계열 ONNX identity

| 변환 repo / 파일 | bytes | 기대 SHA256 |
|---|---:|---|
| 300m `onnx/model.onnx` | 479932 | `ea91fd315a7c152d427d231746f0f811a1ac93beaba656abfdf2b24e091265e4` |
| 300m `onnx/model.onnx_data` | 1234521088 | `ef835ae565d8695236652475903078e8ed794c7c35faf1164d78ec3238e8a88d` |
| 300m `onnx/model_quantized.onnx` | 567874 | `172efde319fe1542dc41f31be6154910b05b78f7a861c265c4600eec906bd6d8` |
| 300m `onnx/model_quantized.onnx_data` | 308890624 | `705626e28e4c23c82ade34566b4197d97f534c12275fa406dfb71e9937d388c0` |
| 300m `onnx/model_q4.onnx` | 519322 | `ad1dfee81a70f7944b9b9d1cc6e48075b832881cf33fab2f2b248be78f3f0043` |
| 300m `onnx/model_q4.onnx_data` | 196725760 | `599962c3143b040de2dd05e5975be3e9091dd067cacc6a8f7186e3203bab9e02` |
| 300m ONNX `tokenizer.json` | 20323312 | `4dda02faaf32bc91031dc8c88457ac272b00c1016cc679757d1c441b248b9c47` |
| 2 text `onnx/model.onnx` | 421421 | `bc47de15f81208a5c99e5ab10f746d5e33b51ea228b7dc0bef9c133a94f1c1c3` |
| 2 text `onnx/model.onnx_data` | 1084170240 | `9fd452bfc6916e5a92f080f8dada35598ca4512b21c65897d4c5974d5f9630a2` |
| 2 text `onnx/model_quantized.onnx` | 495165 | `d06edd601f851c633a2519304cbeb8dc6170d7ceb61b436625c17fb9b6e74953` |
| 2 text `onnx/model_quantized.onnx_data` | 313724928 | `278a7ff1248c3618e4bd11a607fc54f7bdc7778854230f3956d3f86bd9db4f3b` |
| 2 text `onnx/model_q4.onnx` | 490742 | `f9eeba97acddf139b8ee2ddf04bc30dceafa88de93fadf74d7644e0d61a477a9` |
| 2 text `onnx/model_q4.onnx_data` | 174028800 | `c3975f2d1ab7a1878ae31a7d7a9b7804a827aff3800b60dfceafce21cac3df49` |
| 2 ONNX `tokenizer.json` | 32170510 | `4d777ef5bdc1aa36227abdfb77c3e49e7b9c892d16e1b6bda41c393504828be4` |

300m 원본 `model.safetensors` 기대 SHA256은 `cbf5a78393b6a033e0b8a63a57549964f7ed5c6fbeb4ba0694214f36123f2fd2`, 두 Dense는 `c327f2acb00149676ade24a75e11eb6ebbd367f9ee050267ba56829d2979f702` / `ffb6cc5162e11e2ce6bc2367e121ee3bbbc4e82e1ee26826bd7573d4948d81b8`다. 원본 tokenizer JSON 기대값 `6852f8d561078cc0cebe70ca03c5bfdd0d60a45f9d2e0e1e4cc05b68e9ec329e`는 ONNX repo tokenizer와 **다르다**. 동일한 `tokenizer.model` 기대값 `1299c11d7cf632ef3b4e11937501358ada021bbdf7c47638d13c0ee982f2e79c`만으로 전체 tokenizer 동작 동일성을 가정하지 않는다. 실행 variant의 tokenizer/config를 함께 고정한다.

Gemma 2 원본 `model.safetensors`는 1488915288 bytes, 기대 SHA256 `197a32965d4b1105faf060417baa899e193fb73cd401f42ec9295234d5553d79`다. 원본/ONNX tokenizer JSON 기대값은 일치한다. 변환 repo의 model card는 원본 연결과 precision별 파일을 제공하지만 원본 **exact revision**, export command/opset/runtime 버전·quantization recipe의 완전한 attestation은 이번 읽기에서 확보하지 못했다. Google 원본의 QAT 자료를 ONNX Community q4의 계보로 대체하지 않는다. `q4` weight 정밀도, `q4f16` 혼합 연산, MRL 차원 축소, embedding 출력 자체의 binary/int8 양자화는 서로 다른 선택이다.

## 모델별 전처리·tensor·pooling 계약

### E5-small: 첫 후속용 최소 계약

[고정 config](https://huggingface.co/intfloat/multilingual-e5-small/blob/614241f622f53c4eeff9890bdc4f31cfecc418b3/config.json)는 `model_type=bert`, hidden 384, 12 layers, vocab 250037이며 tokenizer는 XLMRoberta 계열이다. BERT 이름만 보고 현재 MiniLM의 WordPiece tokenizer를 재사용하지 않는다. Rust `tokenizers`에 고정 JSON을 읽히고 special-token postprocessor를 켠다. padding을 쓰면 실제 tokenizer의 pad id/mask를 사용하며 config의 숫자만 보고 임의 pad id를 삽입하지 않는다.

검색 query는 **`query: `**, 자막 corpus는 **`passage: `**를 raw text 앞에 한 번만 붙인다. 한국어·일본어에도 영어 접두사를 유지한다. 접두사를 번역하거나 두 입력을 모두 query로 만드는 것은 별도 ablation이다. 공식 예제는 최대 512, attention-mask mean, L2 정규화다. 초기 동일 fixture 실험은 batch 1·최대 128로 제한하되 접두사와 special tokens를 포함한 길이·truncation을 기록한다. 128에서 잘리는 항목은 숨기지 않고 512 계약으로 별도 측정한다.

ORT graph의 **실제** input/output metadata를 Rust로 먼저 기록한다. 기대치는 token IDs/mask 및 graph가 요구하는 token types의 int64 `[B,N]`, float token output `[B,N,384]`다. graph bytes를 읽지 않았으므로 이것을 검증된 정확한 input 목록이라고 주장하지 않는다. `token_type_ids`는 요구될 때만 공급하고, output 이름·dtype·shape를 고정한 후 mask mean/L2를 적용한다. 빈 mask·0 norm·NaN은 실패로 기록한다. output index 0에 대한 무조건 가정은 제거 대상이다.

### EmbeddingGemma-300m: Dense를 생략하지 않는 계약

[공식 HF ORT 예제](https://huggingface.co/blog/embeddinggemma#onnx-runtime)는 tokenizer input으로 session을 호출해 두 output 중 최종 `sentence_embedding` `[B,768]`을 사용한다. 원본 구조는 bidirectional Gemma 3 → masked mean → Dense 2개이며 이 최종 결과를 사용해야 한다. 첫 token output을 mean/L2만 해도 768d가 나오므로 **shape 일치만으로 정답이라고 판정하면 안 된다**. 별도 Rust Dense 구현은 원본 layer config/weights까지 완전히 읽은 뒤에만 가능하다. 이번에는 원본 raw module/config 접근이 인증 제한이라 그 경로를 채택하지 않았다.

query는 `task: search result | query: `, 제목 없는 자막은 `title: none | text: `를 붙인다. ONNX repo의 tokenizer config는 right padding, add_bos/eos true, 최대 2048을 선언한다. Rust JSON postprocessor의 BOS/EOS 동작과 tokenizer_config 의미가 맞는지 token ID 관측으로 확인한다. HF ORT 예제의 최종 output을 이름으로 선택하고 norm을 확인한다. MRL은 먼저 768d 기준을 확보한 뒤 양쪽을 같은 256/128d로 자르고 재정규화하는 별도 실험이다.

### EmbeddingGemma 2: 빈 media feature가 있는 text graph 계약

[고정 Transformers.js 참조 소스](https://github.com/huggingface/transformers.js/blob/511bb61a7c46b73e33f1af3fb308fbd1afe98c02/packages/transformers/src/models/embedding_gemma2/modeling_embedding_gemma2.js)는 text graph에 `input_ids`, `attention_mask`와 `image_features`, `video_features`, `audio_features`를 전달한다. media가 없으면 각각 float32 `[0,text_config.hidden_size]`, 이번 config의 hidden_size는 **512**다. special media placeholder token 수와 feature 수의 일치도 검사한다. text-only는 media graph를 불러오지 않되 이 빈 입력을 없애도 된다고 추론하지 않는다. 이 source를 Rust ORT로 옮길 수 있는 구체적 경로는 확인했지만 실제 graph signature/kernels는 아직 검사하지 않았다.

참조 source는 graph가 `last_hidden_state`와 mean/L2가 끝난 `sentence_embedding`을 반환한다고 명시한다. **`sentence_embedding` `[B,768]`을 사용하고 다시 token pooling하지 않는다.** [Transformers 5.19 공식 문서](https://huggingface.co/docs/transformers/v5.19.0/en/model_doc/embedding_gemma2)의 원본 PyTorch `last_hidden_state`는 이미 512→768 projection을 지난 token output이며 mask mean/L2가 별도로 필요하다. 원본 모델과 변환 graph의 pooling 경계를 구별한다. 원본 `1_Pooling/config.json`은 mean·include_prompt=true, `2_Normalize/config.json`은 sentence_embedding normalization을 선언했다.

query/document 접두사는 300m과 같은 search/title 문자열을 사용한다. prompts는 품질 지침이며 최신 공식 문서는 무접두사 비교도 권한다. 첫 실험은 prefix-on으로 고정하고 prefix-off는 별도 variant로 둔다. tokenizer_config의 model_max_length는 사실상 무한 sentinel이므로 그대로 허용하지 않는다. 초기 상한은 128, 모델 카드 유효 한계는 8192로 명시한다. tokenizers JSON의 postprocessor/special tokens를 확인한다. 양쪽을 768d로 비교하고 MRL은 나중에 수행한다.

원본 카드의 FP32/BF16 권고와 FP16 경고, ONNX 변환 카드의 FP16 제공 및 근접도 수치는 대상 graph가 다를 수 있다. CPU 첫 기준은 FP32로 고정하고 FP16/q4f16을 기본값으로 쓰지 않는다. q4/q8도 ORT backend 지원과 finite-output/순위 차이를 관측하기 전 채택하지 않는다.

## Rust 실행 경로와 CPU 예산 제안

현재 프로젝트는 `ort = 2.0.0-rc.14`, `tokenizers = 0.22`를 사용한다. 새 언어 런타임을 추가하지 않고 infrastructure에서 고정 모델 패키지를 여는 Rust adapter를 만든다는 제안이다. 외부 ORT DLL·버전·SHA256·CPU execution provider·thread/optimization 옵션을 모델과 별도로 기록한다. [ort 공식 예제](https://github.com/pykeio/ort/blob/main/examples/README.md)의 model-info 기능처럼 I/O 이름·정밀도·shape metadata를 먼저 검사할 수 있다. 문서의 예제가 현재 lockfile API와 동일하다고 가정하지 않는다.

[Candle 공식 BERT 예제](https://github.com/huggingface/candle/blob/main/candle-examples/examples/bert/README.md)는 pure Rust 대안이지만 padding까지 평균하는 예제 동작을 E5 정답 구현으로 복사하면 안 된다. E5는 mask mean이 필요하다. Gemma 3 생성 모델 지원도 bidirectional attention·Dense를 포함한 EmbeddingGemma 지원의 증거가 아니며 Gemma 4 PLE인 Gemma 2를 대체하지 못한다. 이번 우선안은 이미 쓰는 ORT의 명시적 Rust adapter이며 pure Rust Candle 동등성은 미검증이다.

아래는 **후속 담당이 조정할 제한 제안**이며 benchmark 성능이나 승인된 제품 SLO가 아니다.

| 항목 | 첫 correctness 스파이크 | 성능 측정으로 진행할 때 |
|---|---|---|
| 실행 | CPU EP, batch 1, intra 2 / inter 1, 최대 128 tokens, 33개 fixture 1회 | 동결된 동일 build/입력/전처리/host에서 warm/cold를 분리, thread 수·token 길이별 비교 |
| 메모리 stop ceiling | E5 FP32 process private bytes 2 GiB, Gemma text FP32 3 GiB; 로딩/임시 allocation 포함 | weights bytes를 RSS로 간주하지 않고 private/working set/peak를 따로 수집 |
| 시간 stop ceiling | load+33개 전체 60초 상한과 취소 전파 | p50/p95 query latency·cue/s·CPU time·cold load를 관측한 뒤 UX 예산 결정 |
| 동시 작업 | 인덱싱 worker 1, bounded queue; 재생/UI와 공통 취소 계약 | 정상 재생 중 영향 별도 측정; 사용 중인 앱/서비스를 종료하거나 전원·VM/DPI 설정을 변경하지 않음 |

E5 FP32는 weight 약 470 MB로 MiniLM과 같은 규모다. Gemma FP32는 1 GB 이상으로 더 큰 로드 예산이 필요하다. q4는 다운로드 크기를 줄이지만 속도·peak RAM·검색 품질 개선을 보장하지 않는다. MRL 384→128 또는 768→256은 저장 vector 비용을 줄여도 transformer inference 비용을 같은 비율로 줄이지 않는다. 예를 들어 10,000 cue의 raw FP32 vector는 384d 약 15.36 MB, 768d 약 30.72 MB이며 metadata/index overhead는 별도다.

## 첫 스파이크의 승인 가능한 결과와 다음 단계

1. **E5 FP32 하나부터** 고정 파일·tokenizer·pooling/prefix 계약을 별도 manifest에 고정한다. 원본/변환 repo SHA, 실제 파일 hashes, adapter frozen EXE/source/lock, ORT DLL, token cap·prefix 원문·pooling version을 label이 참조한다. 기존 MiniLM run의 allowlist/label을 재사용해 결과를 덮어쓰지 않는다.
2. 같은 raw 26 cue와 7 query에 모델별 올바른 query/document 전처리를 적용한다. 현재 FP32·AVX2 임시 결과와 E5 FP32의 per-query 전체 순위·cue 20/고마움 사례·finite/norm/truncation·token IDs를 함께 남긴다. exact-copy 성공을 semantic retrieval 성공으로 집계하지 않는다. 모델별 score 범위가 다르므로 기존 cosine cutoff를 이식하지 않는다.
3. positive 5개의 provisional MRR/hit@3을 동일 정의로 내되 개선돼도 **임시 fixture 개선**만 주장한다. negative 한 개로 자동 no-match threshold를 보정하지 않는다. gold를 모델 결과에 맞춰 고치지 않고 독립 검토·held-out episode/query를 확보한다. 한국어 축약/부정/고유명사·다국어 KO↔JA/EN·겹치는 문맥을 별도 strata로 평가한다. 정확 검색/어휘 baseline도 같은 corpus에서 비교한다.
4. 품질/계약이 통과하면 실제 hardware ISA가 맞는 기존 공식 E5 quantized 파일 또는 별도 검증된 변환을 검토한다. 공식 AVX2가 없는 상태에서 변환 스크립트/파일명을 만들어 진행하지 않는다. 개선이 부족하면 **Gemma 2 text-only FP32**를 다음 후보로 연다. 그때 empty media tensors와 최종 sentence output을 별도 Rust adapter 계약으로 검증한다.

domain은 model 이름과 독립적인 검색 관련성·cue identity·no-match 정책을 담당하고 application은 GUI/CLI 공통 index/query 유스케이스를 담당한다. infrastructure가 tokenizer/prefix/tensor/runtime 변환을, presentation은 검색 입력·결과 상태를 담당한다. 모델·revision·tokenizer·prefix·pooling·차원·precision이 하나라도 바뀌면 index identity를 바꾸고 재색인한다. query와 corpus를 다른 모델 공간에서 점수화하지 않는다. 이 설계 제안은 이번 조사에서 구현하지 않았다.

## 라이선스·계보 확인 범위와 미검증 사항

E5는 HF card MIT 표기이며 조회한 tree에 별도 LICENSE 파일은 없었다. 공식 repo/PR의 provenance는 확인했으나 학습 데이터와 모든 upstream artifact의 권리를 하나의 SPDX 표기로 확정하지 않는다. Gemma-300m 원본/ONNX card는 `gemma`이며 원본 일부 raw 파일은 인증 제한으로 읽지 못했다. 로그인이나 mirror 우회는 하지 않았다. Gemma 2 원본/ONNX는 Apache-2.0이며 Google 카드가 연결한 [Apache 2.0 원문](https://ai.google.dev/gemma/apache_2)을 확인했다. Gemma라는 이름만으로 이전 독자 조건을 Gemma 2에 적용하지 않는다. 런타임 코드·가중치·tokenizer·변환물의 고지는 배포 때 각각 보존한다.

잔여 게이트는 로컬 파일 실제 hash, ONNX signature/opset/operator와 rc.14 runtime 호환, tokenizer parity, 원본→변환 exact source revision/recipe, 실제 한국어 및 교차언어 관련성, no-match calibration, CPU/RAM/latency다. 원본/변환 HF revision과 LFS hash를 고정한 것은 **다운로드·실행·품질·출시 승인 완료**를 뜻하지 않는다. 문서만 갱신했으므로 이번 조사에서는 Cargo 빌드/테스트를 실행하지 않았다.
