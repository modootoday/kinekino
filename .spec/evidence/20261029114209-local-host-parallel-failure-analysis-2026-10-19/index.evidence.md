---
id: evidence:kinekino/20261029114209
title: local_host 병렬 테스트 두 실패 source 분석
status: active
created: 2026-10-29
updated: 2026-10-29
---
# local_host 병렬 테스트 두 실패 source 분석

2026-10-19. 보존 로그와 `tests/local_host.rs`, `src/infrastructure/local_host/`를 읽은 독립 조사다. 관련 exit-code/transport-envelope 계약만 함께 대조했다. **테스트·빌드·제품 실행·모델 추론을 새로 수행하지 않았고 코드·원시 증거를 수정하지 않았다.** SDK source 안정 상태와 root의 다른 진행 작업을 변경하지 않는다.

## 관측과 판정 범위

`runtime/quality-review/model-review-text-full-test-20261019.log`의 local_host integration binary는9건 중7통과/2실패,4.54초다.

| 실패 | 실제 보존 내용 | 해석 한계 |
|---|---|---|
| real_cli_disconnect_during_execution_is_unknown_and_never_falls_back | tests/local_host.rs:283 `rx.recv_timeout(2초).unwrap()`의 Timeout | **테스트 시작 신호** 미수신. transport timeout 응답을 받았다는 뜻이 아님 |
| malformed_oversized_and_truncated_frames_do_not_dispatch_and_host_recovers |181행 CLI code6, 기대0 | code6은PermissionDenied. 단순150ms timeout이면code10이며,둘을동일시하면안됨 |

root는 같은 binary 단독순차9/9/2.39초를 전달했다. 읽은 전체순차 로그 `model-review-text-full-test-serial-20261019.log`에는 local_host9/9/5.28초, 전체합132pass/9ignore가 있다. 전달받은2.39초와 이 로그의5.28초는 다른 실행으로 구분한다. 순차통과는 병렬실패 해결이나 production 신뢰성 승인으로 승격하지 않는다. source변경 없이 병렬잔여가 남는다.

06:00 D:HDD queue6/read1.37MB/s/write4.03MB/s, C:NVMe queue0은 root가 전달한 가능한 동시 자원조건이다. 실패시점별 process/thread/Win32/I/O timeline과 연결되지 않아 원인확정 근거가 아니다. 이 조사에서 디스크·프로세스 지표를 새로 측정하지 않았다.

## 독립 fixture와 공유 여부

각 test는 자기 Arc counters/SDK/LocalHost/runtime/channel을 만든다. `security::endpoint()`는 stable product ID+현재PID+process creation time+BCrypt128bit nonce로 이름을 만든다. 같은 integration process의PID/creation time은 공유하지만 nonce는 호출마다 새로다. 고정 pipe endpoint·공통tempfile·env/current_dir 변경·공용Atomic fixture는 읽은 범위에서 없다. 따라서 **동일 pipe 이름의 결정적 충돌**이나 SDK counter 공유가 source상 드러나지는 않았다. 실제 생성 endpoint 충돌/다른 Win32 identity 문제까지 측정 없이 배제하지 않는다.

공유되는 것은 같은player-cli.exe 파일·동일OS/user/process 자원·test harness의병렬CPU/thread/disk 작업이다. 각LocalHost는 I/O thread와SDK executor thread, 각EndpointClient 호출은 별도private runtime thread, invoke는childprocess와대기thread를만든다. 많은 schema CLI를순차spawn하는발견테스트도 다른local_host테스트와동시진행한다. 이러한부하가대기시간을늘릴가능성은있지만로그에는phase별시간이없다.

peer_token의impersonation은host I/O thread에서 동기query 후 RevertToSelf하며 await/SDK를그사이에두지않는다. LocalHost::start/EndpointClient는 호출thread의impersonation을검사한다. 프로세스전체token변경을원인으로주장할근거는없다. 실제restoration실패나threadtoken오류는별도stage 진단이필요하고 보안검사를생략하는해결은허용하지않는다.

## 실제 host·client·executor 수명과 시간축

