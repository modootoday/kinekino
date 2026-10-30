---
id: plan:kinekino/20261029113931
title: Native caption consumer · 후보19 경계
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native caption consumer · 후보19 경계

2026-10-21 PM 읽기 배정. 제품 union17 3CDF8902와 Meari window11 r2 2593582B/별도union31 후보를 구분한다. 둘은 source 읽기 인수이며 타입/실행/OS 인수가 아니다. 이 문서는 다음 source-only 경로 제안이고 생산 편집 배정이 아니다.

## 실제 연결 순서

1. 원 ViewerHost의 scoped window state와 약한 observer를 준비한다. 원 ShellHostController.start의 preparation 반환 직후 그 preparation.window_actions()를 state에 한 번 전달한다. 별도 driver/loop·raw window를 만들지 않는다. scope는 caller 입력이 아닌 원 포트 snapshot에서 얻는다.
2. 기존 Ready/FrameSubmitted/dispatch 기회에서 snapshot을 읽는다. NotReady/Busy는 성공이 아니며 마지막 유효 scope/revision과 구분한다. **첫 request 전 외부 OS 변화의 독립 Completion은 활성화되지 않는다.** 원 observer가 새 snapshot revision/상태를 보관한 경우에만 borrow 종료 뒤 기존 wake를 보낸다. 같은 snapshot의 FrameSubmitted 반복으로 매번 reproject/wake하지 않는다.
3. callback admission은 원 PluginScope/fullstamp로 검증하고 원 local callback을 bounded queue에 보관한다. 창 action은 이 시점에 실행하지 않는다. min/max/restore는 명시 bool intent이며 관측값을 추정해 toggle하지 않는다. source/doc가 바뀐 queued callback은 기존 registry에서 재검증한다.
4. 원 SDK/read 처리 성공·remaining_callbacks0·원 consumer retry barrier 이후 창 intent를 요청한다. 실패한 SDK outcome에서 창 action을 우회 실행하지 않는다. ID/원 ticket을 유지하고 projection 실패나 Busy에서 새 ID로 재요청하지 않는다. 접수/Issued는 실제 관측과 분리한다.
5. **한 번의 checked 재투영**에 read props와 window props를 합친다. 현재 after_dispatch는 `!read_projection.dirty`이면 조기 반환하고, 성공 시 incoming turn stamp로 한 commit receipt를 만든다. window dirty도 이 결정에 포함하고 같은 try_reproject에서 합쳐야 한다. SDK commit 뒤 두 번째 reproject를 붙이면 원 expected stamp가 stale하거나 불필요한 revision이 생긴다. 성공 시 두 dirty를 함께 clear하고 실패하면 원 데이터·ticket·dirty를 보존한다. 기존 FrameReflection에는 원 incoming turn을 유지한다.
6. 포트의 변화는 원 Completion으로 도착하며 pending turn을 이어갈 수 있다. FrameSubmitted를 OSObserved로 보거나 N+1을 고정하지 않는다. take_result는 보관한 동일 ID의 terminal만 수용한다. 원 host 제거 후 약한 포트는 Closed이고 library record 이력도 소멸하므로 영구 보관을 가정하지 않는다.
7. close는 기존 NavigationGuards→owned wake→child-first barrier→원 Viewer/SDK 종료를 유지한다. action port에 Destroy/Close 우회를 추가하지 않는다. route/wake를 공유하는 좁은 close-intent 함수로 기존 controller와 UI가 같은 판단을 소비하고 전체 controller/runtime를 강하게 캡처하지 않는다.

## 정확 후보19

모두 아래 상대경로이며 source17에 덮는 시점의 before/precedence는 별도 고정한다.

