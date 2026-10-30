---
id: plan:kinekino/20261029113912
title: R0 — 공통 SDK 선택 공급·captured 편집 계약 확인
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R0 — 공통 SDK 선택 공급·captured 편집 계약 확인

2026-10-23, 관측 root `72c8132a01ca4168fdec8a0678121b67b4224b45`, Meari `b532fe0eed06d8526b755b11bc36416070244ef7`. [UI 연결 단계](proposal-viewer-route-integration-stages.md)의 R0 주장만 현재 소스·기존 원 실행 근거로 대조했다. 새 코드/Cargo/S/T/CLI/네트워크/GUI 실행0, 제품 연결 source13 불변이다. 현재 root의 재실행 통과를 주장하지 않는다.

## 주장별 판정

| 주장 | 판정 | 호출 경로와 근거 |
|---|---|---|
| configured SDK에 일반 live chooser용 proposals.list가 있다 | 반증 | `src/application/proposals/mod.rs`의 ProposalStore는 load/capture_current/publish 및 선택적 mutation/output-review 포트다. is_command는 create/import_historical/inspect/edit/accept만 선언한다. `src/application/api/client.rs`와 capabilities COMMANDS에도 proposals.list가 없다. src/tests의 proposals.list/projects.list 문자열 검색0. output_decisions 및 private state_read의 revision 열거는 proposal 목록 포트가 아니다. |
| 생성·수입 결과에서 원 host-held live 선택을 만들 재료와 typed 포트가 없다 | 반증 | ApplicationClient::create_proposal은 ProposalDocument, import_historical_proposal은 StoredProposal을 반환한다. `PendingRequest::Create/Import`와 `PendingOutput::Created/Imported`, async_proposals 등록도 존재한다. 문서에는 proposal/source identity·revision·draft가 있다. 기존 NativeProposalSelection/Target이 Card/Highlight/CueCorrection/DocumentCorrection/명시 DocumentRepair를 표현한다. 새 SDK payload 형식을 복제할 필요는 없다. |
| 위 결과가 기본 Viewer의 live chooser로 이미 전달된다 | 현 소스에서는 반증 | src/presentation의 Create/Import 제출 소비는 pending_host와 시험에 있으며 PendingReviewController의 정상 Phase는 Save/Accept/Inspect다. Created/Imported가 등장하는 output-review observation 분기는 결과 보존이며 chooser 공급이 아니다. `product_viewer_launch/project.rs`는 명시 projects.inspect 결과를 반환하고 `viewer_callbacks/selection.rs`는 ProjectCardChoices→CardSelectionQueue만 구성한다. `product_native_launch.rs`는 외부에서 이미 주어진 NativeProposalLaunchRequest를 읽는 별도 진입이다. 기본 Viewer 생성/수입 결과 handoff는 별도 연결해야 한다. |
| projects/captured guarded 편집은 SDK에서 Card 전용이다 | 반증 | `EditCapturedProposal { draft: ProposalPayload }`; capture_mutation::execute→guarded::edit→domain::edit_state→ProposalPayload::validate. validate는 Correction/DocumentCorrection/Cards/Highlights 모두 처리한다. `ProposalFiles::open_capture_mutation`→guarded::Session은 task 분기가 없다. typed client와 CaptureMutationAdapter::edit도 generic ProposalPayload를 받는다. |
| generic SDK 계약이 있으므로 captured 세 작업의 현재 UI와 CLI가 모두 검증됐다 | 반증 | 현재 CaptureCardModel::selected는 Cards 외 payload를 거부하고 Viewer 선택/표시도 Card 전용이다. 원 guarded unit fixture와 실제 standalone capture CLI 시험은 Correction이다. Highlight/DocumentCorrection captured exact CLI 양성·새 UI 입력/마운트는 이번 조사에서 근거를 찾지 못했으므로 미검증이다. 이를 SDK Unsupported로 바꾸지도, 이미 인수됐다고 표시하지도 않는다. |

## 원 소유·guard는 그대로 재사용한다

