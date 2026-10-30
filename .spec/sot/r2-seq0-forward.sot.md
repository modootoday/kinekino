---
id: sot:kinekino/r2-seq0-forward
title: R2 요청 전 미등록 seq0 forward의 생성 경로
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# R2 요청 전 미등록 seq0 forward의 생성 경로

2026-10-19 source-only 추적. root의 실제 `runtime/ai-spike/results/gemma270-boundary-internal-diagnostic-r3/internal-trace/journal.json`과 D61 vendor checkout을 읽었다. 코드·raw·모델 수정, cargo·추론·취득은 하지 않았다. **현재invalid2는그대로실패이며ignore/0보정하지않는다.** R2진단이새로구분한원인과아직없는관측을분리한다.

## 실제 journal 사실

raw SHA256 `46db1ce4ff620b9abdd7e18b67e319124a94cbabfc2ed4df73944dc02b35fbde`,engine SHA `97a365ae4349d58fc56831ba94fe314ca47ba4f5252a20e87fd8e402d607ccc6`다. schema2/completefalse/invalid2/dropped0/flush_failures0/row_batches2를읽었다.

| ordinal | 실제 stage와 내용 |
|---:|---|
|1|rows_prepared: origin1, sequence_ids[0], registered[false], is_prompttrue, request/casenull|
|2|invalid: row_arity_or_registration,동일origin1/seq0|
|3|invalid: unregistered_forward,origin1/seq0/is_prompttrue/input_shape[1,3]|
|4|첫normal trace request: internal_request_id2/tokens-2047|
|7|첫cohort sequence_bound:sequence1|

따라서이번invalid2는저장실패stickycounter추정이아니라 **같은미등록row의준비+forward진입두검사**로실제구분됐다. rowarity자체는1이며해당reason의실제조건은registrationfalse다. seq0forward가normalrequest보다먼저진입한것은실제기록이다. 그러나미등록으로ForwardKey가없어정상forward_enter/return/modelmetadata/rope/sample/close이벤트가없다. **해당forward의raw3개IDs·실제return성공/오류·출력token·상세stop·flush후상태는이journal에서관측하지못했다.** shape3만으로token문자열을역산하지않는다.

## 직접 source chain과 조건

기준source는 `runtime/ai-spike/source/mistral-gemma-cap-source-r1`이며base `4400935451da5e2dc7379a3f92fbbada66557f6c`와D61sensor다. 읽은 `mistralrs-core/src/lib.rs` SHA256은 `dea6f898efc367c16256768608bd2e3059f97341c6e258eabe32deea2c73b2db`다.

| source 호출점 | 역할·조건 |
|---|---|
|mistralrs-cli/src/main.rs:33 `#[tokio::main]`|기본multithreadTokio진입. CPU옵션이singlethreadruntime을뜻하지않음|
|server-core/src/mistralrs_for_server_builder.rs:933/961|MistralRsBuilder를만들고builder.build().await 호출;다른builder분기1150/1178도동일core진입|
|core/src/lib.rs:701–702 `MistralRsBuilder::build`|`MistralRs::new(self).await`|
|lib.rs:1514 `async fn new`→1589부근|create_engine_instance로engine thread/channel을먼저생성|
|core/src/engine/mod.rs:552|Engine::new에서sensorinitialize. dummy보다먼저recorder존재|
|lib.rs:1619–1636|currentTokio runtime가CurrentThread아닌지검사. !distributed::is_daemon && is_multi_threaded && !loaded_for_uqff_write && categoryText또는Multimodal이면dummy실행|
|lib.rs:1637–1677|block_in_place에서내부NormalRequest 생성: **id0,Completion{text:"hello",echo_promptfalse,best_ofNone},max_lenSome1,SamplingParams::deterministic(),seedNone**,streamfalse/constraintNone/logprobsfalse/truncatefalse|
|lib.rs:1680부근|engine sender에blocking_send하고response channel이닫힐때까지drain. debug의명칭은“Beginning dummy run.”|
|lib.rs:1699–1700|engine logger counters reset. trace recorder reset은아님|

