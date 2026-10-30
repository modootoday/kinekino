---
id: sot:kinekino/proposal-view-input
title: 제안 검토 읽기 모델과 입력 상태
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 제안 검토 읽기 모델과 입력 상태

최신 실제 인수는 [Native Correction gate](../../../verification/presentation/native-review-sdk-2026-10-19.md)다. 원본 Review의 실제 Native 문서/제품 수명/단일 registry/SDK 경로에서23회귀·strict·fmt를 통과했다. 아래 source 준비·headless 기록은 이력이고 다른 payload/전체시각 인수로 확대하지 않는다.

최신 추가 source는 [document 수명 조립](proposal-document-session.md)이다. fmt만 확인한22 gate 이후 변경으로, 기존 통과를 새 수명/Drop 검증으로 확대하지 않는다.

최신 실행 결과: [controller gate](../../../verification/presentation/proposal-controller-2026-10-19.md)에서 현재 Rust source의 fmt/test16/strictClippy를 통과했다. 아래 source 준비/미실행 수량은 당시 이력이며, 최신 결과를 TSX callback·실제 화면·시각 인수로 확대하지 않는다.

이후 [bounded intent/callback adapter](proposal-intent-dispatch.md)와 실제SDK callback을 포함한22검사가 [최종 headless gate](../../../verification/presentation/proposal-callback-sdk-2026-10-19.md)에서 통과했다. 이전16 checkpoint를 보존하며 실제 TSX/renderer 검증으로 확대하지 않는다.

이후 RegistryProposalHost와 기존 actualSDK 회귀의 단일 registry 구성을 [후속22 gate](../../../verification/presentation/proposal-single-registry-2026-10-19.md)에서 test22/strictClippy/fmt로 통과했다. 실제 InteractiveDocument/renderer 조립은 [제품 등록 인계안](../../../../plans/player/presentation/proposal-product-registration.md)을 따른다.

상태: 2026-10-19 현재 mapper/controller Rust consumer의 fmt·locked test16개·strict Clippy가 종료0이다. 이전11개 gate는 이력으로 보존한다. 실제 Meari source callback·화면 입력·시각 인수는 미검증이다.

공통 책임은 `src/presentation/web_ui_adapter/proposals`에 둔다. correction 검사기, CARD 문구, CLIP 추천이 같은 저장·오류·명시 채택 경계를 소비한다. [실제 SDK host](plugin-proposal-host.md)의 canonical DTO와 ApplicationClient를 사용하며 presentation이 정책·CAS·IO를 복제하지 않는다.

## 표시 계약

`ProposalView`는 canonical StoredProposal.validate 이후 원문 identity/SHA/revision, generated raw, 저장된 typed draft, import_error, 상태, acceptance를 분리한다. `ReviewDisplay`는 여기에 사용 중인 working draft, dirty, pending 종류, SDK ApiError를 별도로 제공한다. 수동 수정 후 남아 있는 import_error는 최초 import 관측이며 현재 수정본의 오류 판정으로 바꾸지 않는다.

Cue 원문은 cue ID와 SDK가 제공한 시간 Option을 보존한다. Document 원문은 canonical DocumentTextRef를 유지하고 시간을 생성하지 않는다. CARD의 front/back/source_refs와 CLIP의 title/source_refs는 구조를 보존한다. 생성 원문을 typed draft로 간주하거나 현재 player cue에 임의 연결하지 않는다.

문서 draft_revision과 receipt의 draft_revision/output_revision은 별개다. 검증된 채택 사례의 값은 각각 3/2/3이다. acceptance는 품질 승인·자동 적용을 뜻하지 않는다. 현재 DTO에 연결되지 않은 generation provenance와 quality verdict는 unknown으로 표시한다.

## 입력과 응답 경계

`ReviewScope`는 Meari PluginScope와 proposal/source identity·SHA·source revision을 고정한다. session마다 private owner identity를 갖는 PendingToken을 발급하므로 동일 scope와 sequence로 재생성한 화면에도 이전 응답을 적용하지 않는다. pending 중 새 수정/저장/채택은 Busy이며 dirty 초안의 채택은 SaveRequired다.

저장은 canonical EditProposal과 관측한 expected_revision을, 명시 채택은 canonical AcceptProposal과 원본 source revision을 host에 전달한다. 실패 시 SDK 오류 전체와 작업 중 초안을 보존한다. 성공 시 canonical mutation 응답과 fresh inspect를 함께 받으며 source·종류·revision이 대응하지 않는 snapshot은 초안을 덮어쓰지 않는다. 재시도·새 revision 추정·자동 채택은 하지 않는다.

