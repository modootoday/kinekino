---
id: sot:kinekino/gemma270-trace
title: Gemma270 수정판 내부 경계 trace 설계
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Gemma270 수정판 내부 경계 trace 설계

2026-10-19 source-only 설계. [수정판 실측 SOT](../../../../verification/ai/generation-gemma270-context-boundary-fixed.md)의2047→completion1/total2048,2048/2049→HTTP400을 읽었다. 이 root 실측을 이 세션의 직접 실행으로 합산하지 않는다. **full_boundary_gate_passed=false**, 성능비교 부적격과 품질 승인0을 유지한다. 아래 추가 API/field/module은 제안이며 아직 구현·빌드·실행하지 않았다. 변경은 본 문서와 coordination뿐이다.

## 기준과 남은 관측

현재 checkout은 `runtime/ai-spike/source/mistral-gemma-cap-source-r1`, base `4400935451da5e2dc7379a3f92fbbada66557f6c`와 root 소유 fixed11file 패치다. 수정 engine SHA `9256a093133056cff9a7fbbbe37693a8a11abddd5753fe1598316e883f1d78d0`와 실제 worker/orchestrator SHA `89edc38413dc48f5a2097357e23b131d249b50820976c29114aff8b6b0e97ab5`를 그대로 baseline으로 보존한다. 실제 manifest SHA `ef4cdd0bf0a265bb7f317f89e78e52f509634d4e615a1c87dafefaf7f27e1ef9`, 측정 SHA `aca9587ef28a1e4d27f8315547a75fd87d62d57326d1c8d8a1c72ff8c100b3d3`다.

실제 fixed source의 add_request.rs SHA256은 `9974fbaf0454d45254799f4a936b42a92de025311698116bf54042c472db01e0`, sequence.rs는 `3dcdb77c8ea9847058efe3342bea43a41f3f0e183444c306ee58251606918389`였다. 원본 조사 문서의 이전 SHA와 구분한다. root 동시 변경 후에는 새 snapshot을 재고정한다.

수집 대상은 동일 호출에서 얻는 rendered UTF8/최종 U32LE IDs, BOS suppression 실제 분기, admission 결론, 실제 forward와 RoPE positions, sampling 전후 상세 StopReason이다. HTTP `length`는 `Length`와 `ModelLength`를 합치므로 상세 분기를 증명하지 않는다. HTTP400만으로 forward0을 증명하지 않는다.

## 최소 구조와 식별자 계약

기존 `mistralrs-core/src/lib.rs:80`에는 이미 `mod diagnostics;`가 있고 시스템 진단을 담당하는 `src/diagnostics.rs`가 존재한다. 새 `src/diagnostics/boundary_trace.rs`에 한 책임의 bounded recorder와 path/file 보존을 두고, **기존 diagnostics.rs에 `pub(crate) mod boundary_trace;`를 추가**하는 root 연결 계획을 따른다. diagnostics/mod.rs를 만들거나 lib.rs에 모듈을 중복 선언하지 않는다. 제품 정책·모델 수치·일반 logging 시스템을 바꾸지 않는다. 초기 버전은 **CPU/text-only Gemma/한 model engine/한 sequence/n_choices1/직렬 요청/pagedoff/prefixcache0/speculativeoff/streamfalse**만 추적한다. 승인된 trace profile 밖의 실행은 trace 증거 미완성으로 표시한다. trace 지원을 모든 배치/vision/GPU에 확대하지 않는다.

`Engine::new`(engine/mod.rs:535, anyhow::Result<Self>) 시작에서 trusted child-process의 opt-in 설정을 한 번 읽고 recorder를 초기화한다. 제안 env 이름은 `SOURCE_GEMMA_BOUNDARY_TRACE_MANIFEST` 하나이며 현재 존재하는 설정이 아니다. 없으면 recorder=None, 디렉터리 생성·hash·token 복사·파일 쓰기0이다. 이 경로는 runner가 지정한 로컬 manifest만 허용한다. HTTP body/header/web message로 trace path나 case 이름을 받지 않는다. 추가 traced engine 초기화는 이 제한 profile에서 startup 오류로 거부한다. 이 제한은 새 진단 실행의 precondition이며 일반 요청 정책 변경이 아니다.

