---
id: plan:kinekino/20261029113811
title: 실제 source navigation → Viewing 복귀 R3 최소 제안
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 source navigation → Viewing 복귀 R3 최소 제안

2026-10-24, UI source-only. PM 채택 root `ee595cf20fef754ba1dc7a22429713824cd8c49e`/Meari b532를 읽었다. SDK chooser가 sole-heavy이며 이 문서는 구현·시험 작성·Cargo·S/T·GUI 실행 배정이 아니다. 기존 fixture F948의 typed host return과 실제 source return, 기본 chooser 도달성을 구별한다.

## 읽은 계약과 확정 사항

Meari [seam162A](../../../../packages/meari/.spec/evidence/reviews/chooser-open-navigation-return-seams-2026-10-24.md) 본문과 후속 localbinding 확인까지 읽었다. SHA `162A2A82ADCDD5DC95ECD62EDEC5F73F51639878B6A14EA38F92C6F6AE48FFDB`. 제품 frame_composition.rs, mount_proposal.rs, window_bridge/scoped.rs와 admit_callback, document_session/local_composition.rs와stage/references, 원 return_viewing/request guard, 기존 local scoped 시험을 대조했다. 새 Meari/player-ui-plugins/SDK API가 필요하지 않다.

현재 FrameProjection.root_props.navigation은 Empty다. 이미 studio/navigation.tsx는 StudioNavigationItem→공용 Button을 사용해 Text("VIEW")를 전달한다. navigation-item은 props.workspaceId를 명시 캡처하므로 기존 parser 계약과 맞는다. Jobs가 같은 navigation/source를 closed graph로 합성하고 있다. StudioNavigation·NavigationItem·Button·기존 토큰/아이콘을 재사용하고 return 버튼/스타일을 별도로 복제하지 않는다.

등록선은 기존 ScopedPanelInput.local_signatures → 원 session.bind_local/local_keys → session.references/export → 같은 document props+export atomic commit이다. 원 renderer node/disabled 및 owner/scope/lease/signature 검증 후 session.dispatch_local_after_renderer_guard가 LocalActionInvocation을 만든다. 원 PanelLocalIntentPort에서 현재 session.local_action_key("onNavigate")와 matches_key를 검사한다. Invalid는 SDK dispatch로 fallback하지 않는다.

LocalActionInvocation은 즉시 소비 대상이며 durable grant가 아니다(local.rs27). port에서 인증 후 즉시 typed ReturnToViewing intent로 소비하고 원 invocation을 뒤에 재dispatch하지 않는다. 보류 metadata는 승인된 intent의 provenance/재검증용이며 새 권한이 아니다. 실제 route/dirty/pending/recovery/ACK 정책은 원 ViewerHost::request_viewing_return 및 active.navigation_guards에 남긴다.

## 최소 production exact5 후보

| 경로 | 책임과 범위 |
| --- | --- |
| `src/presentation/web_ui_adapter/proposals/frame_composition.rs` | 기존 navigation/item source를 closed graph에 합성하고 공통 navigation reference/props 조립 helper를 제공. 원 graph/inventory content 충돌 검사는 유지. 기본 root_props는 Empty 계약을 유지하고 Live 조립에서만 실제 NodeSlot 주입. 기존 ProductFrameBlueprint/Jobs가 같은 source 정의를 공유하도록 일치 검증. |
| `src/assembly/meari/viewer_host/source_navigation.rs` (new) | 단일 책임의 bounded1 presentation intent mailbox와 원 PanelLocalIntentPort 합성. 현재 key/owner/Text VIEW 인증, 동일 intent coalesce, 다른 intent Busy, 미지원 workspace 명시 거절. 같은 namespace/key의 Card selection은 기존 port로 위임하고 기존 dispatch를 복제하지 않음. |
| `src/assembly/meari/viewer_host/mod.rs` | mailbox 조립/정리와 원 actual turn의 zero-batch 소비. 같은 active task/document/owner/definition/source generation/current projection metadata 확인 뒤 기존 request_viewing_return 호출. 성공 ticket을 원 ViewingReturn에 보관한 뒤에만 intent 소비. 기존 advance_viewing_return/one-commit/first-error/cleanup 순서 유지. |
| `src/assembly/meari/viewer_host/mount_proposal.rs` | 원 local_signatures에 onNavigate Text를 한 번 추가; 원 session export의 CallbackReference를 기존 navigation NodeSlot props로 조립. 기존 commit closure를 통해 초기와 후속 projection 모두 같은 navigation 유지. 원 selection port와 return port 합성. 원 entry observation gate 전에는 return admission 불가. |
| `src/assembly/meari/viewer_host/live_entry.rs` | 기존 ProductProposalMountInput에 host-owned mailbox만 연결. SDK inspect/ACK/release/LiveEntryRequest/chooser PreparedSelection의 정책·권한·타입 변경0. 이전 mount input literal 시험의 None 설정도 아래 영향 시험 경로에 포함. |