`guarded::open`은 configured ProjectStore에서 project ID/revision/전체 canonical SHA를 읽어 정확한 proposal/capture SHA를 찾고, origin 없는 legacy는 NeedsInput(create_explicit_live_capture)로 거절한다. 이후 **원 ProposalStore writer session**을 열어 origin·proposal ID·source/generated/historical immutable bytes를 대조한다. supplied CaptureExpectation은 draft revision과 전체 state hash의 CAS 기대값이며 새 권한이 아니다. expected 생략 시 immutable capture의 원 revision/hash를 쓰므로 stale capture가 current로 자동 갱신되지 않는다.

edit는 같은 writer 아래 current full-state 기대와 payload task/source refs를 검사하고 publish한다. accept는 source revision과 전체 payload를 검사하며, exact retry도 같은 writer 아래 pre-state와 결정적 post-state를 대조한다. ProposalStoreOrigin 문자열만, source3 행만, current inspect의 같은 ID/SHA만으로 이 검사를 대신할 수 없다. 저장 project의 working_draft는 로컬 초안 보존이며 live 제안 채택과 별개다. projects.save_revision은 원 project CAS를 적용하고 proposal/capture와 cross-root transaction을 보장하지 않는다.

live 진입에서도 `InitialProposalInspection`의 원 ExecutionHandle/context/operation/terminal/ACK, `InitialPanelStartup`, `project_initial`의 proposal ID·draft revision·source ID/revision/SHA·target index·cue/ref 대조를 재사용한다. 이 identity 검사는 captured provenance를 live mutation 권한으로 바꾸는 근거가 아니다.

## 기존 실제 CLI 근거와 한계

`tests/capture_mutation.rs::actual_standalone_cli_capture_edit_edit_accept_retry_and_fail_closed_expectations`는 ignore된 실제 CLI 시험이다. 파일 안 draft helper는 `task:correction`이고, invalid generated JSON에서 명시 수리→continuation edit→accept→exact retry, stale/forged/project mismatch/foreign backend/legacy 거절·project 불변을 요구한다.

원 실행 `runtime/verification/capture-mutation-cli-r1/actual.stdout.log`에서 위 exact1이 **1 passed/0 failed/0 ignored/8 filtered**, `actual.receipt.json`은 terminal=true/exit0/9305ms다. `original-cli-receipts12.json`은 원 child12의 creation identity/family Empty/forced false/stdin EOF를 보존한다. 이는 2026-10-21 이전 기준의 실제 Correction 근거이며 현재 root의 재실행이나 Highlight 양성이 아니다. 세부 BIN/input/실패 이력은 `.spec/sots/verification/application/capture-mutation-cli-2026-10-21.md`를 따른다.

## 최소 후속 계약 제안 — 현재 구현 배정 아님

### A. 먼저 기존 생성·수입 결과를 명시 handoff

새 mutation 정책·SDK 권한 발급 포트는 필요 없다. 기존 configured client가 성공한 create/import 결과를 원 host에 보유하고, 사용자가 문서와 **정확한 item 또는 repair source ref**를 명시 선택한 뒤 기존 NativeProposalSelection으로 변환한다. repair ref 없음은 Unselected로 남긴다. 결과를 받은 terminal은 원 실행 owner가 소비/ACK/release하고 기존 current inspection을 시작한다. renderer는 원 host의 bounded opaque key만 제출한다. 선택 교체 시 scope/generation을 갱신하고 오래된 key·foreign scope·queue 초과·stale inspect는 명시 거절한다. 첫 행 자동 선택·capture→live 전환은 없다.

정확한 UI 소유 후보는 이미 제안된 `src/assembly/meari/viewer_host/live_entry.rs`, `viewer_callbacks/selection.rs`, `product_viewer_launch/project.rs` 및 `src/presentation/web_ui_adapter/proposals/task_choices.rs`다. 생성/수입 결과 수신은 기존 `pending_host.rs` typed result와 원 controller/ExecutionHandle 수명 계약을 소비한다. 이 경로의 구현은 UI R1/R2/R3 별도 배정으로 남기며 SDK에 UI 선택 정책을 추가하지 않는다.

### B. 저장소의 기존 live 문서를 찾는 bounded SDK 읽기

