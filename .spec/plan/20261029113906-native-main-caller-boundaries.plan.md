---
id: plan:kinekino/20261029113906
title: Meari 기본 shell 전환 · 실제 caller 경계 조사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Meari 기본 shell 전환 · 실제 caller 경계 조사

2026-10-21 PM 읽기 배정. root e93a0b7, Meari22fe0d 기준이며 생산 코드/CLI/GUI/빌드 변경0이다. [기존 단계](native-main-integration-checkpoints.md)·[현재 Viewer 종단 입력](../../../../runtime/coordination/ui-viewer-chrome-union12-r1/HANDOFF.md)을 잇는다. 최종 목표는 동일13화면군/모든 하위탭을 갖춘 Meari native-default이며 아래 작은 단계가 완료 정의를 대신하지 않는다.

## 확인한 실제 호출 경로

| 경계 | 실제 source·현재 동작 |
| --- | --- |
| 제품 실행 선택 | `src/main.rs`, `application/launch.rs`: DesktopMode는 Player/DesignPreview/NativeProposal. 기본 Player는 presentation::run_with_native_playback→eframe PlayerApp이다. NativeViewer 선택/caller는 없다. |
| 공유 SDK·endpoint | main이 status/control channel과 저장소로 구성한 PlayerSdk 한 개를 ApplicationClientHandle로 만들고 LocalHost를 시작한다. NativeProposal 분기도 해당 client를 쓰지만 endpoint는 main에 남아 있다. |
| 기존 NativeProposal | `assembly/meari/product_native_launch.rs`: 제안 request 파일/selection·PreparedProductStyles를 읽고 Proposal 실행/창/정리를 조립한다. 일반 Viewer의 준비/종료와 다른 carrier다. 이 분기의 이름만 바꿔 일반 Player에 쓰지 않는다. |
| Viewer 실행 준비 | `native_execution_startup::prepare_native_execution(client,budget,schedule)`는 원 NativeLoopOwner/startup/notifier/ExecutionOwner를 만든다. 실패는 loop/startup/client/notifier를 보존한다. 이미 한 번 만든 startup을 새 loop로 대체하지 않는다. |
| Viewer 조립 | `viewer_native_launch::prepare_viewer_launch(ViewerLaunchInput)`가 공급 client/LocalHost/PreparedNativeExecution을 이동시켜 같은 ViewerHost·actual graph/callback bindings/document를 조립한다. 공개 결과는 NoChild alias다. |
| 실행·정리 | PreparedViewerLaunch.run→run_shell_host→원 owner.run_startup. finish_viewer→동일 typed cleanup driver, Returned runtime Rc 동일성/child barrier/endpoint join/ACK/ExecutionOwner 순서를 사용한다. main caller는 아직 없다. |
| NoMedia opt-in | `viewer_native_launch/no_media.rs`의 별도 public prepare는 QuarantineChild/typed 실패 retention을 사용한다. 테스트의 숨긴 disabled child proof다. 이를 일반 재생·visible video·기본 실행 활성화로 취급하지 않는다. |

main에 Viewer를 연결할 때 **기존 LocalHost를 ViewerLaunchInput으로 이동**해야 한다. 두 번째 endpoint/SDK를 만드는 wrapper를 추가하거나 Viewer-owned endpoint를 main에서 별도 shutdown하지 않는다. 실패 carrier를 anyhow 문자열로 변환해 버리지 않고 원 소유 상태로 report/원 cleanup/명시 process boundary까지 보존해야 한다. NativeExecutionFailure 시에도 이미 준비한 endpoint와 원 execution 실패를 함께 보유할 상위 carrier가 필요하다. 이는 새 종료 상태 머신이 아니라 기존 cleanup 결과를 보존하는 조립 책임이다.

## 기본 전환 전에 남은 실제 제품 경계