`source_navigation.rs`는 제품 application/domain 정책이나 SDK가 아니라 표시 계층의 queue/adapter다. 원 프레임·카드·local session을 재사용한다. `return_viewing.rs`, scoped.rs/document_session, Meari와 독자 router의 구현 변경은 현재 최소안에서 제외한다. 기존 공개 계약만으로 충족되지 않는 정의가 실제 구현 때 발견되면 정확 추가 경로를 별도 제안한다.

mailbox는 host가 소유하고 mount input에는 같은 Rc만 전달한다. active owner는 renderer가 지정하지 않는다. port는 현재 session key, 실제 document metadata와 trusted task/entry gate를 묶어 typed intent를 저장한다. key와 VIEW 문자열만 저장해 나중에 grant로 해석하지 않는다. suffix가 남으면 소비/원 session 교체를 하지 않고, zero-batch 소비 직전 lifecycle/current document 및 같은 active task를 검사한다. source export/projection·owner 변경, return 시작, close에서 pending은 명시 stale/retire 처리한다. old lease를 새 key로 재생하지 않는다.

초기 graph 정의는 항상 같은 closed source를 공유하되 navigation NodeSlot은 명시 Live mount에서만 조립한다. FrameComposition 내부 source inventory에 navigation/item을 추가하면 기존 ProductFrameBlueprint가 생성하는 Studio/Jobs 정의와 내용이 일치해야 한다. 런타임 문자열 모듈 추가나 unknown import 허용은 하지 않는다. 전체 navigation의 VIEW 외 현재 미연결 workspace는 이 scope에서 명시 Unsupported로 거절한다. 그 버튼을 지원 완료나 실제 router 호환으로 표시하지 않는다. 미지원 표시/비활성화까지 바꾸려면 별도 source/컴포넌트 범위를 배정받는다.

## 원 return 경계와 오류 보존

1. 실제 source button → 원 renderer/HostCallbacks/export guard → current local key와 Text VIEW 검증 → typed intent 한 개. 이 단계는 SDK 요청/route commit을 만들지 않는다.
2. 원 actual turn에서 remaining_callbacks==0, 같은 active metadata/observed Studio route/entry 활성 상태를 확인한다. dirty/pending/recovery/terminal ACK·drain·cleanup 거절은 원 request_viewing_return 오류를 보존한다. 저장·취소·Accept를 자동 호출하거나 guard를 직접 clear하지 않는다.
3. route 이전 Busy/borrow 실패는 원 intent/carrier를 보존하고 명시 재시도/취소 경계를 구별한다. 요청 성공 후에는 원 ViewingReturn의 ticket/receipt/first error/owner가 책임지며 새 callback으로 재발급하지 않는다. same current VIEW duplicate는 coalesce하고 서로 다른 owner/key/generation은 거절한다. 백그라운드 자동 retry/spin0.
4. 원 release/session.close/terminal ACK/unbind와 새 Viewing receipt/proof를 그대로 따른다. 첫 receipt는 해당 commit turn의 effect 한 번만 반환하고 rearm-only에 재생하지 않는다. 실제 Viewing retained Some와 원 proof/DTO/current lineage 비교 전 복귀 완료0.
5. invalid/unknown/non-VIEW/local callback을 SDK 호출이나 raw URL/router로 넘기지 않는다. 원 cancellation/partial/first error를 보존하며 부모 SDK 실행 owner는 새로 만들지 않는다.

## 제안 시험 exact5 경로 — 미작성·미실행

아래5와 production5를 합쳐 exact10 후보이며, 배정 시 최신 채택 baseline의 preimage/모든 literal caller를 다시 확인한다.