`server.rs::LocalHost::start`는private current-thread Tokio runtime/첫pipe/Executor를만든뒤 ready채널을보낸다. ready는 **첫pipe 생성·executor thread spawn 성공**이며 CLI executable 시작·client handshake 완료·accept loop 실행ready까지 증명하지않는다. start의ready.recv에는별도timeout이없다. Drop은 stopped=true→cancel oneshot→I/O thread join이며 이미시작된동기SDK call은취소하거나join하지않는다.

acceptloop는한연결씩처리한다. `verify_client`는연결뒤동기검사이며 `timeout(options.timeout, serve(...))` 바깥이다. serve의**한세션deadline**은 length/body읽기→message보안검사→SDK대기→response직렬화/쓰기→ACK읽기를포함한다. ACK까지완료또는timeout/오류후create_next를먼저만들고기존pipe를cancel/drop한다. create_next실패는loop종료이며Host객체가살아있어도실제acceptthread가종료될수있다. 상세오류는현재외부에보존하지않는다.

client.rs는request검증/인코딩·private thread/runtime생성뒤 **그runtime에서** transport timeout을시작한다. 기본5000ms이고해당async구간은pipeopen/busy231재시도→server PID/start/security확인→write/read/decode→ACKtry_write다. require_unimpersonated·thread생성·runtime생성·CLIprocess시작·stdin입력읽기는이timer보다앞이다. 서버deadline과시작점·범위가다르다.

`transmitted=true`는첫framewrite직전에설정한다. 부분write도race가능하므로실행불확실성을보수적으로유지하고자동재시도하지않는다. 성공response뒤ACK는try_write이며실패를무시한다. malformed fixture는write_u8(6).await를한다. ACK나대기순서경쟁가능성을진단하되현재실패의확정원인으로삼지않는다.

Executor는busy CAS와크기1job channel로singlecall을실행하며timeout때queue replay하지않는다. SDK thread는실행뒤busy=false,reply.send다. 서버timeout은이 SDK실행을중단하지않는다. 이정책은응답실패후자동fallback/replay를금지하는계약과맞으며이번조사에서바꾸지않는다.

## line283 시작 신호 Timeout의 후보

테스트순서는hoststart→runner thread spawn→runner내CLI spawn/stdinwrite/wait_with_output→부모rx.recv_timeout2초다. InFlight SDK가실제로호출되면calls++→started.send→200ms sleep→SDK응답이다. 따라서실패는 “**SDK진입신호를2초안에못받음**”만뜻한다.

가능한구간은 runner thread scheduling/childprocessload·CLIinput파싱/EndpointClientthread시작/pipeconnect·identity검사/hostaccept·executor scheduling이다. CLI가초기에code4/6등으로반환해도invoke결과는runner안에만있고부모는started채널만기다려같은Timeout으로보일수있다. counter·childexit·진행stage를실패메시지에남기지않아production실패와fixture startup지연을구분못한다.

2초는테스트wait이고client기본5초보다짧으며CLIstartup을포함한다. 늦은동작이기존정책내에있어도테스트가먼저실패할수있다. 그렇다고2초를늘리거나5초로맞춰해결했다고하지않는다. 해당phase를관측하고유효한disconnect중실행상태를명시동기화해야한다.

200ms sleep은“호스트를drop할때실행중”을보장하지않는다. started신호뒤부모가늦게schedule되면SDK가먼저끝날수있다. test의intendedrace를만들기위해sleep보다실행중barrier가필요하다. 현재실패는started미수신이어서이후race까지검증한것이아니다.

panic시runner JoinHandle은drop되어detach된다. invoke안Child도RAIIkill/join보장이없고executor SDKthread도이미진입했으면hostdrop뒤완료할수있다. receiver가drop된뒤늦은SDKstarted.send().unwrap()는panic하고Executor catch_unwind가Internal로변환할수있다. 이후관측이없으므로실제발생했다고단정하지않지만, 실패fixture가다른병렬test에후행작업을남길수있는구조는고쳐야한다.

## line181 code6의 후보와 production 영향

`invoke`는CLIstdout JSON을파싱했지만호출자는`let(code,_)`로실패응답을버린다. code6은ErrorCode::PermissionDenied다. malformed일련전송에서는calls0을확인한뒤동일host에서새version을호출하며hosttimeout은150ms다.