`run_id`는 runner가 사전 생성해 manifest에 고정한다. case_id는 runner manifest의 enum(`tokens-2047`, `tokens-2048`, `tokens-2049`)이며 사용자 content나 request header에서 만들지 않는다. 신뢰 manifest에 요청JSONSHA와 공통 parse 이후의 canonical normalized text-messages SHA·effective thinking/effort/tools·expected token수를 넣는다. recorder는 **정규화한 messages**와 controls가 exact match인 normal generation request에만 case를 연결한다. raw HTTP SHA는 runner의 request receipt와 이어지며 내부 객체만으로 raw payload bytes SHA를 새로 복원했다고 하지 않는다. unknown/duplicate request가 오면 요청 자체는 기존 처리대로 두고 trace run을 invalid로 표시한다. 관측으로 product admission·sampling을 새로 거부하지 않는다.

`chat_completion.rs:1050`은 NormalRequest.id에 state.next_request_id()를 넣는다. HTTP x-request-id와 이 숫자가 같다고 가정하지 않는다. engine/add_request에서 case→`request.id`를 기록한다. Sequence::new_waiting에는 별도 engine counter가 들어간다. 생성 직후 case/request→`*seq.id()`/response_index를 등록한다. 두 숫자를 같은 ID로 간주하지 않는다. sequence ID registry는 Engine session/run ID와 함께 key를 만들고 종료/거부 시 정리한다. global “현재 case” 슬롯이나 시간순 최근 요청으로 forward를 연결하지 않는다. 이 제한 profile에서는 registry가 단일 traced engine에 귀속되며 다른 engine와 sequence ID 충돌을 허용하지 않는다.

count109회는 Request::Tokenize 경로이며 별도 count 요청이다. normal generation용 trace scope를 만들지 않아 forward/sampling 이벤트와 혼합하지 않는다. 별도로 count trace를 켜려면 event kind와 request identity의 새 계약이 필요하다. 이번 최소 구현에는 추가하지 않는다.

## 정확한 호출점

| 현 함수·타입 | capture와 연결 |
|---|---|
| engine/add_request.rs normal 요청 처리,321행 prompt 준비·341행 부근 processor.process | 정규화 messages/controls로 TraceKey를 얻고 동기 render scope에서 기존 process를 **한 번** 호출 |
| pipeline/processing.rs::Processor::process | 기존 `prompt.starts_with(bos)` 판단과 tokenizer.encode_fast에 실제 전달한 bool; 원본두flag·bos present·startswith·effectiveflag 기록 |
| add_request.rs:422–452 admission | 기존 prompt_requires_truncation 분기 직전 full prompt/IDs·capacity/category/truncate=false; reject 사유와 requestid; accept 후 동일IDs hash |
| add_request.rs Sequence::new_waiting 직후 | requestid/caseid와 독립 seq.id/response_index 연결; 초기 prompt_len 기록 |
| gemma3/inputs_processor.rs ModelInputs 생성667행 | input_seqs와 seq_indices의 실제 row 순서에서 registry TraceKey를 얻어 Gemma3SpecificArgs에 넣음 |
| gemma3/mod.rs::MultimodalModel::forward(244행) | 기존 downcast한 Gemma3SpecificArgs의 trace row key를 ctx에 연결, 바로 기존 self.forward 호출; Gemma 진입/반환 이벤트 |
| pipeline/mod.rs::ModelForwardContext | 현재 `seqlen_offsets()`, `context_lens()`, `position_ids()`, `is_paged()` getter 및 optional trace batch 연결 |
| gemma3/text.rs::TextModel::forward_embeds(605행) | 실제 input shape·dtype/device·ctx getter·max_seq_len/model metadata; 각 호출 ordinal/entry/return |
| gemma3/text.rs Attention::forward,221행 원본 ctx.text_positions 결과 | **원래 계산된 positions tensor**의 CPU 값/형상·min/max·SHA, layer_idx/local-global 분기와 연결; 추가 text_positions 호출 금지 |
| pipeline/sampling.rs::finish_or_add_toks_to_seq(187행) | is_done 직전 및 반환 직후, 기존 add_token 직후의 token수·sampledID·상세reason; 이후 stop/tool/grammar 처리의 최종reason도 별도 |

