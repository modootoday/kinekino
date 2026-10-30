---
id: plan:kinekino/20261029113836
title: "A 우선 검증: Native 단일 창 최소 수직 준비"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# A 우선 검증: Native 단일 창 최소 수직 준비

상태: 2026-10-20 PM 우선 검증 배정·문서 준비. [제품 ADR](../../../adrs/player/presentation/native-top-level-window.md)은 Proposed이며 최종 backend 채택/기본 런처 전환/eframe 삭제는 미확정이다. 현재 source/manifest/cargo/GUI 변경0. B는 A의 실측 제약 발생 시 비교하며 libmpv/공유 GL을 전제하지 않는다.

## 책임별 변경 대상

| 담당 | 최소 책임·예상 접점 | 이번 준비에서 하지 않는 일 |
|---|---|---|
| SDK | 실제 launch/assembly에서 기존 ApplicationClient·선택 store·LocalHost·playback status/control·제품 parent lease 수명을 연결. 기존 browser/video/mpv 인프라를 child lease의 ready/geometry/revoke/ACK에 적응 | GUI 전용 domain/저장/CAS/decision API 복제, 기본 런처 교체, UI 화면/토큰 변경 |
| UI | canonical 명시 선택→초기 props 순수 bootstrap→원본 inventory/ModuleProgram→검증 factory/bridge. 같은 window의 실제 video/browser 영역 geometry와 공용 chrome·focus·상태 표현 | 임시 ProbeScope·fixture ID/항목0 기본값, OS API 직접 호출, Native child 권한/ACK 위조 |
| Meari | 실제 top-level window owner/epoch·child 등록 lease·권한/geometry·revoke/정상 cleanup barrier·오류/forced·wake API를 범용화하고 W01–W13으로 인수 | 제품 SDK/미디어 선택/저장 정책, 임의 child 예산 재사용, raw HWND getter만으로 안전 완료 선언 |
| PM | SDK 공통/fresh CLI gate→Meari lease gate→단일 창 수직 순서·source 동결·소유 앱/CPU 슬롯 조율 | 별도 GUI 프로세스를 제품 단일 창 완료로 처리, 사용자 다른 앱/디스플레이 임의 변경 |

예상 제품 파일 접점은 main/assembly/launch와 infrastructure의 video_surface/browser/mpv/window_activity/window_metrics/file picker, presentation의 공용 bootstrap/geometry·기존 factory/bridge다. 실제 수정 파일은 구현 배정 때 소유자와 확정하며 현재 공유 source는 수정하지 않는다. Meari lifecycle 초안의 타입 이름은 아직 호출 가능한 API가 아니다.

## 고정 fixture 없는 선택 bootstrap

1. SDK가 신뢰한 로컬 입력으로 기존 InspectProposal 및 기대 source ID/revision/SHA와 명시 item index를 받는다. route/entry는 허용된 제품 inventory에서 고른다. 새 CLI flag 이름이나 browser JSON의 프로그램 실행 권한은 만들지 않는다.
2. 같은 client/store로 snapshot을 inspect하고 미선택·stale·범위 초과/작업 유형 불일치를 표시한다. 숨겨진 첫 항목 선택·model 재실행·raw 자동 복구는 없다. R3 원본 선택/NeedsRepair 수동 시작은 기존 계약을 따른다.
3. UI의 초기 props projector는 canonical snapshot·명시 선택·번역 표시에서 값만 만든다. 현재 factory의 initial props 대조는 유지하고 제품 bootstrap에 테스트 StateProgram/ProbeScope를 복사하지 않는다. source refs·시간 None·raw/import_error·품질 미연결을 보존한다.
4. 원본 viewing/studio frame·공용 window chrome·선택 패널 inventory와 CSS/tokens/SVG/Pretendard 자산을 고정한다. 실제 mounted scope의 parent registry/context/session을 한 번 조립해 NativeHostRuntime으로 이전한다. 최소 패널 인수는 전체 route 완료가 아니다.
5. SDK fresh CLI에서 같은 저장소·project/provenance를 확인한다. GUI는 OS 입력과 동일 usecase 도달 표본을 확인하며 CAS/저장 정책 시험 전체를 클릭으로 중복하지 않는다.

