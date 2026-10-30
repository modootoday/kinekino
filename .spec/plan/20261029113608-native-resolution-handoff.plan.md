---
id: plan:kinekino/20261029113608
title: 검증된 회차 결과의 명시 native 재생 연결
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 검증된 회차 결과의 명시 native 재생 연결

검증된 dispatcher 이후 NativeChild/CompletionSignal의 최소 조립 범위는 [child 조립 후속안](native-playback-child-assembly.md)에서 별도로 관리한다.

## P3 parent broker 소스 초안

### 후속 std join dispatcher — 제한 검증 완료, 실제 조립 잔여

P3 검증 커밋 `a50905e` 이후 별도 revision으로 std Rust host-owned dispatcher를 작성했다. Windows pool은 사용하지 않는다. 신규 `infrastructure/thread_completion/{mod,completion,dispatcher,tests}.rs`, 최소 infrastructure 등록, 기존 parent_binding/cleanup_task/직접 tests의 정확8파일이다. 입력 목록·핀은 `runtime/coordination/sdk-git-baseline-r1/parent-dispatcher8.paths`와 인접 pins에 기록했다. UI11 인수 후 명시8 overlay의 집중4+6·lib/bins strict·fmt를 확인했고 [검증 정본](../../sots/verification/application/native-join-dispatcher-2026-10-20.md)에 base/보호 핀과 범위를 기록했다. 기존 P3 fake5 검증을 새 revision의 검증으로 소급하지 않는다.

Host의 단일 dispatcher thread와 bounded sync_channel이 작업을 소유한다. 제출 거부는 아직 시작하지 않은 원 closure/ticket을 돌려준다. 수락된 작업의 child shutdown thread를 실제 join한 뒤 typed result cell을 게시하고 그 다음 주입된 wake callback을 실행한다. callback 반환은 GUI 소비 확인이 아니다. notification panic·오류 및 dispatcher fault는 별도 상태로 기록하며 이미 join된 child 결과를 제거하지 않는다. Host admission close는 수락된 작업을 drain하고 host 자신의 thread 종료·join은 별도로 관측한다. child별 supervisor와 GUI timer polling은 추가하지 않았다.

ParentCleanupTask는 dispatcher handle과 scoped notification을 명시 주입받는다. UI observe는 결과 cell만 읽고 join/wait를 하지 않는다. worker의 cleanup_wait는 동일 predicate mutex와 Condvar로 inflight zero를 기다리며 EngineAdmission의 zero 전환이 통지한다. revoke 뒤 새 admission은 계속 거부한다. 이 worker 대기는 기존 엔진 owner를 바꾸거나 새 SDK를 만들지 않는다.

집중 회귀는 기존 one-shot/epoch/owner4개에 async inflight 종료 전후의 wake 유실 방지·거부된 cleanup 소유2개, dispatcher의 실제 TLS 종료 지연→join→wake·bounded 거부/close drain·notification panic·join 후 dispatcher fault4개로 총10개를 확인했다. 남은 단계는 실제 NativeChild CompletionSignal closure와 !Send lease·OS parent/main 조립이며 이 gate에 포함하지 않았다.

최종8파일 r2핀은 `A8E65EDE052699A142EFA2EE68C640D27A0FA475D2A97563D404DACD63A6D31A`이며 `parent-dispatcher8-r2-pins.json`과 `parent-dispatcher8-r2-source/`에 구분해 보존한다. r1 `B890EF39F3EE6499DDE685FD22A7A6690441BE23E16F98F53B8C74DF743294AF`의 원 소스 사본과 원 pin JSON은 보존하지 못했으며 기록 hash와 tool 변경 이력만 남았다. r2는 tests.rs의 거부된 closure에 유일한 Arc owner를 캡처하고 Full/Closed 재제출/명시 Drop에서 weak reference로 소유 유지·해제를 확인하는 보강이다. 다른 제품 동작·시험 수는 바꾸지 않았다. r2만 이번 제한 gate를 통과했고 r1은 미검증이다.

