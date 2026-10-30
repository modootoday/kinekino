---
id: plan:kinekino/20261029113837
title: Native 영상 surface·생산 child 연결 계획
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native 영상 surface·생산 child 연결 계획

상태: 2026-10-21 PM의 계획 전용 배정. 기준은 UI13r2 통합 `35d257c7d9db1d055afd6f2341b84df74127c956`이다. 제품 구현 lease/Cargo/shared snapshot/OS/GUI/main 변경 권한은 없다. 이 문서는 구현 승인이나 시각 승인 기록이 아니다.

## 관측 사실

UI13r2는 실제 고유29와 영향 시험2, 양 strict/fmt를 인수했다. 읽기3 callback은 실제 mounted scope에 연결됐으나 제품 TSX 응답 표시·버튼 활성화는 후속이다. `ViewerCloseProgress::CallbackPending`은 원 ticket 보존이며 화면 NeedsTerminalAck를 표시하는 기능은 아니다.

현재 `viewer_cleanup.rs`의 ViewerController는 `ShellHostController<ViewerHost, ViewerNoChild>`다. `viewer_native_launch.rs`는 이미 시작한 SDK/endpoint/loop를 받아 동일 host와 CPU 문서를 조립한다. 이 단일 소유 경로를 유지한다. NoSelection에서는 재생 child를 등록하지 않는다.

Meari `infrastructure/window/surface_slots.rs`는 실제 node·문서·window·slot generation과 frame/layout/host geometry revision, rect/clip/plane/visibility를 제공한다. 현재 snapshot focus는 `SurfaceFocus::Unknown`이다. 제품 `parent_surface/geometry.rs::require_start`는 geometry와 rectangular-child 조건을 확인한 뒤에도 `ParentClientOriginUnverified`를 반환한다. 따라서 현 상태에서 visible surface를 시작할 수 있다고 가정하지 않는다.

`NativeSurfaceInput`은 원 lease/slot을 소유하며 raw handle 생성자·Clone/Send를 제공하지 않는다. `NativePlaybackChild`는 원 lease와 router/route를 보존하고 retirement를 정리한다. `PlaybackSurfaceChild<E,S>`는 engine Complete 뒤 surface 정리를 시작하는 기존 seam이다. `RecoveryRegistry`는 실패 시 원 owner를 보존한다. `HiddenSurfaceProbe`는 실제 visible surface 생성자나 영상 성공 증거가 아니다.

## 책임과 조립 제안

Meari는 실제 document node/slot/parent lease·창 child 등록·host runtime·관측 권한을 소유한다. 제품 infrastructure의 Rust visible adapter는 승인된 slot 관측으로 OS 자식 surface를 생성·배치하고 원 OS 소유자와 실제 Destroy 증거를 보존한다. application/domain 및 presentation에서 OS API를 호출하지 않는다.

제품 assembly의 surface factory는 실제 node에 대한 원 registrar를 사용해 단일 child ID를 예약하고 lease/slot을 함께 받는다. factory 밖의 실패 소유자를 먼저 확보한 뒤 NativeSurfaceInput 검사, visible adapter 생성, SDK parent binding/route 준비, engine child 조립을 진행한다. 정확한 생성 순서는 parent binding이 실제 어느 HWND를 요구하는지 SDK와 확정한 뒤 동결한다. 등록 중 Creating 단계의 제한을 정상 접근 권한으로 우회하지 않는다.

생산 `ViewerChild`는 기존 `PlaybackSurfaceChild<NativePlaybackChild, VisibleSurfaceChild>`를 감싸는 단일 native child로 제안한다. 단, lease를 engine과 surface 양쪽에 복제하지 않는다. NativePlaybackChild가 lease를 소유하고 surface가 slot/OS owner를 소유할 수 있는지 실제 생성/cleanup 접근 계약으로 확인해야 한다. 기존 input의 `into_owners`만으로 권한 분배가 검증됐다고 하지 않는다. 필요하면 동일 원 소유를 보존하는 작은 engine cleanup adapter를 SDK와 정하고 기존 retirement state machine을 재작성하지 않는다.