set_draft는 전체 canonical payload를 받는다. fields.rs는 cue/document correction, card front/back, highlight title의 개별 문자열 입력을 같은 payload에 반영한다. opaque 입력 핸들은 session owner와 working-draft generation에 묶이고 전체 draft 교체·각 수정·성공 응답 뒤 갱신된다. 이전 핸들은 거부하며 다른 필드와 source reference는 보존한다. 빈 문구 등 미완성 입력을 SDK 검증 정책으로 재판정하지 않는다. host controller는 같은 Meari registry·실제 SDK consumer에서 검증했고, TSX callback 연결은 후속이다.

이 필드 mapper와 추가 회귀 2개(card 필드 보존/옛·다른 session 입력 거부, highlight 참조 보존/pending 입력 거부)는 source checkpoint다. highlight 검사는 저장 성공 뒤 대기 중이던 옛 입력을 거부하고 새 핸들의 명시 편집을 허용하는 경계까지 확장했다. 앞서 통과한 11개 gate 이후의 변경이다. 당시 호스트 안정화로 cargo/실행을 보류했던 이력은 보존하며 현재 재개 정책은 아래 최신 checkpoint를 따른다. 과거 11개 통과를 신규 source의 검증으로 확대하지 않는다.

## 검증 입력과 남은 인수

`tools/player-ui-sdk-bridge/tests/proposal_view.rs`의 재생성 session의 늦은 응답, 충돌 오류/수정 보존, 채택 revision 구분, 오래된 inspect 거부/구조 보존 4개 검사가 통과했다. 이들은 synthetic UI 상태 검사이며 기존 실제 raw 9개 SDK 사례나 모델 품질 검토를 대체하지 않는다. 기존 SDK 7개와 합계 11개가 통과했고 strict Clippy도 종료0이다.

중앙 배정 gate의 원문은 `runtime/quality-review/player-ui-proposal-view-r1/gate`에 둔다. 공유 native target, BelowNormal, offline/locked, -j1로 실행했다. test compile 1분11초, Clippy 28.34초이며 종료 후 슬롯을 반환했다. root Cargo.lock SHA256은 전후 `0cf44760431b804cdf18afd50aa82f57b50579cfe1465f9f6d6e9ee7b1aa96cc`, 독립 tool lock은 serde 직접 의존 동기화 후 `7fc45c046cda92f93d76d30dd8534e389d25ee697d1c25223224397548c58ab5`다. GUI·추론·독립 plugin6 재검사는 실행하지 않았다.

최소 선언은 `src/presentation/web_ui/verification/proposal-review-intent.tsx`다. inputDisabled/saveDisabled/acceptDisabled를 분리하고 controlled input과 onDraftChange/onSave/onAccept를 보존한다. 고정 consumer r2의 당시 baseline은 exit1, unsupported `prop type must be string/boolean`, span281..320이다. `runtime/quality-review/player-ui-proposal-view-r1/source-binding`에 당시 원본·manifest·진단을 보존했다. 이후 Action refresh 소비를 위해 refreshDisabled와 onRefresh()를 추가했으며 현재 원문의 span으로 이전 진단을 재사용하지 않는다. 구버전 관측은 최신 Meari 지원 판정이 아니다.

다음은 실제 callback 지원 인계 후 같은 source의 입력→canonical intent→SDK 응답→fresh inspect를 연결하고, 중앙 실행 슬롯에서 화면 이동·오류·키보드·좁은 검사기·확대 시안 대응을 검증하는 것이다. 전체 13화면군의 시각 인수는 별도로 남는다.

## Host controller 실패 경계

ReviewSession.require_refresh/complete_refresh는 최종16개 gate에 포함해 format·compile·시험했다. confirmed mutation과 inspect 오류를 분리해 보존하고 refresh_required를 표시한다. 기존 complete의 성공 입력은 mutation과 fresh inspect가 모두 있는 ResolvedMutation이다. controller가 mutation 성공 후 inspect 오류를 일반 mutation 오류로 넘기는 구현은 허용하지 않는다. 저장·채택 성공 여부와 화면 snapshot 갱신 여부는 별개다.

| 관측 | 표시·보존 | 후속 입력 |
|---|---|---|
| mutation 자체가 ApiError 반환 | 기존 snapshot·working draft·SDK 오류 보존 | SDK 결과에 따른 명시 후속 동작, 자동 재시도 없음 |
| mutation 응답 확인, inspect 실패 | 확인된 mutation receipt와 inspect ApiError를 별도로 보존. 저장 실패로 표시하지 않음 | mutation 재전송 버튼 대신 동일 proposal의 명시 새로고침. fresh snapshot 확인 전 새 저장·채택 보류 |
| mutation 응답 확인, inspect scope/revision 불일치 | 관측 receipt·사용자 수정본 보존, snapshot을 임의 병합하지 않음 | 명시 inspect 및 SDK의 최신 상태 확인. expected revision 추정 없음 |
| 둘 다 성공하며 session/token 대응 | canonical mutation과 StoredProposal의 scope/revision 확인 후 snapshot 반영 | 새 field handle 발급, 이전 입력 이벤트 거부 |
| 화면 교체 뒤 지연 응답 | 다른 session 수정본을 덮어쓰지 않음. 실행된 mutation의 결과 자체는 host 상관관계에 보존 | 해당 proposal 재열기 시 canonical inspect |