현재 [union17 실제 입력](../../../../runtime/coordination/ui-native-product-union17-r1/HANDOFF.md)은 원12+caller6r3의 교집합mod1에 caller 우선 적용한 별도 준비 핀이다. source17 3CDF8902...9D35A98/rootbefore 기존9·신규8/원입력차이0/직접fmt17 check0이며 실행0이다. Meari window source11/union31을 합치거나 원 부모핀을 교체하지 않았다.

현재 caller 소스 후보는 [r2 실제6](../../../../runtime/coordination/ui-product-viewer-caller-r1/source-r2/HANDOFF.md)이다. r1의 새 outcome 진단 출력이 stdout/stderr 실패에서 panic할 수 있다는 독립 감사 P2에 따라 fallible writer/secondary 오류 반환으로 최소 수리하고 BrokenPipe 회귀4를 추가했다. 원r1 보존·r2 변경3/불변3, 작성 pure12/직접fmt6 check0·Cargo/실행0이다. 기존 NativeProposal/shared 초기화 진단의 동일 위험은 범위 밖 잔여이며 전체 출력 생명주기 수리 완료로 부르지 않는다.

`player_shell/document::prepare_start_document`는 view.start와 viewing/player.tsx만 수용하며 다른 route는 SourceNotIntegrated다. routes.json은 product-route-contract-not-runtime-registered로 선언돼 있다. 3 viewing route와20 studio route의 선언/일부 source 존재를 실제 mount/기능으로 세지 않는다. 단일 시작 화면 caller 추가만으로13화면 완료라고 하지 않는다.

`components/window-chrome.tsx`의 minimize/maximize/close는 disabled이며 onClick이 없다. 기존 ShellHostController.request_close는 route NavigationGuards→원 NativeWindowWake.request_close의 안전한 close intent 경로를 제공한다. ViewerCallbacks/ReadActions는 현재 capabilities/engine/status와 ACK/projection local actions만 처리한다. caption callback을 추가할 경우 기존 callback scope/queue/ACK와 window 입력 수명을 지키는 별도 presentation/assembly 책임이 필요하다.

읽은 Meari NativeWindowWake는 wake/retry_failed_completion/request_close, NativeWindowDriver는 attach/wake_handle/last_submission/presentation_errors를 공개한다. 이 surface에서는 minimize/maximize/fullscreen command를 확인하지 못했다. 정확한 기존 API가 있는지 Meari 담당에 읽기 확인을 요청했다. **라이브러리 부족 확정 전 후보**이며 raw HWND·별도 loop·새 JS 구현으로 우회하지 않는다. 필요 계약은 원 window/host scope의 UI-thread action 전달과 stale/closed 거절·실제 결과 관측이다.

TransportBar는 이전/재생/다음/음량/회차/전체화면이 disabled 상태다. 기존 SDK PlaybackControl/episode-session 계약을 실제 scoped callback으로 소비해야 하며 GUI 전용 재생 정책을 만들지 않는다. 현재 semantic video는 src/control/autoplay 없는 자리다. 실제 미디어 child는 QuarantineChild와 별개다. NativePlaybackChild는 원 ParentLease/ParentEngineRouter/ParentRouteHandle의 cleanup을 연결하고, PlaybackSurfaceChild는 engine→surface release 순서를 합성하지만 해당 모듈 자체가 실제 slot/OS backend를 제공하지는 않는다.

## 최소 수직 단계 후보 · 아직 생산 편집 배정0

최신 caller 후보는 [r3 실제6](../../../../runtime/coordination/ui-product-viewer-caller-r1/source-r3/HANDOFF.md)이다. r2 outcome I/O 수리의 독립 인수 후 새 NativeViewer의 초기 local-host-ready도 borrowed metadata/fallible writer로 분기했다. 다른 모드는 유지하고 flush 전용 오류 회귀2를 추가했다. source6 6AA30FC9...450C713/작성14/직접fmt6 check0·실행0이며 예상 전체95의 명단도 현재입력 미실행으로 고정했다. 과거 단계의 편집 보류와 후속 exact6 source-only 배정을 구분한다.

