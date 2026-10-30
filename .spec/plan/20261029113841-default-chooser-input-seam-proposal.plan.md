---
id: plan:kinekino/20261029113841
title: 기본 chooser 현재 입력 seam — 읽기·설계 제안
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 기본 chooser 현재 입력 seam — 읽기·설계 제안

## 후속 PM 배정 — I4 private 작성

PM이 아래 I1~I4만 기존2 경로 private 작성·rustfmt까지 배정했다. runtime/development/default-chooser-input-seam-i4-r1의 effective2 472BE14284843473FE001F374FEF7F57A629FD109BE92AEC5BDEBAFC7BA1CC15 / expected4 A4A453F5DED6BF398AA859519A650D52EEF730EC82F325A11AD36082DF382D58을 작성했다. baseline c9c48d20의 before/raw/delta/leaf/HANDOFF와 원 D/B 분기 보존·미검증 영향을 구별했다. rustfmt --check0만 관측했고 Cargo/type/Clippy/실제 등록·실행/shared S/T0이다. Accept dispatch와 E1은 미작성이다. 아래 최초 읽기 전용 설명은 배정 당시 계획 이력이며 실제 실행은 계속 SDK N/C 인수·PM 별도 배정 후다.

2026-10-24 root c9c48d20abddb110d37a810c08c3520c10e955fc / Mearif85 기준. PM 요청에 따른 소스 읽기·계획뿐이다. 코드 작성권·Cargo·OS·physical 입력·새 기능·모델 승인0. SDK fileguard→정상 기본창 N/C 인수와 병렬로 준비하며 실행은 해당 반환·PM 별도 배정 뒤다. [기본 경로 잔여 정본](proposal-default-route-gap-audit.md)의 구체 후속이다.

## 기존 coverage와 실제 빈칸

C21 product4는 명시 NativeProposal의 실제 ProductWindow/Native DOM/controlled textarea/현재 callback path와 같은 SDK worker를 사용하여 Cards·Highlights·CueCorrection·DocumentRepair→typed correction의 Text→Save→별도 Accept를 수행했다. 원 단일 worker/ACK/cleanup도 이미 근거가 있다. 공통 session/controller/bridge의 edit/accept/CAS·immutable publication·receipt를 새로 구현하거나 이 product4를 전체 반복하지 않는다.

현재 live fixture의 Case.binding은 Card=onFrontChange, Highlight=onTitleChange, Cue/Document=onDraftChange다. non-default Positive/SourceReturn은 같은 값으로 restore한 뒤 원 onSave를 return prerequisite로 호출한다. `set_text`는 현재 callback path와 stamp로 원 `set_input_value`를 사용하고 `displayed`는 controlled value와 renderer editor raw_text를 함께 대조한다. Card onBackChange 입력은 이 Case.binding에 없다. D default chooser는 입력하지 않으며, B는 실제 persisted 값을 표시만 한다. 따라서 이미 존재하는 Text 구현과 기본 chooser 진입 뒤 현재 입력의 연결 증거가 구별되어야 한다.

| 후보 | 현재 target/ref와 Text | 현재 빈 관측 | 재사용 근거 |
| --- | --- | --- | --- |
| I1 Card | 현재 selected Card index, 원 source_refs 전체·순서; onFrontChange와 onBackChange 각각 Text | default chooser 진입 뒤 front/back 독립 입력 및 반대 필드 보존·현재 source/ref 유지. back은 기존 live 입력 누락 | C21 Cards 기능 흐름, 기존 source Card front 입력·D 선택·B front/back 표시 |
| I2 Highlight | 현재 Highlight index/source_refs; onTitleChange Text | default chooser에서 현재 title 입력 후 같은 ref·source·다른 항목 불변과 live props | C21 Highlight, 기존 live title 입력·D 선택·B 표시 |
| I3 Cue correction | 현재 index/cue_id/source SHA/revision; onDraftChange Text | default chooser에서 selected cue의 수정 필드만 입력, 원 cue/ref 표시 보존 | C21 cue correction, 기존 live cue 입력·D Cue 선택 |
| I4 DocumentCorrection | 현재 index와 원 typed DocumentTextRef 전체; onDraftChange Text | repair를 새로 시작하지 않고 현재 typed draft로 진입해 입력. ref 전체·source text/다른 segment 불변 | C21 repair→typed 단계·기존 live Document 입력·D 두 번째 문서 진입·B 직접 typed 표시 |

