---
id: plan:kinekino/20261029113908
title: 제품 document host·공통 SDK 등록 checkpoint
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제품 document host·공통 SDK 등록 checkpoint

[UI 조립 인계](proposal-product-registration.md)를 실제 root와 대조했다. root main은 기존 PlayerSdk/명시 ProposalFiles·ProjectFiles·ReviewFiles를 조립한 ApplicationClientHandle 하나를 LocalHost와 GUI에 공유한다. DesignPreview는 SDK 생성 전 반환한다. 이 경계를 보존하고 UI가 두 번째 SDK/store를 만들지 않게 한다.

## 등록 전 감사와 현재 연결

2026-10-20 [parent 등록 실행](../../../sots/verification/application/sdk-product-parent-2026-10-20.md)에서 lib→assembly/meari와 proposal adapter/manifest/context/lease를 root에 등록하고 crate import를 정리했다. 도구는 제품 helper를 재수출하며 중복 manifest/context 본문과 proposal adapter의 과거 path 재컴파일을 제거했다. parent 집중6개, root 전체385pass/19ignored, 독립 target consumer 전체47pass 및 양쪽 strict/fmt가 통과했고 최종 pin/metadata/소유 프로세스 빈 배열까지 인수했다. shared artifact 충돌과 모든 첫 실패는 실행 정본에 보존했다. presentation web_ui_adapter 부모·제품 창/driver 연결은 여전히 미등록이다. tools를 제품 의존성으로 추가하거나 정책을 복제하지 않는다.

현재 presentation::run_with_native_playback는 eframe Glow 창을 만든다. meari-native의 공개 NativeHostRuntime/NativeWindow 경계는 이후 Meari/UI gate에서 검증됐고 [제품 bridge r2 정본](../../../sots/verification/presentation/native-window-bridge-2026-10-19.md)은 실제 SDK/store를 사용한 headless callback/재투영/close와 잔여 OS driver wake를 구분한다. 공개 port 미존재라는 과거 상태를 현재 제품 연결 미완료와 혼동하지 않는다. 기존 eframe 창/공유 GL에 문서를 바로 삽입할 수 있다는 근거는 아직 없으며 실제 제품 창·OS wake 등록은 후속이다.

## 소유와 공개 port 요청

proposal_registration.pending.rs의 초안을 제품 ProposalSdkRegistration으로 등록했다. actual document scope와 같은 registry/context를 받아 기존 SDK adapter를 등록하며 SDK/client/store를 새로 만들지 않는다. context/registry는 try_borrow로 오류를 반환하고, explicit close 실패 때 handle·registry를 보존해 보고/재시도한다. native callbacks→UI session close 뒤 parent SDK unregister 순서를 전제로 한다. Drop은 best effort이며 완료 영수증을 대체하지 않는다. 실제 제품 window hook 연결은 후속이고 headless factory 소비는 이번 consumer gate에서 검사한다.

lease의 두 준비 검사는 다른 scope 거부 후 정상 등록 가능성과 registry borrow 중 close Busy→handle 보존→borrow 해제 뒤 명시 재시도/재호출·기존 handle Stale를 다룬다. context 옵션/borrow·원 문맥 검증과 manifest 역할 pin 검사를 포함한 root focused6개가 실제 통과했다. unsupported fixture client는 이 소유권/문맥 검사에만 쓰며 제품 기본 SDK/저장소로 제공하지 않는다. consumer의 새 검사는 caller-owned 실제 3store SDK를 사용하며 전체47pass에 포함돼 인수했다. held-registry 실패가 UI session 단계라는 한계는 실행 정본대로 유지한다.

runtime/coordination/product-host/proposal_manifest.pending.rs와 request_context.pending.rs는 준비 시점의 후보로 보존했다. 제품의 src/assembly/meari 책임 파일에 연결하고 include_bytes 경로를 실제 위치에서 type 검사했다. Native target4 capability·hash 구성·명시 context 보존/ticket correlation을 유지하며 tools의 함수 본문을 제거하고 제품 함수를 재사용한다. ABI/provider/schema pin과 기존 consumer 회귀의 최종 인수는 현재 실행 정본에서 구분한다.

