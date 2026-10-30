---
id: sot:kinekino/host-2
title: 제안 검토의 제품 host 연결
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 제안 검토의 제품 host 연결

2026-10-19 제품 plugin/host 계약. [실 SDK 제안 gate](../../../verification/presentation/proposal-sdk-plugin-2026-10-19.md)에서 fmt/test7/strictClippy/build·실제9·고정CLI3task/18호출을 확인했다. source callback/UI 연결·시각 인수는 남아 있다. 새 추론 없이 기존 실제 산출물과 SDK 기능을 소비한다.

현재 구현 source는 제품 `TypedSdkPlugin`/typed binding과 공유 `src/assembly/meari/proposals.rs`다. Root DTO를 복제하지 않고 generic input/output 타입으로 host에서 주입하며 Meari registry/permission/lifecycle을 복제하지 않는다. 기존 변환 source는 유지한다. `tools/player-ui-sdk-bridge`의 ProposalBridge는 같은 registry scope에 별도 proposal plugin을 등록하고 canonical Request마다 context를 전달한다. CLI `--proposals STORE REQUEST.json`은 같은 공유 host를 사용한다. root module/Cargo 등록은 변경하지 않았다.

준비한 새 시험3개는 task별3모델로 실제9 raw를 SDK ReviewFiles/inspect_review/read_text_asset로 읽고 create/freshinspect·명시 scripted 기술 edit·stale CAS 거부·accept·새 SDK/Bridge에서 재조회한다. 원본 request/raw SHA와 UTF8를 확인하고 모든 request/result/technical-evidence를 보존하도록 작성했다. 생성 원문은 NeedsRepair를 유지하며 기술 수정 fixture가 자동 복구/사람 품질 승인을 의미하지 않는다. 품질 verdict 기록·자동 적용·새 추론은 없다. fmt check가 통과했고 plugin6+bridge7 시험/strictClippy·portable library WASM check와 고정 CLI 각task 재현을 중앙 슬롯에 예약했다.

## 정본과 연결 경계

ApplicationClient의 create_proposal/inspect_proposal/edit_proposal/accept_proposal 및 projects 계열을 직접 사용한다. 입력은 root `application::proposals::{CreateProposal,InspectProposal,EditProposal,AcceptProposal}`, 결과는 root `domain::proposals::{ProposalDocument,StoredProposal,AcceptanceReceipt}`다. 이 타입을 제품 plugin crate에 복제하거나 root를 역의존하지 않는다. root host가 canonical 타입과 SDK client를 소유하고 typed binding/codec을 주입한다. 필요한 범용 typed codec은 현재 변환 bridge 검증 뒤 별도 source freeze로 설계한다.

공통 Rust SDK가 source 검증·초안 parsing·CAS·immutable 저장·채택을 소유한다. 제품 source는 원문/제안/사람 수정/결과·오류를 표시하고 명명된 입력만 전달한다. 자동 JSON 복구·가짜 시간·자동 채택·SDK 정책 재구현을 추가하지 않는다.

| 단계 | SDK 계약 | UI/host 연결 |
| --- | --- | --- |
| 가져오기 | CreateProposal: source snapshot + generated task/raw_text | 검증된 기존 request/output asset을 원문 그대로 참조. create로 즉시 edit/accept하지 않음 |
| 비교 | InspectProposal → StoredProposal | source/sha/revision, generated raw, draft/import_error/state, acceptance를 구별 |
| 사람 수정 | EditProposal + context.expected_revision | 선택한 cue/document text/card/highlight의 안정 ID·source_refs를 보존하는 canonical draft 전달 |
| 명시 채택 | AcceptProposal + expected_revision + target_source_revision | 사용자가 검토한 해당 proposal revision에만 호출. 충돌 후 자동 재채택 금지 |
| 프로젝트 저장 | 공통 project Client | proposal receipt를 현재 미디어에 묵시적으로 재결합하지 않음. 정본 project binding/CAS 계약을 별도 소비 |