controller는 `GenericViewerController<C>` 형태로 공통 shell 조립을 재사용하고, 실제 생산 child 타입이 확정된 시점부터 `ViewerController = GenericViewerController<ViewerChild>`를 선택한다. NoSelection에는 등록 child가0개이며, 재생 시작 때 같은 controller/원 window에 child를 등록한다. 빈 child 또는 probe를 생산 타입으로 이름만 바꿔 연결 완료로 표시하지 않는다. 재생 전후 controller·SDK worker·endpoint·loop를 교체하지 않는다.

## 수명과 실패 계약

정상 종료는 기존 runtime child-first 정리를 통해 engine retirement/실 worker·process 정리→surface Destroy 관측→child release→host endpoint admission close/기접수 drain/양 join→callback terminal 명시 수락·ACK→attachment release→원 ExecutionOwner close/join 순서를 유지한다. engine이 Pending/ObservationUnavailable/실패면 surface Destroy를 시작하지 않는다. CloseComplete·ACK·Destroy를 Drop이나 elapsed timeout에서 합성하지 않는다.

constructor 실패는 실패 지점 이전에 획득한 원 lease/slot/OS owner/route/retirement와 진단을 반환한다. 등록 후 부분 실패·관측 실패는 동일 child/request/recovery key를 유지하고 원 coordinator가 명시 재시도한다. 독립 cleanup loop나 자동 새 engine/새 window fallback을 추가하지 않는다. parent loss는 admission을 폐기하고 기존 cleanup 전용 접근으로 원 child를 정리한다.

배치 갱신은 node/source generation·window·frame·layout/host geometry revision을 재검증한다. resize/minimize/clip/overlap/route 변경/HMR·node 삭제 시 이전 snapshot을 재사용하지 않는다. unknown·zero area·unsupported plane/focus에서는 신규 생성/재생 admission을 닫고 기존 owner를 보존한다. 어느 경우에 hide/relocate/retire할지는 Meari 관측 계약과 SDK control 계약을 확정해야 하며 정책을 OS adapter에 숨기지 않는다.

## 미결정과 조율

### 1차 격리 child·disabled readproof 최소 배정안 · 계획 전용

freshpaint 소비는 [Meari 기존 경로 정본](../../../../runtime/development/meari-parent-client/.spec/sots/native/surface-redraw-gap.md)과 연결한다. parent-client NotReady는 원 checkpoint의 Ok Pending 상태로 보존하고 HostEffects{committed:None,redraw:true}로 원 요청/owner 세대당1회만 redraw를 요청한다. **실제 FrameSubmitted의 원 N HostStamp observer 안에서** 원 checkpoint.advance와 필요한 slot.with_parent_client/disabled proof를 실소비한다. checked/document/owner borrow 종료 뒤 권한 없는 owned 결과만 저장하고 own wake한다. Completion N+1은 결과 반영만 하며 old proof를 다시 소비하지 않는다. 앞선 "ownwake 뒤 checked 재개" 계획은 next_turn에 따른 full HostStamp 불일치를 놓쳤으므로 정정했다. 원 deadline/budget·owner는 유지한다. 아직 frame이 없을 때 snapshot.slot_generation을 읽을 수 있다고 가정하지 않고 원 checkpoint/slot 객체 identity·source/window 결속을 유지한다. 새 request_redraw API나 stamp 완화는 도입하지 않는다. [host 후속 계획](native-surface-host-follow-up.md)은 이 호출 위치와 최소 경로·기본 ViewerNoChild 전환의 추가 범위를 분리한다. 이는 배선 계획이고 제품 코드/실 OS 실행 근거가 아니다.