이는CUDA전용graphwarmup이나idleemptyscheduler가아니라 **일반initialengine의내부dummy generation 요청**이다. 목적은source주석의dummy run과loggerreset으로명시돼있다. 모델이ready되기전짧은요청을한번수행하는경로이며,정확히어떤최적화/kernel을예열했는지성능효과는이source조사로승인하지않는다. 기본CLI multithread/text-onlyGemma·UQFFwrite아님·daemon아님은기록된실행profile과source조건에맞는다.

## 왜 seq0이 trace 등록을 거치지 않는가

1. engine/add_request.rs:322–378의sensorbegin/render_scope는 **Chat/MultimodalChat arm 안에만**있다. 위dummy는Completion이므로이arm에들어가지않는다.
2. 같은파일379–401의Completion arm은tokenizer.encode_fast(text.clone(),true)를직접호출한다. chattemplate/BOSsuppressionhook도거치지않는다. 따라서source상dummy의입력은"hello"를specialtokens허용으로토큰화한것이다. 실제token3개인지는raw인자값이없고이번journal의[1,3]관측과대응하는수준이다. 새tokenizer실행으로값을만들지않았다.
3. traceadmission은journal.requests에없는request0이면이벤트없이return한다. Sequence::new_waiting은별도enginecounter를ID로쓴다. engine/mod.rs:722은counter를0으로시작하고add_request.rs:794에서읽는다. 1051행에서증가하므로첫dummy는seq0,다음cohort는seq1이된다. requestid0과sequenceid0의우연한값동일을일반동일성으로쓰지않는다.
4. add_request.rs:829는bind(request.id,*seq.id())를호출하지만boundary_trace.rs::bind는request0이journal.requests에없으면return한다. 그래서seq0registry등록이없다. bind함수가호출되지않은것과구분한다.
5. DefaultScheduler의nonempty prompt분기→Pipeline::step/submit_step→pipeline/mod.rs:2036의GemmaInputsProcessor::process_inputs→gemma3/inputs_processor.rs674rows(seq_indices→seqIDs,is_prompt)로진입한다. 이때seq[0]/registeredfalse가ordinal1/2에기록된다.
6. Gemma3SpecificArgs의PreparedRows가MultimodalModel::forward로전달되어boundary_trace::forward_enter가동일seq0을조회한다. 미등록조회는ordinal3 invalid를기록하고keyNone을반환한다. **기존model self.forward 호출은계속된다.** 이후keyNone인metadata/positions/returnhook은이벤트를남기지않는다.

이source연결은request0/seq0/요청전prompt[1,3]/그다음cohortseq1이라는실제기록을설명한다. dummy의직접lifecyclemarker·inputIDs·return/response원문이없으므로 “sourcecallsite와증상일치”와 “모든dummy실행값실측”은구분한다. 다른원인가능성을임의로모두배제했다고하지않지만,이번origin1의두invalid가저장실패나잘못된length라는이전가설은실제reason으로좁혀졌다.

## 숨은 forward와 비간섭 영향

cohort정상forward_enter1개는engine전체forward1개가아니다. source상startupdummy가추가로발생하고이번journal은그진입을invalidcontext로만봤다. 현재missingkey관측으로returncount가없어전체성공forward수를1또는2로단정하지않는다. 다음sensor는**모든model진입**을phase별로account하고시작/끝/오류가맞아야한다.

startupdummy는seedNone/deterministic/max_len1이며cohortseed42/temp0/ignore_eos/max_len2와같은control이아니다. 이를cohort로편입하거나공통DTOControls2를dummy에씌우지않는다.loggerreset은성능통계에서dummy를제외할뿐KV/cache/RNG/모델state·tracehistory를원상복구한다고보장하지않는다. dummy가후속수치결과에영향없다고추정하지않고기존off/witness/full모든층에동일동작을유지한다. trace활성화때만dummy를끄거나코드에서삭제하면init정책과예열state가달라져비간섭비교를해친다.

기존threecaseHTTPstructure/품질0·성능비교부적격은유지한다. r1invalid2와이번r2invalid2는별도engine/실행이며r2reason으로r1counter를소급0으로고치지않는다. 같은source경로가r1에기여했을수있다는추론은가능하나r1reason미관측상태를유지한다.

## 최소 explicit lifecycle/phase accounting 제안

권장은 **dummy생성callsite에서의명시적internal origin**이다. seqID==0/inputwidth3/requestid0/"hello"라는값패턴만보고unknownforward를면제하지않는다. 외부request가이값을모방해trace를우회하지못해야한다.