BOS 분기를 밖에서 재계산한 추정으로 실제 관측을 대신하지 않는다. minimal render scope는 requestid를 public Processor trait 인자로 넓히지 않기 위한 **동기 thread-local stack** 제안이다. scope는 기존 process 호출 한 구간에만 걸고 await/스레드 spawn 전에 종료한다. RAII로 nested/panic 복구하며 thread-local에는 owned TraceKey와 bounded recorder handle만 둔다. forward/sampling의 식별에는 이 TLS를 사용하지 않는다. async task 전체에 thread-local current case를 설정하는 것은 금지한다.

현재 `vision_models/mod.rs::ModelInputs`에는 sequence ID가 없다. generic struct에 필드를 추가해 모든 모델 생성자를 바꾸는 대신 기존 `Gemma3SpecificArgs { image_hashes, packed_layout }`에 optional trace batch를 추가한다. inputprocessor가 정확한 row/seq mapping을 전달한다. `ModelForwardContext`에 optional owned Arc trace batch를 추가하고 기존 new/with_cache는 None으로 초기화한다. Gemma 전용 forward에서만 이를 attach한다. context는 해당 호출에 귀속되므로 다음 요청의 trace key가 남지 않게 한다. default_model_specific_args의 default trace는 None이다. CPU 아닌 trace profile/packed multirow/speculative 경로를 우연히 증명하지 않도록 invalid 처리한다.

`ctx.position_ids()`라는 host metadata와 실제 `ctx.text_positions(...)` RoPE tensor는 다른 관측이다. 둘 다 보존한다. positions를 offset으로 다시 계산해 “실제 tensor”로 기록하지 않는다. attention의 원본 호출 결과를 capture하고 layer0부터 각 layer의 최소 형상·SHA·local/global 분기를 기록한다. 처음 한 번 full tensor를 저장하고 동일 bytes면 뒤에는 같은 blob SHA 참조가 가능하나 실제 호출별 hash는 남긴다. Q/K/logits/weight를 추가 복사할 필요는 없다.

## fenced Rust 최소 변경 형태

다음은 **문서용 제안 diff의 핵심 형태**다. 새 diagnostics API는 존재하지 않으며 crate-level visibility·오류 타입·fixture를 root가 실제 구현하고 컴파일해야 한다. 기존 연산 호출을 observer 구현으로 재작성하지 않는다.

```rust
// processing.rs: preserve the existing expression and the single encode call.
let requested_special_tokens = add_special_tokens;
let add_special_tokens = add_special_tokens
    && !pipeline.get_chat_template()
        .and_then(|t| t.bos_tok())
        .is_some_and(|bos| prompt.starts_with(&bos));
boundary_trace::render_flags(
    add_generation_prompt, requested_special_tokens, add_special_tokens,
    // Capture the actual BOS check inputs/result at this existing branch.
);
// Existing encode_fast(prompt.clone(), add_special_tokens) remains once.
```

```rust
// Gemma-only carrier; proposed field, not present in the current checkout.
#[derive(Default)]
pub struct Gemma3SpecificArgs {
    pub image_hashes: Vec<u64>,
    pub(crate) packed_layout: Option<PackedMultimodalLayout>,
    pub(crate) boundary_trace: Option<Arc<TraceBatch>>,
}
// inputs_processor.rs: preserve input_seqs/seq_indices row order.
// boundary_trace: boundary_trace::rows_for(input_seqs, &seq_indices),
// Gemma MultimodalModel::forward: after the existing downcast:
// ctx.attach_boundary_trace(boundary_trace);
// self.forward(...) remains the same call and argument order.
```

```rust
// sampling.rs: never call is_done a second time for tracing.
let trace = boundary_trace::for_sequence(*seq.id());
trace.before_sampling(seq.get_toks().len(), seq.prompt_tokens(), logprobs.token);
let is_done = seq.is_done(logprobs.token, eos_tok, this.get_metadata().max_seq_len);
trace.pre_add_reason(is_done.as_ref());
// Existing tok_env lookup and decode_ext remain here unchanged.
let mut is_done = seq.add_token(logprobs.clone(), completion_bytes, is_done);
trace.post_add(seq.get_toks().len(), seq.generated_len(), is_done.as_ref());
// Record the final reason at the existing terminal response path as well.
```

