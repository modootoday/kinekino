---
id: sot:kinekino/gui-cli-host
title: GUI 독립 네이티브 CLI host 경계 조사
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# GUI 독립 네이티브 CLI host 경계 조사

상태: **읽기 전용 조사·구현 제안**, 2026-10-19 12:33–12:37 KST source 관측. SDK/UI가 동시에 변경 중이므로 실제 착수 때 source를 다시 고정한다. 새 player/mpv/host·검사·추론·네트워크를 실행하지 않았고 코드/Cargo/lock/기존 파일을 수정하거나 이동하지 않았다. 이 신규 문서만 작성했다. 실제 재생 성공이나 새 기능 지원을 주장하지 않는다.

기준은 [최상위 규칙](../../../AGENTS.md), [공통 SDK 계약](../player/application/api/sdk-contract.md), [작업 API](../player/application/api/jobs.md), [카탈로그 API](../player/application/api/catalog.md)다. 기존 [native 제어](../verification/application/native-playback-control-2026-10-19.md), [GUI endpoint 통합](../verification/application/native-gui-endpoint-controls-2026-10-19.md), [로컬 전송](../verification/application/local-host-transport-2026-10-19.md), [전송 권한 경계](../verification/application/local-host-privilege-boundary-2026-10-19.md)는 각 당시 검증이며 GUI 없는 신규 native host의 증거로 전용하지 않는다.

## 현재 지원과 연결이 끊긴 지점

| 책임 | 실제 source 관측 | GUI 독립 재사용 가능성과 제한 |
| --- | --- | --- |
| 상태 값 | `application/native_playback.rs::NativePlayback` 및 NativePauseIntent | OS/UI 비의존 값·pause intent. bool 기본값과 관측됨을 pause_observed/idle_observed로 구별 |
| 공통 상태 port | `application/playback_status/{mod,observed}.rs`의 PlaybackStatusPort·Publisher/Reader | GUI 없이 생성/게시/조회 가능. 게시 owner가 필요하며 published_age는 native event age가 아님 |
| 공통 제어 port | `application/playback_control/mod.rs`, `domain/playback_control.rs` | Play/Pause/Seek/SetVolume 네 가지 typed 정책·payload/schema·접수 receipt 재사용. Open/Close/Rate/Track/이전다음은 현재 enum/dispatch에 없음 |
| 동적 engine binding | `infrastructure/native_playback_control.rs`의 NativeControlBinder/Port | lifecycle owner가 MpvControlHandle을 bind/clear하고 SDK에 같은 port를 주입할 수 있음. binding은 자체 mpv 생성 기능이 아님 |
| mpv 수명 | `infrastructure/mpv/mod.rs::Mpv` | Child/command handle/updates/snapshot 소유. 현재 launch/launch_local 모두 parent:u32 필수이고 spawn은 --wid를 항상 붙임 |
| IPC | `infrastructure/mpv/{ipc,control,events}.rs` | Windows named pipe+전용 Rust thread/Tokio runtime. GUI API 의존은 없으나 shutdown/join/명령 완료 계약 보강 필요 |
| GUI rendering | `infrastructure/video_surface.rs` | live eframe parent를 빌려 WS_CHILD HWND 생성·layout·파괴. 이것은 CLI host의 필수 공통 domain이 아님 |
| 현재 조립 | `src/main.rs`, `presentation/{player_app,native_controls,transport,workspace}.rs` | GUI가 Mpv 생성·binding·snapshot drain/publish·초기 mute 해제·교체·종료를 소유 |
| one-shot CLI | `bin/player-cli/runner.rs::execute_local` | SubtitleFiles/frame 및 선택 jobs/catalog/review/project/proposal adapter를 조립하지만 native status/control/lifecycle는 주입하지 않음 |
| 명시 host 모델 | `bin/player-cli/{job_host,provider_host}.rs` | LocalHost endpoint 출력·stdin EOF 종료 패턴은 재사용 가능. jobs host는 subtitle 작업, provider host는 catalog.search/episodes.list로서 native host가 아님 |