## 정확한 제안 이름·범위

아래4는 미작성 후보 이름이며 등록·실행 명단이 아니다. full prefix는 `assembly::meari::viewer_native_launch::no_media::tests::end_to_end::live_retained::`다. 한 입력 경우가 다른 target/ref의 실제 source 연결을 대신할 수 없어 correction 두 variant를 구별한다.

- default_chooser_card_current_front_back_input_preserves_selected_refs
- default_chooser_highlight_current_title_input_preserves_selected_refs
- default_chooser_cue_current_draft_input_preserves_selected_ref
- default_chooser_document_current_draft_input_preserves_exact_ref

원 original SDK/client/LocalHost·default ProductFrame choicesNone·실제 source chooser를 사용한다. 기존 SDK fixture 준비를 재사용하고 원 create/setup을 실행 여정의 save 성공으로 세지 않는다. synthetic ViewModel을 실제 renderer로 표시하지 않는다.

각 입력 직전에 **현재** retained frame/stamp와 module projection을 확인한다. 정확 binding에 대해 callback 1개·Text signature·현재 task의 module origin·그 callback이 소유한 실제 editable element를 요구한다. cached path·이전 generation의 key·전체 document contains·직접 ReviewSession mutation·별도 callback을 쓰지 않는다. 원 Text 경로로 고유 기술 문구를 넣고 현재 controlled value와 실제 editor buffer, 선택 target/ref/source, 다른 필드/항목 보존을 확인한다. Card front/back은 서로 다른 문구를 사용해 바인딩 교차를 식별한다. 이 기술 Text는 OS key/IME/focus 인수가 아니다.

Save/Accept는 기존 C21의 기능 검증을 반복하지 않는다. 현재 scope의 `onSave`/`onAccept`가 유일한 Action이고 원 enabled/pending/dirty 표시와 일치하는지 관측한다. dirty 중 return이 금지되면 기존 source fixture처럼 **원 onSave를 정리·복귀의 전제 동작으로 1회** 사용하며 현재 응답이 실제 원 worker로 흘러 표시되고 pending이 drain되는지만 본다. SDK의 CAS 성공·충돌 매트릭스/파일 publication/해시/receipt 재계산을 추가하지 않는다. 버튼을 자동 Accept와 결합하지 않으며 명시 Accept 기능을 새로 검사하지 않는다. 기본 route에서 실제 Accept Action dispatch까지 필요한지 PM이 별도 요청한다면 대표 Card1의 원 Action→원 응답 표시만 추가 경계로 제안하고 나머지3 반복이나 합성 사람 승인을 만들지 않는다.

입력 후 원 source VIEW·현재 반환 proof/새 Viewing frame·fresh chooser/query/key·선택 해제를 재사용한다. B의 저장 상태 재열기3/원 D3 전체 반복을 입력4 각 case에 합산하지 않는다. 동일 client/worker는 원 run_child의 same_runtime 및 Recorded 요청/응답·cleanup으로 관측하고 원 identity private literal이나 새 owner를 만들지 않는다.

## 오류와 재시도 — 종류별 분리

| 종류 | 기존 근거와 한계 | 다음 범위 |
| --- | --- | --- |
| entry arm Busy/거절→원 intent 복구 | 기존 Mode::ArmBusy는 take_live_arm_busy one-shot과 실제 live_entry arm, 최초 UnsupportedCallback 보존·latest 해제·관측 wake를 검사한다. SDK edit 실패나 사용자 재시도 버튼이 아님 | 기본 chooser와 이 기존 seam의 결합만 필요하면 E1 Card 대표 후보를 별도 승인 요청. 같은 owner/원 retained current intent·first error를 관측하며 새 retry 정책/반복 wake 금지 |
| 지역 ViewError 표시·해제 | A typed bridge와 기존 scoped rejected commit/focus/owner 보존이 있음. 실제 default source renderer 실패는 미입증 | I1~I4의 정상 입력과 오류0를 오류 유발 시험으로 계산하지 않는다. 임의 잘못된 callback signature/가짜 SDK 오류를 만들어 일반 사용자 오류로 표시하지 않음 |
| 실제 SDK edit/accept 오류와 명시 사용자 재시도 | 기존 SDK/CLI가 기능 오류·CAS를 검증하며 A는 오류 DTO 표시 계약만 검증 | 현재 두 경로만으로 정책 주입 없이 발생시키는 지원 seam이 확인되지 않았으므로 새 exact 실행 후보에 넣지 않는다. 기존 허용 fault adapter의 실제 caller/응답/owner 경로가 확인되면 정확 경로·최초 오류·현재 입력 보존·명시 Action만 별도 제안. 최초 오류를 지우거나 자동 SDK 재요청을 추가하지 않음 |

