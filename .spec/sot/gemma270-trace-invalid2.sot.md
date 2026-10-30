---
id: sot:kinekino/gemma270-trace-invalid2
title: 첫 Gemma270 trace invalid2 원인 범위와 다음 진단
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 첫 Gemma270 trace invalid2 원인 범위와 다음 진단

2026-10-19 source-only 조사. root의 첫 실제 trace `gemma270-boundary-internal-trace-r1` 결과와 현재 sensor source를 읽었다. **원시 journal/manifest/카운터·코드·모델을 수정하거나 build·추론·취득하지 않았다.** invalid2의 정확한 발생 이유는 현재 파일에 기록돼 있지 않아 확정하지 않는다. trace complete=false와 full boundary/공통검증/비간섭 미검증을 유지한다.

## 실제 증거와 위치

run root는 `runtime/ai-spike/results/gemma270-boundary-internal-trace-r1`이다. journal은 `internal-trace/journal.json`이며 **manifest.json은 internal-trace 안이 아니라 run root**에 있다.

| 파일 | SHA256 |
|---|---|
| internal-trace/journal.json |574b4a87ab54aeeddfe94b3d4dc43d9dec2f162bca6952efd151ce6a297e266c|
| manifest.json |d8f872a984d9d7243f57a3150844742209ae40789d78383710e9f087275fce52|
| trace plan (manifest 기재) |67eb768eab06fd273745ea376130f606cef612125fdeded3fb83577973540ce6|

실제 engine SHA는 `d56e22092f0f0b1a5a9c914f8b3d1d6e3bcde3062fe8329618a58dd5cb3f80b0`다. journal은 parseable, invalid_events2/dropped_events0/complete=false, completed_requests=[2,4,6]이다. ordinal1..38의 이벤트에는 request3/rendered3/admission3/case_closed3, sequence_bound1, forward_enter1/return1, model_metadata1, rope_positions18, sampling4가 있다.

| case | count | generation | journal 연결 |
|---|---:|---:|---|
|2047|200/input_tokens2047|200/completion1,total2048|request2→seq1→call0, sampling_final model_length2048, closed ordinal30|
|2048|200/input_tokens2048|400/no_generation_capacity|request4, closed ordinal34, sequence 없음|
|2049|200/input_tokens2049|400/prompt_too_long|request6, closed ordinal38, sequence 없음|

rendered 원문 bytes4137/4139/4141, U32LE IDbytes8188/8192/8196로 length limit 범위 안이다. 세 admission은count2047/2048/2049,cap2048,truncatefalse다. unknown_request 이벤트는없고dropped0이다. 정상 이벤트가 완전해 보인다는 이유로 invalid를 무시하지 않는다. journal에 없는 미등록forward가 없었다고도 단정하지 않는다.

## invalid 증가 지점 전체 판독

source root는 `runtime/ai-spike/source/mistral-gemma-cap-source-r1/mistralrs-core/src/diagnostics/`다. 조사시점 boundary_trace.rs SHA `ab40c695c96deb1b57b0d8a1926460b46c9043c2f8a04aa2956499240610ee43`, journal.rs `6246e78e14e46fae06d9bd9c912059a6a57e3dca6832642f9a07ebee10ac9e77`, storage.rs `ad063832024cada0eff2448a4cf10f59f287ff14992b4f2b9685a021408ada44`다. 새 snapshot과 실제21source/EXE의compiled-input 연결은 root가 보존한 manifest로 확인한다.

| 현 boundary_trace.rs 위치 | 증가 조건 | 이번 증거로 좁힐 수 있는 범위 |
|---|---|---|
|78 flush|sticky failed=true인 **매 flush마다** invalid+1|최초실패/발생횟수/flush횟수 미기록,2건의논리오류라는뜻아님|
|114 begin|정규화messages hash매칭실패|unknown_request를push하므로 현재 unknown0/dropped0과 맞지않음|
|125 begin|중복request ID 또는중복case slot|reason이벤트없음. 실제3요청 정상등록만으로 숨어있는중복호출을증명/배제못함|
|172 rendered|prompt>65536B 또는IDs>2049|기록된3render는범위안. 별도초과render라면event미기록가능|
|193 admission|plan count불일치/cap!=2048/truncate|기록된3admission은모두일치|
|223 bind|중복sequence 또는한request에2개sequence|기록된bind1개는정상shape; hidden중복은별도reason없음|
|240 rows|row수!=1 또는미등록seqID|잘못된row의값/횟수/호출context를전혀기록하지않음|
|260 forward_enter|단일seq가registry미등록|rows가미등록seq를반환하면여기서두번째증가가능|
|265/270 forward_enter|nonCPU/2049초과 또는input U32읽기실패|실패시enter event없어기록된성공1만으로추가실패를배제못함|
|294/301 positions|nonCPU/2048초과 또는U32읽기실패|관측call0의18layer는모두기록. 다른call의실패가능성은정보없음|
|journal.rs42 push|event Value직렬화실패|event자체가저장되지않음. Value→Vec 정상구조상낮은후보이나counter만으로확정불가|

