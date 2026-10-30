---
id: plan:kinekino/20261029113557
title: Native 영상 slot 소비 계약 준비
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native 영상 slot 소비 계약 준비

2026-10-20. [shared NativeChild 검증](../../sots/verification/application/shared-native-playback-child-2026-10-20.md), [조립 계획](native-playback-child-assembly.md), [기존 native 경계](native-resolution-handoff.md)를 잇는 계약 준비다. Meari 고정 49443cf7e55585937d5a0314826f5642565cbed4와 현재 제품 소스의 읽기 대조만 수행했다. 이 문서 작성으로 새 port/OS/미디어 지원이 생기지 않는다. 추가 제품 코드·Cargo·GUI 실행0, 수동 시각 검토·모델·DPI·VM 보류 유지.

후속 기준 체크포인트는 PM이 SDK5 r2+Meari gitlink49443cf만 exact6으로 통합한 `def8a2e59ad7ac94ae166ff5443ed3bd0c8c867b`다. PM 인계상 index empty/submodule clean이다. 앞선 검증 snapshot에는 자막7이 없으므로 이 통합 root 전체의 결합 시험 완료로 확대하지 않는다. subtitle/PNG API additive merge는 PM 중앙 책임이며 본 source 준비에서 편집하지 않는다.

## 실제 포트와 부족한 권한

| 책임 | 실제 source/API | 지금 입증하지 못하는 것 |
| --- | --- | --- |
| parent provenance | Meari `infrastructure/window/parent.rs`: NativeChildRegistrar.register→borrowed NativeScopedParent+!Send NativeParentLease; with_parent/with_cleanup_parent가 원 window/child/epoch/phase 검증 | 임의 문서/TSX video node의 slot 권한. lease의 Weak 연결만으로 strong coordinator/HWND 생존 보장 불가 |
| 창 환경 | NativeWindowGeometry{revision,environment}, Registrar.geometry; HostEnvironment의 layout/visual viewport,device_scale,layout_zoom,safe_area | video slot client rect·ancestor clip·paint order. whole viewport를 영상 rect로 사용 불가 |
| node 배치 | Meari Native domain LayoutBox{id,x,y,width,height}와 실제 renderer/layout | stable document/node reservation generation, effective clip/focus, OS surface ACK와 결합된 공개 lease |
| 렌더 관측 | Native frame/checked CPU submission | OS compositor present/child HWND overlay 합성 성공 |
| 기존 제품 표면 | `src/infrastructure/video_surface.rs::VideoSurface::{new,id,layout}`: legacy eframe parent에서 STATIC child, bounds×scale/round,SWP_NOZORDER,ShowWindow,부모 focus 복원 | Meari slot/layout provenance·clip·논리 focus. id의 unchecked usize→u32를 새 경로에 복제 불가 |
| 종료 | legacy VideoSurface::Drop은 DestroyWindow 결과를 버림. SDK ClosedEpisodePlayback은 engine/IPC/stdout/stderr 종료와 diagnostics를 가짐 | 실제 surface Destroy receipt. engine joined와 HWND destroyed는 별도이며 둘을 혼동 불가 |
| engine parent | NativeEpisodeEngine의 생성 시 고정 Option<u32>, owned_launch가 검증된 Some 값으로 --wid 전달 | lease/slot 유효성 및 handle 폭. checked 변환 실패는 오류이며 truncate/None fallback 불가 |

고정 producer source에서 NativeSurfaceSlot/SurfaceSlot/surface_slots 공개 구현은 확인되지 않았다. Meari 담당의 native-host-port.md 설계는 source API가 아니다. host rect나 선택 문자열을 raw HWND/slot 권한처럼 취급하지 않는다.

## Meari에 필요한 최소 공통 계약

최종 명칭/정확 경로는 producer가 확정한다. UI-only registrar가 현재 host/document에 존재하는 실제 video node를 resolve하고 opaque slot lease를 발급해야 한다. semantic id·HostStamp·WindowStamp 관측값의 임의 조합으로 권한을 mint하지 않는다. 원 NativeParentLease와 slot lease를 같은 trusted registrar 경계에서 결합한다.