E1의 제안 이름은 `default_chooser_card_original_arm_recovery_preserves_first_error_and_owner`다. **미작성·조건부 1개이며 I4 명단에 포함하지 않는다.** 기존 `take_live_arm_busy`와 `live_error_state`는 Mode::ArmBusy에 한정되고 default chooser dispatch는 Mode::DefaultChooser에 한정되므로, opt-in의 phase 조합을 source2 안에서 표현 가능한지 먼저 설계 검토한다. 기존 Mode 의미/회귀 단언을 바꾸거나 host production에 새 실패 주입 경로를 만드는 방식은 제안하지 않는다. 결합 불가면 exact 추가 경로부터 PM에 요청하고 구현하지 않는다.

## 변경 예상 경로와 비충돌 책임

- `src/assembly/meari/viewer_native_launch/no_media/tests/end_to_end/live_retained.rs`: UI 소유. 후보 exact case·기존 child setup/Recorded transcript/원 cleanup owner·실제 명단 및 범위 관측만. supervisor나 SDK 기능 검증을 재작성하지 않는다.
- `src/assembly/meari/viewer_native_probe/live_retained.rs`: UI 소유. 기존 current source Text/Action helpers와 원 frame/target/ref/editor 관측을 재사용하는 opt-in phase·bounded trace. 생산 policy/host/router를 추가하지 않는다.

변경 예상2는 승인 source 목록이 아니다. production bridge/session/panel/TSX·토큰·SDK API·launch·main·fileguard 변경 필요는 현재0이다. 기존 `viewer_native_probe.rs`의 event enum/고정 budget과 `no_media.rs`의 bound guard 및 supervision을 변경하지 않고 표현해야 한다. helper 부재·추가 event가 필요하면 두 경로를 넘기기 전에 이유와 정확 경로부터 제안한다.

SDK는 `tests/native_product_launch/default_launch.rs`와 그 child 모듈의 pin/pipe/fileguard/정상 N/C를 소유하고 UI 두 파일과 직접 교집합0이다. 실제 실행은 같은 launcher/SDK/공유 S/T/Native 자원을 쓰므로 N/C와 병행하지 않는다. SDK source 채택 뒤 fresh baseline·approved sync·고정 binary/list/ignored 분류·원 보유/복원/첫 실패/실행 lease를 별도 인수한다. 현재 시험 추가·빌드·raw fixture 준비·OS 실행0이다.

예산은 원 supervisor20/reserve2/run18·expiry15/frame64/event80/payload4096을 상속한다. phase가 예산 안에 안 들어오면 여러 기능을 억지로 합치거나 한도를 늘리지 않고 최소 case 분리부터 PM에 제안한다. exact4 정상 입력과 조건부E1을 하나의 성공 수로 미리 합산하지 않는다. 기존 단언·guard 완화/자동 재시도·physical mouse 반복0이다.

## 근거·적용

[C21 coverage](../edited-output-quality-decision.md), [A](../../../sots/verification/presentation/three-task-review-feedback-a-2026-10-24.md), [D](../../../sots/verification/presentation/default-chooser-journey-d-2026-10-24.md), [B4](../../../sots/verification/presentation/persisted-review-reopen-b4-2026-10-24.md)와 현재 Case.binding/set_text/displayed/save_current_source/live_error_state를 읽어 대조했다. 같은 세션에서 읽은 modoo.today frontend-first-journey의 기존 source/typed port 재사용과 actual 경계, test-fixture-and-mock-build의 setup/원 기능/GUI 관측 구분 및 owner/teardown 원칙을 적용했다. 과거 고정 BIN의 기능 통과를 현재 계획의 실행 성공으로 표시하지 않는다.
