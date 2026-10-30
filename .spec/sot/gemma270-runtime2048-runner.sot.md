---
id: sot:kinekino/gemma270-runtime2048-runner
title: Gemma270 runtime2048 경계 runner의 실제 호출 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Gemma270 runtime2048 경계 runner의 실제 호출 계약

2026-10-19 source-only 조사. root가 `runtime/ai-spike/source/mistral-gemma-cap-source-r1`에 확보한 base commit `4400935451da5e2dc7379a3f92fbbada66557f6c`의 source를 읽었다. root는 cap3파일 패치 및 `cargo check --locked -p mistralrs-core --no-default-features -j2` 통과/테스트 컴파일 진행을 전달했다. **이 조사 세션이 check·빌드·테스트·모델 load·HTTP 호출·추론을 실행한 것은 아니다.** [cap 패치 준비](gemma270-runtime-context-cap.md)의 runtime2048/native32768 분리를 유지한다.

## 실제 HTTP route와 schema

`mistralrs-server-core/src/route_registry.rs:27–35`와 `mistralrs_server_router_builder.rs:374–381`에 다음 route가 실제 등록된다.

| route | handler / 실제 용도 |
|---|---|
| POST /v1/messages/count_tokens | anthropic.rs::anthropic_count_tokens; chat 입력의 token **개수**만 반환 |
| POST /v1/messages | anthropic_messages; Anthropic 형태 생성 |
| POST /v1/chat/completions | chatcompletions; 기존 structured spike의 OpenAI chat 생성 |
| POST /v1/completions | completions; 문자열 raw completion, chat template 경로 아님 |

고정 source의 router에는 `/tokenize`, `/v1/tokenize`, `/render`가 없다. 내부 `Request::Tokenize`를 HTTP endpoint로 표현하지 않는다. 읽은 CLI args/commands에서도 독립 tokenize subcommand를 확인하지 못했다. source route의 존재와 기존59ee EXE 또는 신규EXE의 실제 응답 성공은 다르다. 이 세션은 실행 중 server나 바이너리 기능을 probe하지 않았다.

### count_tokens 입력

`anthropic.rs:84`의 **AnthropicMessagesRequest**를 받는다. model은 생략하면 default, messages는 필수 배열이다. 각 message는 role 문자열(user/assistant/system)과 content(문자열 또는 Anthropic content 배열)를 갖는다. 빈 messages/잘못된 role은 validate(false)에서 거부한다. max_tokens는 Option이며 count route에서는 필수가 아니다. system/tools/tool_choice/thinking 및 extension enable_thinking/reasoning_effort/truncate_sequence 등의 필드도 존재하지만 이번 text-only 경계 cohort에는 쓰지 않거나 아래처럼 명시한다.

다음 JSON은 실제 schema의 예시이며 실행한 요청이 아니다.

```json
{
  "model": "default",
  "messages": [{"role": "user", "content": "BOUNDARY_CASE_TEXT"}],
  "enable_thinking": false,
  "truncate_sequence": false
}
```

성공 body는 `{"input_tokens": N}`다(`AnthropicCountTokensResponse`,403행). token IDs/rendered 문자열/최대길이/모델 metadata는 반환하지 않는다. count handler는1930행 이후 Anthropic→OpenAI request 변환→공통 parse_request→RequestMessage::Chat 추출→내부Tokenize를 보낸다. **모델 forward가 아닌 tokenizer/processor 호출**이며 cap admission을 통과했다는 뜻은 아니다.2049도 token count는 성공할 수 있고 생성 route의 admission이 별도로 거부해야 한다.

### 실제 chat 생성 입력

기존 `POST /v1/chat/completions`를 그대로 사용한다. 실제 schema는 `mistralrs-server-core/src/openai.rs:1127`의 ChatCompletionRequest다. messages는 message 배열 또는 raw 문자열도 허용하지만 이번 비교에는 배열 한 user text만 고정한다. model default, max_tokens256, temperature0, seed42, stream=false, truncate_sequence=false, 기존 strict response_format을 유지한다. 경계 테스트용 max_tokens2 등은 아래 별도 boundary arm에만 명시한다.