1. **identity/generation**: 원 host·PluginScope·definition/document revision·retained node identity·slot reservation generation·WindowStamp를 내부에 묶는다. node 재생성/HMR/문서교체/window 재생성은 옛 lease를 폐기한다. 원 metadata는 진단이며 public raw setter/serde token/Send 승격은 없다.
2. **slot snapshot**: 독립 monotonic layout revision과 host geometry revision을 제공한다. client-local rect, scroll/transform 적용 기준, device scale/layout zoom과 validated physical 변환 책임을 명시한다. font/asset/scroll/resize 변동도 revision에 반영한다. SDK가 viewport에서 slot을 추정하거나 DPI를 중복 곱하지 않는다.
3. **clip/visibility/plane**: effective ancestor clip·hidden/zero-area를 관측한다. unsupported rounded/complex clip·UI overlay 조합은 Unsupported/Unknown으로 명시하고 보이는 사각형으로 넓히지 않는다. 실제 backend ACK와 desired layout을 구별한다. HWND airspace 제약을 CSS만으로 해결했다고 표시하지 않는다.
4. **focus/input**: render-only surface와 Meari controls의 논리 focus, Tab 진입/복귀, pointer owner를 협상한다. SetFocus(parent) 성공은 typed node focus 완료가 아니다. 기존 pointer_policy의 커서 정책을 재사용한다.
5. **release**: 원 slot/window/child/generation에 결합한 opaque release owner와 실제 Destroy/partial failure 결과를 제공한다. caller가 자유 Complete DTO로 종료를 주입하지 않는다. 폐기는 admission 먼저 차단하고 원 OS owner/잔여 cleanup을 보존한다. 실제 forced loss는 ForcedIncomplete, 관측 불능은 ObservationUnavailable이며 정상 완료로 합치지 않는다.

producer가 먼저 확정할 책임 경로 후보는 Meari domain/application surface_slot와 infrastructure native_host/window surface_slots다. 제품 mpv/URL/SDK 정책을 Meari로 옮기지 않는다. 현재 49443cf의 cleanup unavailable 계약을 유지하고 별도 task/attempt driver를 만들지 않는다.

### 직접 확정한 초기 provenance 계약

Meari 담당과 PM이 producer 별도 checkout `runtime/development/meari-native-surface-slot`의 exact13 구현을 배정했다. 초기 `NativeWindowPreparation`만 원 document+parent registrar를 결합한 NativeSurfaceRegistrar를 발급하고 reserve_video(expectedHostStamp,id)는 실제 video tag와 원 generational NodeId를 검증한다. 일반 문서 revision 변화와 HMR/원 node 교체를 구분한다. 확정 public source pin은 아직 인수 전이다.

중복 lease 소유 반례를 조율해 register_reserved의 factory에 **(&NativeScopedParent, 원 NativeParentLease, NativeSurfaceSlot attachment)**를 분리 전달하기로 직접 확정했다. slot은 noClone/!Send이며 원 lease를 복제하지 않고 같은 private ParentAccess/child/window/coordinator 검증에 결합된다. with_surface/with_cleanup_surface(request,callback)는 그 권한을 재검증해 borrowed parent+snapshot을 제공한다. 기존 NativePlaybackChild는 유일한 lease를 그대로 소비하고 생성자/parent getter 변경은 필요하지 않다. SDK가 실제 OS release owner/Destroy 증거를 소유하며 Meari에 자유 ACK DTO를 요구하지 않는다.

producer 읽기 사실에 따르면 cached Blitz0.3.0-beta.4 get_client_bounding_rect는 physical unrounded client 좌표이며 layout_environment가 deviceScale×zoom을 적용한다. 따라서 초기 snapshot 좌표 계약은 physical client pixels 방향이고 SDK의 추가 DPI 곱은0이다. 최종 타입/단위는 producer pin 뒤 재대조한다. paint transform/미확인 clip·overlay는 Unknown/Unsupported로 닫으며 wholeviewport로 대체하지 않는다.