다음 snapshot 전환은 현재 UI11 gate의 명시 종료·반환 뒤에만 한다. 원 UI source/receipt/input pin을 보존하고, 대상8경로의 존재 여부·before hash와 기존 파일 사본을 기록한 뒤 exact8만 byte copy한다. before/final pin을 별도 기록하고 UI11 및 미변경 P3 소스의 보호 핀을 대조한다. 현재 public-p3-parent-r1의 archive와 원 검증 입력은 재작성하지 않는다. 줄바꿈·BOM 불일치 조사와 SDK overlay를 임의로 합치지 않는다.

집중 실행 후보는 root 기본 feature에서 `cargo test --offline --locked -j1 -p kinekino-player --lib infrastructure::thread_completion` 및 같은 명령의 `infrastructure::mpv::parent_binding` 필터다. 신규 cfg(test) 코드의 경고까지 검사하도록 root package의 all-target strict 범위를 중앙과 확정하고 fmt를 수행한다. native/bridge 전체 회귀·실OS·mpv·AI는 추가하지 않는다. queue rejection은 아직 시작하지 않은 원 요청, child join 결과, notification 결과, host dispatcher 종료를 각각 구분해 인계한다.

2026-10-20 PM 배정으로 `src/infrastructure/mpv/parent_binding.rs`, `deferred_engine.rs`, `parent_binding/tests.rs` 및 최소 `mpv/mod.rs` 등록을 작성했다. 후속 worker 종료 반례 검토에서 `parent_binding/cleanup_task.rs`를 추가해 정확5파일 입력으로 분리했다. 별도 공개 snapshot의 집중 fake5·lib/bins strict·fmt를 확인했으며 최초 strict 실패와 최소 Box 수리는 [집중 검증 정본](../../sots/verification/application/native-parent-broker-2026-10-20.md)에 기록했다. 기존 실제 native/CLI 관측을 새 실제 parent 연결 성공으로 사용하지 않는다.

ParentBinding은 OS handle이나 !Send renderer lease 없이 한 epoch의 엔진 Arc를 한 번 결합한다. 준비 전 호출은 NeedsInput, 다른 epoch는 RevisionConflict, revoke 후 새 호출은 Cancelled다. 거절된 bind는 제안된 엔진 소유자를 오류와 함께 반환한다. DeferredEpisodeEngine은 같은 configured SDK에 미리 주입할 수 있는 기존 EpisodePlaybackEngine forwarding 포트이며, 새 SDK를 만들거나 CLI의 None standalone 경로를 바꾸지 않는다. main과 NativeChild 조립에는 연결하지 않았다.

revoke_admission은 UI에서 atomic 차단만 하며 동기 shutdown·SDK 재호출·self wait를 하지 않는다. 이미 admit된 호출은 원 엔진으로 끝날 수 있고 그 동안 cleanup은 Pending이다. worker의 cleanup은 기존 엔진 shutdown 포트를 호출한다. 오류와 불완전한 종료 receipt에서는 같은 owner를 유지하고, engine·IPC·stdout·stderr 종료가 확인되기 전에는 Complete를 만들지 않는다. 이 admission 상태는 실제 OS parent의 유효성·handle 재사용 방지·geometry/focus를 입증하지 않는다. UI !Send lease와 NativeChild의 parent 파괴·inflight launch 수명 조율은 실제 연결의 별도 선행 조건이다.

UI 관측용 ParentCleanupTask는 동일 binding/epoch와 cleanup JoinHandle을 보존한다. start_attempt는 worker를 만들고 observe는 is_finished 전에는 join하지 않는다. Finished는 worker를 join한 뒤의 결과이며 그 안의 Pending/Incomplete/Err는 엔진 정리 완료가 아니다. 동일 task가 원 binding으로 retry하며 Complete 후 재시작을 거부한다. assembly는 task를 끝까지 보존해야 하며 Drop을 완료 증거로 사용하지 않는다.