범위는 actual parent-client checked borrow와 원 registrar/lease를 소비하는 NoMedia·hidden·disabled child 및 현재 readproof까지다. engine attach/route 활성화/visible 배치/media open·viewer controller 기본 타입 전환/main/GUI 실행은 제외한다. 현재 start 화면에 실제 video node가 있다는 가정도 하지 않는다. factory는 호출자가 실제 선언 node에 대해 예약한 원 NativeSurfaceReservation을 받아야 하며 div/복사 snapshot으로 대체하지 않는다.

**필수 Meari 입력:** 현재 source 후보는 parent-client `22fe0d952f5d03d28147d12fcce5f1914fe26bbc`, source412 SHA `D60780C56CF3E3756F8202EB25B97F52C76278386BFFB8BF26EE34E9670ADE98`다. 후보의 NativeSurfaceSlot::with_parent_client/NativeCheckedParentClient::with_parent를 직접 읽었다. 실제 첫 구현 lease 때는 중앙 gate 종료·최종 producer pin/채택 commit·Cargo Gdi feature와 lock의 검증 결과를 고정한다. 현재 all-target typecheck 진행 보고를 최종412 gate 통과로 승격하지 않는다. focus Unknown 유지가 입력 계약이다.

제품 원 exact12 후보(아래 PM 승인 +1과 합쳐 현재 source13):

1. `src/infrastructure/mpv/parent_surface/mod.rs` — 제한된 NoMedia child/proof 타입 노출.
2. `src/infrastructure/mpv/parent_surface/input.rs` — 원 parent/slot identity를 유지한 checked-parent callback 연결.
3. `src/infrastructure/mpv/parent_surface/win32/mod.rs` — 내부 child readback 모듈 연결.
4. `src/infrastructure/mpv/parent_surface/win32/construction_guard.rs` — 기존 실제 class/HWND 소유 primitive를 작은 private 함수로 공유하고 checked-parent 안의 격리 생성 경로 추가.
5. `src/infrastructure/mpv/parent_surface/win32/child_input.rs` (신규) — 원 child identity/style/disabled/Destroy readback과 짧은 opaque proof.
6. `src/infrastructure/mpv/parent_surface/win32/child_input/tests.rs` (신규) — 제어 read port의 순서/변화/실패 회귀.
7. `src/infrastructure/mpv/parent_surface/tests.rs` — legacy require_start/Unknown 거절 보존과 타입 경계.
8. `src/assembly/meari/mod.rs` — 격리 factory 모듈 연결.
9. `src/assembly/meari/quarantine_surface/mod.rs` (신규) — 원 registration/lease/input/OS owner·상태·실패 반환.
10. `src/assembly/meari/quarantine_surface/construction.rs` (신규) — inert 등록 뒤 명시 생성 및 proof 소비.
11. `src/assembly/meari/quarantine_surface/cleanup.rs` (신규) — 원 NativeChild request/cleanup 전용 접근·실 Destroy 관측.
12. `src/assembly/meari/quarantine_surface/tests.rs` (신규) — 원 owner/부분 실패/cleanup 회귀와 실제 registrar 타입 계약.

2026-10-21 PM은 `src/infrastructure/mpv/mod.rs`의 feature-gated `pub(crate)` 타입 재노출 1파일 추가를 승인했다. parent_surface가 private이며 기존 선별 reexport로 새 두 타입을 assembly에서 이름 붙일 수 없었던 공개 경계 누락을 해소한다. 외부 API·정책은 추가하지 않는다. 현재 소스 초안·원12/추가1 before·source13 pin·실행하지 않은 검증 행렬과 한계는 [r1 인계](../../../../runtime/coordination/ui-quarantine-r1/HANDOFF.md)에 기록한다. 제품 루트 적용·Cargo·실제 OS 검증·host observer 설치 완료를 뜻하지 않는다.

r1 독립 읽기에서 확인한 class 정리 실패의 재시도 차단과 invalid full-request의 admission 변경은 별도 r2로 수리하고 r1을 보존했다. 현재 검토 대상은 [r2 인계](../../../../runtime/coordination/ui-quarantine-r1/revision-r2/HANDOFF.md)와 해당 source13 pin이다. 실제 coordinator를 사용하는 제어 child 회귀를 포함해 새7개를 작성했으며 실행은 아직 배정 전이다.

