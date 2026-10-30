---
id: sot:kinekino/gemma270-raw-journal-observation
title: Gemma270 raw journal → 공통 Observation 변환 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Gemma270 raw journal → 공통 Observation 변환 계약

2026-10-19 source-only 설계. vendor/model-spike/공통library 코드는 변경하지 않았고 cargo·추론·취득·변환 실행도 하지 않았다. **입력 raw schema2와 출력 Observation schema1은 서로 다른 계약**이다. 단순 schema_version 숫자 치환이 아니다. 변환은 관측값의 명시적 projection이며 sensor 완결성·source/EXE provenance·비간섭을 대신하지 않는다.

## 고정 기준과 실제 실행 상태

vendor 기준은 [D61 R2 source manifest](../../../../../evidence/ai/engine-context-cap/gemma270-internal-trace-diagnostic-source-r2/manifest.json) SHA256 `d61d063a6b06ff02093855737591f9f6bd4140d7f3b5c4f741f45279f3e1090d`다. 이 manifest의 **after 파일**을 읽고 hash를 확인했다. mutable checkout의후속변경을이pin에섞지않는다.

| 원본 source | 고정 사본 / SHA256 |
|---|---|
| diagnostics/boundary_trace.rs | after-18.rs /153f24c12f3f11609fc10a46c949c17cab50d962c841e6f1afc76445474de697 |
| diagnostics/boundary_trace/journal.rs | after-19.rs /9a25c0e4b2e301fc94f3993f5d58b89fde49f24dbdfcfe9bd264f987fd23d7f6 |
| diagnostics/boundary_trace/storage.rs | after-20.rs /6eecf9908702c3ac66bb96314906b5c91139ef68fb45751ff4a843f00c269e7c |

출력은 `tools/boundary-observation/src/domain.rs`의 Observation(schema1), SHA `18b00d9b2f964da174dd2d5c8b36ca0b5df727830a0ba0a6949c971be1be6bcc`; 검증은 `src/application.rs::verify` SHA `dec50f5c79493109dec01a6d0b4448694d5c62f41b0f924bf7853e551209ae6f`다. 이source는공급값검증만하며capture진위를인증하지않는다.

현재 읽은 actual result는 R1 `gemma270-boundary-internal-trace-r1`뿐이며 R2 schema2 **실제 실행 journal은 아직 확인되지 않았다**. D61은R2source고정이지새모델실행의증거가아니다. [R1 actual](../../../../verification/ai/generation-gemma270-internal-trace-actual.md)의rawschema1/completefalse/invalid2/dropped0은그대로실패다. R1 지원은별도명시reader로만하며schema2필드를있던것처럼채우지않는다.

## 변환 입력 묶음과 join 규칙

필수묶음은rawjournal bytes+SHA, rawplan bytes+SHA, frozen3request bytes+SHA, 각count/generation HTTP receipt+SHA, worker/server-start·CLIargv/childenv·model inspection·EXE/source/build receipt다. expected engine/model/3request SHA는이묶음외부의trustedpin에서받아야하며journal자기주장을그대로expected로복사하지않는다.

join key는run_id→plan.case_id/expectedpromptlength→request event의internal_request_id→sequence_bound.sequence_id→(request,sequence,call)다. R2 rows_prepared.origin도추가연결한다. filename2047만보고case를고르거나최근event/시간인접으로zip하지않는다. case_id/promptcount/messageshash/requestbytesSHA가모두대응해야한다. internalrequest/sequence/call은JSON정수만허용하고checked u64/usize conversion을쓴다. float·문자열ID·음수·overflow는실패다.

rawevent ordinal은양의정수·유일·엄격증가이고동일event를두case/forward/sample에재사용하지않는다. stage마다허용필드/필수type를고정하고unknownschema/stage는조용히버리지않는다. raw2의diagnostic stage는명시목록으로sidecar에보존한다. 일대일로선택할수없는중복·누락은변환오류다. 없는값을0/false/빈배열/“expected”로채워DTO를완성하지않는다.

## top-level·Controls 매핑

