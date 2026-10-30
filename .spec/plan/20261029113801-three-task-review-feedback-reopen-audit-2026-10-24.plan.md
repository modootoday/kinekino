---
id: plan:kinekino/20261029113801
title: 3 task 검토 결과 표시·저장본 재열기 연결 감사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 3 task 검토 결과 표시·저장본 재열기 연결 감사

2026-10-24. PM source-only 배정, 기준 root `688a99cbab9a8d371a33d0d3716c34a52f2949b7`. Media가 heavy 슬롯 소유다. 제품 코드/시험 작성·Cargo/GUI/S/T/AI/stage/commit 변경·실행0이며 이 문서만 작성했다. 수동 시각 검토는 계속 보류한다.

## 관측 사실

| 책임 | 현재 소스 | 판단 |
|---|---|---|
| 원문·제안·사람 입력 | `components/proposal-source.tsx`, `components/card-copy-fields.tsx`, `studio/cards/proposal-panel.tsx`, `studio/clips/highlight-proposal-panel.tsx`, `studio/subtitles/correction-panel.tsx` | 세 task 모두 원문 참조/텍스트·generated raw와 controlled 편집 필드를 가진다. Correction은 TextComparison도 사용한다. Card/Highlight에 동일 비교 컴포넌트가 없다는 이유만으로 원문 비교가 없다고 판정하거나 승인 레이아웃을 재설계하지 않는다. |
| 공통 명시 동작 | `components/proposal-actions.tsx:9` | 저장/명시 채택/결과 재확인 버튼은 원 onSave/onAccept/onRefresh, busy/refresh/dirty에서 계산된 canSave/canAccept를 소비한다. 새 저장·채택 정책을 UI에 만들 필요가 없다. |
| 실제 SDK 상태 | `proposals/projection.rs:23`, `session.rs` | ProposalView는 document_revision·state·acceptance를 이미 가진다. ReviewSession은 dirty/pending/error/confirmed mutation·원 inspect 결과를 보유한다. 결과 revision은 SDK 정본 DecimalU64이며 UI가 +1로 만들 값이 아니다. |
| 기본 표시 라벨 | `product_window/labels.rs:16,21`, `panel_model.rs:167,180`, `bootstrap.rs:265,277`, `correction_panel.rs:229,235` | revisionLabel은 기본 “제안 문서”, resultLabel은 빈 기본값이다. Correction의 documentRevision 숫자 prop도 공통 ProposalSource의 revisionLabel로 표시되지는 않는다. Card/Highlight/Correction의 공통 문서 revision/acceptance output revision·저장된 결과 상태를 실제 값으로 보여주는 연결 공백이다. |
| 기본 reactive provider | `assembly/meari/viewer_host/mount_proposal.rs:261,271`, `product_window/mod.rs:257` | provider가 review/index/local ViewError 인자를 무시하고 고정 labels를 clone한다. 최초 표시와 후속 재투영의 결과 문구를 같은 사실 기반 규격으로 정리할 필요가 있다. |
| 오류 표시 | `panel_model.rs:180`, `correction_panel.rs:217`, `window_bridge.rs:513` | SDK review.error.message는 resultLabel에 표시한다. 반면 drain report.view_error를 last_view_error로 보존하고 provider에 전달해도 기본 provider는 무시한다. 모든 오류가 누락된 것은 아니며 지역 ViewError 표시의 누락과 SDK 오류 표시는 구별한다. |
| dirty/pending/recovery 보호 | `window_bridge/navigation.rs:8`, `window_bridge.rs:492` | 원 dirty·pending·confirmed result/cleanup/projection guard가 이동·선택을 막는다. renderer 거부 뒤 controlled props 복구, SDK completion 뒤 재투영·suffix 종료 후 export 교체 경계가 있다. 이번 후속은 보호 정책을 다시 구현하지 않는다. |
| 복귀·fresh generation | `viewer_host/return_viewing.rs:375`, `viewing_chooser.rs:128` | 원 resume export/chooser 재결합과 원 viewing commit을 사용하고, 목록은 실제 draft_revision을 보여준다. 기본 재조회/Open 연결은 D에서 입증했다. 저장된 같은 문서의 새 revision/채택 결과를 다시 열어 공통 헤더와 결과에 보이는 것까지는 별도 근거가 없다. |

## 기존 근거와 남은 범위

