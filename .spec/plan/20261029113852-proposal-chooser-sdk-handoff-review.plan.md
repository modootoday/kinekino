---
id: plan:kinekino/20261029113852
title: R3 chooser — SDK 결과에서 원 host-held 선택까지
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R3 chooser — SDK 결과에서 원 host-held 선택까지

관측 root `09fc7518407798be1480f17aa81f2efa1bdef8d3`의 채택 R1 및 기존 SDK를 읽었다. 목록 후보 source11 EABD4052…는 private/타입·실행0이며 canonical API에 이미 있다고 가정하지 않는다. [UI 단계의 R3](proposal-viewer-route-integration-stages.md)는 제안 상태다. 이 문서만 추가했으며 list11/EXECUTION·UI R2 소스/문서·코드·Cargo·S/T·실제 store scan·네트워크·GUI는 변경/실행하지 않았다.

## 실제 계약과 주장 대조

| 주장 | 판정과 근거 |
|---|---|
| create/import 결과를 얻을 SDK typed port가 없다 | 반증. `application/api/client.rs`의 create_proposal→ProposalDocument/import_historical_proposal→StoredProposal, `pending_host.rs`의 PendingRequest::Create/Import 및 Created/Imported, async_proposals 등록이 있다. SDK 생성·수입 정책을 새로 만들 필요가 없다 |
| SDK 결과가 Viewer의 live choices까지 자동 연결돼 있다 | 반증. `TrustedLiveChoices::from_trusted_local`은 이미 만들어진 정확 NativeProposalSelection 목록을 받는다. create/import가 반환한 문서를 item 선택과 함께 이 목록으로 만드는 기본 Viewer consumer는 별도다 |
| 목록 행을 NativeProposalSelection으로 바로 바꿀 수 있다 | 반증. private list11의 ProposalListRow는 문서 identity/revision/task/count metadata다. cue_id/DocumentTextRef/item 내용이 없고 selection_authority=none이다. `CardCurrentInspection::begin`은 BootstrapSelection을 요구하므로 metadata만으로 index0/ref를 발명해 진입할 수 없다 |
| 기존 original_execution을 복제해 active Viewer와 나란히 raw submit하면 된다 | 반증. sdk_execution/attachment.rs는 하나의 quiescent exclusive consumer를 요구한다. ViewerCallbacks는 원 attachment를 보유하고, R1은 원 Viewing read drain/ACK/release 뒤 같은 ExecutionHandle/context로 inspect한다. clone은 새 submit authority가 아니다 |
| 현재 live choices 설정이 활성 화면의 새 결과에도 바로 적용된다 | 반증. `viewer_callbacks/lifecycle.rs::configure_live_choices`는 scope.is_some/stopping/selection/live_selection이 있으면 “Live choices require the original unexported owner”로 거절한다. 기존 pre-export 조립 API를 dynamic update라고 부를 수 없다 |
| captured generic mutation이 SDK 미지원이므로 Correction/Highlight가 읽기 전용이다 | 반증. 공통 EditCapturedProposal/guarded writer/domain은 generic ProposalPayload다. 현재 CaptureCardModel/ProjectCardChoices와 onCapturedCardEdit가 Card 소비층이다. R0의 실제 Correction CLI와 Highlight/DocumentCorrection 미검증을 유지한다. 현재 UI 미연결을 SDK Unsupported로 표현하지 않는다 |

## 공통 typed handoff의 최소 의미

아래 이름은 새 구현이 아니라 **UI adapter 내부 계약 제안**이다. SDK에서 permission/launch token을 발급하는 새 API는 제안하지 않는다.