`chat_completion.rs:586–603`은 top-level enable_thinking/reasoning_effort를 우선하고, 없으면 `chat_template_kwargs`에서 **enable_thinking bool/reasoning_effort string만** 승격한다. 그 외 kwargs는 warn 후 무시된다. 이는 일반 Jinja 변수를 모두 전달하는 API가 아니다. 충돌을 만들지 말고 count의 enable_thinking=false와 generation의 동일 유효값을 고정한다. Anthropic 변환은 chat_template_kwargs=None으로 설정한다. 따라서 count request에 OpenAI chat_template_kwargs를 복사해 동등성을 주장하지 않는다. Gemma template에 thinking 변수가 실제 쓰이는지/출력에 영향이 있는지도 별도 관측이며 false 전달만으로 제거 효과를 승인하지 않는다.

`chat_completion.rs:1099`와 `completions.rs:294`는 truncate_sequence 생략 시 false를 사용한다. runner는 생략 대신 false를 명시한다. 일반 schema의 다른 optional field·tools/system/session/파일을 혼합하지 않아 count/generation 정규화 차이를 줄인다.

## add_generation_prompt·special token의 실제 처리

`anthropic.rs:1992–1999`는 TokenizationRequest에 **add_generation_prompt=true, add_special_tokens=true**를 고정한다. `engine/add_request.rs:329–337`의 chat 생성도 processor.process에 true/true를 준다. HTTP request에서 이 두 인자를 false로 바꿀 필드는 없다.

`pipeline/processing.rs:38`의 공개 `Processor::process`는 `(Vec<u32>, String)`을 반환한다. 순서는 template 렌더→BOS 중복 방지→encode_fast다. `add_special_tokens=true`여도 rendered prompt가 template의 bos_tok으로 시작하면 실제 tokenizer 인자는 false로 바뀐다. 이것은 **Gemma의 double BOS 방지**다. 단순 tokenizer.encode(text,true)나 다른 tokenizer의 chat_template를 복제하면 같은 조건이 아니다. `vision_models/gemma3/inputs_processor.rs:243`의 Gemma3Processor는 process를 override하지 않아 이 기본 구현을 사용한다.

`pipeline/chat_template.rs:712`의 공개 apply_chat_template_to는 messages/add_generation_prompt/enable_thinking/reasoning_effort/template/BOS/EOS/UNK/tools를 받는 낮은 단계 렌더 함수다. 실제 pipeline의 processor가 수행하는 template_action/BOS suppression/최종 encode까지 대체하는 단독 호출은 아니다. native Rust MiniJinja 경로이며 새 Python 렌더 코드를 만들 필요가 없다.

## IDs·문자열을 실제로 관측하는 선택지

### 존재하는 내부 Rust API

`mistralrs-core/src/request.rs:543`의 TokenizationRequest는 다음 필드다.

- text: `Either<Vec<IndexMap<String, MessageContent>>, String>`
- tools: Option<Vec<Tool>>, add_generation_prompt: bool, add_special_tokens: bool
- enable_thinking: Option<bool>, reasoning_effort: Option<ReasoningEffort>
- response: `tokio::sync::mpsc::Sender<anyhow::Result<Vec<u32>>>`

Rust runner가 같은 엔진 instance를 소유한다면 `MistralRs::get_sender(model_id: Option<&str>)`(`lib.rs:1781`)로 Request::Tokenize를 보내고 소유한 response channel로 IDs를 받는 계약을 사용할 수 있다. 이것은 **존재하는 library API**이며 외부HTTP서버에 이 enum을 JSON으로 보내는 기능이 아니다. model default는 None, text는 Left(chat messages), tools=None, true/true, thinking=false/effortNone를 고정한다. count handler와 같은 정규화 messages가 들어가야 한다.

`engine/add_request.rs:1024–1060`의 tokenize_text는 processor 결과 `(tokens, rendered)` 중 rendered를 버리고 IDs만 반환한다. 따라서 internal Tokenize도 rendered 문자열을 반환하지 않는다. Right(raw string)는 encode_fast를 직접 호출하며 generation prompt/template를 적용하지 않으므로 Left와 동등한 검증이 아니다.