client에서PermissionDenied는 require_unimpersonated,pipeopen권한오류,security::verify 오류에서나올수있다. verify는GetNamedPipeServerProcessId/PID일치/OpenProcess/starttime/현재및servertoken/permits_peer 전체오류를PermissionDenied로매핑한다. **실제권한불일치뿐아니라연결수명에따른Win32조회실패도이코드로묶일수있다.** 이경로는transmitted전이다. 현재stdout응답이없어어느단계인지모른다.

가설중하나는connection후150ms세션이client handshake/frame도착전에끝나기존pipe가cancel되는것이다. client가늦게serveridentity를조회하면끊긴handle에대한조회실패가code6으로나타날수있다. source상가능하지만해당Win32오류·시각이없어확정하지않는다. 반대로아직accept/verify_client가늦어진단계는서버timeout이시작하지않았을수있으므로모든지연이150ms에포함된다는설명도잘못이다.

이test는잘못된length·truncatedbody·malformedJSON·ACK뒤새연결복구를한fixture에묶었다. 마지막ACK와create_next전환을명시적으로기다리지않고childspawn을시작한다. pipebusy231는client재시도로처리하지만securityverify/peercheck오류는재시도하지않는다. stalehandle/name전환·보안오류/세션expiry를분리한회귀가필요하다. 보안실패를무시하거나완화하는것은금지다.

production에도짧은customdeadline/부하/연결교체중동일수명race가있을수있다. 기본5초에서실제실패가재현됐다는증거는없다. code6이연결상실과진짜accessdenied를구분하지못할가능성은사용자오류분류영향이다. class를바꾸려면Win32stage/cause와transmission단계별정책을먼저고정해야한다. 요청전문·SID/token값·endpoint를오류로그에노출하지않는다.

## 최소 진단과 원인별 수정안

### 우선 test receipt·stage 관측

- invoke의childPID/start/stdinclose/exit·stdoutJSON·stderr존재·phase별monotonic elapsed를구조체로반환하고실패assertion에비밀없는error code/delivery/retryable·host.is_running/calls를붙인다. raw endpoint는hash/fixtureID로대체한다. stdin EOF는take한handle의scope종료로명시하고child수명guard를둔다.
- disconnect test는started채널과runner결과채널을**함께관측**한다. earlyCLIexit를2초Timeout으로숨기지않고어느event가먼저왔는지분류한다. 잠금·채널wait중clientstdout을읽지않아deadlock되지않도록소유thread가wait_with_output을끝내고bounded결과를보낸다.
- host진단은acceptok/verify_client/framelen/bodyread/verify_message/dispatch/responsewrite/ACK/create_next/stop 이유에정적stage·monotonictime·Win32ErrorKind/rawcode를붙인다. client도open/verify/write/read/decode를분리한다. diagnostics는opt-in소유fixture채널/파일이며일반protocol·SDKpolicy와분리한다. 원문·securitysecret는보존하지않는다.
- deadline은기존150ms/기본5초/fixturewatchdog를유지해각각어떤timer가끝났는지기록한다. 첫관측단계에서임의연장·성공retry·직렬화만으로통과시키지않는다.

### fixture 수명·동기화 수정 제안

1. InFlight는started뒤explicitrelease gate로막고부모가started를확인한뒤hostdrop→CLI결과확인→release→executorfinished확인순서를고정한다. 기존200/250ms sleep을정확한실행중/완료barrier로대체한다. safetywatchdog는실패경로cleanup을위해유지하며성공조건을시간확장으로만들지않는다. SDKcall중단/rollback을보장한다고하지않는다.
2. 모든return/panic경로에서소유childkill/wait·runnerjoin·gate release·executor완료관측을RAII로보장한다. 사용자process나타testchild를종료하지않는다. 다른app/VM/DPI/서비스조작은필요없다. panic을조용히무시하지않고먼저실패receipt를보존한다.
3. malformed/oversized/truncated가dispatch0인contracttest와deadlineelapsed·ACK/sessionrotation복구test를책임별로분리한다. 완전잘못된frame의즉시거부는deadline으로성공시키지않는다. idlepartialtimeout은partialhandle을유지하고timeoutstage관측으로검증한다. recovery는명시적인sessionclosed/nextinstancecreated event뒤같은실CLI를사용한다. 이는protocol수명barrier이며deadline증가가아니다.