| 담당 | 책임/다음 계약 |
| --- | --- |
| SDK | 기존 client/3store/LocalHost 유지, launch 명시 모드·선택 검증, root Cargo/mod 등록, 공통 manifest/ticket correlation 이관과 도구 재사용, SDK/CLI/project/provenance 연결 |
| UI | fixture 상수 없는 명시 proposal/source 선택으로 ModuleProgram/inventory와 동일 registry/context/scope를 묶는 document factory; session/bindings·reproject·close의 소유/호출순서 |
| Meari | 실제 window/event-loop 또는 embedding 공개 경계; OS dispatch/registry borrow 밖 callback drain·재투영/redraw·source 교체·close/오류·접근성 지원 범위 |
| infrastructure/assembly | window/WebView/mpv/LocalHost/document/plugin의 실제 소유와 종료, 동일 SDK facade 주입. 플랫폼 호출을 domain/presentation 정책으로 옮기지 않음 |

Meari/UI에 위 port 경계를 요청했다. 이는 현재 Repeat·원본 소비 gate를 바꾸거나 새 heavy 실행을 요청한 것이 아니다. 공개 API의 존재/계약을 확인하기 전 기능을 가정한 제품 코드를 등록하지 않는다.

UI가 [제품 document factory 계약](../../../sots/player/presentation/shared/proposal-document-factory.md)을 인계했다. 동일 actual scope 안의 Rc<RefCell<PluginRegistry>>/context slot과 session이 UI thread에 속하므로 라이브러리 host hook에 근거 없는 Send/Sync·별도 UI worker 이동을 요구하지 않는다. document와 callback/session owner의 소유 분리는 공개 hook 계약과 함께 확정한다. Meari는 자체 Native winit host를 generic 공개 port로 제품에 조립하는 경로부터 구체화하며, eframe embedding/GL context 공유는 미검증이다. Q07/Q08 host 통합은 Repeat 이후 잔여이고 현재 headless gate로 대체하지 않는다.

## 다음 검증

2026-10-20 오디오 checkpoint 반환 뒤 parent lease·공통 manifest/context의 제품 root 등록과 기존 도구의 단일 소스 재사용을 인수하고 root 입력 동결/single-heavy 슬롯을 중앙에 반환했다. fresh CLI/proposal/project/provenance 저장 결합은 그 다음 별도 checkpoint로 나누며, 실제 NativeWindow/OS wake·시각 검토를 앞당겨 완료로 표시하지 않는다. 현재 검사 상태와 첫 오류는 위 실행 정본을 따른다.

최소 공유 입력은 root Cargo/lock의 meari core·player-ui-plugins path 의존과 필요한 nested workspace 제외, src/lib.rs→assembly/meari 부모, proposals adapter·proposal_manifest·request_context·proposal_registration, tools/player-ui-sdk-bridge의 helper 재수출/호출부다. meari-native/renderer feature·paint patch·제품 창을 이 첫 등록에 추가하지 않는다. tools를 제품 의존성으로 추가하지 않으며 tools의 기존 proposal manifest/context 본문을 제품 구현 재사용으로 바꾼다. include_bytes contract/provider/schema 경로와 provider hash를 실제 등록 경로에서 다시 검증하고 독립 도구 lock은 복사하지 않는다.

기존 같은 ApplicationClientHandle/ProposalFiles/ProjectFiles/ReviewFiles와 actual Rc registry/context/scope를 lease에 주입한다. headless factory의 ParentProposalLease 경계에는 제품 lease를 위임하는 얇은 consumer 연결만 둔다. 두 번째 SDK/store나 기본 fixture 선택은 만들지 않는다. root assembly의 crate import와 도구의 과거 path import를 함께 정리해 정책 본문을 두 번 소유하지 않는다.

이 추출만으로 저장 포맷·안정 ID·설정 이름을 변경할 계획은 없다. 직렬화 필드/기존 DTO/store diff가 없어 이번 추출은 migration이 불필요하며, provider/content hash가 달라지면 새 pin으로 기록한다. 후속 provenance 원자 결합이 저장 규격을 바꾸는 경우 별도 버전·fresh CLI 재열기·tamper 검사와 migration 판단을 준비한다. sidecar나 메모리 DTO만으로 영속 결합 성공을 대신하지 않는다.

이전 Repeat·원본 UI·SDK 회차 전환 인수 이후 제품 등록/CLI/provenance 잔여를 통합 큐에서 대조한다. 현재는 core/plugin path 의존만 root lock으로 resolve하며 독립 도구 lock을 복사하지 않는다. Native renderer feature/paint patch는 후속 창 checkpoint에서 따로 검증한다. 등록 뒤 SDK/CLI 회귀와 실제 창의 원본 문서 입력→callback→SDK 저장/명시 채택→재투영→fresh CLI/project 재열기→source 전환/창 종료 정리를 단계별로 검사한다. headless 성공을 player.exe 완료로 표시하지 않는다. 전체13화면 시각 비교와 사람 품질 승인은 별도다.