StopReason의 `as_ref()`는 borrowed observation이며 take/replace 하지 않는다. off-path helper는 no-op handle 또는 Option 분기이며 결과값/오류를 sampler로 전파하지 않는다. 실제 구현에서는 `trace` helper가 enum 원시 variant와 payload를 명시적으로 encode한다. Display/to_string의 length로 축약하지 않는다.

## 이벤트·바이트 보존

각 event는 schema_version/run/case/internal_request_id/sequence_id/response_index/monotonic event ordinal 및 stage를 갖는다. ordinal은 recorder의 순서 번호이며 RNG·UUID 생성·wall-clock에 의존하지 않는다. forward_call_id는 trace batch 생성/실제 forward에서 부여하고 entry/positions/return/sampling을 동일 seq key로 연결한다. forward 시점과 sampling 이벤트를 시간 근접으로 zip하지 않는다.

- Render: exact UTF8 `rendered-prompt.utf8.bin`; BOM/개행/공백 정규화 없음. U32 IDs는 각 to_le_bytes를 이어붙인 `prompt-token-ids.u32le.bin`; count×4B/shape/ID범위/SHA를 manifest로 기록한다. JSON pretty print는 원시 바이트 대체물이 아니다.
- BOS: requested flags/effective flag, 실제 template BOS 문자열/ID와 starts_with 결과. 시작 token2의 존재만으로 suppression 분기를 소급 추정하지 않는다.
- Admission: native32768/runtime2048, 실제 category/capacity/token수/truncate, accepted/rejected 및 exact reject variant. 거부 cases에는 sequence가 없으므로 seq_id=null이다.
- Sampling: **`Sequence::get_toks().len()`**을 사용한다. `Sequence::len()`은 cache/prefill 상태를 보아 raw vector 길이와 다를 수 있다. prompt_tokens/generated_len/sample ID/EOS설정/현재cap/생성budget/is_done variant와 payload/add_token전후·최종reason를 보존한다. seed 값/choice_seed 정책은 기존 manifest에서 연결하며 seed 계산 함수를 다시 호출하지 않는다.
- Forward: Gemma entry/exit 수, input shape/dtype/device/실제 input IDs, offsets/context_lens/hostpositionIDs/actualRoPEpositions/native/runtime/modelmetadata/paged/packed·pixel유무. return에는 성공/오류·logits shape만 기록하고 logits 값은 이번 범위 밖이다.
- Completion: HTTP receipt SHA/usage/finish_reason와 trace manifest SHA 연결. HTTP400 case의 forward0은 해당case의 forward event0 및 완결된 run accounting이 모두 있어야 주장한다. 단순 빈 로그/누락파일은0회가 아니다.

## opt-in localfile 경로·동시성·누출 방지

runner가 workspace 아래 새 전용 run directory와 고정 local manifest를 만들고 engine에는 그 경로 하나만 전달하는 제안이다. path root는 미리 승인한 `runtime/ai-spike/results/<fresh-trace-run>/internal-trace`로 confinement한다. canonical parent와 고정 workspace root를 확인하고 `..`·absolute child name·UNC/device/ADS·reparse/symlink/junction을 거부한다. Windows TOCTOU를 막으려면 startup에 디렉터리 handle을 열어 최종 경로/파일 identity를 확인하고 생성한 파일 handle의 최종 경로도 재확인한다. canonicalize 문자열 starts_with 검사만으로 race 안전을 주장하지 않는다. 파일명은 fixed enum+숫자 ordinal, CreateNew, 기존 파일 overwrite0, 사용자 메시지/요청ID 문자열을 경로로 쓰지 않는다. 테스트 fixtures로 동작을 검증한다.

상위 디렉터리는 runner 소유로 다른 writer가 없음을 보장하고, 보장할 수 없으면 trace profile startup을 실패시킨다. readonly manifest의 frozen SHA와 open handle identity를 읽기 전후 확인한다. 전역 환경변수/사용자설정은 변경하지 않고 child env만 쓴다. `.spec` 복사는 root가 worker 종료·flush·SHA 확인 뒤 별도 보존한다.

