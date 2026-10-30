---
id: plan:kinekino/20261029113824
title: Proposal UI 제품 연결 인수표
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Proposal UI 제품 연결 인수표

## 품질 UI reachable 재감사

2026-10-20 현재 root의 C21 exact21은 원 pin과 byte 차이0이다. 공통 품질 UI는 세 원본 패널·Native8 등록·factory/controller·bridge로 이미 연결됐으며 원27개 headless 기능 인수와 런처2 공통 frame 인수가 있다. [최신 상태·동시변경 계약과 reachable 표](../edited-output-quality-decision.md)의 현재 root 재감사를 따른다. 기본 Player/eframe 경로와 명시 NativeProposal 경로를 구분한다. 미연결 기능으로 재구현하지 않으며 새 source/Cargo/GUI 실행은0, 사용자 수동 GUI 보류를 유지한다. 기본 제품 전환·native video/전체 route·시각 인수는 별도 계약과 배정 범위다.

## 2026-10-20 C21 수용 뒤 실행 파일 전달 차이

후속 실행 인수: `runtime/quality-review/ui-native-launch-frame-r1/handoff.md`에 원 receipt/실제 바이너리 증거를 보존했다. 입력 pin `9EEEE91536A2934EB0F77BCB112D052C01BD495506069D264240E3AC6301800C` exact2에서 실제 launcher unit2pass/0fail/기존ignored1(80159ms), root default/native all-target strict(2225/39708ms), fmt(1593ms), native feature `--bin player` 실제 dev 빌드(73632ms)가 모두 exit0이다. 수리·재pin0. 보호 snapshot570 delta0, live/snapshot exact2 delta0, 원 실행 handle5 종료·receipt5 terminal0·기록 PID10 소멸을 확인하고 heavy/snapshot을 PM에 명시 반환했다. 실행 파일은 `runtime/quality-review/ui-native-launch-frame-r1/artifact/player.exe`에 보존했으며 SHA256 `94B223D6E580317C65E180A5D04B1CBDAA5AF60421DC5C600800480BC50314BA`, 146540032bytes, feature `native-proposal-window`, dev/debug profile이다. GUI 실행·배포0이며 인수 없는 기본 Player 전환·실제 영상·캡션 호스트 기능·수동 시각 일치 인수는 포함하지 않는다. 아래 source-only 단락은 실행 전 준비 이력이다.

PM 후속 배정으로 명시 NativeProposal 공통 프레임 연결 source-only 구현을 준비했다. 정확 범위는 `src/assembly/meari/product_native_launch.rs`와 기존 하위 `product_native_launch/tests.rs` 두 파일이다. 런처의 비공개 `prepare_launch_window`가 기존 `prepare_product_frame_window`에 제품명·StudioFrame·Wide Inspector 구성을 전달하며 실제 실행과 unit test가 같은 함수를 호출한다. 기존 loop/SDK owner·관측·실패 반환·cleanup은 유지한다. 기존 ignored one-loop authority 회귀를 보존하고 호출만 같은 helper로 연결했다. 신규 시험2는 명시 SDK 문서의 원본 프레임 headless mount/품질 callback과 설정 실패의 원 요청/owner cleanup을 검사하도록 작성했다. source pin은 `runtime/coordination/ui-native-launch-frame-r1/`에 둔다. 파일별 rustfmt 적용만 수행했으며 Cargo 시험·clippy·전체 fmt gate·OS 실행은 중앙 슬롯 배정 전 미실행이다. 아래 표는 이 source-only 변경 전 감사 기준이며 코드 소비/기능 통과를 혼동하지 않는다. 기본 mode·main·Cargo·제품 식별·영상 계약은 변경하지 않았다.

PM 수용 커밋 `b0c19f17a59f54a6469361b05b7fbae3ef349a85` 기준 읽기 감사다. C21의 원본 세 패널·공통 프레임 자동 기능 회귀27개와 strict/fmt 통과는 코드 인수이며, 새 `player.exe` 빌드·배포·OS 실행 또는 시각 인수를 의미하지 않는다. 아래 기존 표의 런처/등록/비동기 연결 설명은 당시 관측 기록이며, 현재 상태는 이 절과 [품질 판정 연결 계획](../edited-output-quality-decision.md)의 최신 인계를 우선한다.