### 중요한 두 가지 중복 계수 경로

1. `rows([unregistered_seq])`는 invalid+1 후에도 **Some(ids)**를 반환한다. 이어서실제Gemmaforward의 `forward_enter`가 같은ID를못찾으면 invalid+1을더하고None을반환한다. 따라서 **한 원인의 두 검사로invalid2**가 가능하다. rows(empty/multi)는+1이지만forward_enter의len!=1은추가증가없이None이므로 동일모양이아니다. sensor가None을주더라도실제모델forward를막는코드는아니다.
2. `failed`는 edit의journal mutex poison,flush의journal/sinkmutex poison, sink.write 실패에서true가되고해제되지않는다. flush는 이를읽을때마다+1한다. sink.write실패는해당 snapshot을만든 **뒤** failed=true가되므로다음flush부터invalid에반영된다. 단일storage오류뒤두flush로invalid2가가능하다. 최초실패errno/op/ordinal이없다. parsed최종파일만으로과거write/sync_all실패를배제하지못한다.

이둘은현재동등하게검토할후보이며 “rows가원인” 또는 “디스크실패”로확정하지않는다. flush의counter증가방식에문제가있어도counter를0으로바꾸거나최종complete를true로고치지않는다.

## 실제 호출 경로에서의 제한

### count/template

`anthropic_count_tokens`는정규화parse후 `Request::Tokenize`를보낸다. `engine/add_request.rs::tokenize_text`는processor.process→encode만호출하고GemmaImageProcessor::process_inputs/rows/모델forward를호출하지않는다. generation의render_scope는동기호출후RAII로복구된다. 따라서정상count경로만으로rows invalid2가발생한다는설명은source에없다. Tokenize자체가normal begin으로중복진입했다면unknown/중복호출reason이필요하며현재journal만으로가정하지않는다.

### normal generation/admission/rows

Gemma inputs_processor.rs:674의rows는 `seq_indices.iter().map(|idx| *input_seqs[*idx].id())`에만연결돼있다. seq_indices는get_prompt_input/get_completion_input의실제출력이다.정상 2047은admissionaccepted→Sequence::new_waiting→bind→scheduler→process_inputs→rows→Gemmaforward순서다.2048/2049는admission에서return하여sequence/rows를만들지않는다.

engine/mod.rs DefaultScheduler분기는 `!scheduled.completion.is_empty()` 및 `!scheduled.prompt.is_empty()`에서만pipeline.step을부른다. **정상idle/empty scheduler가무조건rows(empty)를호출한다는주장은맞지않는다.** 내부get_prompt_input필터로seq_indices가비었거나별도직접process_inputs호출이있었는지는rows context없어미확인이다. 실제모델출력뒤새decode가없어야하는cohort지만 sensorevent가없는추가실행을counter만으로배제하지않는다.

### warmup/startup

trace초기화는Engine::new552행으로모델loader준비뒤다. Engine::new의graph precapture는pagedmanager조건을가지며CPU/pagedoffprofile의일반경로와다르다. lib.rs의cuTile warmup과multimodal CUDA eagerwarmup은feature/device별경로다. AMoE에는dummySequence학습루프도있지만이번profile의실행증거는없다. 이런이름을찾았다는이유로원인으로승격하지않는다. rowcallsite에phase/device/seqids를넣어실제caller를관측해야한다.

### runner 읽기와저장

tools/model-spike/src/application/generation_trace.rs:98이후는generation3회뒤,server종료전에journal을250ms간격최대10초poll한다. closed response뒤flush가늦을수있다는주석이있다. 이점은원시파일재쓰기중부분read/parse실패가능성을설명하지만, **read 실패는runner parse결과이지sensor invalid 증가함수가아니다**. Windows공유조건이실제sink.write/sync_all오류를일으켰는지는errno없어미확인이다. runner를범인으로단정하지않는다.

Sink::write는ordinal낮은snapshot을무시하고,같거나큰ordinal에seek/write_all/set_len/sync_all한다. event ordinal은invalid증가만으로바뀌지않는다. 동일ordinal의snapshot이라도invalid수는달라질수있고현재last_written은이구분을하지않는다. run중snapshot을여럿보존하지않아현재파일로invalid가어느event사이에증가했는지복원할수없다.

## 최소 reason 진단 추가 제안

목표는새실행에서원인을분류하는것이며r1의카운터를다시해석해통과시키는것이아니다. 일반로그에원문/IDs를복제하지않고기존ownedjournal에boundedreason을추가한다.