- `HostProposalCandidate`: 원 configured client/context·원 scope/generation·출처 종류·수신한 typed 결과 또는 metadata witness를 원 host에 보유한다. renderer에는 bounded opaque key와 표시 필드만 제공한다. 종류는 create 결과/import 결과/list live metadata/captured read-only를 구별한다. 출처 label 자체가 권한은 아니다.
- `DocumentChoiceKey`와 `ItemChoiceKey`: 문서 선택과 item/ref 선택을 구별한다. 문서 metadata의 item_count는 버튼/빈 상태 힌트이며 index의 소유권 증명이 아니다. 각 key는 host-held 행·generation·실제 LocalActionKey/scope에 묶인다. query/page/문서 교체는 이전 key를 retire한다.
- `HostInspectedDocument`: 기존 InspectProposal의 typed StoredProposal와 원 terminal/owner·ACK 상태를 보존한다. 목록/생성 결과 witness와 proposal ID·draft revision·source ID/revision/SHA·task를 대조하고, 여기서 얻은 실제 item index+cue/ref만 명시 선택할 수 있다.
- 최종 전달 값은 기존 `NativeProposalSelection`/`NativeProposalTarget`이다. 이를 R1의 TrustedLiveChoices/원 current inspection에 전달하며 새로운 SDK policy DTO로 복제하지 않는다. 전달 뒤에도 `project_initial`의 full identity·target membership/cue/ref 검사와 기존 SDK mutation guard는 유지된다.

Raw path/URL·renderer가 보낸 ID/SHA/ref/origin/정수 index를 조립 권한으로 받지 않는다. 빈 목록/repair count0/실패에 첫 행 fallback을 두지 않는다. 동일 current 선택의 중복은 원 intent coalesce만 허용하고 재inspect/재commit 성공을 만들지 않는다. 다른 선택은 bounded Busy/교체 절차로 명시 처리한다.

## A. create/import 결과의 즉시 handoff

이 경로는 저장소 전체 목록을 기다릴 필요가 없다. 원 SDK 요청의 결과를 받고 사용자가 그 결과 안의 문서·item을 **명시 선택**한다. “즉시”는 자동 첫 item 열기나 생성 성공=편집 활성이라는 의미가 아니다.

1. 원 configured SDK/동일 ExecutionOwner가 create/import를 수행한다. create의 ProposalDocument와 import의 StoredProposal를 구별하고 원 request_id/envelope·typed decode·terminal 결과를 보존한다. create 결과를 임의 StoredProposal로 확장해 acceptance/historical 상태를 제조하지 않는다.
2. 전체 결과의 draft/source를 원 host에 보관한다. 이미 명시된 caller target이 있으면 반환 결과의 membership·cue/ref와 정확히 대조한다. target이 없으면 item 선택 UI를 제공한다. 잘못된 generated output/repair에는 자동 target을 부여하지 않는다.
3. 원 terminal을 소비/ACK하고 원 attachment를 명시 해제한 뒤, 기존 pre-export launch 단계라면 `prepare_product_viewer_launch_with_live_choices`에 기존 TrustedLiveChoices를 공급할 수 있다. 새 worker/ExecutionOwner/store scan은 필요 없다.
4. Viewer가 이미 export된 경우 현재 configure_live_choices 호출로는 갱신되지 않는다. UI가 별도 dynamic handoff 계약을 구현해야 한다. 이전 키/선택/dirty·pending intent/terminal/attachment를 원 owner가 정리하고 같은 Viewer context의 명시 generation 교체·props commit으로 새 선택을 게시해야 한다. 현재 guard를 삭제하거나 scope=None으로 조작하지 않는다.
5. 사용자의 Open intent는 R1의 동일 route owner→Viewing drain/ACK/release→CardCurrentInspection→project_initial 대조를 거친다. R2의 editor mount/actual proof·복귀가 성공해야 실제 입력을 활성화할 수 있다. 이 문서에서 R2 성공을 전제하지 않는다.

기존 R1은 DocumentRepair target을 “Live repair entry is not connected”로 명시 거절한다. 생성 결과가 NeedsRepair인 경우 기존 별도 수동 repair 기능을 보존하며 R3 typed draft 진입의 성공으로 바꾸지 않는다. repair ref를 선택할 추가 연결은 별도 요구/계약으로 남는다. SDK create/import 자체의 지원을 이 UI guard 때문에 Unsupported로 바꾸지 않는다.

## B. 목록 조회에 기반한 두 단계 선택