### root가 추가해야 하는 관측 계약

동일 pipeline을 직접 소유한 Rust runner는 공개 `pipeline.get_processor().process(...)`의 반환쌍을 읽을 수 있다. 다만 MistralRs 외부 consumer가 공개 getter로 내부 pipeline을 얻을 수 있다고 가정하지 않는다. 독립EXE에 붙는 기존HTTP만 사용하는 runner라면 **현재 API로 문자열/IDs를 얻을 수 없으므로** root가 별도 diagnostic Rust 경로를 구현하고 freeze해야 한다.

가장 직접적인 제안은 engine/add_request.rs의 실제 chat processor 반환 직후·admission 전에서 tokens/rendered를 복사하고, admission 후 사용 tokens도 함께 기록하는 **소유된 진단 capture**다. 공개 endpoint를 새로 만들었다고 서술하거나 일반 제품 흐름에 넣지 않는다. source hook 위치에 대한 제안이며 현재 구현된 기능은 아니다. 관측은 값을 교체하지 않는다.

manifest에는 case ID/엔진EXE·patch·compiled-input SHA/모델GGUF SHA/templateSHA/요청JSONSHA/effective thinking·reasoning/tools/두 요청 flag와 실제 BOS suppression/UTF8 rendered bytes·SHA/IDs의명시적U32LE bytes·SHA/ID수/처음·마지막IDs/native32768/runtime2048/truncation전후동일 여부를 남긴다. IDs를 decode한 문자열은 byte-for-byte 원래 rendered 문자열의 증거를 대신하지 않는다. 원시 내용을 같은artifact로 보존한다.

## 2047/2048/2049 cohort 구성

1. 먼저 비추론 tokenizer/processor만 사용한다. token 수는 **최종 rendered chat prompt** 기준이다. 문자수·UTF8 bytes·user text만의token수를2048로 취급하지 않는다.
2. Rust runner에서 제한된 후보 문자열 집합과 deterministic padding 조합을 탐색해 count endpoint 또는 실제 processor의 count가 각각2047/2048/2049인 **정확한 문자열**을 찾는다. 단어 반복이 항상1token을 추가한다거나 binary search의 단조성을 가정하지 않는다. 탐색 예산/후보·seed·실패를 고정하고 정확한 길이를 못 만들면 blocked fixture로 기록한다. 문자열을 임의로 token slice/decode해서 chat round-trip이 같다고 하지 않는다.
3. exact 후보마다 internal IDs/실제 render capture와 HTTP count가 같은 수인지 확인하고 freeze한다. 생성 request messages와 count messages를 exact 비교한다. raw special token 삽입을 padding으로 사용하면 role/template 의미를 바꾸므로 허용하지 않는다.
4. 별도의 내부 **CompletionTokens(Vec<u32>)** fixture는 exact length를 만드는 방법이지만 HTTP `/v1/completions`의 prompt schema는 String이다(openai.rs:1448). raw-token fixture는 engine core 경계 unit test용으로만 분류하고 chat/BOS/template 검증을 통과한 것처럼 사용하지 않는다. `/v1/completions`에 IDs 배열을 보낼 수 있다고 명령을 만들지 않는다.

### 독립 boundary arm과 품질 arm

| arm | 요청/기대·관측 |
|---|---|
| count-only2047/2048/2049 | tokenizer만 실행; 각 count exact. count 성공은 생성 허용 아님 |
| prompt2049, truncate=false | engine/add_request.rs:427의 >runtimecap 검사에서 명시적 ValidationError; forward0회; truncation0 |
| prompt2048, generation>0 | 한도가 이미 찬 상태. native32768로 fallback하거나 추가token을 조용히 허용하면 실패. 정책은 root가 명시적으로 admission거부 또는 generation0종료 중 고정 |
| prompt2047, max_tokens2 | EOS/stop 조기종료와 구분하며2048 경계 도달/추가token/마지막forward를 관측. 단순length 문자열만으로 통과하지 않음 |
| structured3task | 기존 max_tokens256·temp0·seed42·schema를 유지; prompt+256<=2048 admission. boundary실험과 별도 집계 |