| Observation 필드 | 정확한 원천 | 변환 전 실패 조건 |
|---|---|---|
|schema_version|**출력DTO정의1**|rawschema2→DTO1reader가고정pin과다르면실패. rawversion을1로속이지않음|
|complete|raw.complete를보존; 외부완결검사실패면승인불가|rawfalse를재계산해true로변경금지. 필수mapping실패는DTO생성오류|
|invalid_events/dropped_events|raw동명u64|누락/음수/overflow 실패. invalid event를필터한뒤0설정금지|
|engine_sha256|raw.engine_sha256|trustedEXEbytes/SHA·server-start·build와 일치하지 않으면 실패|
|model_sha256|raw.declared_model_sha256|**declared**이므로inspection/실모델bytesSHA와trustedpin의별도대조필수|
|cases|plan3case와normalrequest3개의strictjoin|중복/여분/unknown_request/서로다른requestID아님 실패|
|controls.max_tokens|request.data.controls.sampling_params.max_len 및원본HTTPmax_tokens|각case2 exact;null/alias충돌실패|
|temperature|sampling_params.temperature 및HTTP|수치0/0.0만. 문자열0/NaN/추론한default금지|
|seed|request.controls.seed 및HTTP seed|u64 42 exact;실제RNG적용검증으로확대하지않음|
|ignore_eos|sampling_params.ignore_eos 및HTTP|booltrue exact|
|truncate_sequence|requestcontrols·HTTP·admission모두|boolfalse exact;불일치실패|
|n_choices|sampling_params.n_choices 및HTTP n/default의명시정규화근거|1 exact;unknowndefault를1로발명금지|
|stream|controls.stream 및HTTP|false exact|
|prefix_cache_entries|maxseq/speculative와같이**외부실행증거**|아래외부표참조;raw없음에서0추론금지|
|max_sequences|실제CLI/workerresolvedconfig|1 exact|
|speculative|실제CLI/modelprofile의resolvedmode|false exact;flag없음만으로false확정하지않음|

3case의controls를필드별exact검사한뒤동일한공통Controls하나로묶는다. thinking=false/reasoning_effort=null/tools_present=false는DTO에없어도raw/HTTP/정규화controls에서별도검사한다. CPUthread4·offline·pagedoff·native/runtime·engineflavor도DTO에모두담기지않으므로receipt로보존한다. DTO의필드집합이작다는이유로통제를삭제하지않는다.

## Case·BOS·Admission 이벤트 매핑

| Case 필드 | raw event/field | exact 검증 |
|---|---|---|
|prompt_len|plan.prompt_tokens와admission.data.prompt_tokens|2047/48/49각1,IDbytes/4와같음|
|request_sha256|plan.case.request_sha256와trusted원본requestbytes의**정의된serialization/hash**|HTTPreceipt에저장된request도같아야함. raw메시지hash와혼동금지|
|internal_request_id|request.internal_request_id|request event1개,동일case모든 event 일치|
|sequence_id|sequence_bound.data.sequence_id|accepted1개;rejected는bind없음+closed.seqnull|
|accounting_closed|case_closed event 및raw.completed_requests membership의결합|둘다필수;event만보고true/빈로그로closed추론금지|
|render_event/admission_event/closed_event|각event.ordinal|원본ordinal그대로.새1..n재번호부여금지|
|rendered_utf8|rendered.data.rendered_utf8|JSON각정수0..255→exactbyte;UTF8valid·<=65536;재렌더금지|
|prompt_ids_u32le|rendered.data.prompt_ids_u32le|exactbyte·4배수·prompt_len×4·vocab<262144|
|bos.text|rendered.data.bos|nonnull문자열;empty실패|
|bos.requested_generation_prompt|rendered.data.requested_generation_prompt|true exact|
|bos.requested_special_tokens|rendered.data.requested_special_tokens|true exact|
|bos.starts_with_bos_observed|rendered.data.starts_with_bos_observed|true exact;변환기재계산값을실제분기로대체금지|
|bos.effective_special_tokens|rendered.data.effective_special_tokens|false exact;rawUTF8실제prefix도교차검사|
|admission|admission.data.decision|accepted/no_generation_capacity/prompt_too_long의enumliteral만매핑|
|http_status/usage|각generation HTTPreceipt|아래외부표;admission에서status를합성금지|

case_closed outcome은accepted의response_send_completed/rejected의admission_rejected만현profile에대응한다. null sequence와 bind 부재만으로 거부 case의 forward0을 증명할 수 없고 raw complete·진단오류 부재·전체 이벤트 accounting이 필요하다. 공통verify가확인하는render<admission<close외에adapter는request<render,bind의admission뒤/forward앞,completed IDs의exactset도검사한다.

## Forward·LayerPositions 매핑