원 Meari 공개 NativeChild.begin_cleanup은 stamp에 묶인 CompletionSignal을 제공하며 notify는 active/dirty와 lifecycle waker를 사용한다. 기존 signal을 통한 후속 연결 통로는 있으나, shutdown worker가 반환 직전 notify하면 UI가 is_finished=false를 본 뒤 최종 wake를 잃을 수 있다. 실제 종료·join 확인 뒤의 최종 signal 조율은 미연결이다. 새 supervisor를 만들더라도 그 notifier thread가 아직 실행 중인 사실을 전체 worker 종료로 표현하지 않는다.

후속 구현 제안은 child별 supervisor Rust thread를 추가하는 대신 host 수명에 소유된 join-completion dispatcher를 명시 주입하는 것이다. dispatcher가 기존 shutdown JoinHandle을 인수해 실제 join한 뒤 결과를 유지하고 그 다음 CompletionSignal을 알린다. UI 관측은 준비 신호가 아니라 joined 결과를 읽는다. 실패한 dispatch는 원 JoinHandle/owner를 반환해야 한다. dispatcher의 OS/runtime worker는 child shutdown worker와 다른 수명이며, child 정리 결과를 전체 runtime 종료로 부르지 않는다. 알림 context에는 raw parent·!Send lease·engine owner를 남기지 않고 결과와 scoped signal만 둔다. host 종료는 dispatcher의 남은 delivery를 별도로 회수해야 한다. 아직 해당 dispatcher를 구현하거나 선택하지 않았다.