후속 producer 소스/연구 인계로 이 좌표 표현을 더 좁힌다. source-only r2 `a77f04884119faf44aa649f961048b8ff700123b`/manifest401 SHAA4B3554AAFC077EB6FA865D3A0649F34AB161348D664086C09F111545017C6A2는 원 BaseDocument::id 재검증을 추가했고 public 서명은 유지한다. DOM client box는 viewport-relative layout 수치여서 실제 parent-client 원점에는 safe-area paint offset까지 필요하다. **physical_client_rect라는 필드 이름만으로 SDK OS 배치를 허용하지 않는다.** 추가 DPI 곱0과 실제 원점/clip/plane/focus 미증명은 함께 유지하며 require_rectangular_child_window의 현 거부를 우회하지 않는다. paint clip/layer/stacking/transform·checked scene commit 관측이 선행이고 rounded bounding rect·unsupported clip_path None·layer drop을 지원으로 승격하지 않는다.

현재 전달된 실제 서명 후보는 Preparation.surface_slots→reserve_video(&HostStamp,&str)→register_reserved(factory(&NativeScopedParent,NativeParentLease,NativeSurfaceSlot))와 slot.with_surface(callback(parent,snapshot)), with_cleanup_surface(CleanupRequest,callback(parent)), revoke다. 이는 source-only producer 인계이며 SDK gate는 아직 이 새401 입력을 소비하지 않았다. 실제 Destroy proof는 여전히 SDK OS owner 책임이다.

### a77 typed input 소비 exact5 source 작성

독립 `runtime/development/sdk-surface-input-r1`/base a611f791bad209f156a95d1f2e26cc2dce8499ec에서 mpv/mod.rs와 parent_surface/{mod,input,geometry,tests}.rs를 작성했다. frozen source/등록 patch/정확 producer a77 manifest 의존은 `runtime/coordination/sdk-surface-input5-r1/`에 보존한다. 기존 root/public snapshot/producer 소스는 변경하지 않았다.

실제 NativeSurfaceSlot.with_surface/with_cleanup_surface와 원 NativeParentLease 타입을 직접 소비하는 입력 소유 코드다. constructor child mismatch와 fresh snapshot window mismatch를 거부하고 원 owner/typed 오류를 유지한다. 공개 snapshot을 다시 주입해 원 slot 권한을 mint할 수 없다. 유한/음수/정수범위·scale·rectangular clip의 수치를 검사하되 좌표에 DPI를 다시 곱하지 않는다. start 검증은 visibility/producer clip-plane-focus gate와 parent-client origin 미검증으로 닫혀 있다. source 입력은 실제 backend/ready token/Destroy ACK가 아니다.

시험8은 공개 metadata와 함수 서명 소비만 작성했다. actual NativeParentLease/slot 행사·constructor 실패 시 실제 owner 반환·실제 cleanup 권한은 미검증이며 가짜 객체를 만들지 않았다. 직접 rustfmt/diff0, Cargo/OS/GUI0이다. producer가 예고한 후속 Option<PhysicalClientRect>/FrameTicket/logical_focus 변경은 a77 고정 타입과 별도 revision으로 소비하며 아직 섞지 않는다.

### 후속 실제 Win32 owner의 좁은 책임

다음 구현의 신규 경로 후보는 `mpv/parent_surface/win32/{mod,geometry,release,window_events}.rs` 및 `assembly/meari/playback_surface/surface_release.rs`다. 실제 서명/정확 개수는 검증 producer의 paint origin·clip·plane·focus 계약을 받은 후 확정한다. 임의 HWND/wholeparent id를 입력 DTO로 받지 않는다.

- create는 같은 with_surface borrowed parent와 재검증된 현재 snapshot에서만 수행하고 원 child HWND/생성 generation/실제 thread 소유를 보유한다. mpv parent u32는 이 **생성된 child**에서 checked 변환하며 실패 시 원 partial child를 반환한다. parent HWND 자체를 --wid로 대체하지 않는다.
- geometry는 같은 paint-origin/clip 관측 revision과 결합하고 이미 physical인 좌표에 추가 배율을 적용하지 않는다. bounds*scale/unchecked cast/SWP_NOZORDER만으로 clip·plane 지원을 주장하지 않는다. visibility/focus 요청과 actual OS 결과·Meari 논리 owner를 구분한다.
- release는 원 HWND의 단순 숫자 재사용/IsWindow(false)만을 완료 proof로 삼지 않고 원 생성 identity에 결합한 실제 Destroy/WM_NCDESTROY 관측을 보존한다. native callback은 상태/wake만 기록하며 SDK command/join을 재진입하지 않는다. 확인 불능/실패는 원 OS owner를 유지한다.
- NativePlaybackChild actual joined 이후에만 UI-only surface release를 시작하는 source4 outer에 연결한다. engine lease는 surface 정리 동안 유지한다. actual complete receipt가 있어야 S의 Complete이며 이미 joined된 engine을 surface-only retry에서 다시 시작하지 않는다. Drop의 best-effort Destroy를 정상 receipt로 승격하지 않는다.