| 정확한 경로 | 현재 소스 관측 | 최소 후속 |
|---|---|---|
| `Cargo.toml` | `native-proposal-window`는 선택 feature이며 default feature가 없다. bin은 `player`/`player-cli`다. | 배정된 빌드에서 feature·source·실행 파일 SHA를 고정한다. feature 활성화만으로 기본 화면이 바뀐다고 표시하지 않는다. |
| `src/domain/product.rs` | `PRODUCT`가 표시 이름과 `player.exe`/`player-cli.exe`를 소유한다. | 새 런처·전달 파일명도 이 책임을 유지한다. UI 구현을 위해 제품 이름을 분산하지 않는다. |
| `src/application/launch.rs`, `src/main.rs` | 인수 없는 실행은 `DesktopMode::Player`다. Native 조립은 명시 NativeProposal 분기이고, 기본 실행은 `presentation::run_with_native_playback`으로 간다. 같은 configured SDK/client와 LocalHost 조립은 이미 존재한다. | 기본 단일창 제품 진입점에서 같은 client/LocalHost/playback 소유권을 전달한다. 기본 실행에 시험용 proposal JSON·fixture 저장소·별도 SDK 생성을 강제하지 않는다. |
| `src/assembly/meari/product_native_launch.rs` | 현재 `prepare_product_window`를 호출한다. 공통 프레임을 받은 자동 제품 회귀의 `prepare_product_frame_window`와 다르다. | 먼저 같은 configured client와 기존 owner/실패 보존/cleanup을 유지하며 공통 프레임 조립을 연결할 정확 변경 범위를 배정한다. 명시 검토 창 전환만으로 기본 제품 완성을 선언하지 않는다. |
| `src/presentation/web_ui_adapter/proposals/product_window/mod.rs` | `prepare_product_frame_window`와 기본 output-review 등록이 존재한다. 해당 프레임 조립은 현재 P1a 기능 검증 경계다. | 기존 FrameComposition·원본 세 패널·공통 품질 컴포넌트를 재사용한다. C21 동작을 새 화면에 복제하지 않는다. |
| `src/presentation/mod.rs` | 기존 제품 경로는 `eframe::run_native`, `Renderer::Glow`다. | GL 사용 자체를 신규 Native UI/영상 결합의 근거로 삼지 않는다. 기존 경로의 교체 시점은 실제 단일창 인수 후 결정한다. |

실제 native video는 [child assembly의 slot 계약](../native-playback-child-assembly.md)에 의존한다. document/source generation·slot identity·layout revision에 결합한 실제 client rect, effective clip/visibility/z-order/focus, invalidation 및 Destroy 관측을 먼저 확보해야 한다. 전체 창 geometry나 borrowed parent lease만으로 video slot을 대신하지 않는다. SDK parent router의 기능 회귀는 실제 OS parent·media decode·재생 시간 진행 검증을 대신하지 않는다.

작업 순서는 공통 프레임 런처 연결 범위 고정 → 기본 단일창의 명시 선택/저장소 및 수명 배선 → 공개 video slot 계약과 SDK parent route 연결 → 중앙 배정 빌드 및 실제 실행 인수다. 현재는 읽기 감사·후속 범위 정리만 수행했다. Cargo·snapshot 변경·GUI·실행 파일 갱신0이며, Meari의 진행 중 heavy 작업과 겹쳐 실행하지 않는다. 수동 GUI/시각 검증 보류와 사용자의 데모 종료 소유권을 유지한다.

2026-10-20 읽기 대조. UI source/consumer 동결·추가 cargo/GUI0이다. SDK가 parent 등록/manifest/context 이관·공통 request context 및 전체 gate를 소유한다. PM의 isolated 선택20pass 인계와 진행 중인 최종 검증을 구별한다. UI는 공유 등록 source를 수정하지 않는다.