`application/api/capabilities.rs`에 playback.open/engine/rate/track/previous/next/close의 이름은 있으나 현재 supported 판정은 `playback.status`와 네 가지 `playback_control::is_command`에만 연결된다. 명령 목록 존재를 실행 지원으로 세면 안 된다. GUI endpoint에 CLI가 붙어 네 제어를 수행한 기존 시험과, GUI 없이 CLI host가 mpv를 생성하는 새 기능도 서로 다르다.

## GUI parent와 rendering의 분리

현재 GUI는 `presentation/workspace.rs:23`에서 local Mpv launch에 `self.surface.id()`를 주고, `presentation/transport.rs:39–49`는 원격 source에도 같은 HWND를 준다. `VideoSurface::new`는 live eframe Win32 parent를 요구하며 id()는 HWND를 u32로 변환한다. 기존 서명을 재사용해 CLI가 임의 숫자0이나 사용자 앱 HWND를 넣는 것을 해결책으로 채택하지 않는다. pointer 폭/유효성·owner 수명은 플랫폼 adapter에서 별도 점검할 대상이며 현재 관측만으로 실제 truncation 장애가 발생했다고 주장하지 않는다.

최소 설계는 application에서 OS handle 없는 **출력 의도**를 구별하고 infrastructure가 실제 rendering host를 조립하는 것이다. 아래 이름은 후보이며 새 API가 구현됐다는 뜻이 아니다.

| 출력 모드 후보 | 필요한 host | 증명할 수 있는 범위 |
| --- | --- | --- |
| MpvOwnWindow | CLI native host가 소유 mpv process를 만들고 mpv 자체 top-level video output을 사용. embedded parent --wid 경로와 분리 | 실제 표시 가능한 video output·decoded dimensions·time 진행·window/VO 상태를 독립 관측. **player.exe 제품 GUI 실행/수동 QA 아님** |
| EmbeddedSurface | 제품 GUI가 자기 surface adapter를 제공하고 mpv가 그 parent 안에서 그림 | 현재 GUI 통합 경로. CLI 계약이 arbitrary HWND를 외부 입력으로 받지 않음 |
| AudioOutput | audio device/session이 사용 가능하고 명시 mute/volume 정책이 적용됨. 영상 표시 의도가 없으면 그렇게 명시 | audio decode/time/device·출력 상태. 실제 들림은 장치/별도 관측 필요. video UX 완료가 아님 |
| DecodeOnly / NoVisibleVideo | 명시 headless/decode 출력 adapter | time/decoder/metadata 등 기술 검증. visible video frame·사용자 시청·GUI 합격으로 세지 않음 |

현재 spawn은 `--force-window=yes`, `--mute=yes`, 키·OSC 비활성화, `--wid`를 고정한다. CLI 자체 창 모드는 이 프로필을 그대로 쓰는 대신 output 정책을 명시해야 한다. mpv 자체 창에서 키 입력이 꺼져 있어도 명령형 CLI/IPC 제어는 별도 가능하지만 실제 동작은 인수 시험이 필요하다. **현재 muted startup의 해제는 PlayerApp::update_native**에 있으므로 GUI를 없애고 spawn만 호출하면 음소거가 남을 수 있다. 새 host owner가 시작·준비·명시 mute 해제/보존을 정책으로 관리해야 한다.

mpv 자체 영상 창은 GUI 라이브러리·제품 창을 요구하지 않는 native rendering 방식의 후보이며 OS의 interactive desktop/graphics output/device가 필요하다. hidden WebView parent가 존재한다고 mpv의 표시 가능한 영상 surface가 확보되는 것은 아니다. 어떤 mpv 옵션/VO 조합이 지원되는지는 trusted 고정 binary와 실제 host에서 검증해야 하며 이번 조사에서는 옵션·rendering 지원을 실행 확인하지 않았다.