1. 각invalid++를 `record_invalid(reason_code, context)`로연결한다. 기존invalid증가의뜻은첫진단판에서유지하고 reason별건수·first발생시점·현재eventordinal·별도mutation_revision을저장한다. invalid이벤트가push실패를다시호출하는재귀를만들지않도록별도boundedreason표를쓴다. dropped/overflow도보존한다.
2. rowsreason은 EmptyRows/MultiRows/UnregisteredRows로분리한다. caller staticlabel `gemma3_process_inputs`, is_prompt, input_seqs.len, seq_indices.len,선택seqIDs,registry등록여부,각seqrawlen/promptlen,num_computed_tokens,phase를남긴다. 호출counter는유효/무효를포함해증가한다. 임의backtrace·prompt원문은필요없다. 범위를넘는행이나lookup실패는원본modelpolicy를바꾸지않고traceinvalid를유지한다.
3. forward_enter_missing_sequence에는앞rows진단callID를연결해같은원인의두검사를구분한다. 등록된case/request가없으면null로남기고case를가짜로배정하지않는다. rows길이검사시invalid무시/None조기return으로문제를숨기는수정은원인확인전금지다.
4. sticky failed는 atomicflag와별도로 first_failure{op,ErrorKind,raw_os_error,ordinal,mutation_revision}를 **이벤트journal mutex밖**의한정상태에보존한다. sinklock/journallock/seek/write/set_len/sync_all을분리한다. flush_attempts/write_successes/write_failures와snapshotordinal/revision을보존한다. 실패flag를매flush+1하는현재계수는관측한대로명시한다. 이후semantic변경이필요하면invalid_occurrences와failed_flush_observations를별도schema로분리하고migration/회귀를선보존한다. r1에소급적용하지않는다.
5. printstderr나현재응답payload를바꾸지않는다. 저장불능이면대체성공파일을조용히생성하지않고runner가실패flag/incomplete를확인할명시적receipt계약을추가한다. journalmutex poison시에도reason보존과overflow가bounded이어야한다. 실패가model/seed/stop호출순서를바꾸지않게한다.

```rust
// Document-only diagnostic shape; not an implemented API.
enum InvalidReason {
    EmptyRows,
    MultipleRows,
    UnregisteredRow,
    ForwardSequenceMissing,
    AdmissionMismatch,
    DuplicateBinding,
    StickyRecorderFailure,
    EventSerialization,
}
// Preserve the observed counter, add a bounded reason record and revision.
// Do not set invalid=0 or reinterpret the r1 journal as complete.
```

## 회귀와root재실행 조건

모델없는Rust회귀를먼저추가한다. 정상singleboundrow0invalid,emptyrow1·후속forward추가0,multirow1,미등록single row→forward2를현재정책으로재현하고reason이둘의공통origin을연결해야한다. 같은case중복begin/bind/admissionmismatch/renderlimit/CPUdtype읽기실패/positions실패각각reason을확인한다. countprocess와idleempty scheduler가rows를부르지않는실제callcount계약을검증한다. mapper/processor직접준비가필요한경로는phase를명시한fixture로분리한다.

fake Sink에서seek/write/set_len/sync_all각실패와mutexpoison을주입한다. 첫실패한번후여러flush가현재invalid를증가시키는형태를재현하고first_failure는유실되지않아야한다. 동일ordinal/다른revision·동시flushsnapshot순서·runnerread중write·parse불능·최종flush실패는명시incomplete다. 실제Windows공유회귀와pathconfinement를유지한다. 실패주입으로모델실행은필요없다.

root는새reason진단source/EXE·compilerinputs·diff/whitelist(신규파일포함)·DTO검증기·fmt/tests/strictclippy/lockedbuild를선보존한뒤**별도r2label**에서고정동일3payload/count3/2048cap을재실행한다. r1 d56e엔진/원시journal/manifest/실행측정은그대로보존한다. 새실행에서normalrequest3/render3/admission3/closed3·bind1·모든rows호출·reason과flushreceipt를완결회계로대조한다. reason원인에따른최소수정은그뒤별도patch로검증한다. 지금바로invalidcounter제거·emptyrow예외·0보정은허용하지않는다.

r2reason이실제로빈prep/미등록row를보이면어떤caller/phase가정상인지정책을검토하고그범위만명시적으로구분한다. 실modelforward를미등록상태로흘렸다면허용으로바꾸지말고identity전달을고친다. storage오류면실제errno/op와atomicrewrite/공유·flush완료조건을고치고counter통계는원인진단과분리한다. 일반API구조가맞아도recording완결·공통contract·비간섭은별도다. 성능비교부적격·품질승인0·A완전off rawexact미검증을유지한다.

이번조사결론은 **정상cohort의명시적정합성과invalid발생경로의정보부족을함께확인했고,rows/forward이중계수와stickyflush계수가둘다invalid2를만들수있다는것까지좁혔다**는것이다. 정확한원인은새reason관측전미확정이다. 이번변경은본연구문서와coordination뿐이다.