현재 source는2048 prompt를 >조건으로 거부하지 않는다. 위 표의 가득 찬 prompt 정책은 **runner/engine이 충족해야 하는 요구이며 구현된 보장 아님**이다. boundary arm의max_tokens2는 context를 제거하거나 기존 품질arm256을 낮추는 우회가 아니다. 경계 검증을 위해 fixture와 기대값을 별도 선언한다.

## stop/off-by-one의 정확한 관측 지점

`pipeline/sampling.rs:196`은 `seq.is_done(sampled_token, eos, metadata.max_seq_len)`을 **추가 전** 호출한다.208행에서 `seq.add_token(...)`을 부른다. `sequence.rs:1646`은 stop reason이 있어도 tokens.push를 수행한다. `sequence.rs:1713–1718`은 생성budget에는 `generated_len+1>=max_len`을 쓰지만 모델길이는 **현재 tokens.len()>=max_model_len**을 검사한다. 이 구조는 strict total length 경계의 off-by-one 후보가 source에 존재함을 보여준다. 실제 EOS·stop·budget 우선순위와 response count까지 관측하기 전 실제 모델 실행의 초과를 확정하지 않는다.

`StopReason::Length`와 `ModelLength`는 둘 다 외부 finish_reason `length`로 매핑된다(sequence.rs:85/99). 따라서 HTTP의 length만으로 어느 한도가 걸렸는지 알 수 없다. 최소 capture는 request/case ID, 현재prompt_len/tokens_len/generated_len, sampled ID, EOS/stop 여부, effective cap, is_done의구체variant, add_token전후길이, 총forward수와각forward의seqlen_offset/입력길이/최대position, 최종usage prompt/completion/total 및 response finish_reason이다. spec의 cap이 forward position만 제한하는지 emitted sequence까지 제한하는지 혼합하지 않고 이번 runner는 **전체 sequence<=2048 및 처리 position<2048**을 명시적으로 검증한다.

우선 모델 없이 기존 Sequence Rust fixture로 boundary와 두 stop variant를 검사한다. prompt2047/max_tokens2에서 후보token 추가 전후, prompt2048에서 no-room admission을 각각 검증한다. root의 새cap3파일패치가 loader metadata만 수정했다면 이 stop 경계는 자동 해결되지 않는다. 필요한 stop/admission 수정은 별도 실제diff/회귀로 증명하며 기존 cap/수치정책을 제거하지 않는다. source-only runner 계약으로 현재실행을 통과 처리하지 않는다.

## provenance·검증 상태

읽은 로컬 source SHA256(조사 시점; root의 동시 작업 이후 manifest는 별도 재고정):

| 파일 | SHA256 |
|---|---|
| core/src/engine/add_request.rs |4e724836f6f7d9616bdaee5e9f49a302fec09c21bcce99c690f3b22c5eca04a7|
| core/src/sequence.rs |e5feb88548ca018e42f623294433ffb9c15d68a208cf2f82192d97c65524e005|
| core/src/pipeline/processing.rs |1a42d64b30d4ca8d1edd3ff805f79e18950ed24bd1ffe1204be2650460465dd7|
| server-core/src/anthropic.rs |f8850f7e4d92878d0bfed0bbadbe21e7006330446a09832ab143efa38a358b94|
| server-core/src/route_registry.rs |d07d114c51cc15f86fd3518e4a3d88c613c3efd6e2a62f5d4d1ebb9474f33370|

표의 core/server-core는 각각 저장소의 mistralrs-core/mistralrs-server-core다. 원본 위치는 [고정 engine source](https://github.com/EricLBuehler/mistral.rs/tree/4400935451da5e2dc7379a3f92fbbada66557f6c)이며 로컬 checkout의 변경 파일은 root 소유다. 이 조사에서는 수정하지 않았다.

실제 신규 CLI ready/HTTP count/ID와render capture/정확한세길이fixture/forward·stop경계/수치·품질은 미검증이다. root가 수행한 check 통과는 전달받은 사실이며 본 조사검사에 합산하지 않는다. 개발코드 변경이 없어 cargo검사도 실행하지 않았다. 변경은 이 신규 정본과 research coordination 인계만이다.