이 경로는 **list11의 실제 SDK/CLI 인수·채택이 선행**한다. 현재 canonical에 없으므로 capability를 추측하거나 파일을 직접 열거해 대체하지 않는다.

1. 원 Viewing consumer가 typed ListProposals를 요청한다. 기존 Viewer read codec의 READS는 capabilities/playback.engine/playback.status 세 개뿐이며 ReadInput도 empty다. list payload/typed reply는 아직 이 profile에 없다. 기존 async proposal manifest도 create/inspect/edit/accept/import만 있고 list는 없다.
2. 따라서 기존 원 ExecutionAttachment/AsyncSdkPump에 명시 chooser read profile을 추가하는 연결이 필요하다. 기존 READS3의 의미·contract를 보존하며 list/inspect codec·manifest를 별도 opt-in으로 조립한다. 원 attachment의 SdkPort를 공유하고 새 attach/worker/endpoint를 만들지 않는다. raw original_execution.submit으로 현재 exclusive owner를 우회하지 않는다.
3. metadata 응답은 **문서 선택 목록**이다. 원 SDK dataset/snapshot/query/continuation은 host에 남고, 다음 페이지도 같은 원 consumer로 요청한다. SourceExpired/RevisionConflict/ResourceExhausted/Unsupported는 해당 목록의 오류로 표시한다. stale 후 silent refresh·자동 첫 선택·다른 dataset fallback은 없다.
4. 사용자가 문서를 고르면 같은 원 consumer에서 `InspectProposal {proposal_id}`로 **선택 준비용 문서 inspect**를 한다. 이는 item target이 아직 없기 때문에 R1의 target-bound CardCurrentInspection과 다른 목적이다. 기존 InspectProposal SDK/원 terminal·ACK를 재사용하며 새 일반 SDK 정책을 만들지 않는다.
5. inspect 결과가 metadata의 identity/revision/source/task와 다르면 Stale을 표시하고 명시 재조회/재선택을 요구한다. 새 snapshot을 옛 행의 권한으로 채택하지 않는다. 일치하면 host-held full document의 실제 item/ref 목록을 보여주고 사용자가 하나를 명시 선택한다. captured row를 이 목록의 live 후보로 자동 변환하지 않는다.
6. 정확한 NativeProposalSelection을 만들고 원 chooser read terminal/ACK·attachment를 drain/release한 뒤 R1/R2 진입을 요청한다. 진입 직전의 **target-bound current inspect**는 유지한다. 문서 선택 준비용 inspect와 mount 직전 검사에는 서로 다른 stale window가 있으므로, 후자를 삭제해 중복을 줄이지 않는다. 반대로 선택이 변하지 않은 같은 intent의 재시도마다 새 준비 inspect를 무제한 제출하지 않는다.

`InitialProposalInspection::start`는 target 없는 InspectProposal을 받을 수 있지만 trusted startup 전용이며 내부에서 raw execution.submit을 호출한다. 이를 active Viewing attachment와 함께 직접 쓰는 것이 재사용은 아니다. pre-export 단계에는 기존 의미대로 사용할 수 있고, 활성 단계는 위 원 consumer의 typed inspect binding으로 연결한다.

## captured 작업의 별도 경계

captured 읽기는 ProjectTaskChoices의 working draft/capture/source/ref를 보존한다. 같은 proposal ID가 live store에 있거나 current inspect가 일치해도 기존 captured user intent를 live intent로 바꾸지 않는다.

captured 편집은 원 project ID/revision/hash/capture hash의 CaptureAnchor와 원 configured proposal writer·origin·immutable state/full-state expectation을 공통 SDK가 검사한다. 현재 Card 버튼/모델만 이 UI 경로를 소비한다. Correction/Highlight/DocumentCorrection을 지원하려면 generic CaptureMutationAdapter를 같은 원 owner에 연결하는 UI 입력/표시·종류별 SDK/CLI/정리 근거를 별도 확보한다. onCapturedCardEdit를 일반 live chooser callback으로 전용하지 않는다. originNone legacy는 원 NeedsInput/read-only 정책을 유지한다.