현재 수리 재검토 대상은 [r3 인계](../../../../runtime/coordination/ui-quarantine-r1/revision-r3/HANDOFF.md)다. 원 uncertain attempt의 접근 실패와 늦은 Destroy 확인 뒤 class 마감 분류를 각각 보완해 새9개를 작성했다. r1/r2는 보존한다. [첫 gate 계획](../../../../runtime/coordination/ui-quarantine-r1/revision-r3/gate-plan.md)은 컴파일·제어 회귀만 수행하며, [own-window 후보 설계](../../../../runtime/coordination/ui-quarantine-r1/revision-r3/own-window-fixture-plan.md)는 별도 OS 배정 대상으로 분리한다. 현재 Cargo/shared 변경/OS 실행0이다.

첫 gate 최종은 [r4 인계](../../../../runtime/coordination/ui-quarantine-r1/revision-r4/HANDOFF.md)다. r3 Native strict의 enum 크기101을 보존하고 원값 Box 이동1파일만 수리했다. r4 actual Native 고유84(새9 포함)·default/native all-target strict·workspace fmt가 통과했다. ignored4/actual registrar·slot/실 OS는 실행하지 않았다. source13·비입력·원 library412/CargoLock/SDK A9r2 보호차이0, 19 receipt terminal(원101 포함)/기록38PID부재/index0을 확인해 heavy/shared를 PM에 명시 반환했다. 제품 root/index 채택과 실제 host 연결은 후속이다. 앞선 r1~r3의 작성/미실행 상태는 당시 기록으로 보존한다.

후속 [own-window source6 r5](../../../../runtime/coordination/ui-own-window-r1/revision-r5/HANDOFF.md)는 실제 순수85+지정 OS composite1=고유86·양 all-target strict/fmt를 통과했다. actual registrar/원 partial/현재 N-frame proof/같은 full scope의 정확 Completion N+1/원 hidden disabled child와 parent 정상 정리·remaining0을 확인했다. 원 strict101 둘/수리2를 보존하고 root1433/snapshot1393 비입력·library412/CargoLock/기존 SDK·media 입력·binary/inner 근거 보호차이0,15 outer terminal/기록31PID부재/index0으로 heavy/shared를 반환했다. 다른 ignored4·실 OS 실패 주입·최초 admission false 부작용0·기본 Viewer host 배선·시각/미디어는 여전히 미검증/후속이다. root 반영은 PM 인계 대기다.

기존 HiddenSurfaceProbe의 public 진단 성공/ProbeObservation은 권한으로 소비하지 않는다. 공유 대상은 UI thread/class registration/WndProc event 소유/hidden WS_CHILD|WS_DISABLED 생성의 private OS primitive뿐이다. 기존 probe 경로의 동작은 기계적 대응과 회귀로 보존한다. readproof는 별도 타입이며 public bool/복사 observation/숫자 HWND에서 만들 수 없다. 기존 geometry.rs::require_start와 snapshot.focus를 수정하지 않는다. 실제 source 의존 때문에 owner/release/window_events 수정이 필요하면 추가 파일과 이유를 먼저 보고하고 exact12를 조용히 확대하지 않는다.

제안 API 반환 형상(아직 공개 API 아님): `register_no_media(original registrar, reservation)`은 원 ChildLease·동일 coordinator/recovery 결속과 제한된 construction checkpoint를 반환한다. registrar의 실제 register_reserved factory 안에서는 input/OS-owner 상태를 inert로 보존하고 OS Create를 실행하지 않는다. 등록 후 정상 parent 접근이 가능한 단계에서 `construct_quarantine(checkpoint)`가 with_parent_client→with_parent 안에서 NoMedia hidden disabled child만 생성한다. checkpoint는 원 owner 상태를 가리키는 제한된 Rust capability이며 cleanup 권한을 복제하지 않는다.