원문·IDs는 opt-in artifact에만 저장한다. tracing/debug/info/HTTP error/stdout/stderr에 prompt·token 배열·완전 경로를 출력하지 않는다. 운영 logger에 복제하지 않고 notification에는 case/event수와 결과만 둔다. 원문을 보존하면서 “마스킹된 원문을 exact 증거”라고 하지 않는다. ACL/접근조건은 runner 소유 디렉터리 기준으로 검증한다.

recorder는 bounded owned event buffer다. registry mutex를 쥔 채 model/processor/sampler/await/파일 I/O를 호출하지 않는다. append는 짧은 mutex 구간에서 복사만 하며 entry 단위 원자 순서를 보장한다. trace queue overflow/오류/poison을 tensor 반환·seed·stop 정책에 전파하지 않고 atomic invalid flag와 누락수로 기록한다. **trace 완결 게이트는 실패**시키되 결과를 추정으로 채우지 않는다. 실제 작업 중 background disk writer를 두지 않고 최대3case/제한바이트를 메모리에 보존한 뒤 runner의 정상 shutdown/명시flush에서 기록하는 방식이 최소다. writer가 필요한 경우 EOF/flush acknowledgment·소유thread join 없이는 완료하지 않는다. 강제 kill이면 incomplete며 forward0 판정 불가다.

IDregistry는 전역 singleton이더라도 immutable run/engine identity와 per-seq map만 보유하며 “current request”를 공유하지 않는다. duplicate/missing row mapping/겹치는 generation/다른 model·engine는 trace invalid로 남긴다. count 요청의 rendering을 generation trace의 TLS에 섞지 않는 회귀를 둔다. engine lifetime 종료 전 registry를 임의 제거하지 않고 완료 accounting 후 정리한다.

## 불변성·회귀·완료 기준

trace는 관측 오버헤드를 가지므로 **성능비교 부적격**이다. disabled path와 enabled path에서 같은 원본 요청·모델·engine patch·CPU4/threads·seed42/temperature0·ignore_eos=true/max_tokens2/truncatefalse 및 native/runtime 값으로 대조한다. 출력 token IDs·raw stop variant·response/usage가 같아야 한다. elapsed/peak 메모리 동일성을 요구하지 않고 개선 주장도 하지 않는다.

모델 없는 Rust 회귀부터 수행한다: disabled filesystem0, exactUTF8/U32LE roundtrip, binarySHA/count, BOS 실제분기1회, nested/panic TLS복구, independentseq/request IDs, count와generation 구분, row mapping과group, path traversal/ADS/reparse/race/overwrite거부, bufferoverflow/flush실패=incomplete, seed함수/encode/is_done/add_token/modelcall 호출횟수 불변. fake recorder가 관측할 뿐 원본 함수를 다시 호출하지 않음을 검증한다. StopReason Length/ModelLength를 같은 문자열로 저장하면 회귀 실패다.

root 실제 후속은 **새 trace engine source/EXE 선보존→trace off 기존3case 통제→trace on 같은3case**다. 2047은 full prompt2047/BOS suppression/actualmaxposition2046인 prefill과 cap도달ModelLength(add전2047/add후2048)·completion1/total2048을 연결한다. 2048/2049는 admission명시거부·sequence미생성·forward0·sampling0을 완결 accounting으로 증명한다. forward chunking이 있을 수 있으므로 prefill이 반드시1호출이라고 가정하지 않는다. actual positions 전체가0..2046을 누락없이 덮는지와 어떤position도2048이상 아닌지를 검사한다. 실제 sampling이 EOS/다른stop이면 expectedboundary검증으로 승인하지 않는다. 이 설계는 새 모델추론을 이 세션에서 허용하는 것이 아니며 root 실행 계약이다.

trace off/on이 다르면 observer가 원인인지 먼저 조사하고 값을 보정하지 않는다. capture 없이 HTTP통과만 재확인하면 현재SOT의 partial 게이트를 유지한다. full gate는 root가 모든 internalartifact·source/compile attest·동일입력통제를 검증한 뒤에만 갱신한다. 품질max256/3task는 별도이며 이번 boundary 추가실행을 품질표본으로 합산하지 않는다.