## exact 경로 제안과 선행 gate

R3의 기존 exact9는 공통 task_choices·TSX 목록·selection callback·product caller·Viewing/frame 조립 책임을 이미 나눴다. 기본 사용자에게 보일 목록/선택 UI는 그 범위에 두고, SDK 정책·filesystem은 추가하지 않는다.

추가가 필요한 seam 후보는 다음이며 **지금의 코드 작성 배정이 아니다**. R2 변경을 읽거나 재작성하지 않고 채택 R1의 포트에서 도출했다.

| 경로 | 최소 책임/필요 이유 |
|---|---|
| 신규 `src/assembly/meari/viewer_callbacks/chooser.rs`와 `/chooser/tests.rs` | 원 attachment를 소비하는 typed list/선택 준비 inspect·host-held candidate/terminal carrier. 별도 SDK worker 없음 |
| 기존 `src/assembly/meari/viewer_callbacks/mod.rs`, `codec.rs`, `lifecycle.rs` | opt-in chooser profile 등록, 기존 READS3 보존, active result handoff의 generation/retire·drain/release/close 연결. 현재 configure guard를 우회하지 않음 |
| 기존 `src/assembly/meari/viewer_read_actions/mod.rs` | 새 명시 list/page/document choice intent가 필요할 때만 기존 admission/원 key 규칙과 조립. generic empty ReadInput에 payload를 숨겨 넣지 않음 |
| 기존 `src/assembly/meari/viewer_host/live_entry.rs` | 이미 선택된 typed live 값을 원 route 단계에 전달하는 제한 seam이 실제로 부족할 때만 추가. CurrentInspection/JobsLedger를 복제하지 않음 |

새 후보 수를 R3 exact9에 묵시적으로 더하지 않는다. 실제 조립은 caller/consumer/lifetime 검토 후 최소 exact 집합을 PM이 배정한다. 목록을 async_proposals 범용 profile에도 제공하기로 결정하면 별도로 proposal_manifest.rs/async_proposals/mod.rs/pending_host.rs가 필요하다. **Viewer 전용 chooser profile과 범용 proposal profile을 동시에 중복 등록하지 않는다.** 현재 목적에는 원 Viewing consumer의 opt-in list/inspect 연결이 최소 후보이며 새 universal SDK API는 필요 없다.

순서는 SDK list11 실제 CLI gate/채택(목록 경로만)→R2의 원 route/mount/proof 인수→R3 UI 정확 선택 공급/표시/키·scope/원 cleanup 연결이다. create/import 즉시 handoff는 기존 SDK gate 근거를 확인해 목록 없이도 별도 R3 caller 후보로 진행할 수 있다. 둘을 한 가지 “기본 chooser 완료”로 합치지 않는다. 기본 launcher/URL 정책 전환은 계속 R4 별도 결정이다.

SDK/CLI는 create/import→inspect/edit/save/명시 accept/reopen 정책, list→page/stale/명시 inspect를 입증한다. UI gate는 callback origin/key/generation, 문서·item 명시 선택, 원 결과 표시·route proof·입력 활성·복귀/close만 추가 입증한다. GUI로 CLI 미지원 경로를 우회하지 않는다. source11의 current_evidence=not_checked, R1의 Inspected, R2의 actual checked frame, 실제 user input, 사용자 품질/시각 인수는 서로 다른 결과다.

## 출처와 적용

modoo.today의 `modootoday/software-qa/code-change-review-verification` 본문 SHA9f73865f96ec0cefcf3a4023b294400258473640df3b1d2add03dcca8192d64b와 필수 verdict 참조 SHA9d9619a829ae24d73645ef5f7c8126c8257caeddac80800632866257008ad2a9를 실제 읽었다. R3/R0의 “SDK port 부재”, “metadata에서 직접 선택 가능”, “원 handle clone을 병행 사용 가능”, “captured SDK 미지원” 주장을 기존 호출부·guard에 대조해 위 판정과 미연결 seam을 구별했다. 신규 구현·타입·SDK/CLI/GUI 실행 성공을 주장하지 않는다.