### source 수정은 원인 확인 뒤

- stale/disconnectedhandle identityquery 실패가확인되면security조건은그대로유지하고운송오류분류를Win32cause와transmitted전후에맞춰분리한다. 진짜PID/start/token불일치는계속PermissionDenied다. 보안query를재시도해다른peer에붙거나실행unknown요청을다시보내지않는다.
- create_next/ACK전환문제가확인되면한이름소유handle을유지하는현재조건과취소순서를보존해수정한다. accept실패를무조건성공으로바꾸지않고host의terminalreason/health를관측한다.
- 일시적부하로CLIload전2초만넘은것이면fixturephase/cleanup결함을고치고production5초내실제transport를별도로측정한다. 테스트전체직렬화는원인분리대조로쓸수있지만병렬동작보장대체물은아니다.

## 결정적 Rust 회귀와 실제 후속 게이트

모델없이privateRust test port/transport/securitystage seam에서조건을만든다. 예를들어child가SDK진입전에code6으로끝나는경우와단순startup지연을각각주입해별도receipt를확인한다. 실행중releasebarrier는hostdrop시responseunknown/retryablefalse/calls1을결정적으로검증한다. receiver취소/started중복/workerpanic도자원정리를확인한다.

namedpipe회귀는clientopen뒤identity검사직전에barrier를두고server세션expiry/취소를일으켜Win32cause·dispatch0·전송상태를확인한다. 정상identity거부fixture와별도로실행해진짜securitydenied를transport끊김으로잘못분류하지않는다. ACK전후/partialframe/length0/overlimit/malformed 각각에서oldpipe/nextinstance이벤트와새client호출이상호간섭하지않는지검증한다. timer동작은가능하면Rust가상clock/explicitgate를쓰고새타이밍정책을도입하지않는다. 실제Win32에서는ownedbarrier/receipt로뒷받침한다.

다음실행은root가새source/EXE/fixture입력·소유receipt를선보존한뒤수행한다. 동일9test의병렬원래조건과순차대조를모두남기고finite반복횟수/실행순서/자원조건을미리고정한다. 반복통과율만으로원인을확정하지않는다. 실패가한번이라도나오면stage/JSON/childstatus/host수명자료로원인별분류한다. 그뒤SDK전체fmt/test/strictclippy와실제CLI경로를재검증하며root/UI기능품질과별도판정한다.

## 근거 식별자와 현재 결론

| 읽은 파일 | SHA256 |
|---|---|
| tests/local_host.rs |a028ab4d74faa072d9800d8c23b05110e2ea0c08d443f0132d4752088caf2e90|
| local_host/server.rs |231b469be50383d594e84cbbe0e2a3c372bbbf3c4234fd3586273141480b5522|
| local_host/client.rs |250e1a00d669b60a93e3f085072c7649a688b68cad3eef3dca9bb9451ab7f6ca|
| local_host/executor.rs |40f7853689200c173b713993f1b517c42408e4403946d4b38a11596fb3ccc8e9|
| parallel log |b1862eb4779138d64c172f86b52430b4637791a4b688e3ca5a2e93703db22fab|
| full serial log |2abedb2633d45483354587fcd90192735b2058e0a6baee0dc7ed81fd22f980979|

현재 관측에는 phase 정보가 부족하다. 2초 started 미수신은 CLI startup/early exit/dispatch 대기를 구분하지 못하며 code6은 PermissionDenied로 timeout code10과 다르다. 직접 공유 fixture 충돌의 source 근거는 없고, 짧은 session과 security query의 수명 race·진단 누락·실패 cleanup이 구체적인 조사 지점이다. 정확한 원인은 미확정이며 위 수정안도 미구현·미검증이다. 병렬 실패는 잔여로 유지한다. 연구 세션은 지정 문서와 coordination만 변경했다.