## source snapshot whitelist·compile gate 영향

현재 `tools/engine-lab/src/application/patch_snapshot.rs::Scope::Boundary`는 exact11file diff를 요구한다. 현11개는 cap3개, compatibility4개(Cargo.toml/lock/code-exec session/varbuilder), boundary4개(add_request/sequence/server lora_adapters/CLI manage)다. 새 trace가 이를 통과할 수 있다고 주장하지 않는다.

제안은 **새 Scope::BoundaryTrace와 새 Revision**, 기존 Boundary whitelist/고정SHA/fallback정책 유지다. 위 최소 설계를 구현하면 기존11개 중 add_request는 추가변경되고, 신규/추가 변경 경로는 다음8개다.

1. mistralrs-core/src/diagnostics.rs (기존 시스템 진단 모듈에 child module 연결)
2. mistralrs-core/src/diagnostics/boundary_trace.rs (신규)
3. mistralrs-core/src/engine/mod.rs (startup recorder)
4. mistralrs-core/src/pipeline/processing.rs (BOS/render 동기 observer)
5. mistralrs-core/src/pipeline/sampling.rs (상세stop 전후)
6. mistralrs-core/src/pipeline/mod.rs (forward context trace carrier)
7. mistralrs-core/src/vision_models/gemma3/mod.rs (전용args/forward 연결)
8. mistralrs-core/src/vision_models/gemma3/inputs_processor.rs (row key 연결)

gemma3/text.rs는 기존11개 안에 있으며 추가 forward/positions capture가 들어간다. 이 제안대로면 whitelist 잠정 합계는 **11+8=19개**다. 실제diff가 더 필요하면 root가 그 책임을 검토해 명시적으로 갱신하며 숫자19에 맞추려고 변경을 숨기지 않는다. **신규 untracked 파일은 현재 git diff --name-only/base diff --binary에 잡히지 않는다.** snapshot 도구가 tracked diff와 untracked 목록을 함께 검증하고 CreateNew source 복사·hash·manifest로 포함해야 한다. 기존 git show base:path는 신규파일에 실패하므로 before absent/afterSHA를 구분한다. 빈 before 파일을 원본에 있었던 것처럼 만들지 않는다. 기존 diagnostics.rs와 신규 boundary_trace.rs의 compiler inputs 포함도 확인한다.

기존 모듈을 확인하지 않고 diagnostics/mod.rs와 lib.rs 추가를 제안했던 초안의 경로·20개 집계는 위 내용으로 정정했다. root는 별도 `tools/boundary-observation` Rust 공통 DTO/검증기의 synthetic10tests 완료를 전달했다. 이는 sensor 연결·실제 모델 trace 실행·19파일 engine 컴파일 통과와 구분하며, 이 세션이 테스트를 직접 실행한 것도 아니다.

root는 새engine-lab snapshot/build/CLI보존/revision선택/runner manifest 지원을 구현할 때 도구자신의 compiled-input snapshot과 fmt/test/strictclippy/lockedofflinebuild를 새로 보존한다. 이 조사에서는 도구를 수정하지 않았다. 기존 cli_snapshot의9256 SHA/98790400B를 새trace EXE에 재사용하지 않는다. core/server-core/CLI 실제 feature 조합에서 corecheck·관련tests·strictclippy·CLI build를 수행하고 no-default-features 경로에서 unused import/field/deadcode가 없는지 검증한다. 새crate 의존성 없이 std/fs/sync와 기존직렬화·hash기능을 우선하며 새dependency가 필요하면 Cargo.toml/lock 및closure attestation을 명시한다. 허용목록에 담긴 diff가 전체native/transitive compiler attest라는 뜻은 아니다.

이번 source 조사·설계·인계만 완료했다. 실제 trace module/API/path confinement/동시성/수치불변·BOS/forward0/positions/StopReason 수집 및 컴파일은 모두 미검증이다. root fixed11file·engine·모델·기존evidence·도구·SDK·UI·다른앱은 수정하지 않았다.