현재 controller는 기존 Meari plugin registry를 통한 typed host 호출을 소비한다. ApplicationClient를 UI에서 별도로 우회 호출하거나 새 registry·CAS·idempotency 정책을 만들지 않는다. mutation 직후 inspect에는 mutation의 expected_revision을 복사하지 않고 canonical inspect 계약에 맞는 별도 context를 사용한다. request ID는 기존 host의 ticket 상관관계를 따른다.

인수에는 저장·채택 각각의 성공 후 inspect 실패, 명시 새로고침의 실패/성공, 그동안의 중복 mutation 방지, 화면 전환 후 응답 보존을 포함한다. 이 상태가 session/controller source에 구현되고 실제 같은 SDK consumer에서 통과하기 전에는 입력 연결을 완료로 표시하지 않는다. 당시 호스트 안정화 실행 보류는 이후 중앙 재개 지시로 대체됐다.

controller 도입 전에는 저장 성공 뒤 inspect 실패·재실패·오래된 snapshot 거부·정상 refresh·중복 mutation 거부의 synthetic 회귀 1개를 추가해 예상14개였으며 실행하지 않았다. 이후 controller/양쪽 mutation 회귀를 추가한 현재 범위는 아래16개 checkpoint를 따른다. 기존11개 통과 기록은 이 신규 source를 입증하지 않는다.

## 중앙 재개 이후 controller checkpoint

중앙의 즉시 재개 지시로 저자원 source 구현과 format을 재개했다. 무거운 검증만 한 담당씩 -j1·BelowNormal·전용 실행기 최대 두 logical CPU로 배정한다. 신규 추론/다운로드·Pages·VM·DPI 보류는 유지한다. 기존 goal은 get_goal에서 blocked로 확인했으며 도구의 update_goal에는 active 재개가 없어 PM에 제한을 보고했다. 미완료 goal을 완료 처리하거나 다른 목표로 교체하지 않고 구현은 계속한다.

`controller.rs`의 ReviewController는 typed ReviewHost를 통해 edit/accept를 호출하고 확인된 mutation을 먼저 보존한 뒤 inspect한다. inspect 실패나 모순 snapshot에서 receipt와 수정본을 보존하고 명시 refresh만 허용한다. `tools/player-ui-sdk-bridge/src/review_host.rs`의 ProposalReviewHost는 기존 ProposalBridge→Meari registry→실제 ApplicationClient 경로를 소비한다. inspect에 expected revision을 복사하지 않으며 SDK ApiError는 canonical details에서 복원한다. 별도 registry나 정책을 추가하지 않는다.

현재 fmt/test16/strict Clippy가 모두 종료0이다: 기존 SDK7 + 기존 view4 + 필드2 + refresh상태1 + controller 실패주입1 + Windows 실제SDK controller1. 실패주입에서 edit와 accept 각각 mutation1회·inspect2회·동일 mutation 재전송0을 확인했다. 실제SDK에서는 필드 입력→저장→inspect→명시 채택→3/2/3 revision을 같은 registry로 확인했다. 실제 raw9 사례도 기존3개 검사에서 통과했다. 자원 receipt·원실패/수리·전후 lock·관측 범위는 [최종 gate 기록](../../../verification/presentation/proposal-controller-2026-10-19.md)에 둔다. 슬롯은 반환했으며 TSX source callback·실제 화면·시각 인수는 여전히 미검증이다.

교정 검사기 source는 공용 components/proposal-actions.tsx를 소비한다. refreshRequired 상태에서 입력·save·accept를 막고 명시 refresh를 별도 callback으로 전달한다. 기존 footer/button 토큰을 재사용하며 새 장식 자산이나 색상은 추가하지 않았다. 이 선언 변경은 실제 변환/렌더 증거가 아니다. Meari 담당이 인계한 최소 경로는 원본 onDraftChange(string)/onSave()/onAccept()→scoped host callback→기존 PluginRegistry이며 아직 본체 source callback/dispatch는 미지원이다. onRefresh는 같은 명시 inspect 경로의 후속 소비 요구다. callback에 임의 SDK 명령·URL·셸을 문자열로 싣지 않는다.

CARD/CLIP 후속은 [패널 source 기록](../../../verification/presentation/card-highlight-proposal-source-2026-10-19.md)에 둔다. canonical payload/source_refs를 묶은 ReviewItem을 추가하고 같은 필드 핸들·controller·토큰·ProposalActions를 소비한다. 관련 assert는16개 최종 gate에서 통과했다. 전체 카드/클립 편집기 구현과 실제 callback 연결 완료를 뜻하지 않는다.