제안 소스 경계는 `infrastructure/thread_completion/`의 소유 task/dispatcher 계약과 Windows adapter, 기존 `mpv/parent_binding/cleanup_task.rs`의 명시 dispatcher 주입, 향후 `assembly/meari/playback_child.rs`의 NativeChild/CompletionSignal 조립이다. application DTO·CLI wire·새 SDK는 추가하지 않는다. 우선 fake dispatcher로 early-ready 후 실제 worker 종료 지연, 제출 실패의 handle 반환, stale signal, 동일 attempt 재관측, runtime delivery 잔여를 별도로 검사한다. Windows threadpool 후보는 [QueueUserWorkItem](https://learn.microsoft.com/en-us/windows/win32/api/threadpoollegacyapiset/nf-threadpoollegacyapiset-queueuserworkitem)의 runtime-owned callback이며 전용 child thread의 join을 확인한 뒤 알리는 역할만 맡는다. 이 후보의 callback 자체가 반환하기 전에 전체 runtime quiescence를 주장하지 않는다.

thread object wait 후보는 [RegisterWaitForSingleObject](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-registerwaitforsingleobject)로 종료된 thread를 관측할 수 있지만 one-shot도 unregister가 필요하다. [UnregisterWaitEx](https://learn.microsoft.com/en-us/windows/win32/api/threadpoollegacyapiset/nf-threadpoollegacyapiset-unregisterwaitex)의 비동기 unregister에는 pending callback이 남을 수 있어, wait context와 handle의 수명을 별도로 보존해야 한다. UI blocking unregister/self-wait나 단순 Drop으로 완료 처리하지 않는다. 이 정리 부담 때문에 첫 구현 전에 dispatcher 수명과 완료 범위를 먼저 인수하며, 현재 P3 커밋의 실제 wake 지원으로 표현하지 않는다.

fake 회귀5개는 준비 전/epoch/중복 결합, 같은 엔진의 media close/reopen과 parent 영구 revoke 구분, slot 소유 중 빠른 revoke와 late bind, clone의 inflight open 및 late control 차단·실패/불완전 cleanup의 동일 owner retry, 결과를 준비해도 worker가 아직 반환하지 않았으면 Pending을 유지하는 반례를 확인했다. 최초4·추가5·최소수리r2 핀은 `runtime/coordination/sdk-git-baseline-r1/parent-broker*-pins.json`에 구분한다. 남은 핵심은 caller-driven observe를 NativeChild의 실제 최종 completion/wake와 연결하는 것이다. 결과 전달 wake만으로 worker 종료를 추정하거나 정기 GUI polling을 가정하지 않는다. 이 단계는 OS/Mpv/미디어 통합 인수가 아니다.

2026-10-20 P3 읽기 전용 준비: 아래 r3는 초기 standalone native 인수이며 이후 [옵션 CLI](../../sots/verification/application/native-options-cli-2026-10-19.md)·[회차 전환 CLI](../../sots/verification/application/native-transition-cli-2026-10-19.md)·[포인터](../../sots/verification/application/native-pointer-cli-2026-10-19.md)·[오디오/WebView](../../sots/verification/application/native-audio-webview-2026-10-20.md)의 별도 관측이 있다. 해당 checkpoint의 성공/원실패/미검증 범위를 유지한다. 새 Meari TSX video child 통합은 아직 없으며 하단 공개 포트 대조와 선행 조건을 따른다. 이번 준비에서 source 수정·빌드·미디어 재생·다운로드는 하지 않았다. Meari가 단일 heavy를 소유한다.

초기 [r3 실제 인수](../../sots/verification/application/native-episode-playback-2026-10-19.md)는 전체295·18ignored/strict/bins와 actual1pass44.39초다. 새 resolve→native 프레임·시간 진행·네 CLI 제어→close/reopen/stale ID/EOF를 통과했다. 일반 close와 host 영구 종료를 구분했다. 당시 다음은 rate/tracks·회차·GUI 조립과 나머지 전체 기능이었다. 아래 단계별 초기 준비 기록은 역사적 구간이다.

기준은 [manifest-r3 실제 인수](../../sots/verification/application/player-manifest-wire-cli-2026-10-19.md)다. 저장된 master 구조의 유효성과 현재 URL·참조 미디어·decode는 별개다. 전체 SDK/CLI·회차 전환·자막·제작·AI 범위를 유지한다.

1. application의 재생 수명 계약을 분리한다. open은 결과 artifact ID만 받으며 공통 결과 reader로 원문을 재검증한다. 원 URL·셸·mpv 옵션을 외부 payload로 받지 않는다. 프로세스 시작과 파일 로드·decode 관측을 구별한다.
2. 명시적으로 선택한 trusted mpv 경로의 Rust 어댑터를 만든다. absent/empty/URL Referer를 임의 보충하지 않는다. 현재 legacy GUI 어댑터의 `referer.is_some()`과 network/local 구분을 분리한다. HTTPS 전송·TLS 검증·스크립트/설정 차단을 명시하며 엔진 옵션 실제 지원을 확인한다.
3. 한 host가 한 native 인스턴스를 소유한다. close/control은 실제 instance ID를 검사하고 늦은 요청·관측을 새 인스턴스에 적용하지 않는다. EOF 뒤 자식과 IPC 작업자 종료를 확인한다. GUI와 CLI는 같은 application 계약을 쓴다.
4. 공통 SDK·schema·capabilities·typed client와 `--serve-native` 조립을 연결한다. transport ready·process started·media loaded·시간 진행·명령 접수/적용을 각각 반환한다. URL·토큰·원본 헤더는 공개 출력에서 제외한다.
5. 속도·트랙·previous/next는 같은 소유 수명과 회차 목록의 순서를 사용한다. next를 임의 URL 산술로 구성하지 않는다. 원 회차 실패·교체 실패·늦은 제어·원본 보존을 검증한다.
6. 실행 슬롯 배정 후 fmt/전체 tests/strict Clippy/두 bin을 고정하고 실제 회차를 새 CLI로 resolve→native open→독립 관측→제어→close/EOF까지 검사한다. master 참조 playlist·segment 수신/디코드·실제 시간 진행이 없으면 playback 성공으로 표시하지 않는다. 실패 원문과 원래 deadline을 보존한다.

Meari 반환 뒤 SDK native checkpoint 실행 슬롯을 배정받았다. [native-resolution-cli-r1](../../sots/verification/application/native-resolution-cli-2026-10-19.md)의 등록 source는 fmt0·workspace/all-targets strict Clippy20215ms0·전체293pass17ignored336902ms0·두bin944ms0이다. 고정 CLI의 미디어 없는 host 시작/EOF5443ms0를 확인했다. 실제 회차 native open·decode·제어·엔진 종료 인수는 아직 미수행이다.

`application/playback_session`, `infrastructure/mpv/{host,launch,network_plan,session_engine,trusted_executable}`와 SDK·typed client·CLI `--serve-native`를 등록했다. invalid 입력/변조 결과의 engine 호출 차단과 URL/헤더/실행 옵션 payload 거부 인수도 Rust 테스트로 준비했다. 실제 pin 대상 `runtime/mpv/mpv.exe` SHA256은 `f8bf7e8fd8d6696e35441c55091498060e4594f724a132fd6c46579341bc0790`로 다시 확인했다. 파일의 존재·해시 확인은 새 경로에서의 실행 증거가 아니다.

현재 adapter는 wire `Absent`에 referrer 옵션을 추가하지 않고 network whitelist는 독립적으로 유지한다. `Empty`는 mpv 전송의 동일성이 미검증이므로 Unsupported로 남긴다. URL은 기존 Referer 정책에 통과한 값만 별도 argv로 전달한다. stored manifest를 live capability나 decode 성공으로 승격하지 않는다. 소유 worker join·instance 제어 원자성·현재 엔진 오류·native host/SDK 등록은 코드에 반영했으나 실제 동작 인수는 남는다. IPC 관측은 큐를 계속 쌓는 대신 최신 스냅샷 하나를 보관한다. host 종료 gate는 진행 중 SDK 호출이 뒤늦게 새 엔진을 열지 못하게 한다.

미등록 실제 harness는 `runtime/coordination/native-resolution/actual_cli.pending.rs`다. 기존 강화 resolve 인수를 재사용하고 새 native fixture에서 진행·제어·인스턴스 교체 거부·close/EOF 증거를 보존하는 초안이다. 현재 고정 입력의 전체 검사 뒤 구간을 나눠 등록한다. 실제 프레임/디코드 증거 검사를 추가하기 전에는 실제 playback 완료 gate로 실행·승격하지 않는다. 크기·time-pos만으로 픽셀 디코드/출력 품질을 증명하지 않는다.

mpv 옵션은 [공식 manual](https://mpv.io/manual/stable/)에서 referrer/network timeout/load-scripts/load-unsafe-playlists 계약을 확인했다. 이 문서 확인은 로컬 고정 mpv 실행·전송·출력 보안의 실제 인수를 대신하지 않는다.

## P3 Meari NativeChild 연결 사전 대조 (읽기 전용)

SDK 제품 연결5의 [proposal Native 실행](../../sots/player/application/native-proposal-execution.md)은 실제 main 분기·폰트/CSS·proposal scope의 시작/close 증거다. 영상 child나 새 TSX 재생 제어 통합으로 승계하지 않는다. 기존 [legacy GUI 고정 실행](../../sots/verification/application/native-gui-pinned-execution-2026-10-19.md)도 Meari 부모/child의 overlap·입력·OS 수명 증거가 아니다. 아래는 현재 source를 읽은 계약 대조이며 Native316 이후 P1a가 구현 중이므로 P3 시작 전에 최종 공개 API·핀을 다시 인수한다. 후보 포트나 파일은 기존 구현으로 표기하지 않는다.

### 재사용할 실제 SDK·엔진 포트

| 책임 | 실제 파일/공개 계약 | P3에서 유지할 경계 |
| --- | --- | --- |
| 공통 회차 재생 | application/playback_session/mod.rs의 EpisodePlaybackEngine: Send+Sync, EpisodePlaybackSessions | GUI/CLI 모두 같은 usecase. NativeChild/HasWindowHandle을 application에 넣지 않음 |
| open 입력 | playback_session/types.rs의 OpenEpisodePlayback{resolution_artifact_id,paused,volume_percent} | 검증된 결과 reader→EpisodePlaybackSource의 비공개 생성 경로 유지. URL/Referer/mpv args/parent HWND를 외부 wire에 추가하지 않음 |
| 인스턴스 제어·종료 | ControlEpisodePlayback/CloseEpisodePlayback의 DecimalU64 engine_instance_id, OpenedEpisodePlayback/ClosedEpisodePlayback/EpisodePlaybackObservation | stale instance 원자 검사, started/accepted/unobserved와 실제 media/속성/exit 관측 구분 |
| 옵션·회차 | SetEpisodeRate, SelectEpisodeTrack(expected_tracks_revision), playback_session/transition::{EpisodeTransitions,TransitionSlot}, EpisodeReplacementOwner | 기존 track 목록 revision/전환 slot·durable resolve·명시 commit 및 CAS 정책 재사용. UI에 이전/다음 URL 계산을 복제하지 않음 |
| SDK 등록 | application/api/mod.rs::PlayerSdk::with_episode_playback/with_episode_transitions, api/client.rs 및 native_session.rs | 기존 playback.open/engine/close/set_rate/select_track·개별 playback control과 capture 라우팅 사용. Native context의 revision/idempotency/timeout 미지원 거부를 임의 삭제하지 않음 |
| 명시 엔진 host | infrastructure/mpv/host.rs::NativePlaybackHost::{start,engine,shutdown}, session_engine.rs::NativeEpisodeEngine | start는 trusted executable path/SHA·parent Option<u32>·diagnostics store를 받음. 현재 parent는 생성 시 고정이며 자동 GUI 검색/공유/재바인딩 없음 |
| 통합 host | infrastructure/playback_workspace.rs::PlaybackWorkspaceHost::{start,sessions,transition_service,jobs,catalog,shutdown} | provider/jobs/catalog/native/transition owner가 이미 있다. 별도 GUI용 default SDK나 중복 store/host를 만들지 않음. CLI playback_host.rs는 parent None으로 같은 조립을 사용 |
| 실제 프로세스 소유 | mpv::{owned_launch,owned_startup,session_engine,shutdown_observation}.rs | Child + IPC/stdout/stderr join 소유, 초기 실패의 Mpv recovery 보존, diagnostics 발행 한계 유지. control handle clone은 Child 소유권이 아님 |

`Mpv::shutdown`은 control permission을 닫고 Child.try_wait/kill/wait와 각 JoinHandle.join을 수행하는 동기 함수다. NativeEpisodeEngine.shutdown은 closed AtomicBool을 먼저 설정하고 active Mutex 아래 정리한다. 정상 playback.close는 현재 instance를 닫지만 host를 영구 종료하지 않고, host.shutdown은 기존 SDK clone의 늦은 open도 차단한다. Drop의 best effort를 Complete로 해석하지 않는다. 현재 공개 API에는 UI-thread에서 즉시 반환하는 begin/observe shutdown이나 독립된 nonblocking revoke-open 포트가 없다. NativeChild 콜백에 동기 shutdown을 그대로 넣어 event loop를 막지 않는다.

### 실제 Meari 부모·child 계약

`application/window_lifecycle/ports.rs::NativeChild`는 'static이며 Send를 요구하지 않는다. 메서드는 parent_lost, begin_cleanup(CleanupRequest,CompletionSignal), observe_cleanup(CleanupRequest)다. resize/focus/미디어 제어 메서드는 없다. begin의 Pending은 같은 정리 작업을 계속 소유한다는 뜻이고 signal은 변경 알림뿐이다. Err는 정리 시도가 terminal failure이며 정리 작업을 뒤에서 새로 시작/계속하는 뜻이 아니다. IO worker/잔여 process와 cleanup attempt의 완료를 분리해 실제 종료 proof를 검사해야 한다.

`ChildLease`는 Rc 권한을 가진 non-clone/non-serialize UI-thread 토큰이다. WindowStamp(owner/epoch/surface_generation)·CleanupStamp(child/revocation/attempt)는 metadata이며 raw HWND나 engine_instance_id의 대용 권한이 아니다. WindowLifecycle::{register,validate_access,release_child,suspend,begin_close,poll,retry_failed,force_loss}의 같은 coordinator를 사용한다.

실제 부모는 infrastructure/window/parent.rs의 NativeChildRegistrar::register(expected WindowStamp, create closure) 안에서만 NativeScopedParent: HasWindowHandle과 NativeParentLease를 얻는다. parent lease의 with_parent/with_cleanup_parent는 매번 epoch·정상/cleanup 상태를 검사하며 weak owner로 순환 소유를 피한다. raw handle을 한 번 정수로 복사해 이후 유효성이 보장된 것으로 쓰지 않는다.

같은 NativeStartupContext::prepare<C:NativeChild> → NativeWindowPreparation::{children,registrar} → ChildWindowHost<H,C>와 같은 runtime → attach_prepared 경로가 있다. 준비 자체는 OS parent를 만들거나 접근 권한을 주지 않는다. 실제 ready/등록 후 parent를 얻으며, 실패는 preparation/runtime/partial child를 원 상태로 반환한다. 현재 proposal controller의 고정 Host=ProposalWindowBridge 및 attach_observed를 그대로 media child controller라고 부르지 않는다. playback용 host/controller 조립은 후속 UI/assembly 소유다.

geometry는 NativeWindowObservation/NativeChildRegistrar의 geometry와 NativeWindowNoticeKind::GeometryChanged 및 HostEnvironment로 관측한다. 이는 바깥 window 관측이며 TSX의 video slot rect/clip/z-order/focus 정책 포트가 이미 있다는 뜻이 아니다. 현재 video_surface.rs의 VideoSurface::{new,id,layout}는 legacy eframe parent에서 STATIC render child를 만들고 rect·scale·visible 및 foreground focus 복원을 처리한다. NativeParentLease/slot/clipping 연결은 없다. `id()->u32`와 engine.parent Option<u32>는 HWND 전달 값이며 process PID가 아니다. P3에서 실제 borrowed handle의 폭/변환 계약을 대조하고 unchecked truncation을 새 어댑터에 복제하지 않는다. 현재 코드만 읽고 기존 ABI 결함이 확정됐다고 판단하지 않는다.

### 구현 전에 확정할 연결 공백

1. **parent 준비 순서와 단일 SDK**: NativeEpisodeEngine은 parent를 생성 시 고정하지만 Native parent lease는 실제 ready 이후에만 생긴다. 지금처럼 SDK를 먼저 구성하는 main에 raw parent의 늦은 setter나 새 SDK 교체를 넣지 않는다. trusted infrastructure의 명시 parent-capability broker/ready gate 후보를 먼저 확정한다. Send+Sync EpisodePlaybackEngine과 !Send parent lease는 원 HWND를 application DTO에 넣어 우회하지 않는다. parent 미준비/상실 중 open 거부·pending 요청 회수·늦은 spawn 정리의 원자 경계도 함께 정의한다.
2. **child-first 비동기 정리**: adapter가 NativePlaybackHost/engine과 VideoSurface·parent lease를 실제로 소유한다. parent_lost는 빠른 부모 권한 차단이며 SDK command를 호출하지 않는다. begin_cleanup에서 같은 owned cleanup 작업을 한 번 시작하고 pending 결과를 보존, 완료 뒤 CompletionSignal을 보낸다. observe_cleanup은 같은 attempt/receipt를 읽고 UI-thread surface 정리를 확정한다. 동기 mpv shutdown을 SDK execution worker에 다시 요청해 self-wait/queue 뒤 대기하지 않는다. 정확 nonblocking revoke/shutdown port는 아직 후보이며 구현 배정이 필요하다.
3. **수명 분리**: 미디어 instance close/reopen·회차 교체는 같은 유효 parent 아래 허용하지만 parent close/suspend/forced loss는 그 host의 새 open/늦은 IPC·geometry·focus를 거부한다. child process·세 worker 종료 및 surface 정리가 확인된 후에만 child barrier Complete다. force loss/부분 startup 실패·join 실패/timeout은 원 owner와 retryable evidence를 보존하며 OS/SDK 정상 종료로 승격하지 않는다. Native child 먼저, 그 뒤 business host/SDK worker, 마지막 실제 parent Destroyed 순서를 적용한다.
4. **실제 video slot과 입력**: P1 공통 frame의 slot rect/scale/clip/visibility/focus 최종 포트가 선행한다. HWND 영상과 Native softbuffer overlay의 z-order/겹침은 CSS만으로 해결된 것으로 보지 않는다. dialog/menu/subtitle/transport overlay·키보드 focus·pointer owner의 계약을 UI/Meari와 정한다. 임의로 영상/메뉴를 숨기거나 별도 떠 있는 mpv 창으로 바꿔 디자인 계약을 축소하지 않는다. 기존 pointer_policy의 engine cursor visibility 및 audio handoff coordinator도 재사용하며 새로운 mpv 기본값에 위임하지 않는다.

### 후보 파일과 배정 경계

| 담당 | 기존 재사용/후속 변경 후보 | 선행 인수 |
| --- | --- | --- |
| SDK application | playback_session/{mod,types,options,transition}, api/{native_session,client,schemas} | 현재 DTO/정책 유지가 기본. 새 공통 수명 포트가 필요한 경우만 별도 계약; UI/OS 타입 금지 |
| SDK infrastructure | mpv/{host,session_engine,owned_launch,owned_startup,shutdown_observation}, video_surface; 신규 meari/playback_child.rs 및 parent/cleanup broker 책임별 후보 | nonblocking revoke·owned pending shutdown·원 Child/worker receipt·parent 폭/guard 계약. 아직 신규 파일 없음 |
| Meari | window_lifecycle/NativeChild·parent lease·prepared attach·frame slot/focus/clip 공개 포트 | P1a 최종 API/소스 핀 뒤 소비. 제품 mpv/URL/SDK 정책을 라이브러리로 이동하지 않음 |
| UI | 공통 player frame의 video slot/controller와 얇은 typed playback 입력·상태 표시 | Item proposal startup과 media child startup을 구별. geometry/overlap/focus를 임의 화면별 정책으로 복제하지 않음 |
| SDK assembly/main/CLI | 기존 PlaybackWorkspaceHost·NativePlaybackHost를 동일 configured client에 연결하는 assembly 후보, main의 명시 mode 조립 | host/store/client1개. CLI standalone 경로/기존 protocol 유지 및 parent 모드의 owner 인수 계약 |

### P3 검증 순서와 완료 한계

먼저 Rust 계약/CLI로 artifact 재검증, unsupported/invalid 입력의 process 차단, instance·track revision·전환 slot/stale 거부, host 영구 revoke와 close/reopen 구분을 검사한다. trusted partial startup/parent loss/cleanup failure는 원 process·worker·partial child와 같은 attempt를 남기는 bounded seam으로 관측한다. 필요한 실제 media decode·frame/시간·제어 속성·재생/정리 receipt는 **새 CLI pin**으로 기존 resolved-result 유스케이스를 실행한 뒤 인수한다. 과거 URL/Referer가 지금도 유효한 capability라고 가정하거나 모델/다운로드를 재개하지 않는다.

그 다음 새 Meari parent/NativeChild 실제 GUI는 geometry/resize·overlay overlap·focus/pointer·suspend/close/forced loss 및 실제 OS parent 수명만 배정된 범위에서 확인한다. raw handle 확보/프로세스 생성/검은 placeholder/child cleanup signal을 decode·UI 입력·Destroyed로 대신하지 않는다. 모든 현재 포트·DTO가 존재한다는 사실과 새 TSX video child 통합의 실제 성공은 별개다. 현재 단계는 계획만 준비했으며 구현/컴파일/미디어/GUI/다운로드를 수행하지 않았다. 전체 playback·자막·제작·AI·S2 목표와 DPI/VM 보류를 유지한다.