등록 실패는 기존 ChildCreationFailure/원 NativeParentLease·NativeSurfaceSlot·partial child를 보존한다. 등록 후 생성/readback 실패는 동일 original registration/coordinator/recovery key·input·partial OS owner·첫 오류를 유지하는 failure carrier를 반환한다. 새 coordinator/child를 만들어 실패를 덮거나 Drop으로 정리 완료를 만들지 않는다. 외부 checkpoint와 coordinator가 같은 원 상태를 관측하더라도 cleanup은 원 NativeChild 경로 한 곳만 수행한다.

현재 구현의 `checkpoint.with_disabled_child(consumer)`는 원 input의 private slot에서 checked parent를 직접 연다. caller가 별도 checked proof를 바꿔 넣을 수 없다. 같은 callback 안에서 원 child generation/WindowStamp/parent 관계·현 process/thread·WS_CHILD/WS_DISABLED·IsWindowEnabled=false·hidden 유지·WM_NCDESTROY 미관측을 다시 읽고 짧은 borrowed proof를 전달한다. proof는 non-Clone/!Send/!Sync/no-serde/private constructor이며 consumer 진입 직전에 parent/frame/slot과 child readback을 재검증한다. 복사 진단은 별도로 반환해도 실행 권한이 아니다. 현재 disabled 상태 증거일 뿐, 미래 mpv 자손·메시지 전송·실제 렌더 가능성·RenderOnly focus DTO를 보장하지 않는다.

NoMedia 단계에는 engine이 없으므로 원 CleanupRequest window/child/generation을 확인한 NativeChild cleanup만 OS Destroy를 시작한다. 기존 release primitive의 원 WndProc/event identity·실 NCDESTROY 및 class release 관측을 유지하고 Pending/ObservationUnavailable은 원 owner와 함께 반환한다. media를 붙인 뒤의 engine-first retirement/실 IPC worker join→surface Destroy는 후속 단계의 별도 조립이며 여기서 fake engine Complete를 만들지 않는다.

Rust 회귀 명세는 (a) factory Creating 단계 OS 호출0/등록 후 checked callback에서만 생성, (b) focus Unknown 상태 격리 생성과 visible/media 권한0, (c) 현재 child generation/parent/process/thread/style/enabled/Destroy 교차·변화 거절/consumer0, (d) frame/slot/HMR/revoke·parent move/resize 재검증 실패, (e) class/Create/installed callback 부분 실패 원소유 및 첫 오류 보존, (f) cleanup 중 원 request 정합·Destroy 미관측 Pending·실 receipt 전 release 금지, (g) 기존 probe/legacy start 거절의 기계적 호환이다. 제어 OS port와 pure gate는 실제 HWND/disabled 렌더 실행 증거가 아니다. 첫 source 배정은 source-only·직접fmt/읽기 인수로 닫고, 실제 OS 생성/입력/렌더/Destroy는 별도 명시 gate로 남긴다.

### 생성 전/후 권한 순환 해소 · source 대조 결론

현재 `parent_surface/geometry.rs::require_start`는 snapshot.require_rectangular_child_window를 먼저 호출하며, 해당 Meari 검사는 RenderOnly focus를 요구한다. 그 뒤에도 제품은 ParentClientOriginUnverified로 거절한다. 실제 child 입력 비수용 proof를 생성 후에만 발급한다면 이 helper를 child 생성 전 필수조건으로 그대로 사용하는 새 visible factory는 순환한다. 따라서 한 번의 require_start로 생성과 media admission을 모두 승인하는 계약 대신 아래 두 단계가 필요하다. 현재 거절을 삭제하거나 snapshot.focus를 RenderOnly로 고치는 방식은 사용하지 않는다.