후속 PM exact6 source-only 배정으로 [caller6 실제 소스](../../../../runtime/coordination/ui-product-viewer-caller-r1/source-r1/HANDOFF.md)를 작성했다. 원 source12 기반 overlay에 명시 --native-viewer 선택과 원 owner 조립을 구현했으며 root/default 값은 바꾸지 않았다. source6 SHA9738F8B0...1AAA6B, mod.rs 교집합1의 before/precedence를 따로 고정했다. 작성 pure8/최종 직접fmt6 check0이고 컴파일/실행0이다. 아래 표는 최초 제안이며 이 문서만으로 새 공동17/실행을 승인하지 않는다.

후속 Meari 교차 확인에서 해당 공개 창 동작 port 부재를 확인했다. source-only 최소 사례·기존 공개/내부 경계·typed 동작/원 WindowStamp와 lifecycle·접수/OS 관측 분리·원 close 유지 인수 조건은 [UI-HOST-WINDOW-ACTIONS-01](native-window-actions-gap.md)에 분리했다. 원 NoMedia API 부족으로 확대하지 않는다.

| 단계 | 정확 후보 경로 | 유지할 계약·인수 |
| --- | --- | --- |
| Caller 조립 | `src/main.rs`, `src/application/launch.rs`, 신규 `src/application/launch/native_viewer.rs`, `src/assembly/meari/mod.rs`, 신규 `src/assembly/meari/product_viewer_launch.rs`, 신규 `src/assembly/meari/product_viewer_launch/tests.rs` | 기존 client/endpoint/prepare_native_execution→공개 prepare_viewer_launch→run→finish_viewer를 한 번 연결. mode/옵션은 trusted local Rust 입력으로 설계하고 기존 args·제안 모드를 보존. NoChild 시작/read 실제 기능의 단계이며 기본 전환 완료가 아님. 실패의 원 endpoint/native/typed owners를 끝까지 보유. |
| Caption/host intent | `window-chrome.tsx`/Viewing props graph 및 callback/window-action 책임별 모듈 | close는 기존 guards/owned wake 소비. min/max/fullscreen은 실제 Meari host action API 확인·필요 시 본체 보완 후 같은 consumer로 검증. 버튼을 enabled로 바꾸기만 하지 않음. provider 입력 변경도 정확 인계. |
| 전체 route/SDK 입력 | 기존 player_shell route/document/host callbacks와 각13화면군 source·공통컴포넌트 | 실제 선택 route의 graph/typed view model·공통 입력/저장·키보드/접근성/동일 토큰·폰트/자산을 소비. 선언만 등록하거나 flatten하지 않음. 단계별 정확 파일은 해당 구현 전 별도 고정. |
| 실제 media child | 기존 generic Viewer carriers, playback_child/surface와 원 SDK parent/episode 계약의 별도 조립 | same Native document/slot/registrar/coordinator에 원 playback owner를 연결. NoMedia proof를 재사용하지 않고 실제 미디어·surface·engine 관측/종료/ACK를 검증. 임의 selector/raw handle/서명 URL 재구성0. |
| Native 기본값 전환 | 위 caller와 application launch 정책의 후속 최소 delta | 전체 기능·종료·필수 caption/route·실제 재생 인수 뒤 별도 pin에서 기본 선택을 전환. eframe에 새 정책/디자인을 확장해 최종 방향을 바꾸지 않음. |

Caller 후보6은 제안이며 현재source12와 `assembly/meari/mod.rs`가 겹친다. 배정 시 source12 인수 상태와 정확 delta/before를 먼저 고정하고 무단 union을 만들지 않는다. config·report·owner carrier가 커지면 source 책임을 별도 모듈로 나눈 뒤 추가 경로를 조율한다. resource/SDK snapshot은 중앙 슬롯 반환 후 확인한다.

현재 source12는 코드 읽기·직접fmt까지며 CPU74/OS2는 미실행이다. 새 caller/main 기본값 변경·Meari API 구현·child cursor 포트·OS·수동 시각·추론·VM/DPI 작업은 하지 않았다. 실제 미구현을 인수 조건에 남기고 현재 SDK 단일heavy를 보존한다.