| DTO 필드 | raw source | 변환 전 exact 조건 |
|---|---|---|
|entered_event/returned_event|forward_enter/forward_return.ordinal|동일request,seq,call의1:1pair|
|ordinal|forward_enter.data.call|case별0부터연속;events정렬로새call번호발명금지|
|sequence_id|forward_enter.data.sequence_id|boundseq와같음;return/meta/layers도같음|
|input_ids_u32le|forward_enter.data.input_ids_u32le|byte그대로. renderedprompt의해당contiguousslice와대조|
|input_shape|forward_enter.data.input_shape|정확히rank2 `[1,width]`;rank축자동추가/제거금지|
|offset|forward_enter.data.seqlen_offsets[0]|**길이정확히1**,checkedusize;min/첫값임의축약금지|
|native_capacity/runtime_capacity|동일call의model_metadata.data.native_capacity/runtime_capacity|각32768/2048. plan/상수로채우지않음|
|cpu/paged/packed/pixels_present|forward_enter.data동명bool|true/false/false/false exact|
|returned_successfully|forward_return.data.success|booltrue;누락return/오류→실패|
|layers[].event/layer|rope_positions.ordinal/data.layer|동일call내18개distinct0..17|
|layers[].actual_positions_u32le|rope_positions.data.actual_positions_u32le|실제byte그대로. offsets로재생성금지|

rawextra인input_dtype는U32,actualpositionsdtype는U32/shape와 count 일치,hostposition_ids/context_lens·is_prompt·origin·model_metadata.layers18/slidingwindow512·return.logits_shape도sidecar사전검사한다. DTO에없는metadata를무시하면잘못된join이나phase를숨길수있다. 그형식은D61source/실제고정processormetadata에서정의하고라벨만보고값을추정하지않는다.

R2 rows_prepared는internal_request_id/case_id가null일수있다. `origin`으로forward_enter와연결하고sequence_ids/registered/is_prompt를검사한다. rows중empty/multi/unregistered·unmatchedorigin/extra준비가있으면현strictprofile에서는실패하며정상성공rows만골라DTO를만들지않는다. row_batches와rows_prepared개수도 일치를 요구한다. 실제forward카운터가없는sensor누락가능성은완결accounting과별도provenancegate로판정한다.

verify는forward0..N의offset이coverage와같고inputIDs가promptslice와같으며모든layerpositions가covered..end이고마지막coverage가prompt2047전체인지검사한다. 최대32forward의분할prefill은허용한다. adapter가한전체forward로합치거나ordinal을조작하면안된다. raw모든enter/return/meta/layer가정확히한DTO참조또는명시diagnostic으로account되어야한다.

## Sampling 4event → DTO Sampling 매핑

raw에는sampling_before_decision/pre_add/post_add/final이있고DTO에는event필드3개가있다. 네개를시간근접으로뭉치지않는다. 현accepted2047은**한sample**만허용한다.

| Sampling 필드 | 정확한 매핑 |
|---|---|
|before_event|**sampling_pre_add.ordinal**. reason이계산된뒤이벤트이며before_decision을대신넣지않음|
|after_event/final_event|sampling_post_add.ordinal / sampling_final.ordinal|
|sequence_id|네event의data.sequence_id 모두같음|
|raw_len_before|sampling_pre_add.data.raw_token_len;before_decision값과같아야함|
|raw_len_after/generated_len_after|sampling_post_add.data.raw_token_len/generated_len|
|prompt_len|네event data.prompt_len 일치값|
|sampled_id|네event data.sampled_id exact일치;vocabbound|
|pre_add_reason/post_add_reason/final_reason|각pre_add/post_add/final.data.reason의variant→StopKind|

sampling_before_decision은DTO의직접eventfield가없어receipt에남긴다. lastreturn<before_decision<pre_add<post_add<final<close, before_decision.reason=null,원시길이2047→2048,post/final길이동일·generated1을사전검사한다. finalevent만보고이전값을복원하지않는다. runtime_capacity2048/generation_budget2도네event에서대조한다.

Stop 변환: null→None,model_length→ModelLength,generation_budget→GenerationBudget,eos→Eos,원본literalother→Other다. **unknownvariant는Other로흡수하지않고실패**한다. `value`payload는DTOenum에서사라지므로adapter가model_length.value==2048/generation_budget.value==2를원문에서검사하고receipt에보존한다. 모델길이variant가맞아도value999를버려통과시키면안된다. 이번verify는pre/post/final모두ModelLength만허용해다른reason는정상적으로실패한다. HTTPfinish_reason=length로이enum을채우지않는다.

현재DTO의sampledID/1개length는terminalgeneratedsuffix전체를직접보존하는witness가아니다. [terminal witness 계약](gemma270-terminal-witness-noninterference.md)의B/C rawexact나A완전off미관측문제를이변환으로해결했다고하지않는다.

## 필요한 외부 증거

