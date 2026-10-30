---
id: sot:kinekino/gemma270-terminal-witness
title: Gemma270 terminal witness와 비간섭 증거의 범위
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Gemma270 terminal witness와 비간섭 증거의 범위

2026-10-19 source-only 후속 검토. 현재 root checkout의 response/sequence/sampling 및 sensor source를 읽었다. root는 sensor21파일 구현, check와 CLI strictclippy1 통과/coretests 빌드 중, source snapshot `EF2F4F69...r1`을 전달했다. 이는 전달받은 상태이며 줄임 SHA를 완전 hash로 복원하지 않는다. 이 세션은 다운로드·빌드·모델 load·추론·기존 SOT/코드 변경을 하지 않았다. [기존 내부 trace 설계](gemma270-internal-boundary-trace.md)의 off/on exact raw-ID 요구를 삭제하지 않고, 실제 관측 가능한 수준으로 분리한다.

## 기존 독립 witness API 판독

source root는 `runtime/ai-spike/source/mistral-gemma-cap-source-r1`다.

| 실제 위치 | 판독 |
|---|---|
| mistralrs-core/src/response.rs:32 ResponseMessage | content/role/tool_calls/reasoning_content; raw IDs 없음 |
| response.rs:87 Choice | finish_reason String, stop_sequence, index, message/logprobs; 상세 StopReason enum 없음 |
| response.rs:359 Response::Done | 일반 ChatCompletionResponse; core channel로 받아도 위 정보 이상은 아님 |
| response.rs:379 Response::Raw | logits_chunks와 tokens가 실제 존재하나 별도 raw-logits 실행 경로 |
| request.rs NormalRequest.return_raw_logits | 일반 요청에서 false; 이를 true로 바꾸면 같은 실행 정책이 아님 |
| sequence.rs:2373 maybe_send_raw_done_response | raw_choices에서 tokens/logits를 꺼내 전송; 일반 chat terminal witness가 아님 |
| sequence.rs::get_toks/prompt_tokens | live Sequence의 실제 전체IDs와 prompt길이를 읽는 기존 내부 getter |
| pipeline/sampling.rs::finish_or_add_toks_to_seq | live Sequence와 상세reason를 동시에 갖는 실제 terminal 호출점 |

고정 source의 일반 HTTP/내부 Done 응답에서 **payload·실행 경로를 유지한 채 final raw generated IDs와 상세 StopReason을 얻는 독립 API는 확인되지 않았다.** Response::Raw의 tokens 이름만 보고 generated suffix와 같다고 하지 않는다. return_raw_logits/raw channel로 대체하거나 logprobs를 켜지 않는다. token 문자열/UTF8 bytes/content를 decode·재encode해서 원래 IDs를 재구성하지 않는다. EOS/stop/tool/reasoning 처리와 decode의 비일대일성 때문에 같은content는 IDs exact의 증거가 아니다. 외부length도 Length/ModelLength를 구별하지 못한다.

## 세 실행층의 계약

모델GGUF/원본config/template·요청 bytes·thinking·CPU4/seed42/temp0/ignore_eos/max_tokens2/truncatefalse 및runtime2048 정책은 동일하게 고정한다. 기존 경계입력2047/2048/2049를 재사용한다. observer설정만 run manifest에 분리하며 HTTP payload에는 관측 옵션을 넣지 않는다.

| 층 | 관측 | 허용하는 결론 |
|---|---|---|
| A: 모든 observer off | 기존9256 boundary-fixed baseline API와 **새 sensor CLI의 none mode** 각각 HTTP receipt·content·usage·error·finish_reason | 새binary의 observer-off 외부행동이 기존baseline과 같은지. A raw IDs/상세reason는 unavailable |
| B: full trace off + terminal witness only | 같은새CLI, 공통 최소witness만 on; 최종 rawgeneratedIDs/상세terminalreason/최소연결정보 | A↔B의 외부행동 및 B의 실제 terminal 증거. A raw exact를 주장하지 않음 |
| C: full trace on + 같은terminal witness | B와 동일 witness 구현·호출점·serializer에 fullsensor 추가 | **B↔C rawgeneratedIDs 및 상세reason exact**와 fullsensor 내부연결 검증 |

A에 기존 baseline receipt만 놓고 새binary의 완전off를 생략하면 binary 변경과 witness효과가 섞인다. 따라서 A에는 두 실행의 비교가 필요하다. 과거보존결과는 새실행처럼 재집계하지 않는다. B/C는 같은EXE SHA로 mode만 바꾸는 것이 우선이며 CLI를 두 번 다른소스로 빌드해 observer효과와 binary효과를 섞지 않는다.