이 후속 OS 코드는 아직 작성·실행하지 않았다. 실제 backend 미지원인데 metadata gate를 상수 Supported로 바꾸거나 no-op Destroy adapter를 연결하는 경로는 만들지 않는다.

### 독립 순차 소유 정책 source4 작성

독립 checkout `runtime/development/sdk-playback-surface-r1`에서 `assembly/meari/mod.rs` 등록과 `playback_surface/{mod,cleanup,tests}.rs` exact4를 작성했다. 고정 49443cf NativeChild 계약만 소비하며 실제 slot/OS adapter는 미연결이다. source pin과 준비 상태는 `runtime/coordination/sdk-playback-surface4-r1/`에 보존한다. 컴파일/Cargo/GUI 실행0, 직접 rustfmt/diff만 확인했다.

generic PlaybackSurfaceChild<E,S>는 UI-thread 전용이며 원 두 component를 단일 outer child로 소유한다. engine Complete 뒤에도 engine object/lease를 보유한 채 surface release를 시작하며 둘의 port Complete에서만 outer Complete를 반환한다. surface-only terminal 실패의 새 유효 attempt는 surface만 begin하고 engine을 재시작하지 않는다. 원 full stamp/reason/signal, unavailable/원 typed 오류 및 stage별 잔여 진단을 유지한다. concrete Error Clone(Arc-backed) 제약은 SDK 합성기에만 있으며 Meari NativeChild global Clone/Send 제약은 늘리지 않는다.

공개 WindowLifecycle/CompletionSignal 회귀6을 작성했다: stage 순서/양 owner 해제, surface-only retry/원 typed fault, engine 실패/명시 retry, engine unknown+deadline/같은 attempt, surface unknown+forced loss, duplicate/foreign/reason 거부. 시험 port는 명시 fake backend이며 실제 SDK joined/NativeParentLease/slot/HWND/Destroy/미디어를 입증하지 않는다. 실제 engine join은 이전 SDK35 gate의 별도 증거이고 향후 concrete 조립 인수는 다시 필요하다. 원 SDK35 입력/공유 snapshot/root/index/gitlink는 바꾸지 않았다.

동결 source4의 독립 읽기 검토는 추가 확정 결함 없이 마감됐다. 시험 한계는 성공 뒤 generic component Drop까지이며 pending/unknown outer/lifecycle 최종 Drop·재진입·무신호 adapter 상태 변화는 미포함이다. surface unknown의 원 오류는 variant만 확인하고 모든 stage/모든 오류의 Arc 동일성을 검증한 것으로 넓히지 않는다. 실제 lease 생존/해제 증거는 producer pin을 소비하는 concrete adapter 이후 별도로 필요하다. 현재 Cargo0/회귀6 미실행을 유지한다.

첫 실제 소비 gate는 [build4 admission 거부](../../sots/verification/application/playback-surface-sequence-2026-10-20.md)로 child 생성 전에 종료됐다. CPU74.21875%/receipt125, Cargo·시험0이며 자동 재시도/후속 실행은 하지 않았다. source4/보호596 일치와 owned[]를 확인하고 heavy/snapshot을 PM에 반환했다. snapshot에는 exact4 미검증 overlay와 before 원문이 남는다. 회귀6/관련 회귀·strict/fmt 인수는 중앙 재배정 후의 남은 작업이다.

후속 새 명시 conservative2/-j1 배정에서 r1 surface6+기존child7/router12 실제25pass를 확인했다. native strict의 collapsible_if 원101은 보존하고 같은 조건 let-chain1만 합친 r2 source4 pin9AAF1E25354AC7B6933A94FBA37FD9FAC69E31024F2DBDFBD4B3AB971EACD4DD로 native all-target strict/fmt0을 확인했다. 원 build4 child0 거부와 구별하며 자동 fallback/4CPU Cargo 성공이 아니다. 최종 source4/보호596 delta0·receipt7 terminal·PID13 부재, snapshot r2 보존/PM 반환. actual lease/OS/Destroy와 producer 새 slot14/PNG 미연결 한계는 유지한다.

