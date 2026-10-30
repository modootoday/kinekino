---
id: adr:kinekino/20261029113541
title: "제안: React→Meari 제품의 최상위 창 소유"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제안: React→Meari 제품의 최상위 창 소유

상태: **Proposed·제품 ADR 입력·미채택/미구현**, 2026-10-20. source/manifest/빌드/GUI 변경0. 사용자 확정 목표는 React→Meari 기반 공용 컴포넌트·라우팅·전체13화면을 하나의 제품 창으로 구현하는 것이다. Meari의 [native-child 초안](../../../../packages/meari/.spec/sots/integration/native-window-lifecycle.md)과 [ADR0016](../../../../packages/meari/.spec/adrs/0016-native-window-child-lifecycle.md)은 범용 수명 제안이며 제품 backend를 결정하지 않는다.

PM 조율 상태: A를 **우선 검증 경로로 배정**했다. 최종 제품 backend 채택·기본 런처 전환·eframe 삭제 승인은 아니다. B는 A의 실제 제약이 확인될 때 비교할 대안으로 유지한다. 다음 [최소 수직의 책임/선택/종료 의존성](../../../plans/player/presentation/native-window-minimal-vertical.md)만 준비하며 SDK 공통 검증·fresh CLI 우선순위를 유지한다.

## 권고

**A: Meari가 최상위 창을 소유하고 제품 인프라가 범용 native-child lease로 영상/WebView를 연결하는 경로를 먼저 검증한다.** 이미 검증한 원본 TSX/ModuleProgram/NativeHostRuntime·factory·동기 receipt·입력/close 경계를 주 UI 경로에 직접 연결하므로, 장기적으로 두 UI/layout/input/a11y 체계를 유지하는 중복이 적다는 판단이다. 이는 source에 근거한 우선 검증 권고이며 도입 비용·성능 측정이나 채택 완료가 아니다.

B: 기존 eframe 창 import는 별도 비교 대안으로 보존한다. eframe GL 표면 유지의 장점이 있지만 현재 driver가 기존 창을 import하지 않으며 event/render/a11y/lifecycle 연결 adapter가 새로 필요하다. lease가 불가능하거나 기존 renderer 성능/미디어 수명 요구를 A가 충족하지 못하면 B의 실제 spike로 재검토한다. 먼저 기존 제품을 삭제하거나 기본 런처를 미검증 backend로 바꾸지 않는다.

## 실제 고정 버전 관측

아래는 읽은 manifest/lock 값이다. 설치된 전역 최신 버전·호환성 보장이 아니다. SDK gate 진행 중인 읽기 관측이므로 실행 인수 pin은 최종 owner 결과에서 다시 고정한다.

| 책임 | 기존 제품 root | Meari Native |
|---|---|---|
| UI/runtime | eframe/egui0.36.2 | meari-native0.1.0, blitz-shell/dom/html/paint0.3.0-beta.4 |
| event loop | winit0.30.13 | winit0.31.0-beta.3 |
| 현재 창 renderer | eframe Renderer::Glow, glow0.17.0 | NativeWindowDriver의 VelloCpuWindowRenderer, anyrender_vello_cpu0.18.0/anyrender0.14.0 |
| GL 관련 | glutin0.32.3 | 선택 feature gl의 glutin0.32.3/glow0.16.0/anyrender_skia0.12.0/skia-safe0.153.3 |
| native handle | raw-window-handle0.6.2, wry0.57.0 | raw-window-handle0.6.2 |

root Cargo.lock SHA 관측526da706…bf4a4, Meari lock f029e934…af87와 원 파일 hashes는 runtime/coordination/ui-native-window-architecture-r1/read-only-observed-pins.json에 보존했다. 숫자가 같은 glutin/raw-window-handle만으로 창·context·FBO 소유권이 공유되는 것은 아니다. 서로 다른 winit Rust 타입을 그대로 위임할 수 있다고 가정하지 않는다. Meari의 별도 gl feature는 현재 CPU driver의 외부 eframe context import 증거가 아니다.

## A/B 비교

| 기준 | A: Meari 최상위+child lease | B: eframe 창 import |
|---|---|---|
| React→Meari UI | 원본 module/라우팅·공용 토큰/컴포넌트를 주 UI로 연결 | 같은 UI 이관 외에 eframe과 Meari renderer/event bridge 필요 |
| 중복 owner | 하나의 Native window/driver·DOM/input/a11y owner로 수렴 | 기존 eframe owner를 유지하되 Meari의 새 창 생성 경로를 import 경로로 보완하고 양쪽 권한·frame/close를 조율해야 함 |
| 기존 미디어 부모 | eframe cc 대신 lease의 실제 parent를 주입하도록 infrastructure 조립 변경 | 현재 cc 기반 부모를 유지하기 쉬움. 다만 Meari가 외부 owner의 ready/revoke/teardown을 관측하는 계약은 여전히 필요 |
| renderer | 현재 CPU로 제한된 prototype 인수부터 가능. GL 방향은 별도 결정/실측 | 기존 Glow 유지 가능성. NativeScene의 표시/입력/a11y·외부 context/FBO·loss/복원 구현은 미지원 |
| 이관 비용 | PlayerApp/StudioShell의 eframe UI·frame 이벤트·chrome/overlay·단축키 연결을 Meari로 이관해야 함 | 즉시 legacy 화면 보존은 쉬우나 완전한 React→Meari 목표에서 그 UI 이관은 없어지지 않음 |
| 현재 결정 불가 | lease·정상 child barrier·실제 OS parent/geometry/suspend·전체 화면/시각 인수 필요 | pinned loop/type 경계·실제 import/공유 GL·이중 처리 없는 입력/표시·동일 수명 인수 필요 |