## 최소 공통 책임 경계

1. **domain**: source kind(local video / 검증된 provider media), SessionId/engine generation, 출력 의도, 제어 값, 회차 인접 선택·범위·pause intent. HWND/Child/WebView/pipe/JSON mpv command가 들어가지 않는다.
2. **application**: 기존 PlaybackControlPort/StatusPort를 유지하고 lifecycle용 작은 typed port를 추가 검토한다. open/close 결과는 명시 세션/instance·접수/관측 상태를 반환한다. SessionOwner는 source resolve→engine 생성→bind→관측→교체→종료 순서를 일관되게 조율한다. UI event에서만 기능을 구현하지 않는다.
3. **infrastructure/mpv**: trusted executable·소유 child·출력 adapter·IPC·events·명령 correlation·종료와 reap를 담당한다. spawn 설정은 EngineLaunchOptions 같은 typed 값으로 내부 조립하며 브라우저/CLI payload가 임의 argv·실행 파일·shell을 지정하지 못하게 한다.
4. **assembly/host**: 명시 `--serve-native` 같은 mode 후보로 lifecycle owner와 status/control ports를 PlayerSdk에 주입하고 기존 LocalHost를 연결한다. 준비 endpoint와 media loaded/render ready를 구분한다. 사용자 입력 없이 host를 자동 시작하거나 GUI로 fallback하지 않는다.
5. **presentation**: CLI는 명령 파싱/구조화된 응답, GUI는 동일 client·surface selection만 맡는다. GUI의 workspace layout/fullscreen/자동숨김을 headless host의 책임으로 옮기지 않는다.

라이브 재생은 지속 세션이며 “파일 결과 검증 후 Completed인 durable job”과 동일하지 않다. jobs.submit에 playback 명령을 무조건 넣는 대신 session lifecycle를 별도 계약으로 두고, provider resolve 등 긴 준비가 jobs가 필요하면 명시 source/result ref로 연결한다. WorkerHost/ExecutorRegistry의 취소·deadline·Restart Interrupted를 playback 상태에 그대로 복사하지 않는다. host 종료 후 media를 자동 재생하는 정책은 별도 승인·계약이다.

## IPC·관측·종료에서 재사용/보강할 점

- `MpvControlHandle::submit`은 engine instance ID와 live/connected 상태를 점검하고 bounded64 큐에 enqueue한다. not-ready/ended/full/closed 오류를 그대로 재사용한다. bind 교체 후 이전 handle이 새 engine을 제어하지 않는 수명 규칙을 유지한다.
- IPC는 pipe에 최대50×100ms 연결 시도 후 property observation을 등록한다. 현재 request_id는1000부터 증가하지만 application 명령 ID와 correlation receipt는 없다. events는 모든 성공 reply의 `acknowledgements` 수를 증가시키므로 observe_property 성공도 섞일 수 있다. 이 합계로 개별 pause/seek 적용 완료를 주장하지 않는다.
- 현재 네 제어 receipt는 **accepted/unobserved**다. SDK에서 applied/failed를 추가할 경우 command/request/session/generation과 mpv response를 연결하고 독립 property readback을 보존해야 한다. response 성공과 실제 media/output 준비도 서로 다른 상태다.
- `Mpv::snapshot`이 updates를 drain하고 Child::try_wait를 검사한다. GUI frame가 없어도 host loop가 이를 호출해 publisher에 전달해야 한다. 동일 state를 주기 재게시하면 published_age가 새로워져도 event freshness는 새로워지지 않는다. raw event sequence/time과 publication 시각을 필요에 맞게 구별한다.
- status에는 engine/session identity·rate·track/mute·render mode·최근 native event age가 없다. 새 open/교체·previous/next 검증에서 상태를 잘못된 source에 귀속하지 않도록 보강 계약을 검토한다. 누락 값을 false/0·기본 track으로 채우지 않는다.
- 현재 `Mpv::Drop`은 state close→직접 child.kill/wait이며 IPC thread JoinHandle은 보관하지 않는다. 명시 close는 graceful quit 시도/유한 deadline/owned child reap/IPC loop 종료와 실패 receipt를 설계해야 한다. command handle clone이 남아도 종료를 확인할 수 있어야 한다. 프로세스 트리/자손까지 현재 보장한다고 표시하지 않는다.
- 현재 mpv pipe 이름은 PID+시각 nonce다. 제품 LocalHost의 current-user/same-privilege 검사와 mpv 자체 IPC endpoint의 ACL·접속 주체 보장은 같은 것이 아니다. mpv side security를 별도 확인하며 외부 arbitrary pipe 주입을 허용하지 않는다.
- binary 선택은 current_exe 옆 mpv.exe 존재 확인이다. 기존 오류 문구에 가제 exe명이 남아 있는 점도 관측됐다. 제품 식별 중앙화와 trusted binary/version/SHA preflight는 조립 책임으로 검토한다. 임의 browser data로 실행 파일을 바꾸지 않는다.