## SDK의 좁은 소유 경로

구현 배정 가능 영역은 독립 SDK checkout의 assembly/mpv다. 새 source 후보는 `src/assembly/meari/playback_surface/{mod,cleanup,tests}.rs`와 실제 Win32/mpv 연결의 `src/infrastructure/mpv/parent_surface/{mod,geometry,release}.rs`다. 이는 후속 책임 후보이며 지금 생성·수정한 파일이 아니다. 공통 geometry/slot 정책을 이 후보에 복제하지 않고 인수된 Meari snapshot·opaque 권한만 소비한다. producer API가 확정되기 전 exact 구현 개수/함수 서명을 발명하지 않는다. main/UI/application DTO·CLI의 추가 필요 변경은 별도 exact 범위로 제출한다.

trusted UI assembly가 strong coordinator+router+dispatcher host를 유지한다. 실제 registrar closure에서만 slot/borrowed parent 검증→surface 생성→checked parent id→NativeEpisodeEngine 준비를 수행하고, 등록 성공 뒤 현재 lease/layout을 다시 확인한 경우만 router.activate한다. source resolution/remote message/CLI에 raw handle을 받아 이 경계를 우회하지 않는다. factory 부분 실패는 error+원 partial surface/route/예약 child를 반환한다. SDK instance/store/LocalHost는 하나다.

현재 NativePlaybackChild는 engine retirement의 proof만 처리한다. 실제 surface wrapper가 유일한 coordinator child가 될 때 원 engine child/lease+surface owner를 함께 보유한다. 같은 CleanupRequest/CompletionSignal로 engine 원 task를 관측하고 actual joined 이후 UI-only surface release를 시작·관측한다. 둘의 실제 완료가 있어야 outer child Complete다. surface 단계만 실패했을 때는 이미 joined인 engine을 다시 retry/생성하지 않고 원 surface 잔여 단계만 명시 재시도한다. Unknown/partial receipt와 canonical 오류를 유지한다. worker에 !Send lease/raw UI owner를 넘기거나 UI에서 join/shutdown을 기다리지 않는다.

## 실제 CLI 기능 인수 대상

기존 `src/bin/player-cli/native_host.rs`의 --serve-native는 parent=None standalone을 조립한다. 그 성공은 새 Meari slot 성공이 아니다. GUI를 클릭해 CLI 미지원 기능을 우회하지 않는다. 다음 실제 기능 검증은 배정된 fresh player-cli pin에서 같은 configured SDK/LocalHost endpoint를 사용하며 raw slot/HWND 입력 명령을 새로 노출하지 않는다.

| 단계 | 공통 SDK/기존 명령 | 요구 근거 |
| --- | --- | --- |
| 재생 전 | capabilities/schema, catalog/episodes stored resolution 재검증 | slot 미준비/폐기 시 open 차단, 잘못된 media/source에 spawn0, 과거 URL/Referer를 무검증 재사용하지 않음 |
| 재생/관측 | playback.open/engine/status | 실제 decode·시간/프레임 관측, 원 engine instance ID, 단순 transport-ready/프로세스 생성/검은 영역과 구별 |
| 제어 | play/pause/seek/volume/muted/rate/select_track | 동일 instance/track revision과 실제 속성, stale/foreign 요청이 새 세대로 remap되지 않음 |
| 회차 | previous/next/transition.observe/commit/release | 기존 transition 유스케이스, old owner 종료/새 instance 및 rollback/partial 실패 근거 |
| 캡처/종료 | production.capture_frame, playback.close 및 host shutdown | 실제 media frame와 diagnostics, engine/IPC/stdout/stderr joined receipt, 별도 surface Destroy receipt·마지막 parent 제거 순서 |

먼저 Rust headless 계약 회귀로 identity/clipUnsupported/ready·revoke/partial owner/unknown/same attempt를 검사한다. 실제 OS slot 배치·clipping·focus·surface Destroy는 별도 중앙 슬롯에서만 검증하며 CLI 기능 증거와 구분한다. 수동 GUI·시각 인수는 여전히 보류다. 현재는 위 실제 CLI/OS 검증을 실행하지 않았다.