1. lib.rs의기존dummy요청생성직전에서기존init조건과purpose `startup_dummy_generation`의lifecyclebegin을등록한다. trustedinternalconstructor가EngineSessionID+LifecycleID를발급하고해당NormalRequest/sequence까지전달한다. 기존publicHTTPschema에는origin필드를추가하지않는다. defaultexternalorigin은CohortOrUnregistered이며사용자입력/ID값으로internalmark를만들지않는다.
2. add_request의normal공통진입은origin을먼저읽는다. CohortChat은기존begin/plan검증, StartupDummy는별도lifecycle registry에등록한다. bind도각namespace에서request→sequence를명시연결한다. cohort requests/completed3개와lifecycle counter를따로유지한다. callback/TLS를 async 전체에 유지하지 않고 request 소유 carrier 또는 명시 registry로 전달한다.
3. rows/forwardhook은enumkey `Cohort{case,request,seq}` 또는`Lifecycle{purpose,id,request,seq}`를전달한다. 최초 row 준비·실제 enter·actual IDs/shape·metadata/RoPE·return·sampling reason·response channel drain·종료를 phase별로 기록한다. origin 불명·registeredfalse는 계속 invalid다. 정상 startup이라는 명칭만으로 무시하지 않는다.
4. lib.rs에서기존response drain뒤lifecycleend를기록한다. 단순received_any는성공/실패를구별하지않으므로실제Response variant/오류/생성길이와마지막reason를기록하고현재원본drain/sampling순서는바꾸지않는다. `logger.reset()`직전/후marker는실제위치에둘수있지만trace를reset하거나앞events를버리지않는다.
5. rawschema는새phase/namespace를명시한version으로고정한다. topcomplete는cohort3closed뿐아니라enabledlifecycle의declared/entered/returned/closed accounting·invalid/dropped/storagefailure0까지필요하다. complete=true를얻기위해기존invalid를감산하는변환은금지다. 새로운actual로만새상태를판정한다.

```rust
// Document-only shape: no vendor code was edited.
enum TraceOrigin {
    Cohort,
    StartupDummy { lifecycle_id: u64 },
}
// Only the existing internal dummy construction site may create StartupDummy.
// Unknown requests/rows never acquire it by matching request=0 or width=3.
```

기존NormalRequest에privatecarrier를추가하면crate내모든structliteral/initializer와distributedclone경로를점검해야한다. 범위가크다면core내trustedregistry연결을택할수있지만명시requestownership/nonce/EngineSessionkey를확인하고stage완료때정리한다. 선택에따른정확한changedfiles는새snapshotwhitelist에리뷰하고현재21파일수에억지로맞추지않는다. lib.rs는새실제변경대상으로추가선보존해야한다. 이설계는실제RustAPI구현이아니다.

## 회귀와 실제 재실행 게이트

모델없는Rust회귀부터: initialruntime조건(CurrentThread/multithread/daemon/UQFF/category),내부dummy만origin발급,외부request0/seq0모방거부,Completion분기identity전달,rows→forward 동일 origin,namespace별counter,missingreturn/drain오류/중복 close/저장실패시incomplete를검증한다. cohort3개와startup1개를의도적으로뒤섞은syntheticrun에서unknown을그냥필터하면실패해야한다. 공통DTO는3cohort만projection하되lifecycle를별도receipt로완결검증한뒤에만허용한다.

root후속actual은새source/EXE/빌드입력·patch·schema·동일model/3payload를선보존한별도label에서수행한다. dummy자체를제거/추가실행하지않고원래call을한번관측한다. stage 순서 lifecyclebegin→dummyrequest→seqbind→rows→actualforwardenter/return→sampling/response→lifecycleend→cohortrequest가완결됐는지확인한다. dummy 실패를 received_any만으로 성공 처리하지 않는다.

원래R1/R2raw·97a3EXE·현재D61source는변경없이보존한다. 새phase회계가맞아도fullsensor상대비간섭은별도Bterminalwitnessonly↔Cfull+samewitness대조다. 완전offA rawIDs미관측한계와performance부적격/품질0은계속유지한다. 이번조사는원인sourcechain과정책수정안까지만완료했다.