현재 완료 근거는 [bridge r2 전체46·strict/fmt](../../../sots/verification/presentation/native-window-bridge-2026-10-19.md), [factory](../../../sots/verification/presentation/product-document-factory-2026-10-19.md), [원본 CardHL](../../../sots/verification/presentation/original-review-sdk-2026-10-19.md), [실제 R3 NeedsRepair](../../../sots/verification/presentation/document-repair-sdk-2026-10-19.md)다. 모두 기능 범위이며 제품 창·fresh CLI·사람 품질·전체 시각 인수의 완료가 아니다.

## 기능 gate: SDK/CLI에서 확인

| ID | 실제 인수·근거 | GUI에 중복시키지 않을 범위 |
|---|---|---|
| F01 명시 선택/수입 | 명시 proposal/source ID·revision/SHA·항목, package/item/candidate/companion의 immutable asset SHA를 기존 inspect/read/source_import로 대조. 원 raw/NeedsRepair/import_error와 working draft 분리 | GUI 클릭으로 수입/저장 정책을 다시 구현하지 않음. UI는 선택 표시·오류 전달만 연결 |
| F02 mutation/오류 | 같은 SDK의 expected revision/CAS·typed edit/accept·receipt draft/output revision·automaticfalse·stale/refresh/NotFound/corrupt/ref mismatch. parent registry/actual SDK counter·cleanup/error 보존 | 기존46 기능 회귀를 창에서 전부 재실행하지 않음. GUI는 실제 입력이 같은 usecase에 도달하는 표본과 오류 상태 표현 확인 |
| F03 fresh CLI/프로젝트 | 고정된 실제 제품 실행·명시 저장 root/같은 client에서 저장한 상태를 별도 PID의 player-cli로 inspect. project Live/Saved capture·working draft·정본 source binding과 새 SDK/store reopen을 구별 | GUI 조작 자동화로 fresh CLI를 대체하지 않음. same-process test를 fresh process 성공으로 집계하지 않음 |
| F04 provenance/decision | 동일 원 candidate/item decision을 재조회하고 수정 output receipt와 별도 대상임을 확인. package/item/candidate/companion/decision 정본 결합이 없는 현재 gap은 별도 추적 | acceptance를 quality verdict로 바꾸거나 원 후보의 verdict/decision SHA를 수정 output에 상속하지 않음 |
| F05 의미/저장 | evidence_verified=저장 증거 무결성, review_ready=baseline 존재. ASR 원입력을 human gold로 표시하지 않음. ModelCondition을 라이선스 승인/실측 정밀도로 오인하지 않음 | UI 문구/상태는 이 의미를 공유. 신규 GUI 전용 SDK/domain 정책 없음 |

프로토콜/프로젝트 ID는 표시 제품명과 분리한다. 제품 표시 이름·출력 파일명은 단일 제품 식별 설정을 따른다. SDK 오류/receipt는 canonical 타입으로 유지한다. 기능 gate의 정확 최종 source/해시·실제 PID·저장소/endpoint·원 receipt를 SDK가 인계한 뒤 root 제품 통합 기준으로 고정한다.

## GUI gate: 실제 OS와 시각 표현만 확인

| ID | GUI에서만 필요한 인수 | 성공 한계 |
|---|---|---|
| G01 실제 terminal wake | 실제 parent backend가 registry terminal을 반영→해당 driver.wake_handle().wake→OS loop Completion→suffix 정확1회 admission. 포화 중 input blocked·무한 spin 없음, 중복 wake/close 뒤 stale 요청과 SDK counter·동기 receipt를 연결 | headless 명시 Completion과 실제 OS wake를 구별. SDK mutation 결과 판정 자체는 F02 재사용 |
| G02 실제 입력/접근성 | OS pointer·keyboard·한국어/일본어 IME·focus/tab·접근성 action이 현재 node/scope를 통과해 독립 field와 공용 controller에 도달. disabled·stale·선택 전환의 표시·포커스 확인 | synthetic document input/Tree snapshot을 OS 키보드·스크린리더 성공으로 대신하지 않음 |
| G03 frame/오류/종료 | resize/minimize/maximize·복원, 사용자 close→callback receipt/session/parent 정리→driver 자원 제거, parent 실패/retry 동안 문서 유지. checked last_submission·presentation_errors와 실제 화면을 대조 | last_submission은 compositor ACK가 아님. Drop/forced incomplete는 정상 CloseReceipt가 아님. 사용자가 데모 player 종료 |
| G04 단일 창 미디어 | 실제 제품 root의 video/WebView·자막/overlay/PiP·조작부와 Native UI의 부모창·focus·수명 연결. 원래 client/LocalHost/playback binder를 공유. 커서 enter/move/controls 복귀·mute/volume 관측은 기존 후속과 연계 | 다른 GUI 프로세스/창을 띄워 단일 제품 통합이 된 것으로 표시하지 않음. 실제 청취·seek stall 원인 미검증을 유지 |
| G05 디자인 전체 | `.spec/artifacts/design/`의 승인 버전·화면 ID 기준 전체13화면군/모든 하위탭과 empty/demo/loading/error/selected 상태를 확대1:1 비교. 공통 frame·header 여백·min/max/close·SVG·Pretendard/다국어 fallback·색/spacing/border/radius 토큰을 실제 실행에서 확인 | 기능46 통과나 한 패널만의 일치로 전체 승인하지 않음. 승인 공통 컴포넌트 규격과 생성 이미지 장식/오류를 구별 |