- D 정본 `.spec/sots/verification/presentation/default-chooser-journey-d-2026-10-24.md`: 기본 choicesNone/source List·명시 선택·실제 진입·source VIEW·fresh query/다른 문서·항목·재진입·원 cleanup. 수정/Save/Accept/CAS0이므로 편집·채택 완료 근거로 확대하지 않는다.
- `source-navigation-return-r3-headless-2026-10-24.md` 후속 O1: 기존 explicit Live4는 실제 source 입력·수정값 표시·원값 복구·Save를 복귀 전제 조건으로 사용했고 원 create/inspect/edit/inspect였다. 입력/dirty/원 source VIEW의 범위는 인정하되 새 결과 revision/명시 accept 표시·저장본 재열기 인수가 아니다.
- `project-capture-readonly-2026-10-21.md`: live Card source edit/save/accept와 API 연결의 기존 headless 근거가 있다. capture-origin live mutation은 SDK 원자 guard 부재로 차단하는 별도 범위다. captured 화면을 새 live 편집 여정 대신 쓰거나 capture 지원을 복구했다고 표시하지 않는다.
- `live-frame-fixture-r1-2026-10-24.md`: typed/실제·모델 frame 경계를 구분한 기존 근거다. 독립 SDK/CLI Save/CAS/Accept 결과는 해당 정본 소유이며 GUI에서 다시 같은 정책 시험을 하지 않는다.

## 최소 후속 A — 공통 사실 기반 feedback 투영 제안

구현 권한은 아직 없다. 새 정책/SDK/owner/라우터/비동기 작업 없이 기존 typed 결과를 표시하는 순수 Rust feedback DTO/formatter 한 책임을 제안한다. 최소 production 후보는 existing5/new1 총6경로다.

1. 신규 `src/presentation/web_ui_adapter/proposals/review_feedback.rs`: 저장된 document revision·acceptance output revision·현재 dirty/pending·confirmed mutation/refresh needed·SDK 오류·지역 ViewError를 구분한 표시 DTO와 주입 가능한 문구 계약. 저장·채택 가능 여부나 오류 재시도 정책은 계산하지 않는다.
2. 기존 `.../proposals/mod.rs`: 좁은 module 연결만.
3. 기존 `.../proposals/bootstrap.rs`: 최초 mounted props에도 원 StoredProposal의 동일 feedback을 소비한다. 기존 callback/export/선택/owner는 유지한다.
4. 기존 `.../proposals/panel_model.rs`: Card/Highlight 공통 projection이 동일 feedback으로 revisionLabel/resultLabel을 채운다.
5. 기존 `.../proposals/correction_panel.rs`: 같은 feedback을 소비하여 Correction과 다른 두 task의 표시 의미를 맞춘다. DocumentRepair 선택 계약은 유지한다.
6. 기존 `.../proposals/window_bridge.rs`: 기존 last_view_error를 표시 입력으로만 연결하고 최초/후속 feedback의 우선순위를 한 경계로 합친다. completion/ACK/drain/guard/commit 로직은 변경하지 않는다.

현재 공통 source의 revisionLabel/resultLabel이 이미 렌더되므로 이번 A에는 TSX/토큰/새 패널/전체 provider·기능 코드 수정이 필수라고 단정하지 않는다. 구현 중 localized contract 또는 최초 factory 연결에 추가 경로가 필요하면 정확 이유/경로를 먼저 PM에 제안한다. 실제 API 문구/서명은 승인된 후보에서 정리하며 이번 문서가 메서드 추가 승인은 아니다.

표시 규칙은 관측 사실을 따른다. pending과 dirty를 저장 실패로 표시하지 않는다. mutation 응답은 확인됐지만 inspect가 실패한 confirmed+refresh 상태는 “결과 재확인 필요”로 구별하고 원 onRefresh만 유지한다. inspect 완료 후는 저장된 실제 revision/accepted receipt 상태를 표시한다. 세션에 마지막 작업 종류 이력이 없으면 “방금 저장 성공”을 지어내지 않고 “저장된 수정본 revision X” 같은 현재 사실만 보여준다. acceptance output_revision과 document draft_revision은 원값을 각각 명명하고 모든 값은 정확 decimal 문자열로 유지한다. 과거 receipt/모델 결과를 현재 성공으로 승격하지 않는다. SDK·지역 오류는 원 코드/메시지 출처를 구분하며 기존 첫 오류/실패 carrier는 버리지 않는다.

원문 source identity·selected index/ref와 편집값은 현재 원 SDK snapshot/선택에 묶는다. feedback 표시는 새로운 선택 세대·SDK 소유자를 만들거나 기존 선택을 되살리지 않는다. canSave/canAccept/refresh/dirty navigation 정책은 그대로 소비한다. 결과 문자열만 보고 같은 SDK 기능의 완료를 판정하지 않는다.

## 검증 최소안 — UI 계약만, 아직 미작성·미실행

정확 작성/실행 명단은 구현 후보 승인 후 동결한다. 현 단계에서 새 시험 이름·성공수를 등록됐다고 표시하지 않는다.