A도 단순 getter로 끝나는 작업이 아니다. parent owner/epoch·surface generation·geometry revision·child 등록/revocation/cleanup ACK와 예산·실패 retry를 구현하고 Meari W01–W13을 인수해야 한다. 현재 HostCleanupComplete는 child-aware barrier가 아니므로 기존46의 close 성공을 이 인수로 확대하지 않는다. 독립 두 번째 창은 진단 prototype이며 제품 단일 창 완료가 아니다.

## 현재 접점과 재사용 범위

PlayerApp::new는 eframe CreationContext를 VideoSurface::new·Browser::new·WindowActivity·WindowMetrics·LocalFilePicker에 전달한다. VideoSurface는 generic HasWindowHandle에서 부모 HWND를 얻어 소유 STATIC child를 생성하고, logical bounds×scale·focus/visibility를 조정한다. Browser는 같은 generic parent로 wry build_as_child를 호출하고 logical bounds·visible·기존 navigation/event/audio port를 사용한다. 이 generic parent 접점은 A에서 재사용할 후보이며 장기 안전성을 이미 제공하는 lease는 아니다.

현재 mpv 프로세스 adapter는 VideoSurface의 numeric id를 --wid로 받고 IPC·상태/control·출력을 관리한다. **A의 첫 미디어 수직은 이 기존 프로세스 adapter와 child surface를 유지할 수 있는지 검증한다.** libmpv Render API나 GL texture 공유를 채택 전제로 두지 않는다. 부모 UI가 CPU인 것과 mpv 영상 디코드/출력 backend는 다른 책임이다. 새 source에서 hwdec/실제 영상 표시가 무엇인지 별도 관측한다.

VideoSurface::id의 u32 전달은 현재 adapter 형식이며 WindowOwner/epoch/lease 권한을 대신하지 않는다. handle 폭·부모 관계·focus·DPI mapping은 실제 adapter 인수에서 확인한다. scoped HasWindowHandle borrow만으로 child의 장기 수명을 보장하지 않는다.

기존 PlayerApp Drop은 mpv를 먼저 take하고 Mpv::shutdown은 소유 프로세스 정리·wait/worker join을 수행하지만 Drop은 오류를 무시한다. 새 정상 close에는 이 명시 terminal/오류를 ACK/checkpoint로 보존해야 한다. mpv 사용 중단→video child 해제, WebView 사용 중단/COM thread 정리→parent 제거 의존성을 조립한다. 특정 SDK/media 순서를 Meari generic 라이브러리에 하드코딩하지 않는다. forced destroy는 정상 ACK가 아니다.

## UI 이관 범위와 실제 인수

UI는 기존 client/SDK·read model·정책/저장소·plugin capability와 provenance 의미를 재사용한다. PlayerApp의 eframe Ui 렌더 함수를 그대로 Meari 컴포넌트로 사용할 수 있는 것으로 표시하지 않는다. viewing/player/start·공용 window-chrome/transport·studio/frame/navigation·모든13화면군/하위탭, 선택·empty/demo/loading/error 상태와 키보드/a11y·focus·overlay·native 영역 geometry를 React→Meari 원본 inventory/라우팅에 연결해야 한다.

SVG 아이콘·Pretendard/다국어 fallback·색/spacing/border/radius 및 공용 토큰·shadcn식 소스 소유 컴포넌트, 실제 demo media/이미지·SDK 인터페이스를 유지한다. 현재 [headless bridge46](../../../sots/verification/presentation/native-window-bridge-2026-10-19.md)은 이 범위의 일부 입력/SDK 인수일 뿐 전체 route·Q01–Q09·시각 일치가 아니다. 기준은 승인 `.spec/artifacts/design/`의 전체 시안이며 확대1:1·실제 실행 재검토를 생략하지 않는다.

채택 전 순서는 (1) SDK parent 최종 gate·fresh CLI/provenance (2) generic child lease/동일 OS parent·실패 cleanup 인수 (3) 현재 mpv 프로세스+WebView를 같은 Native 창에 연결하는 최소 수직 (4) 실제 terminal wake/사용자 close·input/a11y (5) 전 화면 이관·시각1:1·renderer 성능/GL 대안 비교다. 실제 조작은 메인 모니터·소유 앱1개·중앙 슬롯·사용자 다른 앱 보호를 따르며 DPI/VM 보류를 임의 해제하지 않는다.

현재 모두 설계 권고와 source 관측이다. 제품 source/manifest·버전 pin 변경, 새 model/libmpv/GL/창 실험 및 실행0이다. [기능/GUI 인수표](../../../plans/player/presentation/proposal-product-registration.md)를 따른다.