GUI는 최신 사용자 지침에 따라 메인 모니터에서 사용자가 눈으로 확인할 수 있도록 소유 앱1개·중앙 슬롯 배정으로 진행한다. 사용자의 다른 앱과 PiP를 보호하고 보류된 DPI/VM 범위는 유지한다. 이 계획 수정으로 디스플레이 설정을 바꾸거나 앱을 실행·종료하지 않는다. screenshot/crop·원 시안·실행 바이너리/source/asset/font pin과 실제 실패를 연결한다. 공통 컴포넌트·토큰·asset의 전 화면 일관성을 유지하며 기능시험을 GUI 클릭으로 중복하지 않는다.

## 현재 조립과 미연결 지점

2026-10-20 후속: [bootstrap consumer 기능 gate](../../../sots/verification/presentation/bootstrap-preparation-2026-10-20.md)에서 명시 선택 순수 props와 원본 Card/HL factory 소비를 61개·strict/fmt로 검증했다. 제품 root 등록·실제 창/wake·시각 인수는 추가되지 않았다.

최신 document 3/historical binding 1과 bootstrap의 읽기 호환성, 현재 4개 GUI capability와 공통 import의 차이, 명시 launch 설정·Unsupported·원본 품질 표시의 잔여는 [historical import UI 연결 감사](historical-import-ui-connection.md)에 분리했다. 읽기 감사이며 현재 실행 입력이나 지원 capability를 확장하지 않는다.

| 지점 | 읽은 사실·다음 책임 |
|---|---|
| SDK 부모 등록 | 현재 lib.rs는 assembly를 등록하고 assembly/meari는 proposal_manifest/proposal_registration/proposals/request_context를 공개한다. root Cargo에 meari core/player-ui-plugins가 있다. SDK gate 진행 중이므로 최종 인수는 owner 결과를 기다림 |
| 실제 런처 | main은 동일 client에 선택 Proposal/Project/Review store와 playback port를 조립하고 LocalHost와 run_with_native_playback에 공유한다. DesignPreview는 SDK 조립 전 반환·저장소 옵션 금지. 이 분기에 편집을 몰래 연결하지 않음 |
| 기존 GUI | presentation/mod.rs는 여전히 eframe::run_native·Renderer::Glow 경로이며 web_ui_adapter/factory/bridge 제품 부모 등록과 Native runtime 런처 연결은 보이지 않음. root/launch 변경은 SDK와 조율 |
| factory/선택 bootstrap | 검증한 factory→실제 scope/registry/session→bridge를 연결. 테스트 Lease/항목0/fixture package를 제품 기본값으로 가져오지 않음. 초기 props는 canonical DTO/명시 선택의 순수 투영이어야 하며 제품용 bootstrap projector와 선택 목록 배선은 추가 준비 필요 |
| 공개 Native driver | attach(&blitz_shell::EventLoop,runtime,options)는 기존 pinned loop 위임용이고 내부 add_window+VelloCpuWindowRenderer로 새 창을 만든다. show_with_runtime은 독립 loop convenience. eframe 기존 창/GL import를 제공하는 것으로 해석하지 않음 |
| 실제 wake | NativeWindowWake는 host ID/atomic wake bits만 전달한다. 제품 backend terminal 소유자가 clone을 보유해 wake하고, Rc document/registry/GL은 worker로 보내지 않는다. 제품 backend→wake→Completion 배선은 아직 없음 |
| provenance 실행 지연 | SDK 담당 관측상 pinned Rust subprocess reader는 매 store 검증에 약 450ms 이상 걸릴 수 있다. 현재 Immediate plugin을 UI thread에서 실행하면 창을 막을 수 있다. CLI 기능 인수 후 공통 SDK scheduling port와 비동기 실행·typed Pending→실제 terminal wake를 조율한다. 기존 Pending 방어 회귀는 실제 provider 비동기 완료 근거가 아니다. bootstrap은 순수 투영으로 유지하며 worker/store 정책을 구현하지 않는다. Rc document/registry/GL은 worker로 이동하지 않는다 |
| 영상/WebView 부모 | driver 공개 메서드는 attach/wake_handle/last_submission/presentation_errors이며 읽은 source에서 제품 인프라가 사용할 parent window handle·ready/suspend/destroy epoch port가 보이지 않음. SDK video/browser adapter를 같은 창에 연결하려면 Meari와 구체 계약 확정 필요 |