| 경로 | 책임 |
| --- | --- |
| src/assembly/meari/mod.rs | 제품 window consumer 모듈 등록 |
| src/assembly/meari/player_shell_host.rs | 원 preparation의 포트 전달 및 선택적 observer 조립 |
| src/assembly/meari/player_shell_host/delivery.rs | 기존 observer에 snapshot 신호 결합·borrow 뒤 원 wake |
| 신규 src/assembly/meari/player_shell_host/window_intents.rs | 원 route guards/close intent 전달 공통화 |
| src/assembly/meari/viewer_native_launch.rs | 같은 Viewer state/observer 설치, 원 public owners 유지 |
| src/assembly/meari/viewer_host/mod.rs | SDK/suffix barrier 후 action 처리·단일 props commit |
| src/assembly/meari/viewer_callbacks/lifecycle.rs | 원 registry의 window local key·queue 위임, SDK read 역할 보존 |
| src/assembly/meari/viewer_callbacks/codec.rs | 새 실제 callback 구현/props source의 provider 입력 대응 |
| 신규 src/assembly/meari/viewer_window_actions/mod.rs | 상태·local binding의 제한된 facade |
| 신규 src/assembly/meari/viewer_window_actions/state.rs | 원 weak port·snapshot/명시 intent·retained ticket/dirty |
| 신규 src/assembly/meari/viewer_window_actions/callbacks.rs | 원 local callback 검증·키/queue, SDK 명령 생성0 |
| 신규 src/assembly/meari/viewer_window_actions/tests.rs | stale/중복/Unknown·props/registry·commit barrier 회귀 |
| src/presentation/web_ui_adapter/player_shell/document.rs | 원 graph/기본 props와 명시 창 상태 props 조립 |
| 신규 src/presentation/web_ui_adapter/player_shell/document/window_actions.rs | renderer용 DTO·label/disabled 상태·callback reference projection |
| src/presentation/web_ui/viewing/read-panel.tsx | 원 read props와 caption props 전달 |
| src/presentation/web_ui/viewing/player.tsx | 같은 공통 WindowChrome에 상태/동작 전달 |
| src/presentation/web_ui/components/window-chrome.tsx | min/max/restore/close 공통 컴포넌트 상태·입력 |
| src/presentation/web_ui/styles/components.css | 기존 토큰으로 restore 의미 아이콘 대응 |
| src/infrastructure/meari/product_styles.rs | 기존 직접작성 assets/design/window/restore.svg의 검증 asset 등록 |

현재 restore.svg는 실제 존재하지만 Native product asset 등록은 minimize/maximize/close만 있고 CSS restore 클래스도 없다. 새 그림을 임의 대체하지 않고 같은 SVG를 등록한다. fullscreen port는 유지하되 첫 caption 단계의 UI fullscreen 버튼/exit SVG는 별도 transport 단계의 필수 잔여다. 이 분리가 전체 요구를 삭제하지 않는다.

## 첫 구현에서 보호할 source 표현

WindowChrome은 ViewingPlayer뿐 아니라 StudioFrame도 소비하며, callback-free 기본 Viewer graph와 read graph도 기존 회귀다. 새 required callback props를 일괄 추가해 이들을 깨뜨리지 않는다. 원 static/interactive 경로의 기본값·callback bindings를 actual module graph CPU mount로 먼저 확인한다. SourceModuleGraph는 모듈당 component 하나와 명시 ComponentReference.export를 검증하므로 **동일 파일에 복수 export variant를 만들 수 있다고 가정하지 않는다**. slot/조건부 prop 표현이 현재 IR로 안 되면 최소 재현/진단으로 추가 파일 또는 Meari 요구를 조율한다. 이 호환성 확인 없이19가 최종 충분한 구현 경로라고 주장하지 않는다. 기존 markup을 화면마다 복사하거나 flatten해서 맞추지 않는다.

최소 회귀는 first Ready/NotReady·같은 snapshot 반복/외부 관측·원 scope stale·동일 request ID/반복 callback·Busy/Queued/Issued/Observed/Closed·원 SDK error/suffix 시 side effect 금지·단일 commit/실패 dirty 보존·원 read/NoMedia 회귀다. 원 library fake ledger와 제품 CPU props 검사를 실제 창 동작으로 승격하지 않는다. library accepted pin을 받은 뒤 별도 source 배정으로 구현하고 actual min/restore/max/close·입력/키보드·전체13화면 시각은 중앙 슬롯에서 검증한다.
