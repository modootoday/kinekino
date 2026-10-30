---
id: sot:kinekino/spike
title: 경량 생성 모델 다음 spike 준비 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 경량 생성 모델 다음 spike 준비 계약

후속 root 실제 결과: [고정 취득·헤더 검사](../../../../verification/ai/generation-gguf-header-actual.md), [Llama1B 실제3요청](../../../../verification/ai/generation-llama1b-structured-smoke.md), [Gemma context2048 준비 실패](../../../../verification/ai/generation-gemma270-loader-failure.md)를 따른다. 아래는 조사 시점의 미취득/미실행 범위이며 source 지원과 실제 Windows 실행 성공을 구분한다.

2026-10-18. [기존 후보군](lightweight-model-families.md)을 family별 한 개로 좁힌 읽기 전용 조사다. 작은 HF metadata·카드·공식 engine source와 현재 Rust adapter만 읽었다. 모델/배포물 취득·설치·빌드·추론은 하지 않았으며 root tool/기존 frozen 증거·UI/SDK를 수정하지 않았다. 다음 실행은 root 단일 worker가 소유한다.

**추천:** 새 family 대조는 **Gemma 3 270M IT Q4_K_M**부터다. 이미 받은 파일만으로 즉시 준비 비용을 줄이려면 **Qwen3 0.6B Q4_K_M**의 동일 조건 structured 대조가 먼저 가능하다. **Llama 3.2 1B Instruct Q4_K_M**는 세 번째다. 이는 파일/준비 비용과 새로운 비교 정보의 순서이며 의미 품질 예상 순위가 아니다. 각 family의 최종 선정은 아래 세 개뿐이다. Gemma4 E2B·Gemma3 1B·Qwen3.5 0.8B는 이 선택을 설명하는 비교 대상이다.

## 현재 실제 baseline과 실행 코드

[구조화 SOT](../../../../verification/ai/structured-text.md)의 baseline은 `unsloth/Qwen3.5-2B-GGUF`, revision `f6d5376be1edb4d416d56da11e5397a961aca8ae`, `Qwen3.5-2B-Q4_K_M.gguf` **1280835840 B**, SHA256 `aaf42c8b7c3cab2bf3d69c355048d4a0ee9973d48f16c731c0520ee914699223`다. 실제 제약 arm 구조/참조9/9와 교정 누락·카드 사실 오류·하이라이트 유용성 미승인을 유지한다. 기존 무제약0.8 온도 결과와 섞지 않는다.

읽은 현행 코드:

- `tools/model-spike/src/application/structured_text.rs`: `download-text-balanced-q4.json`을 고정해서 읽고 `.gguf` 첫 항목을 선택한 뒤 SHA를 검사한다. 동일 세 task·3회, 무제약/제약 arm 순서 교대, 총18요청이다. 후보 model 인자를 받는 공개 use case가 아니다.
- `tools/model-spike/src/infrastructure/local_text_server.rs`: 기존 `runtime/ai-benchmark/mistralrs-v0.9.4-cpu/mistralrs.exe`로 localhost private server를 소유한다. child stdin null, UI 없음, offline/token-source none, CPU/context2048/paged-attn off/max-seqs1/prefix-cache0/4 threads다. private Windows Job을 사용한다.
- 실제 요청은 `/v1/chat/completions`, `model=default`, 한 user message, max_tokens256, temperature0, request seed42, stream=false다. 제약 arm은 `response_format.type=json_schema`, strict=true를 쓴다. global seed는 CPU 오류 때문에 생략한다. request seed가 실제 적용됐다고 새로 인증하지 않는다.
- `chat_template_kwargs.enable_thinking=false`는 현재 요청에 있지만 전달 값과 template 적용은 별개다. 새 model에 `/no_think` 문구를 추가해 비교 prompt를 바꾸지 않는다. 기존 `run` 기반 단발 CLI와 이 structured 서버/API 경로도 구분한다.

기존 실행의 engine SHA는 server-start 증거에서 **`59ee8ff009a06cee8c39890cf6fd90d74f4617e46e88ac9b46a28a11538a9d73`**다. source는 v0.9.4 tag가 가리키는 **`4400935451da5e2dc7379a3f92fbbada66557f6c`**에 대조했다. tag source와 설치 EXE의 전체 빌드 입력 동등성을 새로 입증한 것은 아니다. 기존 실제 Qwen load와 아래 source 지원을 구분하며 Gemma/Llama의 Windows load 성공은 아직 없다.

## 선정 후보의 원본 identity·파라미터·접근 조건