1. **격리 child 준비/생성:** 원 coordinator에 inert owner/recovery를 먼저 보존하고 원 child ID·lease·slot을 등록한다. 원 checked parent borrow/lease의 현재 identity·UI thread 권한으로 NoMedia·hidden·disabled child만 생성한다. 생성 전 child RenderOnly proof는 요구하지 않는다. 이 단계는 보여주기/재생/engine route 활성화를 허용하지 않으며 placement0·크기0 자체를 실제 renderer 배치 근거로 해석하지 않는다. 생성 중 실패도 partial OS owner·원 parent/slot·첫 오류를 반환한다.
2. **생성 후 입력/배치 검증과 admission:** 같은 원 child generation·parent/lease·process/thread/WS_CHILD·실 disabled 상태·Destroy 미관측을 실제 child readback으로 검사한다. 이를 현재 parent/frame/slot/geometry·실 renderer 배치/clip/plane 증거와 같은 짧은 checked borrow에 결속하고 별도 child-input proof를 만든다. 두 증거가 모두 현재일 때만 visible 배치/media admission을 허용한다. public snapshot의 focus는 그대로 Unknown일 수 있으며 실제 권한은 불투명 proof로 전달한다. 새 typed admission과 legacy snapshot helper의 책임을 분리하는 정확 API는 별도 lease에서 확정한다.

원 source의 HiddenSurfaceProbe construction_guard는 이미 원 parent borrow 아래 WS_CHILD|WS_DISABLED·hidden zero-area로 생성하고 inert owner/partial construction을 보존한다. 이는 격리 생성에 필요한 상태 분리의 참고 근거일 뿐, 해당 probe에 visible/route/engine 권한을 추가하거나 그 readback을 새 proof로 승격하는 근거가 아니다. 새 생산 adapter는 별도 책임으로 설계한다.

attach/resize/mpv child 생성·parent/enable 변화 뒤에는 원 proof를 재사용하지 않고 새 readback을 요구한다. Unknown/불일치/실패는 admission을 닫고 원 owner를 보존한다. media 시작 전 실패에는 engine이 없음을 명시하고 원 OS child의 실제 Destroy를 관측한다. engine/route를 이미 획득했다면 기존 engine-first retirement·실 worker/IPC join 뒤 surface Destroy 순서를 지킨다. Drop/timeout으로 완료를 만들지 않는다.

향후 회귀는 생성 전 focus Unknown이어도 격리 생성만 가능한 상태, child readback 전 visible/media 거절, proof의 child/parent/frame 교차 사용 거절, stale/enable 변화 재거절, partial 생성 실패 원소유, engine 없는 정리와 engine-first 정리를 구분해야 한다. 이는 작성할 명세이며 실제 disabled child 렌더·입력/IPC·OS 생성/Destroy 검증은 아직 없다. retry13/현재 Meari 순수 gate와 코드·실행을 혼합하지 않는다.

### PM 후속 읽기 반영 · origin과 child focus 분리

PM의 별도 MS 공식 문서 읽기 연구는 제품 infrastructure가 소유 surface HWND를 처음부터 WS_DISABLED로 생성·유지하는 최소안을 권고했다. 원 child generation/parent/lease/process-thread/WS_CHILD·IsWindowEnabled=false·Destroy 미관측을 parent-client checked borrow의 같은 callback에 묶는 짧은 opaque proof 후보이며 아직 구현/실검증이 아니다. attach/resize/mpv child 생성 뒤 재검증하고 enable/parent 변경/stale는 Unknown으로 닫고 원 owner를 보존한다.

담당 연구 출처는 [EnableWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enablewindow)와 [WM_NCHITTEST](https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-nchittest)다. PM 연구에서는 disabled child의 mouse/focus 제외와 HTTRANSPARENT의 같은 thread 한계를 구분했으며 NOACTIVATE/GetFocus만으로 충분하다고 보지 않았다. 이 세션은 해당 OS 동작을 실행 검증하지 않았다. 실제 disabled 상태 mpv 렌더·자손 재enable·parent 입력·IPC controls/Destroy는 계속 미검증이다. Meari parent/frame, 제품 OS child-input/Destroy, SDK IPC/retirement, assembly 결속의 책임을 분리하고 retry13/현재 gate와 혼합하지 않는다.