| 경로 | 검증 목적 |
| --- | --- |
| `src/assembly/meari/viewer_host/source_navigation/tests.rs` (new) | authentic key+scope/Text VIEW, wrong signature/non-VIEW/foreign/old export, capacity/coalesce/다른 owner Busy, stale document/task, 성공 전 보존·성공 후 한 번 소비, close retire. Rust 시험. |
| `src/assembly/meari/viewer_host/mount_proposal/tests.rs` | 같은 session export에 plugin+navigation key 한 번, NodeSlot 실제 source 경로/서명, 실패 시 original partial carrier·old export 보존, mount input 새 optional field 영향. 재사용 Card 선택 port 회귀. |
| `src/assembly/meari/viewer_native_probe.rs` | 새 source navigation action Kind만 추가해 typed fixture return과 구분. 기존 bounded80/4096 및 checkpoint cap 유지. |
| `src/assembly/meari/viewer_native_probe/live_retained.rs` | 별도 SourceReturn 모드에서만 기존 typed-host fixture 자동복귀를 사용하지 않고, 실제 현재 onNavigate source button/path로 VIEW를 한 번 dispatch. 실제 props/buffer·frame/원 guard·새 Viewing proof를 관측. 기존8 fixture 모드는 불변. |
| `src/assembly/meari/viewer_native_launch/no_media/tests/end_to_end/live_retained.rs` | 같은 원 Supervisor/SDK owner/20total2reserve 안에서 Card/Highlight/Cue/Document source-return 양성4. actual source action1/local admission1/route commit1/actual entry·return Some/원 EOF·Empty·forcedfalse·WorkersQuiescent를 구분. source_return_action=true는 이 새4에만. dirty/pending/foreign 등은 headless owner 시험으로 검증하고 창에서 CAS 기능을 반복 인수하지 않음. |

headless queue·원 SDK/CLI guard 시험이 먼저이며, 실제 window4는 별도 PM 자원 배정 후다. fmt/test/clippy와 CargoJSON 고정 BIN/list/정확 시험/실패 보존/원 S 복구 원칙을 유지한다. 현재 이10개 파일은 수정하지 않았고 신규 시험도 작성하지 않았다. 기존 F948 W8와 실제 source navigation4, renderer source action과 OS pointer/key·IME/시각 인수는 별도 범위다.

## SDK chooser와 충돌/작업 순서

현재 SDK 동결 source6 `4CC6D6CF…`의 정확 경로는 viewer_callbacks/{chooser.rs,chooser/tests.rs,mod.rs,codec.rs,lifecycle.rs}, viewer_read_actions/mod.rs다. 위 UI exact10과 교집합0. SDK가 공유 S/T를 소유하는 동안 UI는 이 source-only 제안만 작성하며 shadow source나 private 구현도 시작하지 않는다.

다음 [PreparedSelection→Open 계약](proposal-chooser-open-handoff-contract.md)의 SDK successor는 viewer_host/live_entry.rs와 live_entry/tests.rs를 포함하므로 production 경로1개가 직접 겹친다. 현재 UI 최소안은 LiveEntryRequest·chooser carrier/selection/lifecycle를 수정하지 않지만 동일 파일은 동시 소유하지 않는다. SDK chooser gate 채택 후 최신 root로 preimage를 다시 고정하고, PM이 live_entry 조립 변경을 직렬 배정해야 한다. 비교 대상 raw와 현재 파일을 덮어쓰지 않는다. 표시 기본 chooser/Open 버튼은 별도 R3 진입 배정이므로 source-return 성공만으로 기본 앱 도달성을 주장하지 않는다.

## 스킬 적용과 미검증

같은 세션에서 탐색·본문을 읽은 modoo.today `modootoday/software-qa/test-fixture-and-mock-build`를 원 owner/error/ACK/fixture-vs-real/source/BIN 출처 분리에 적용했다. `modootoday-free/design-delivery-plus/frontend-first-journey`를 공통 컴포넌트·기존 typed 경계와 실제 reachable/미지원 구분에 적용했다. LocalActionInvocation의 즉시 소비 계약을 실제 정의에 대조해 durable grant 저장을 제외했다. 새 언어/SDK/domain 정책이나 Meari API를 도입하지 않는다.

source-only 결과: 제품 코드/시험 구현0, Cargo0, S/T0, GUI0, stage/commit/push0. 제안은 아직 type/실제 source action/기본 chooser/OS·시각 검증되지 않았다.