## 같은 실제 창과 child 의존성

Ready 전에는 Browser/VideoSurface/mpv child를 생성하지 않는다. Meari가 실제 창 owner와 lease를 승인한 뒤 SDK infrastructure가 UI thread에서 generic parent 접근으로 child를 생성·등록한다. 실패 시 생성 자원 또는 미완료 cleanup owner를 보존한다. API 표현·예산·window-only/surface-dependent profile은 Meari 인수 때 확정한다.

video path는 우선 기존 VideoSurface child+mpv 프로세스 --wid/IPC를 이용할 수 있는지 검증한다. mpv의 실제 영상/오디오/제어 관측은 기존 SDK 경로를 사용한다. Browser는 기존 wry build_as_child/navigation/audio/generation 경로를 유지하고 같은 실제 parent로 연결한다. 두 child가 동시에 표시되는 경우 clipping/z-order·focus·overlay 및 다른 영역 가림은 실제 인수 항목이다.

UI layout 영역은 logical 단위/scale/zoom을 명시해 인프라에 전달하고, parent의 실제 geometry_revision/bounds 관측과 대조한다. geometry 변경과 window epoch/surface generation을 혼동하지 않는다. suspend 때 child를 유지할 수 있는지는 profile과 실제 자원 사용을 확인하며 무조건 유지/무조건 재생성을 약속하지 않는다.

## 정상 close ACK의 부분 순서

정상 close는 일반 callback/input·새 child 접근을 먼저 차단한다. 그다음 제품 coordinator가 child와 SDK cleanup의 의존성을 지킨다. 현재 headless bridge의 session→parent 완료는 child-aware barrier가 아니므로 그대로 새 parent 창 제거 조건으로 사용하지 않는다.

| 선행 조건 | 후행 동작·완료 근거 |
|---|---|
| 기존 mpv/control 사용 중단 요청 | 실제 소유 프로세스 terminal·IPC/worker 종료·오류를 보존. Drop 결과 무시를 ACK로 바꾸지 않음 |
| mpv가 video parent를 더 사용하지 않음 | VideoSurface child native 정리·해당 lease/revocation ACK |
| WebView 작업/관찰·child 사용 중단 | 실제 COM/UI-thread 정리·browser child lease ACK. generation과 늦은 메시지 차단 |
| child teardown에 필요한 SDK/registry 자원 유지 | 제품 session/parent registration의 명시 cleanup. child에 필요한 자원을 먼저 해제하지 않음 |
| 모든 실제 child와 host cleanup ACK 확인 | driver view/a11y/surface·실제 parent 창 제거 후 Closed |

독립인 video/browser 단계는 서로의 고정 순서를 라이브러리에 강제하지 않는다. coordinator가 완료 checkpoint/owner/epoch/lease/revocation을 유지하며 retry는 미완료 단계만 실행한다. Pending은 실제 terminal+wake 후 재개하고 busy spin·SDK mutation replay는 없다. forced destroy/Drop은 즉시 권한을 폐기하되 정상 Complete receipt를 만들지 않는다. 이 합성 barrier 구현은 아직 없다.

## 다음 인수와 유지 범위

SDK parent/common context 최종 gate와 fresh CLI/provenance 기능 우선순위를 지킨다. 이후 Meari W01–W13 전체 계약을 축소하지 않고 단계별 state/thin consumer·실제 parent/child·cleanup/forced OS 증거로 인수한다. 최소 단일 창 수직은 명시 선택 원본·실제 backend terminal→wake·OS 입력/IME/a11y·child geometry·사용자 close를 연결한다. 이를 Q01–Q09/모든 hook·라우팅·플러그인/전체13화면 완성으로 확대하지 않는다.

실제 조작은 메인 모니터·소유 앱1개·중앙 슬롯·사용자 눈 확인과 다른 앱 보호를 따른다. DPI/VM 보류는 유지한다. 전체 승인 시안의 header/chrome/각 상태·모든 탭·아이콘/폰트/토큰을 확대1:1 재검토하며 theme 교체나 일부 패널만의 일치로 범위를 줄이지 않는다.