parent origin의 최소 경계는 원 NativeScopedParent callback 안의 opaque checked borrow다. 원 lease/window/host/slot/frame/geometry와 실제 OS client rect·client→screen origin, 실제 renderer 배치를 같은 관측으로 결속해야 한다. public snapshot·offset0 가정·같은 크기·DPI 재곱으로 이 권한을 만들지 않는다. 아직 구현·실 OS 검증된 API가 아니다.

parent GetFocus 관측은 아직 생성되지 않은 child가 입력을 받지 않는다는 증거가 아니다. child focus/입력 비수용은 별도 checked child 계약으로 검증해야 하며 Unknown 거절을 유지한다. origin bridge 성공을 RenderOnly focus 성공으로 승격하지 않는다.

Meari 후속 경로 후보는 parent.rs·신규 parent_client.rs·surface_slots.rs·checked_window.rs·window.rs와 해당 tests다. 이는 정확 구현 lease가 아니며 별도 경계·회귀 확정이 필요하다. 현재 local-action Core gate나 제품 retry13에 합류하지 않는다. A/B surface 구현 및 실제 visible 지원은 계속 미인수다.

| 담당 | 확정할 계약 | 완료 조건 |
| --- | --- | --- |
| Meari | viewport 상대 rect와 실제 parent-client origin의 대응, 같은 frame/geometry의 유효기간 | caller boolean 없이 원 host 관측으로 검증 가능한 API |
| Meari | RenderOnly focus·clip·plane·overlay/resize/HMR invalidation | 실제 입력/paint 관측과 권한의 구분 및 stale 거절 |
| SDK | visible surface HWND와 parent binding/route prepare·activate·원 PlayerSdk engine 주입 | 같은 router/instance/source revision을 CLI와 GUI가 사용 |
| SDK/UI | lease/slot 소유 분해, engine retirement·dispatcher 종료와 surface Destroy 순서 | partial failure에서 원 owner를 잃지 않는 타입 계약 |
| SDK | media source 해석/파일 선택 이후 admission·open/observe/control/close | 새 GUI 전용 정책 없이 application usecase·CLI 계약 확보 |

SDK/Meari 세션에 위 기존 계약·미결정 확인을 요청했다. 답변 전에는 확정된 API처럼 구현하지 않는다. 이 계획만으로 Meari 소스나 SDK 경로 lease를 확보한 것으로 해석하지 않는다.

### Meari 계약 확인 · 2026-10-21

Meari 담당이 d669 계열(+98f test-support)의 현재 계약을 확인했다. CPU 실제 Scene 관측은 video node/slot generation/전체 HostStamp/frame ticket/geometry revision에 결속한 rect·clip·visibility·plane까지다. parent-client origin과 Native focus는 여전히 Unknown이며 공개 API로 positive RenderOnly/Supported 권한을 만들 수 없다. 논리 focused_node/surface_focusable나 probe 자체 OS readback도 이 증거를 대신하지 않는다.

기존 invalidation은 renderer Weak·DOM document ID·generational node ID·slot generation·전체 host stamp·geometry revision/frame을 확인한다. 새 CPU prepare/submission만 publish하며 pending/실패/stale는 commit하지 않는다. owner Drop·node 제거/HMR 원문 교체·geometry 변경·revoke는 이전 접근을 거절한다. resize/clip/overlap은 새 frame에서 다시 관측하고 rounded/unknown clip/overlay/transform은 보수적으로 거절한다.

담당이 제안한 최소 origin bridge는 원 NativeScopedParent의 같은 UI callback에서 WindowStamp/HostStamp/frame ticket·실 renderer surface offset/actual surface size·실 parent-client OS readback을 결속하는 borrowed non-Clone/!Send token 또는 checked outcome이다. 이는 아직 API/구현/OS 검증이 없는 설계 후보다. focus 역시 passive RenderOnly 선택만으로 올리지 않으며 실제 OS 관측 또는 강제되는 비상호작용 계약을 확인할 별도 checked 경계가 필요하다. 따라서 위 배정 A는 이 Meari 경계의 별도 설계·lease 확정에 의존한다. SDK 답변과 정확 서명은 계속 미결정이다.