- 순수 feedback projection matrix: 기존 SDK 계약이 유효하다고 검증한 StoredProposal/MutationReceipt/ApiError fixture를 입력으로 Card/Highlight/Correction의 최초와 후속 표시를 대조한다. revision을 2^53 초과 decimal로 포함하고 accepted output revision·dirty/pending·confirmed+inspect error·local ViewError를 구분한다. Save/CAS/Accept를 다시 실행하거나 정책을 시험하지 않는다. 제안 테스트 경로는 신규 `.../proposals/review_feedback/tests.rs` 한 책임이다.
- 기존 `.../proposals/window_bridge/scoped/task_tests.rs`에 최소 표시 연결 회귀 제안: 원 scope/원 handle/SDK owner를 유지한 현재 controlled input·dirty/지역오류·기존 props commit에 feedback이 반영되는지 확인한다. 이미 검증한 mutation 순서·CAS·ACK/취소·파일 저장 suite를 추가하지 않는다. 원 typed 응답을 UI projection에 전달하는 검증과 실제 SDK 재실행을 구별한다.
- 저장본 재열기는 A 표시 투영 뒤 별도 B 제안이다. 같은 원 SDK/store에 이미 검증된 저장/채택 결과 fixture를 명시 seed하고, 실제 기본 chooser가 fresh List로 같은 proposal의 현재 revision을 표시→명시 item/Open→실제 editor/source/ref/현재 document revision·acceptance output revision을 노출하는지만 읽기 관측한다. GUI Save/Accept 재실행·CAS/저장 파일 재검사0이다. 기존 D의 actual source/frame/owner/세대 계약을 재사용하며 원 frame budget 안 최소 task 매핑을 먼저 제안한다.
- B의 후보 기존 경로는 `viewer_native_launch/no_media/tests/end_to_end/live_retained.rs`(read-only persisted-result fixture)와 `viewer_native_probe/live_retained.rs`(현재 표시 관측)다. existing D 여정을 덮지 않고 별도 case/정확 명단·최소 일반 영향 및 원20/2·15/64를 PM 검토받는다. API 작업 후 상태 fixture가 실제 저장 결과라는 근거 없이 “저장 후 재열기” 성공으로 부르지 않는다. 실제 저장·채택 동작 검증은 SDK/CLI 근거를 연결한다.

source 리뷰/일반 pure·typed UI 검사로 공백을 먼저 좁힌다. 모든 화면의 클릭 반복·수동 시각/OS pointer 검사를 선행 gate로 두지 않는다. default player.exe noargs launcher reachability와 이 API 조립/표시 여정도 구분한다. 새 실행은 Media 반환 및 PM 별도 자원 배정 이후다.

## 적용 스킬와 인계

이 세션에서 본문을 읽은 modoo.today `modootoday-free/design-delivery-plus/frontend-first-journey`의 공통 source/typed 경계·실제 도달성 구분을 적용했다. `modootoday/software-qa/test-fixture-and-mock-build`의 기존 기능 gate 재사용·fixture 입력/실제 결과/원 teardown 범위 구분을 적용했다. Rust 표시 DTO·정확 DecimalU64 및 Result 경계는 rust-pro 기준으로 제안했다. 타 언어 정책·검증 도구는 만들지 않았다.

PM에게 A production6+신규pure시험1/기존typed시험1의 source-only 후보 검토를 먼저 요청한다. B 실제 읽기 재열기2경로는 A 정합 뒤 별도 계획·배정 후보이며 자동 확장하지 않는다. 현재 코드 변경·추가 시험·실행0, 전체 편집/채택 UX·수동 시각 완료0이다.

## A source-only 후속 구현

PM이 최초8경로와 명시 번역 catalog 주입에 필요한 기존 literal6경로를 추가 승인했다. 비공개 후보는 `runtime/development/three-task-review-feedback-r1/HANDOFF.md`이며 총existing12/new2다. PanelLabels.feedback required field의 public Rust source compatibility 영향을 명시하고 전체 기존 literal6을 갱신했다. 승인추가6은 catalog 필드 삽입만이며 기존 assertion·정책은 유지했다. 공통 사실 DTO/초기·후속 투영/지역오류 표시와 순수8+typedbridge1 신규 시험을 작성했다. proposed 영향3 포함 GATE-expected12와 상세 GATE-PLAN을 분리했다. 파일별 fmt/check만 수행했으며 타입/등록/실행0, Media heavy 유지/Cargo/GUI/S/T0이다. B2경로는 계속 미배정이다.

A14 후속 단독 gate에서 sourceF6DA/BIN7870DFE2의 fmt/default/native strict/no-run·실제 등록 및 신규9+영향3 exact12가 모두 통과했다. 기존12 bytes/mtime·신규2부재 복원과 보호 감사 완료. 실제 범위·source compatibility·SDK 기능 중복0·B 미완료를 구분한 정본은 `.spec/sots/verification/presentation/three-task-review-feedback-a-2026-10-24.md`다. PM은 A 반환/채택 뒤 B 기존2 private source-only 구현을 후속 배정했으며 B 실행은 별도다.