**판정:** 세층은 fullsensor가 공통terminalwitness 조건에서 결과를 바꾸지 않았다는 controlled 증거를 제공한다. 그러나 이것만으로 “모든observer off인 A와 C의 raw IDs exact”를 논리적으로 증명하지는 못한다. 원래 강한 raw-ID 비간섭 목표는 삭제하지 않고 `A_to_C_raw_ids_exact_verified=false / unavailable_without_observer`로 남긴다. `B_to_C_raw_ids_exact_verified=true`는 실제비교후에만 가능하다. 단일 full_noninterference_passed를 true로 올려 이 차이를 감추지 않는다.

완전off 상태의 원시내부값은 관측하지 못한다는 한계다. 향후 같은정책의 독립 engine-owned terminal export가 정식으로 있거나 더독립적인 witness를 검증하지 않는 한 이 한계를 남긴다. 두observer가 일치해도 공통 witness의 systematic 오류 가능성은 별도 회귀·cross-check로 줄이는 것이며 수학적 독립증명으로 승격하지 않는다.

## 최소 terminal witness 구현 계약 제안

현재 sensor의 `diagnostics/boundary_trace.rs::sampling`은 journal의 sequence→request mapping이 있을 때만 기록한다. `closed`도 같은 journal에 의존한다. **이 fullsensor registry를 witness의 source로 재사용하면 B에서 독립 witness가 사라진다.** terminal witness는 fulljournal/renderTLS/forwardkey/trace이벤트에서 IDs나 reason을 재구성하지 않는 별도 작은 terminal-only recorder를 둔다. DTO/바이트writer는 기존 공통검증기와 공유할 수 있지만 원시관측 경로와 enabled 조건은 분리한다.

실제 호출점은 `pipeline/sampling.rs`의 nonstream text terminal branch다. 현재652행 부근 `sampling_final` 호출 직전에는 live seq와 **최종reason**가 있다. tool parser가 ToolCalls로 바꾸는 처리가 그앞에 있으며 기존 add_token은 이미 수행됐다. timing/group response send 이후에는 Sequence정리나전송실패와혼동될수있다. 따라서 **현재 terminalreason가 확정된 뒤·seq.update_time_info/group send 전 한 번** snapshot을 가져간다. 원본 is_done/add_token/seed/modelcall을 다시 호출하지 않는다. 현재 boundary는 tools없음/streamfalse/n_choices1이라 이한정scope로 시작한다. streaming/cancel/error/image/speech 경로에 임의 적용하지 않는다.

핵심 field는 run_id/mode/seq.id/response_index/prompt_len/rawtotal_len, `seq.get_toks()[seq.prompt_tokens()..]`의 exactU32LE와 최종StopReason variant/payload다. 범위검사 실패면 witness incomplete이며 빈IDs로보정하지 않는다. rawgeneratedIDs에는 원본vector에있는 EOS/stop special token도 포함하고 HTTPcontent에없는token을 임의제거하지않는다. emitted-visible-content IDs라는 이름을 붙이지 않고 **committed_generated_token_ids**로 명명한다. 요청한 “최종rawgeneratedIDs”의정의가이것임을manifest에고정한다.

Reason은 pre_add/post_add/HTTPfinishreason중 하나를 임의선택하지 않는다. B/C 공통 terminalreason는 실제 terminal branch의 최종reason이며 C의 pre/post samplingtrace는 이를별도로cross-check한다. Length(n)/ModelLength(cap)/Eos/StopTok(id)/StopString(index,offset등 실제payload)/기타원본variant를명시encode한다. Display의length로축약하지않는다. 이번 boundary2047은ModelLength2048을예상하지만 실행전승인하지않는다.

```rust
// Proposed document-only insertion at the existing final terminal branch.
// This API does not exist yet; no code was changed in this review.
if terminal_witness::enabled() {
    terminal_witness::observe_committed(
        *seq.id(),
        seq.get_response_index(),
        seq.prompt_tokens(),
        seq.get_toks(),
        &reason,
    );
}
// Existing sampling_final / timing / group-send order remains unchanged.
```