## 빠진 명령과 최소 typed 계약 후보

| 기능 | 현재 source 범위 | 추가할 application 계약·검증 |
| --- | --- | --- |
| playback.open | Mpv launch/launch_local는 infrastructure API, SDK open dispatcher 없음 | 검증된 SourceRef+OutputIntent+시작/pause/volume, Session/Generation·lifecycle receipt. source bounds·실행 실패·readiness 명시 |
| playback.close | GUI stop_native/drop만 존재 | 대상 session/generation, 이미 종료/다른 세션/timeout 구분. control/status clear와 실제 child exit·IPC 종료 관측 |
| playback.engine | PlaybackRoute의 native/web preference와 generation별 재시도 제한 | 실제 engine availability/mode 선택·대체 실패. GUI 없는 host가 web playback을 지원하는지 별도 선언 |
| playback.set_rate | 명령 이름만 catalogue에 존재, enum/wire/status 미구현 | 유한 rate 범위·engine capability·command correlation·관측 speed. 지원 범위 결정 전 기본 success 금지 |
| playback.select_track | 명령 이름만 존재, track list/선택 status 없음 | engine track identity·종류·없음/auto 정책·선택 검증·재조회. cue speaker ID와 혼동 금지 |
| playback.previous/next | EpisodeQueue.adjacent와 GUI episode panel·EpisodeTransition 존재 | queue/source revision·현재회차·방향·경계/no-target·resolve ticket·new generation. stale response 거부와 pause intent 유지 |

기존 `PlaybackControl`에 open/close와 모든 job 책임을 무조건 추가하는 거대 enum보다, lifecycle/transport/short controls의 port를 구별하고 SDK가 명령별로 dispatch하는 방향을 권고한다. 새 Request schema·error/capabilities는 실제 주입 port의 지원에 따라 노출한다. 중복 Request/Response나 GUI 전용 정책 복제를 만들지 않는다.

## 회차·provider 연결의 현재 한계

`EpisodeQueue`는 검증된 EpisodePage/current URL로 회차 목록을 고정하고 adjacent에서 경계를 검사한다. GUI episode_panel과 EpisodeTransition은 선택·page ready·native handoff·autoplay·30초 expiry를 조합한다. 이 값 정책은 CLI에서 재사용 가능하지만 실제 source resolver는 별도다.

provider application에는 BrowserProviderPort(try_start/poll/cancel/navigate/release), BrowserMedia/BrowserResult, BrowserWork의 ticket·document generation·stale 거부가 존재한다. infrastructure browser_provider는 STA/hidden HWND/메시지 펌프/WebView2를 소유한다. 이는 사용자 Chrome이나 player.exe가 없어도 사용할 수 있는 adapter 구조지만 **브라우저 엔진·윈도우 자원 자체가 없는 구현은 아니다**.