수치는 고정 HF API/card 조회 결과다. tensor inventory·물리 파일 bytes·논리 파라미터·실행 peak를 구분한다. 세 후보 모두 dense 계열이며 MoE의 소수 expert 활성 개념을 붙이지 않는다. embedding lookup이 있어도 전체 weight 파일이 자동으로 사라지는 것은 아니다.

| family 선정 | 원본 HF revision | 파라미터 관측 | 원본 파일 bytes / 접근·라이선스 |
| --- | --- | --- | --- |
| google/gemma-3-270m-it | `ac82b4e820549b854eebf28ce6dedaf9fdfa17b3` | HF BF16 tensor inventory268098176; 약270M dense | model.safetensors536223056 B; gated=manual, license=gemma |
| meta-llama/Llama-3.2-1B-Instruct | `9213176726f574b556790deb65791e0c5aa438b6` | HF BF16 tensor inventory1235814400; 약1.24B dense. 1B라는 표시와 구분 | model.safetensors2471645608 B; gated=manual, license=llama3.2 |
| Qwen/Qwen3-0.6B | `c1899de289a04d12100db370d81485cdf75e47ca` | 원본 카드0.6B/비embedding0.44B; HF 저장 tensor inventory751632384와 서로 다른 집계 | model.safetensors1503300328 B; gated=false, Apache-2.0 |

Qwen config의 tie_word_embeddings=true를 확인했다. 카드의 논리 파라미터와 HF 저장 tensor 합계가 다르므로 **751M active 모델** 또는 모든 bytes가 중복 없이 상주한다는 식으로 확정하지 않는다. 실제 safetensors header/alias와 loader 선택을 이번에 검사하지 않았다. 숫자 이름으로 메모리를 계산하지 않고 실제 파일/로드 peak를 기록할 것이 핵심이다.

Gemma/Llama의 비인증 tree는 weight SHA를 별표로 가린다. raw config body 요청도401이었다. 위 bytes는 API metadata이지 새로 내려받아 hash 검증한 값이 아니다. 가려진 SHA를 만들어 넣거나 변환 repo의 공개 상태를 원본 동의/접근 조건의 해소로 취급하지 않는다. root의 기존 인증/약관 수락 여부는 이번에 조회하지 않았다.