| 외부항목 | 원천과 대조 | raw만으로 불가능한 이유 |
|---|---|---|
|HTTPstatus|generation-{2047,2048,2049}.json.response.http_status|accepted를200/rejected를400으로만들면측정이아님|
|Usage|response.body.usage.prompt_tokens/completion_tokens/total_tokens → Usage.prompt/completion/total|2047/1/2048실제수치. rejected는usage필드부재또는명시null,부분object는실패|
|HTTPerror|rejectedbody.error.type/messagecode와상태|DTO에없지만전송오류를admission거부로혼동하지않아야함|
|count|countreceipt각200/input_tokens2047/48/49 및같은messages/controls|별도비추론요청이며generationevent로섞지않음|
|workercontrols|actualCLIargv/resolvedconfig·childenv/workercompiledinputs|prefixcache0/maxseq1/speculativefalse·CPUthreads/offline를raw에서추정못함|
|engine/modelidentity|실제EXE/모델byteSHA·inspection/source/buildreceipt|rawdeclared값은인증아님|
|requestidentity|고정입력manifest·plan·HTTPrequestbody의정의된hash|JSON표현순서/Eitherinternaltag와HTTP표현차이를정규화했다고값을바꾸지않음|
|완결flush|최종journalstableSHA/worker정상종료·flushreceipt·failedstate|parseable snapshot이마지막write성공/후속오류없음을자동보장하지않음|

HTTPresponse usage의성능수치는DTOusage로넣지않는다. payloadseed전달은actualCPU RNG적용증거가아니다. transport오류/부분response/timeout/missingreceipt는status0또는usage0으로채우지않고실패한다. externalreceipt가같은run/engine/model/case라는연결이없으면cross-run결합을거부한다.

## DTO에 없는 R2 진단은 버리지 않는다

raw schema2의mutation_revision/row_batches/flush_attempts/flush_failures/first_storage_failure 및invalid·rows_prepared·unknown_request reason/context는별도conversion receipt에원문SHA와eventordinal/JSONpointer로전부연결한다. raw complete가 true여도 flush_failures>0, first_storage_failure nonnull 또는 invalid 이벤트가 있으면 fail-closed다. mutationrevision과flush횟수의관계는sourcecounter정의대로검증하고event수와같다고가정하지않는다.

모든 event의 매핑 위치 또는 sidecar 보존 이유를 기록하고 unknown·duplicate·orphan을 버리지 않는다. DTO verify는 전역 ordinal 재사용·여분 raw event·진단 origin·storage error payload 전체를 검사하지 않으므로 adapter 사전 검증이 필요하다. DTO 성공 후에도 provenance·sensor accounting·shared contract·noninterference는 별도 상태로 유지한다.

R1 schema1은별도reader경로로진단 사유 필드의 부재를 unavailable로 기록한다。materialize가능한경우에도Observation.schema1/complete=false/invalid_events=2/dropped0를그대로보존하여verify는incomplete accounting으로실패해야한다. rows/flush기록없는R1을R2필드의0값으로보충하거나reason 추정으로 invalid를 차감하지 않는다。DTO를만들수없는필수누락이면conversion_error와raw상태를보존하고“불변통과DTO”를생성하지않는다.

## root 구현·검증 후속

변환기는Rust application usecase와rawschema별infrastructuredecoder/외부receiptadapter·domainjoin정책을분리한다. libraryverify의I/O없는성격은유지한다. 외부source/EXE/모델의성공hash를검증하지못하면caller가expected를자기생성하지않도록한다. DTO JSON·SHA와conversionreceipt·검증Result를새label에보존하고raw를덮어쓰지않는다.

모델없는회귀는D61shape syntheticvalid→DTOverify통과,모든필드누락/타입오류/unknownenum,중복case/seq/call/layer/globalordinal,pre_add와beforedecision혼동,model_lengthvalue잘못됨,orphanrows/invalid/flushfailure,외부 HTTP status/usage 누락·cross-run request SHA·worker controls 불명 및 R1 completefalse/invalid2 유지가 포함된다. 이는 새 테스트 구현 제안이며 이 세션에서 실행하지 않았다.

R2 actual이 없는 현재는 D61 source 대응 설계만 완료했으며 shared contract 실제 통과는 없다. root가 새 actual journal을 고정한 뒤 raw 완결 gate→외부 provenance/controls gate→변환→pure verify→결과 보존 순서로 진행한다. synthetic/DTO 통과는 sensor 완결성이나 capture 진위를 대신하지 않는다. root/vendor/model-spike/library 코드·모델·기존 evidence는 변경하지 않았다.