## 최소 경로별 배정 후보

다음 목록은 승인 요청용 후보이며 현재 변경 허용 경로가 아니다. 미결정 API가 닫힌 뒤 PM이 exact 경로를 다시 동결한다. 한 번에 전체를 구현하지 않는다.

첫 배정 A는 visible OS adapter의 독립 source8 후보: `src/infrastructure/mpv/parent_surface/{mod.rs,input.rs,geometry.rs,visible/mod.rs,visible/construction.rs,visible/placement.rs,visible/cleanup.rs,visible/tests.rs}`다. 생성·배치·정리를 분리하고 현재 geometry 거절을 관측 근거 없이 제거하지 않는다. 기존 probe 경로를 수정·승격하지 않는다. OS 실행 전 source 읽기와 제어 포트 회귀부터 진행한다.

배정 B는 생산 child 조립 source8 후보: `src/assembly/meari/{mod.rs,viewer_child/mod.rs,viewer_child/surface_factory.rs,viewer_child/tests.rs,viewer_cleanup.rs,viewer_cleanup/tests.rs,viewer_native_launch.rs,viewer_native_launch/tests.rs}`다. 기존 playback_child/playback_surface/recovery를 소비하며 router/SDK 주입에 추가 경로가 필요하면 별도 lease로 분리한다. GenericViewerController/실 ViewerChild 선택과 NoSelection child0·동일 controller 유지·실패 owner 반환을 함께 검증한다. 기존 원13 전체를 재작성하지 않는다.

병행 가능한 후속 C는 읽기3 응답과 CallbackPending 표시다. `viewer_callbacks`의 typed 결과→presentation 관측 DTO→원 document state의 얇은 연결을 설계하고 버튼은 실제 capability/관측 상태에 따라 활성화한다. unknown을 Playing/time0/volume100으로 만들지 않는다. source graph/TSX·공통 adapter의 exact 경로는 별도 화면 계약과 lease에서 정한다. 종료 pending 표시와 명시 ACK 사용자/CLI 동작을 함께 정의하며 close/after_dispatch에 자동 ACK를 추가하지 않는다.

## 검증 순서

1. SDK CLI/API 선행: 동일 configured client에서 media source 검증·open/observe/control/close, expected instance/source revision·stale 거절, unknown/Unsupported, retirement/retry/terminal ownership을 인수한다. HWND·raw pointer를 외부 CLI 입력으로 받지 않는다. CLI headless 통과는 visible HWND 성공 증거가 아니다.
2. source gate: identity mismatch/Creating 접근 제한/stale layout/zero area/unknown focus/부분 생성 실패/engine Pending·실패 시 surface 미정리/같은 recovery 재관측/NoSelection 등록0을 최소 Rust 회귀로 검증한다. fake adapter 결과와 실제 OS 관측을 구분한다.
3. 별도 중앙 OS gate: 원 slot/lease로 visible child 생성·실 rect 배치/resize·hide·clip/Destroy 관측, 실제 router/engine 연결·미디어 open/frame/control/close, parent loss·partial unwind를 검증한다. OS 권한은 새 배정이 필요하다. 이번 계획에 자동 허용하지 않는다.
4. typed 화면 gate: 읽기3·CallbackPending/명시 ACK의 실제 화면 투영과 같은 SDK instance 보존을 인수한다. 전체 시안/토큰/공통 컴포넌트·캡션 버튼 확대 대조는 수동 GUI 재개 배정 후 수행한다.

각 구현 배정은 pin·before 해시·독립 읽기→지정 중앙 gate→보호 파일/원 실행 receipt·소유 반환으로 종료한다. main/기본 DesktopMode 전환은 surface·controls·회차·오디오·입력·창 chrome·복구·시각 gate 이후 마지막 별도 배정으로 유지한다.