현재 BrowserCatalogJobs::operations는 `catalog.search`, `episodes.list` 두 개다. CLI provider host도 같은 명령만 등록한다. MediaReady 타입·observer 코드의 존재만으로 episodes.resolve와 실제 회차→media→mpv 연결이 지원됐다고 주장하지 않는다. 기존 [provider 검증](../verification/application/browser-provider-cli-2026-10-19.md)과 별도로 resolve 허용 navigation/referer·manifest·광고 차단·source generation·취소/종료를 인수해야 한다. 저장 목록 재조회와 live media 재생도 별도 gate다.

## 실제 CLI 인수 항목과 담당

**전체 CLI 기능 검증을 먼저** 수행하고 제품 GUI 수동 QA는 이 연구/선행 작업에 넣지 않는다. 아래 항목은 아직 실행하지 않은 인수 기준이다. VM·DPI·새 장시간 시험·추론도 실행하지 않는다.

| 순서 | 실제 인수 항목 | 소유 담당·source 지점 |
| --- | --- | --- |
| 1 | 명시 native host 시작·구조화된 endpoint, 기본 in-process/다른 host에서 unsupported, 별도 process CLI discovery/schema/errors. 동일 user/privilege·busy/timeout/execution_unknown 유지 | SDK: bin/player-cli 새 host 조립, application/api/client/capabilities/queries, infrastructure/local_host |
| 2 | 고정 trusted mpv/source SHA·local media open, pause intent, MpvOwnWindow 실제 output/time/video observation. decode-only 결과와 구별. product player.exe는 실행하지 않음 | SDK: lifecycle typed port·mpv launch profile/owner; UI: embedded surface 요구만 계약 검토 |
| 3 | play/pause/seek/volume 실제 CLI→동일 SDK→mpv, enqueue receipt와 command reply·독립 property를 구별. source/session/generation 일치 | SDK: 기존 playback_control/status/native_control/mpv IPC. UI는 기존 client 사용 유지 |
| 4 | close/EOF/signal/실패·media없음·mpv종료·IPC단절·큐 full, 실제 owned PID/handle cleanup·후속 status·중복 close. 다른 앱/세션 영향 없음 | SDK: lifecycle owner/mpv IPC Join·shutdown. 현재 Drop 성공을 추정하지 않음 |
| 5 | rate/track 지원이 추가된 경우 모든 schema·범위·invalid/없음·실제 관측까지. 구현 전 unsupported 유지 | SDK typed 정책/MPV adapter; UI 후속 표시 계약 |
| 6 | 목록·resolve ticket과 이전다음 경계·빠른 재선택·오래된 media callback·취소·원래 pause intent·source 바뀜·각 close 후 재open, 실제 CLI source별 readback | SDK: EpisodeQueue/Transition의 application orchestration+provider port; UI 담당: browser/회차 DTO 일관성 검토·별도 구현 소유 조율 |
| 7 | 기능별 새 bin/source/args/원본 응답·실패·media/process/관측 evidence를 보존. fmt/test/clippy all-targets-Dwarnings, legacy GUI endpoint contract 회귀는 CLI 시험과 분리 기록 | SDK 검증 소유, PM 전체 gate 조율. UI 수동 QA는 CLI 전수 인수 후 별도 |

최소 구현 인계는 SDK가 lifecycle owner/typed port·mpv output profile·host 조립·CLI 인수를 맡고, UI 담당은 **기존 PlayerApp의 lifecycle 조립을 동일 application owner로 옮길 때** surface layout/parent 수명과 shared client 연결을 맡는 방식이다. presentation/main 파일은 소유 담당이 변경한다. 공통 API와 mpv adapter의 계약을 먼저 맞추고 검증된 owner를 두 소비자가 조립한다. 리서치 서브에이전트는 실제 변경을 수행하지 않는다.