SDK concrete parent→ParentProposalLease 위임과 manifest/context 단일 소스는 SDK 소유다. UI projection/bindings/factory/bridge는 UI 소유, Native driver/표면·event loop API는 Meari 소유다. 공통 client·단일 registry·실제 source generation·opaque callback handle을 유지한다. [factory 계약](../../../sots/player/presentation/shared/proposal-document-factory.md), [host 계약](../../../sots/player/presentation/shared/proposal-window-host-port.md)을 따른다.

## Meari에 확인할 공개 계약

단일 창 A=Meari 최상위+native-child lease/B=eframe import의 pinned 버전·미디어 parent·UI 이관 비교와 우선 검증 권고는 [제품 ADR 입력](../../../adrs/player/presentation/native-top-level-window.md)에 분리했다. Proposed이며 A의 채택/구현 완료나 libmpv·GL 공유 채택을 뜻하지 않는다.

PM은 A를 우선 검증 경로로 배정했다. 최종 제품 backend/런처 전환은 미확정이며 [최소 수직 준비](native-window-minimal-vertical.md)의 책임별 변경 대상·명시 선택 bootstrap·child/host close ACK 의존성만 현재 정리한다. SDK 공통 검증/fresh CLI 우선과 source/build 동결을 유지한다.

Meari 담당의 읽기 검토로 현재 실제 parent window handle/lifecycle port가 없음을 확인했다. attach의 새 CPU 창과 eframe 기존 창/GL import는 별개다. owner·epoch/generation·등록 lease·revocation/cleanup ACK 및 정상/forced close·실패 retry 계약은 Meari 문서 초안 인수 후 조율한다. raw HWND DTO나 일시 borrowed closure만으로 장기 영상/WebView child 수명이 보장된다고 가정하지 않는다. 현재 미지원이며 source/build는 동결 상태다.

필요한 것은 새 SDK mutation이 아니다. (1) caller의 기존 eframe loop/창에 위임할 실제 supported 경로 또는 주 loop를 Native로 전환할 때의 소유 경계, (2) 같은 실제 window의 scoped native handle/lease와 ready/resize/suspend/destroy epoch를 제품 infrastructure에 제공하는 port, (3) CPU window/GL renderer 선택과 외부 context/FBO import·상태 복원/소멸의 지원 범위다. 이름/시그니처를 발명하지 않고 owner가 타입/얇은 소비자로 확정한다. 별도 standalone 창은 진단 spike이며 제품 단일 창 완료가 아니다.

순서는 SDK parent 최종 인수→fresh CLI/provenance 기능 gate와 제품 선택 bootstrap→실제 window/wake 수직→전체 디자인 확대 비교다. 정확 슬롯은 PM이 배정한다. 이전 누적 조립 기록은 [과거 인계](history/proposal-product-registration-before-acceptance-split-2026-10-20.md)에 보존했다.