A만으로 저장소 전체 chooser를 완성했다고 표시하지 않는다. 일반 목록은 새 read 포트가 필요하다. 최소 제안은 proposals.list의 문서 metadata page와 이후 **명시 inspect→host-held item 선택**의 두 단계다. 목록 응답에 원문/generated/draft/파일 경로/권한을 넣지 않는다. 행은 proposal ID/task/current draft revision/state/source ID/revision/SHA/typed item count 등 bounded metadata만 제공하며 revision은 선택 witness다.

제안 상한은 기본50/최대200행, 응답은 기존 transport byte budget 이내, continuation은 query와 configured dataset에 종속한다. private filesystem 열거·현재 revision 읽기·writer reservation·root identity guard는 infrastructure만 수행한다. 목록을 반환할 때만이 아니라 scan/정렬/읽기 수도 제한해야 한다. 초기 bounded scanner는 예컨대4096 후보를 넘으면 ResourceExhausted로 명시 실패하고 불완전 목록을 전체 성공으로 반환하지 않는다. 대규모 저장소 지원은 별도 index/transaction 설계 없이 약속하지 않는다. page 간 동시 수정의 snapshot 일관성·취소/close와 cursor stale 의미는 구현 전 계약으로 고정한다. 같은 cursor/row가 mutation permission은 아니다.

SDK/application exact 후보(코드 미작성):

| 경로 | 책임 |
|---|---|
| 신규 `src/application/proposals/list.rs` | strict request/page/cursor·상한·metadata 계약과 read use case |
| 기존 `src/application/proposals/mod.rs` | optional list port 기본 Unsupported·dispatch 등록. legacy provider가 강제로 목록을 지원하지 않음 |
| 신규 `src/infrastructure/proposals/list.rs` | configured 원 root/writer 아래 bounded 열거·metadata 검증·교체/손상/scan 초과 실패 |
| 기존 `src/infrastructure/proposals/mod.rs` | concrete optional list 등록만 |
| 기존 `src/application/api/client.rs`, `capabilities.rs` | typed list 호출·실제 provider별 지원 표시 |
| 기존 `src/application/proposals/schema.rs` | strict request/response·cursor·오류 계약 |
| 신규 `tests/proposal_list.rs` | 실제 CLI 목록/페이지/빈목록/부재 backend/손상/stale/상한과 명시 inspect 연결 |

목록을 Meari async consumer에 연결할 후속 exact 후보는 기존 `src/assembly/meari/async_proposals/mod.rs`, `proposal_manifest.rs`, `src/presentation/web_ui_adapter/proposals/pending_host.rs`다. 기존 worker/registry/terminal·ACK를 재사용하며 새 owner/transport를 만들지 않는다. 이 후보 목록은 SDK 읽기와 UI 연결 책임을 분리한 제안이며 한 번에 합성할 배정이 아니다. 수정·저장·Accept 정책은 기존 공통 SDK에 그대로 둔다.

## 잘못 승격할 수 있는 문구의 대조

UI 단계 문서의 “captured Card만 기존 별도 명시 guarded edit 경로가 있다”는 **현재 UI route** 범위라면 맞다. 이를 SDK 범위로 일반화하면 `pub draft: ProposalPayload`와 `Self::Correction`/`Self::Highlights`를 처리하는 domain 검증에 모순된다. “자막/Highlight captured mutation의 SDK 계약이 없다면 편집 불가로 유지한다”는 조건문은 계약 부재를 확정하지 않는다. 실제 계약은 있고 UI 소비/종류별 CLI 실증이 남았으므로 후속 UI 문서에서는 이 둘을 구별해야 한다. peer 소유 단계 문서는 직접 수정하지 않았다.

## 적용 스킬

modoo.today `modootoday/software-qa/code-change-review-verification` 본문 SHA256 `9f73865f96ec0cefcf3a4023b294400258473640df3b1d2add03dcca8192d64b`, 필수 `references/verdict-table.md` SHA256 `9d9619a829ae24d73645ef5f7c8126c8257caeddac80800632866257008ad2a9`를 실제 읽었다. R0의 목록 존재·typed 결과 handoff·Card 전용 주장별로 caller→port→guard→시험을 추적하고 반증/미검증을 분리했다. 최소 계약 제안은 PM의 별도 조사 요청 범위이며 스킬의 새 일반 리뷰로 확대하지 않았다.