helper는 enabled時에만 bounded복사·borrowed enumencode를하고 모델제어값을반환하지않는다. mode=None이면 module이링크돼있어도보존/I/O/registry0, mode=Witness/Full이면 정확히같은단일호출을쓴다. fullsensor기존branch와공통helper의중복호출회귀를둔다. terminalwitness는seed/RNG/stopcheck/tokenappend/decoder를호출하지않고rawID·reason사본만다룬다.

## 식별자·거부요청·I/O와 독립성

HTTP nonstream ChatCompletionResponse 생성은현재 `sampling.rs`에서id를 `seq.id().to_string()`으로넣는다. witness seq.id와 runner의같은HTTP응답id를 **실제비교**해case를연결할수있다. 응답id포맷을API일반규격으로확대하지않고현source계약으로고정한다. request numericid와seq.id는다르며같다고쓰지않는다. runner는직렬요청/한choice와requestbodySHA·HTTPresponseSHA를보존한다. witness의case mapping을fulltracejournal에서얻지않는다.

2048/2049 admission거부는Sequence/terminalsampling이없으므로witness도없어야한다. **witness0은forward0의독립증거가아니다.** B에서는HTTP400와terminal0만판정하고forwardcount는unavailable이다. C에서완결admission/forwardaccounting으로forward0을증명한다. B/C간거부case에빈token배열을합성해rawIDs exact로집계하지않고`not_applicable_admission_rejected`로기록한다.

witness전용localmanifest/path는fulltrace설정과분리하며childlocaltrustedpath·freshCreateNew/confinedroot·reparse/ADS/UNC/TOCTOU검증과실패=incomplete를유지한다. rawIDs를stdout/stderr/HTTP에추가하지않는다. terminal에서boundedmemory한번copy후응답송신완료·runner shutdown에flush하는것이우선이다. 엔진결과와witness쓰기성공을같은boolean으로합치지않는다. B/C의공통witness는동일schema/serializer/byteorder·같은오류처리조건이어야한다. 어느mode에도파일쓰기오류가sampling이나stop을바꾸면안된다.

동일seq에terminal1건만허용하고중복/누락/overflow/flush실패시witness gate실패다. B와C의메모리/I/O오버헤드가다르므로성능비교부적격을계속유지한다. observer가스케줄링에아무영향도없다는주장은하지않는다.

## 검증·manifest와 남은 승인

모델없는synthetic회귀는EOS/hidden stop을 포함한 raw suffix·prompt경계·invalidslice·모든 StopReason variant의명시payload·exactU32LE·duplicate/flusherror·none모드I/O0・B/C동일witness・fulljournal없는B동작・추가모델/stop/seed호출0이다. `tools/boundary-observation`의공통DTO/검증기를사용할수있지만기존synthetic10tests가새witness계약을이미검증했다고하지않는다.

root실제후속순서는A새binaryoff↔기존baseline외부대조, Bterminalonly, Cfull+samewitness다. 2047의B/C suffixIDs exact/최종reason exact/HTTPusage exact를필수로하고Cfulltrace가동일suffix/terminalreason와일치하는지도검증한다. fulltrace사전/사후sampledIDs와terminalsuffix의cross-check는연산흐름이맞는지추가근거다. rawsuffix전체를sampledID1개만으로대체하지않는다.2048/2049거부는위의NA계약을적용한다.

보고서에는 `baseline_api_compatibility`, `none_to_witness_api_compatibility`, `witness_to_full_raw_exact`, `witness_to_full_stop_exact`, `full_internal_boundary`, `none_to_full_raw_exact`를별도상태로둔다. 마지막은관측한적없으므로미검증이다. 내용동일을원시ID동일로승격하지않는다. root가상대적fullsensor비간섭의증거수준을수용해도A원시게이트를소급통과로바꾸지않는다. 제품품질승인과도독립이다.

현재source21파일sensor 구현상태는root소유진행사항이다. witness추가시해당sampling/기존diagnostics연결과새witnessmodule/설정/공통DTO/검증기변경의정확한diff·untracked·Cargo 입력을다시snapshot한다. 종전19개설계수나현재21개를그대로완료수로강제하지않는다. 기존snapshot/EXE/실측을덮지않고새source/EXE SHA, 관련Rusttests·fmt·strictclippy·lockedbuild를새로고정해야한다. 이세션은수행하지않았다.

결론은 **3층설계는원래목표에필요한현실적인추가증거이며,공통witness조건의비간섭검증에는타당하지만완전off와rawexact의강한목표전체를충족했다고볼수없다**는것이다. 이잔여를명시하고요구를유지한다. 이번변경은본신규연구문서와coordination뿐이다.