원본 근거: [Gemma 고정 카드](https://huggingface.co/google/gemma-3-270m-it/tree/ac82b4e820549b854eebf28ce6dedaf9fdfa17b3), [Llama 고정 카드](https://huggingface.co/meta-llama/Llama-3.2-1B-Instruct/blob/9213176726f574b556790deb65791e0c5aa438b6/README.md), [Qwen 카드](https://huggingface.co/Qwen/Qwen3-0.6B/blob/c1899de289a04d12100db370d81485cdf75e47ca/README.md), [Qwen config](https://huggingface.co/Qwen/Qwen3-0.6B/blob/c1899de289a04d12100db370d81485cdf75e47ca/config.json). 조회 가능한 metadata와 권한이 필요한 파일 바이트를 구분한다.

### Gemma4 E2B 대신 작은 Gemma를 고른 이유

`google/gemma-4-E2B-it` revision **`3e22461f65e89153144f8adb70e3b8c2cc9845a7`**은 gated=false/Apache-2.0이며 HF tensor 합계5123178051, 원본 **10246621918 B**다. LFS SHA는 `2db5482b20d746879bb3ef79b5203e9075a2e2b98f54ec7c2f281c1477ddc550`이다. [공식 카드](https://huggingface.co/google/gemma-4-E2B-it/blob/3e22461f65e89153144f8adb70e3b8c2cc9845a7/README.md)는 **2.3B effective / embedding 포함5.1B**를 구분한다. E는 PLE lookup 비용과 관련된 effective 명칭이고2B의 상주 가중치라는 뜻이 아니다. 전용 image/audio encoder도 존재한다.

이번 검토의 v0.9.4 GGUF 경로는 gemma4를 projector 필수 architecture 목록에 둔다. 더 큰 파일·template18569 B·추가 modality 계약을 동시에 가져오는 경로는 최소 text 비교와 다르다. text-only로 얼마나 줄일 수 있는지 실측하지 않았다. Gemma4를 실패 모델로 분류하는 것이 아니라 이번 최소 자원 단계에서 유보한다.

Gemma3 1B IT도 원본 `dcc83ea841ab6100d6b47a070329e1ba4cf78752`, 999885952 tensor params/1999811208 B로 확인했다. 270M보다 높은 용량을 처음부터 택해 의미 품질이 좋아진다고 보장할 수 없으므로 첫 후보를270M으로 좁혔다. [Google의270M 소개](https://developers.googleblog.com/introducing-gemma-3-270m/)는 task-specific fine-tuning과 text structuring 용도를 설명한다. 이 카드의 강점은 우리 한국어/일본어 자막 정확도 증거가 아니다. 낮은 용량으로 의미 개선이 안 보이면 JSON 성공만으로 승격하지 않는다.

Qwen3.5 0.8B는 기존 실제 파일532517120 B이고 text0.8B/전체tensor873438784(vision 포함)다. 이미0.6B와0.8B의 무제약 단발 실패를 보유한다. 이번 family 내 최소 파일396705472 B·기존 캐시 재사용이 가능한0.6B를 고르며 후보를 추가하지 않는다. 이 선택은 DeltaNet 기반0.8B가 지원되지 않아서가 아니다.

## 변환 artifact·quantizer·현재 고정 SHA

아래 세 변환 artifact가 다음 실행의 허용 후보다. Q4_K_M 이름은 모든 tensor가 균일4bit라는 의미가 아니다. 실제 tensor dtype 분포·GGUF tokenizer/template metadata는 root가 파일 취득 후 Rust reader로 검사한다. 이 연구는 새 GGUF를 읽거나 취득하지 않았다.

| 선정 변환 repo@revision | 파일 | bytes | LFS SHA256 |
| --- | --- | ---: | --- |
| unsloth/gemma-3-270m-it-GGUF@`c90975dbd40c0c7b275fefaae758c3415c906238` | gemma-3-270m-it-Q4_K_M.gguf |253115424|`b1baabd6b729e4041822220d3e648e00d99cac5df86b10dffb77bcccf0688e39`|
| bartowski/Llama-3.2-1B-Instruct-GGUF@`067b946cf014b7c697f3654f621d577a3e3afd1c` | Llama-3.2-1B-Instruct-Q4_K_M.gguf |807694464|`6f85a640a97cf2bf5b8e764087b1e83da0fdb51d7c9fab7d0fece9385611df83`|
| unsloth/Qwen3-0.6B-GGUF@`50968a4468ef4233ed78cd7c3de230dd1d61a56b` | Qwen3-0.6B-Q4_K_M.gguf |396705472|`ac2d97712095a558e31573f62f466a3f9d93990898b0ec79d7c974c1780d524a`|

Qwen 행은 현재 `runtime/ai-benchmark/q4-model.json`과 repo API identity가 일치한다. 이번에 weight 전체 SHA를 다시 계산하거나 과거 파일의 완전 무결성을 재감사한 것은 아니다.

변환 계보의 확인 범위:

- [Gemma Unsloth 카드](https://huggingface.co/unsloth/gemma-3-270m-it-GGUF/blob/c90975dbd40c0c7b275fefaae758c3415c906238/README.md): base_model은 원본 이름과 연결되며 license=gemma다. 읽은 범위에서 exact 원본 commit·quantizer commit·imatrix calibration artifact를 확인하지 못했다. 조회한 original SHA와 변환 생성 parent가 같다고 주장하지 않는다.
- [Llama bartowski 카드](https://huggingface.co/bartowski/Llama-3.2-1B-Instruct-GGUF/blob/067b946cf014b7c697f3654f621d577a3e3afd1c/README.md): llama.cpp **b3821** quantization과 imatrix 데이터 gist를 명시한다. b3821은 `70392f1f81470607ba3afef04aa56c9f65587664`다. dataset 링크의 고정 revision·실제 calibration hash·원본 weight commit까지 완전 연결한 것은 아니다. converter가C++라고 runtime을 반드시C++로 실행해야 하는 것은 아니다.
- [Qwen Unsloth 카드](https://huggingface.co/unsloth/Qwen3-0.6B-GGUF/blob/50968a4468ef4233ed78cd7c3de230dd1d61a56b/README.md): base_model 이름과 Apache 선언을 확인했다. exact 원본 commit/quantizer version/calibration은 미확인이다. 공식 Qwen GGUF repo를 이름만 보고Q4 후보로 대체하지 않는다. 이번 고정 공식 tree에서 확인한 artifact는Q8_0뿐이다.

세 repo는 gated=false지만 **원본 권리/사용조건과 변환 수치 동등성은 별도**다. metadata SHA는 취득 bytes 검증 전까지 관측된 expected hash다. source tokenizer와 embedded tokenizer가 동일하다고 이 단계에서 쓰지 않는다.

## Tokenizer와 chat-template identity

| 모델 | 원본에서 확인한 identity | 다음 root gate |
| --- | --- | --- |
| Gemma270M | Gemma3ForCausalLM/gemma3_text. tokenizer.json33384570 B(가려진LFS SHA); tokenizer_config Git blob`550423832ceb5344d2910f2a099d79da8e850686`; 별도chat_template.jinja1532 B/blob`1117055ab8e8c90e1b200be00cddb78943616d9e` | GGUF gemma tokenizer·BOS/EOS/turn special IDs·embedded template bytes를 덤프하고 원본 API template와 비교. `<bos>`, start_of_turn user/model, end_of_turn 및 generation prefix를 보존 |
| Llama1B | LlamaForCausalLM/llama. tokenizer.json9085657 B/Git blob`5cc5f00a5b203e90a27a3bd60d1ec393b07971e8`; tokenizer_config54528 B/blob`4ff488a165e900e5129cda7c20ab32d568d2a475` | Llama3 BPE/headers/eot와 BOS를 확인. 원본 template의 날짜 기본값/strftime 경로를 기록하여 날짜가 실험 사이 달라지지 않도록 freeze |
| Qwen0.6B | Qwen3ForCausalLM/qwen3, Qwen2Tokenizer, tokenizer.json11422654 B/LFS`aeb13307a71acd8fe81861d94ad54ab689df773318809eed3cbe794b4492dae4`; tokenizer_config9732 B/blob`417d038a63fa3de29cfde265caedae14d1a58d92` | embedded BPE pre-tokenizer와 template를 원본에 대조. add_bos=false, eos im_end, enable_thinking=false시 빈 think block 생성 여부를 실제 rendered prompt로 확인 |

위 Gemma/Llama template 내용은 공개 HF API의 config/tokenizer metadata에서도 읽었다. gated raw 원본 파일을 인증 취득한 byte hash 검증과 같지 않다. 각 GGUF의 embedded metadata는 아직 미검증이며 새 모델 파일명으로 identity 검사를 대신하지 않는다. generation prompt와 원본 user prompt를 나눠 SHA/토큰 수를 기록하고 template가 다르면 모델 비교의 명시 변수로 남긴다.

## 현재 Rust 엔진 호환성

v0.9.4 source의 [GGUF pipeline](https://github.com/EricLBuehler/mistral.rs/blob/4400935451da5e2dc7379a3f92fbbada66557f6c/mistralrs-core/src/pipeline/gguf.rs)은 `gemma3` main-only 파일을 `load_native_gemma3_text`로 보낸다. [Gemma3 config](https://github.com/EricLBuehler/mistral.rs/blob/4400935451da5e2dc7379a3f92fbbada66557f6c/mistralrs-core/src/gguf/gemma3_config.rs)는 Gemma3ForCausalLM/gemma3_text와 GGUF metadata config 합성을 지원한다. Gemma3는 projector 필수 목록이 아니고 Gemma4는 해당 목록에 있다. 따라서270M text-only 경로의 구체적 source 근거가 있다. 실제 파일의 hparam/tensor layout과 Windows load는 root가 확인해야 한다.

[GGUF enum/dispatcher](https://github.com/EricLBuehler/mistral.rs/blob/4400935451da5e2dc7379a3f92fbbada66557f6c/mistralrs-core/src/gguf/mod.rs)에 Llama/Qwen3가 등록돼 있고 native llama/qwen3 모델 경로가 있다. [quant loader](https://github.com/EricLBuehler/mistral.rs/blob/4400935451da5e2dc7379a3f92fbbada66557f6c/mistralrs-quant/src/gguf/mod.rs)는 Q4K/Q6K/Q8_0를 해석한다. CPU는 Candle quantized tensor/matmul 경로를 사용한다. baseline의Q4_K_M 실행 성공은 kernel 존재의 호스트 관측이지만 모든 새 model tensor가 같은 조합이라는 증거는 아니다.

[tokenizer reader](https://github.com/EricLBuehler/mistral.rs/blob/4400935451da5e2dc7379a3f92fbbada66557f6c/mistralrs-core/src/gguf/gguf_tokenizer.rs)는 gemma unigram과 Llama3/Qwen2 pre-tokenizer 분기를 구분하고 필요한 GGUF pre metadata가 없으면 오류를 낸다. [chat-template reader](https://github.com/EricLBuehler/mistral.rs/blob/4400935451da5e2dc7379a3f92fbbada66557f6c/mistralrs-core/src/gguf/chat_template.rs)는 GGUF template를 읽는다. 임의 공통 ChatML로 세 family를 덮는 수정은 제안하지 않는다.

동일 기능을 제공하는 **기존 Rust mistral.rs/Candle 경로를 우선**한다. llama.cpp는 양자화 제작 도구/대안 native engine이고 Rust wrapper를 쓰더라도 C++ engine이라는 점은 바뀌지 않는다. 이번에는 engine 교체나 새 서비스 설치를 준비 조건으로 추가하지 않는다.

## 필요한 adapter 변경과 실제 다음 경계

현재 CLI를 새 후보 실행 명령으로 제시하지 않는다. `structured_text::execute`의 manifest 경로는2B에 고정돼 있고 첫GGUF 선택도 다중 파일 계약으로 안전하지 않다. 필요한 root 수정은 다음 세 책임으로 좁힌다.

1. **입력 계약:** 후보 중 정확한repo/revision/파일/SHA 및 허용 engine을 갖는 typed run spec. 모델 표시명과 파일 선택을 분리하고 `.gguf` 첫 항목 검색을 explicit filename으로 바꾼다. 기존2B manifest/출력/label을 덮어쓰지 않는다.
2. **engine adapter:** 기존 LocalTextServer의 준비된 argv·Windows Job·offline/CPU·request schema를 재사용하되 선택된 model spec만 주입한다. template/model metadata를 새 preparation 결과로 보존한다. 현재 코드가 이미 실행하는 `serve ... -f ABS_GGUF` argv와 HTTP API를 근거로 삼으며 검증하지 않은 CLI flag를 새로 만들지 않는다.
3. **평가 정책:** 같은3개 prompt/schema/cue hash·context2048/max256/temperature0·requestseed42·prefixcache0를 유지한다. thinking 제어는 model별 rendered template에서 확인한다. schema 통과, 원문 단어/순서, 사실 근거, 누락·환각, 시간/peak와 거절을 분리한다.

추천 다음 단일 실행은 Gemma270M의 고정Q4 파일로 **교정·카드·하이라이트 각1건 제약 smoke**다. 이는 기존3회 arm의9/9와 직접 같은 표본 수가 아니므로3/3을9/9와 동등하다고 쓰지 않는다. load/template/schema가 성립하면 동일18요청 계획으로 후속 측정할 수 있다. 라이선스/접근 조건 또는 실제metadata가 아직 준비되지 않았으면 기존Qwen0.6 파일로 adapter 계약을 검증하는 것이 새모델 취득을 우회하는 것보다 좁다. root는 기존 자원 정책과 단일worker를 유지하고 다른 앱을 종료하지 않는다.

메모리 상한은 파일크기로 발명하지 않는다. 현재2B 파일1.28GB 대비 실제 최대private 약2.99GB였고 baseline은 다른빌드 병행 관측이다. 후보들의 상대속도·idle peak·한국어/일본어 품질은 모두 미실측이다. 더 작은 모델이 기존2B의 의미 오류를 해결할 것이라고 약속하지 않는다.

## 권리·접근 및 남은 증거

Gemma3는 [Gemma Terms](https://ai.google.dev/gemma/terms)이며 Gemma4/EmbeddingGemma2의 Apache 선언을 상속하지 않는다. Llama는 [고정 카드 내Community License](https://huggingface.co/meta-llama/Llama-3.2-1B-Instruct/blob/9213176726f574b556790deb65791e0c5aa438b6/README.md) 및 Acceptable Use Policy를 구분한다. 원본 tree의LICENSE.txt7712 B/USE_POLICY.md6021 B identity를 조회했지만 인증된 원문파일 취득/배포 고지 묶음을 만든 것은 아니다. Qwen 원본과변환본 Apache 선언·LICENSE 파일을 확인했다. 이것은 legal 검토나 출시 고지 완성의 대체물이 아니다.

Llama 원본의 명시 지원8개 언어는 영어·독일어·프랑스어·이탈리아어·포르투갈어·힌디어·스페인어·태국어다. 한국어·일본어가 그 명단에 없으므로 해당 자막 품질을 가족명으로 보장하지 않는다. Gemma의140+언어 설명과 Qwen의100+언어 설명도 이번 자료의 실제 품질 통과가 아니다.

이번 산출물은 본 신규 문서와 `runtime/coordination/research.md`뿐이다. 코드 변경이 없으므로 cargo fmt/test/clippy를 새로 실행하지 않았다. 실제 모델 취득·GGUF header/template/tensor 검사·Windows load/run·의미 승인·원본/변환 수치 parity는 root 후속으로 명시한다.
