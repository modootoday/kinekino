---
id: plan:kinekino/20261029113854
title: Viewing chooser 표시·명시 Open·재열기 최소 연결 제안
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Viewing chooser 표시·명시 Open·재열기 최소 연결 제안

2026-10-24, root314d468e7a43b3ef3b70126cf3f0ed9ec11d5d19/Mearib532/lock8F 기준 읽기·설계만 수행. 구현·새시험작성·Cargo/S/T/GUI·stage·commit0. SDK Open partial3와 현재 SDK 소유 host2는 미인수 상태이며 이 문서는 실행 권한이 아니다.

## 관측 사실

- viewer_native_launch.rs::prepare_viewer_document_profile은 기존 read-panel을 준비하고 선택적 Live/Legacy/Captured를 pre-export 설정한다. choicesNone은 chooser를 구성하지 않는다. 제품 client/context/ExecutionOwner는 이미 원 조립 경로로 들어오므로 새 worker/store/endpoint를 만들 필요가 없다.
- viewer_callbacks/lifecycle.rs에는 configure_proposal_chooser, chooser_phase/documents/items/prepared/page/document/failure/terminal, queue_chooser/drain_chooser가 있다. chooser.rs는 ACK 이후 PreparedSelection을 제공하며 scope/generation/Arc identity를 소유한다. 원 READS3와 공유된 pump/terminal retaining consumer를 사용한다.
- read-panel.tsx는 player/기존 captured 선택/Jobs/READS3만 표시한다. ViewerHost::admit은 queue_card_selection→queue_local→dispatch이며 chooser 입력 조립은 없다. after_dispatch는 drain_local/consume_reads 후 projection.dirty를 기준으로 reproject하므로 chooser-only 상태 변화는 별도 얇은 표시 변경 감지가 필요하다. local_props에는 chooser 표시 데이터가 없다.
- return_viewing.rs는 원 scope release/ACK→resume_released_route→Viewing commit/retained proof/관측을 수행한다. resume은 export를 다시 호출하고 chooser.bind는 generation진행·선택폐기·Idle로 돌아간다. 따라서 이전 page/item/Open key 재활성화가 필요하지 않다. 관측 전 active_task/live_opening/return owner를 무시하면 안 된다.
- SDK private partial의 register_prepared_open(scope)는 원 host 보유 PreparedSelection을 등록해 opaque String key를 반환하는 제안 구현이다. UI는 ID/ref/index/SHA나 새 Arc로 selection을 만들지 않는다. SDK host2가 route 이전 재검사와 ticket→LiveOpening→소비 경계를 소유한다.

## 화면과 데이터 흐름

기존 ViewingReadRoot 안에 소형 ProposalChooser 공통 표시 컴포넌트를 넣는다. 같은 Button/RadioGroup/RadioGroupItem·기존 토큰/글꼴을 사용하며 문서 목록과 해당 문서의 item 선택을 구별한다. 첫 항목 자동선택은 하지 않는다. 표시 props는 phase/status/error, opaque document/item key와 표시문구, selected key, 다음페이지 가능 여부, Open key 및 disabled 상태뿐이다. raw scope/ref/SDK envelope/콜백 handle은 TSX로 내보내지 않는다. localized label key와 표시문구를 Rust 표시 어댑터에 모아 추후 i18n 경계를 유지한다. 이번 연결안에 새로운 색상·SVG·토큰은 필요하지 않다.

Idle의 명시 조회(onChooserList)→PageReady 문서 선택(onChooserDocument Text)→원 Inspect terminal/ACK→DocumentReady item 선택(onChooserItem Text)→Prepared의 host register_prepared_open→명시 Open(onOpenLiveProposal Text) 순서다. Next는 기존 onChooserNext Action과 원 cursor를 사용한다. SDK busy/ACK실패/terminal보존과 UI disabled를 동기화하되 UI에서 지원/권한을 판단하지 않는다. 확인 실패는 기존 typed 오류를 표시하며 마지막 정상 결과를 새 선택으로 위장하지 않는다.

원 zero-callback barrier에서만 drain_chooser를 호출한다. chooser-only props의 실제 값 변화와 기존 projection.dirty를 합쳐 한 번 reproject하며 scope/generation 전환 뒤 이전 props로 덮어쓰지 않는다. 원 consume_reads/ACK retry와 공유 pump를 유지하고 새 poller/재시도 타이머/SDK owner를 만들지 않는다. Open pending/Jobs/active route에는 SDK 원 배타 guard를 따른다.

chooser callback 4개 이름은 기존 host의 입력 분기 선택에만 사용하고 실제 권한은 queue_chooser의 dispatch_local/local key/export lease/scope 검사로 판정한다. Invalid를 다른 SDK/READS3 fallback으로 넘기지 않는다. 현재 queue_chooser는 타 로컬 키에 Invalid를 낼 수 있으므로 모든 입력에 무조건 먼저 호출하지 않는다. Open은 SDK의 기존 admit_live_entry로 그대로 전달한다.

복귀는 검증된 source navigation과 원 request_viewing_return을 그대로 사용한다. export직후 Idle/선택없음/Open비활성 표시를 원 Viewing props에 반영하고, 실제 view.start 관측·원 owner정리가 끝난 뒤 사용자가 새 조회를 누를 수 있다. 새 조회→새 문서·item 명시선택→새 Prepared/Open key→기존 R1 current inspect로 재열기한다. 자동 조회·자동 선택·오래된 carrier replay는 제안하지 않는다. Save/Edit/Accept는 기존 proposal window/SDK ports를 소비하며 정책·CAS·저장 기능을 재구현하지 않는다. Save와 Accept의 결과 표시/재열기 정합 검증은 별도 후속 인수다.