create/inspect는 expected_revision을 받지 않고 edit/accept는 이를 요구한다. proposal 명령의 idempotency_key/timeout은 SDK에서 거절한다. host가 context 옵션을 몰래 제거하거나 버튼 클릭을 자동 retry하지 않는다. target_source_revision은 저장된 immutable source snapshot과 대조되는 값이며 현재 player 소스 상태를 입증하지 않는다.

## 데이터와 표시

Correction/Cards/Highlights는 ProposalTask이며 각 payload는 같은 모양이 아니다. Cue source의 correction은 cue_id/text, Document source correction은 canonical document selector/text reference, cards는 front/back/source_refs, highlights는 title/source_refs를 유지한다. 첫 행/현재 cue/현재 프로젝트로 식별자를 대체하지 않는다. 원문과 draft가 같은 문자열이라고 자동 확인 상태로 만들지 않는다.

NeedsRepair는 import_error와 generated.raw_text를 별도 표시한다. raw output을 정식 typed 초안처럼 표시하지 않는다. draft=None이면 기존 SDK의 NeedsInput/수정 경로를 사용한다. 새 revision 응답은 요청을 보낸 proposal/document scope에만 적용하고 사용 중인 다른 초안의 textarea를 덮어쓰지 않는다. pending 중 선택 이동·돌아오기·stale 결과의 UI 인수는 후속 중앙 슬롯에서 검사한다.

from_request_asset는 cards/highlights의 cue 시간을 None으로 보존한다. 원본 request에 나타난 시간 문자열에서 새 PTS를 추정하거나 실제 영상 재생 구간으로 승격하지 않는다. 표시 시간 미상과 연결 실패를 구별한다. UI read model에 필요한 실제 source range/provenance 확장은 SDK 담당과 조율한다.

AcceptanceReceipt의 automatic_application_enabled는 false다. 채택은 immutable 출력 receipt이며 자동 player 적용·파일 내보내기·모델 품질 승인을 의미하지 않는다. 현재 source의 GeneratedResult는 task/raw_text만 보유하므로 model revision/license/precision·review artifact/output SHA provenance의 공용 연결 경로를 SDK 담당에게 요청했다. 없는 필드는 만들어 채우지 않는다.

## 후속 인수

공통 읽기 모델·작업 중 초안·응답 scope는 [검토 입력 계약](proposal-view-input.md)에 둔다. 새 상태 회귀 4개와 기존 SDK 7개가 같은 consumer gate에서 통과했다. 실제 화면 입력·시각 검증과는 구별한다.

실행 완료 범위는 기존9 raw 전부의 NeedsRepair/raw/SHA 보존·명시 scripted 기술수정·stale CAS 거부·accept·새 SDK 재열기와 task별1개 고정 CLI fresh process/store다. 이것을 실제 사용자의 편집·품질 승인으로 표현하지 않는다. 다음은 동일 host를 correction/CARD/CLIP의 공통 Rust read model과 입력으로 연결하는 source checkpoint다. Meari callback 미지원은 원본/진단/필요binding/인수로 본체 담당과 조율하며 JS 정책/runtime으로 우회하지 않는다.

기존 실제9근거를 정확히 선택한 뒤 원본 asset identity/SHA를 보존해 source→비교→사람 수정→명시 채택을 같은 Meari registry와 실제 ApplicationClient로 검사한다. 오류는 ApiError 전체와 request context를 보존한다. stale expected revision·target source mismatch·NeedsRepair·중복 사용자 호출/범위 변경·동시 host 문맥을 구별한다. SDK의 기능 회귀를 GUI 클릭으로 대체하지 않는다.

제품 source 연결 대상은 자막 correction inspector, CARD 문구, CLIP highlight 추천이며 공통 비교/초안/명시 채택 책임을 공유한다. 이3경로를 연결해도 전체13화면군의 다른 기능/레이아웃·키보드·자산·시각 인수는 남는다.

modoo.today frontend-first-journey에서 읽은 typed port/서버 confirm/wiring 상태 구분과 design-to-code-fidelity-diff의 표시/실행 증거 분리를 적용한다. 프로젝트의 SDK/CLI 선행·Rust·품질 미승인 규칙이 우선한다.