## exact 경로 제안 — production8 + 시험3 = 총11

| 경로 | 최소 책임/소유권 |
|---|---|
| src/presentation/web_ui/viewing/read-panel.tsx | 기존 root에 chooser 표시 props/컴포넌트 연결 |
| src/presentation/web_ui/viewing/proposal-chooser.tsx (신규) | 공통 Button/RadioGroup으로 목록·선택·상태·명시 Open 선언 |
| src/presentation/web_ui_adapter/player_shell/document.rs | 고정 source inventory와 초기 Idle/빈선택 props 연결 |
| src/presentation/web_ui_adapter/player_shell/proposal_chooser.rs (신규) | SDK 독립 순수 표시 DTO→Value/CallbackReference, 공통 label/disabled 투영; document.rs에서 명시 module 선언 |
| src/assembly/meari/viewer_host/viewing_chooser.rs (신규) | 기존 callbacks getters/queue/drain/register_prepared_open 소비, 표시 스냅샷·zero-batch 연결과 원 observed Viewing guard |
| src/assembly/meari/viewer_host/mod.rs | 신규 얇은 모듈 연결·입력 분기·기존 props commit 흐름에 chooser 변경 통합. 직접 ViewerHost field 추가는 피하고 실제 필요시 별도 제안 |
| src/assembly/meari/viewer_native_launch.rs | 기본 product-frame의 choicesNone에 원 pre-export configure_proposal_chooser(ListProposals::default()) 조립; 기존 explicit 선택 프로필과 배타 유지 |
| src/assembly/meari/viewer_host/return_viewing.rs | 원 resume/export의 새 Idle chooser props를 기존 Viewing commit에 함께 반영; route/receipt/owner 정책 불변 |
| src/presentation/web_ui_adapter/player_shell/proposal_chooser/tests.rs (신규) | 빈·목록·실패·Prepared 표시 및 opaque key/callback/disabled/escaping 투영 시험 |
| src/assembly/meari/viewer_host/viewing_chooser/tests.rs (신규) | 실제 원 SDK/export로 list→document→item→Open/실패·재export·복귀 fresh query 시험 |
| src/presentation/web_ui_adapter/player_shell/preparation_tests.rs | 실제 고정 source graph/초기 root/기존 callback-free 준비 경계 회귀 |

새 host상태 저장이 불필요한지 구현 전 검토한다. 변경감지는 기존document props와 값 비교 또는 짧은 조회 스냅샷으로 처리하고 임의 거대 state/cache를 만들지 않는다. 추가필드/새파일/기존 caller 변경이 필요하면 이11 밖으로 조용히 확대하지 않는다.

기본 '제품 실행' 전체를 Native로 바꾸는 제안은 아니다. application/launch.rs·main.rs·product_viewer_launch/project.rs의 기본 분기·설정·선택 구성은 별도 application 소유 결정이다. 여기서는 기존 product-frame Native 경로의 기본 Viewing 표시/입력 연결만 대상으로 하며 여전히 전체 사용자 기본 실행 reachability는 별도 통합 인수가 필요하다.

## SDK 소유권과 순서

| SDK 소유 경로 | UI 제안과 교집합 | 조건 |
|---|---|---|
| viewer_callbacks/selection.rs, lifecycle.rs, chooser/tests.rs partial3 | 없음 | 현재 frozen source를 수정하지 않고 register_prepared_open 및 원 chooser 포트를 소비. API 부족시 SDK에 exact 제안 |
| viewer_host/live_entry.rs, live_entry/tests.rs host2 | 없음 | SDK최신314preimage 구현·인수/채택 후 UI를 최신root로 다시동결. 같은 LiveEntryRequest/PreparedSelection sidecar·원 ticket소비 경계를 재구현하지 않음 |
| UI host/mod.rs·return_viewing.rs | 직접 경로교집합0, 의미상 Open/return 연결 의존 | SDK host2 진행 중 UI는 source읽기/제안만. 후속 UI필요변경이host2를건드리면별도직렬배정 |

## 후속 검증 계획 — 전부 미작성·미실행

먼저 SDK host2/공통CLI 지원을 인수하고 UI exact preimage를 재고정한다. fmt/default·native strict/lib no-run JSON/list로 새등록을 대조한 뒤 표시 순수시험과 headless 실제SDK 연결을 먼저 검증한다. 최소 headless는 initial빈선택/Open불가, 명시list/page/document/item ACK전불가, authenticOpen·동일원Arc, foreign/old export key 거부, pending 보존·동일route 중복0, 복귀후oldkey거부·freshquery새명시선택·R1inspect, typed오류표시/원closecleanup이다. SDK/CAS/SaveAccept 기능 suite를 GUI에서 재검사하지 않는다.

그 뒤 별도 PM 자원/OS 배정에서 실제 고정 product source의 명시 callback 흐름과 retained entry/return/reopen을 관측한다. 테스트 전용source치환/TrustedLiveChoices만으로 새기본chooser도달성공을주장하지 않는다. OS포인터/키/IME·접근성·수동이미지1:1시각인수는별도다. 현재 검증된 R344는 이 새chooser 흐름의 시험 결과가 아니다.

적용 스킬은 앞선 동일UI작업에서 탐색·본문을 읽은 modootoday-free/design-delivery-plus/frontend-first-journey(공통 컴포넌트·typed SDK 경계·연결/시각인수 구별)와 modootoday/software-qa/test-fixture-and-mock-build(실제원fixture·실행/모델·기존원owner 보존)이다. 신규 정책·타언어 검증도구를 도입하지 않는 좁은 연결 설계에 적용했다. Meari product-review-edit-accept-reopen-seams-2026-10-24.md와 기존 chooser-open-handoff-contract의 원문 추적을 참고했다.
