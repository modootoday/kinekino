---
id: plan:kinekino/20261029113628
title: 자막교정·카드·하이라이트 제품 I/O의 다음 최소 체크포인트
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 자막교정·카드·하이라이트 제품 I/O의 다음 최소 체크포인트

Status: active (PM 검토 제안). 2026-10-24 C2 반환 뒤 저자원 소스/기존 근거 읽기만 수행했다. 공유 코드/Cargo/OS/AI/다운로드0이다. 이번 제안은 **명시 채택된 timed-cue 교정본의 SRT/VTT 새 파일 내보내기** 한 유스케이스다. 모델 실행기·Accept/CAS·재열기 UI를 다시 만드는 계획이 아니다.

후속 PM domain 구현 배정으로 `runtime/development/accepted-subtitle-projection-r1/`에 private exact2/source2 `16B639708D2CBC10F602C9CD34657C2D230924F46F1E6A25AE942C16737DB398`와 순수6을 작성했다. source/before/delta/expected 및 HANDOFF의 SDK 계약을 인계한다. 기존 StoredProposal/AcceptanceReceipt/SourceCue/DecimalU64를 재사용하고 borrowed projection만 제공한다. module 등록/공유 SDK 파일 편집0, application/storage/CLI/OS 의존0, fmt/check2만 통과·타입/list/actual0이다. 원 source/accepted text/순서/정확us를 유지하며 render precision은 기존 range loss/collapse 검사를 어떻게 재사용할지 SDK/PM 구체 조율 항목으로 남긴다. 아래 구현0 표현은 최초 제안 이력이고 이 private domain 작성 범위만 갱신한다.

## Outcome

PM이 precision A안을 선택했다. 후속 SDK는 기존 range::select([0,u64::MAX))를 그대로 소비하고 새 public precision helper를 만들지 않는다. 빈 입력/ms 붕괴 거절과 기존 loss 사용자 확인을 유지한다. HANDOFF에 nonzero원점+unsorted멀티/upperMAX+loss/collapse의 새 소비자 경계 최소3조건을 명시했다. 이 결정은 소스 설계이며 source16B639/expected6·코드·시험·actual export 지원은 변경0이다.

기존 correction의 원문·생성원문·수정본·explicit AcceptanceReceipt를 그대로 둔 채, 사용자가 고정한 accepted output을 공통 Rust SDK/CLI에서 새 SRT/VTT 파일로 명시적으로 내보낼 수 있다. 입력 receipt의 proposal/output/source revision·SHA와 출력 bytes/SHA/format을 연결한다. 원 source/proposal/project를 수정하거나 자동 적용하지 않는다. card/highlight는 현재 읽기·편집·채택·저장 근거를 그대로 소비하고, 새 모델생성/카드파일렌더/영상자르기 지원은 별도 잔여로 둔다.

## Authority

| 범위 | 소유·결정 |
| --- | --- |
| 이번 조사·후속 제안/Media projection 책임 | Media, 문서만 허용 |
| typed 공통 API/CLI·dispatch/capability 등록·root index | SDK/PM, 구현 배정 미확정 |
| 기존 Accept/CAS·immutable store·file publication | 기존 SDK 계약 재사용, 정책 재작성0 |
| 모델/추론·worker·미디어 엔진·실제 자원 gate | 중앙 별도 배정, 현재 보류 |
| 사용자 품질·explicit accept | 사용자 판단, 모델 성공/기술 시험으로 대체하지 않음 |

## Evidence

2026-10-24의 정확 소스 읽기/명령어는 `rg`를 src/application/proposals, src/application/api, src/bin/player-cli, src/infrastructure/jobs.rs 및 packages/media-jobs·media-subtitles 명시 소스에 한정했다. runtime/cache/target 재귀 탐색·실제 CLI 실행0이다. 기존 SOT의 과거 fixed producer 결과를 현재 binary로 재실행한 것처럼 표시하지 않는다.

| 경로·symbol | 확인한 사실 | 제품 연결 판정 |
| --- | --- | --- |
| src/application/proposals/mod.rs::CreateProposal/create_bound | source와 **caller 제공** GeneratedResult{task,raw_text}를 받아 typed ProposalPayload로 parse. 잘못된 형상은 NeedsRepair. 원 generated는 보존 | create가 AI 추론 executor라는 주장은 refuted. candidate import/편집 시작점이지 새 생성 기능 아님 |
| src/domain/proposals/mod.rs·validation.rs | Correction{cue_id,text}, Cards{front,back,source_refs}, Highlights{title,source_refs}; DocumentCorrection도 별도. cue시간은 Optional | text/ref만으로 subtitle timing·highlight PTS/clip을 만들 수 없음 |
| src/domain/proposals/transitions.rs::edit_state/accept_state | revision 검사, 수정 시 acceptance 해제, 명시 accept와 원 snapshot source revision 대조. AcceptanceReceipt에 output/source identity | 기존 CAS/Accept 재구현 불필요. 새 export는 이미 채택한 immutable 출력 소비만 |
| src/application/proposals/schema.rs | output_scope=immutable_typed_output_revision, no subtitle/card rendering or project overwrite, automatic_application_enabled=false | 현재 채택=파일 렌더/프로젝트 적용이라는 주장은 refuted |
| packages/media-jobs/src/infrastructure/subtitle_executor/operation.rs::OPERATIONS | subtitles.convert/export_range 두 개. src/infrastructure/jobs.rs의 기본 subtitle_executor도 이 구현 | correction 생성·cards·highlights generator executor 지원으로 부를 수 없음 |
| media-jobs executor_registry, src/infrastructure/download/product_host.rs, vad_product_host/mod.rs | 명시 operation 등록, 없으면 Unsupported. 추가 download/VAD는 별도 adapter 주입 | 기존 jobs transport가 있다고 새 3task 모델 IO가 연결된 것이 아님. 조사한 기본 registry에 생성용 operation 구현 근거 없음 |
| src/application/api/capabilities.rs, src/bin/player-cli/runner.rs | export_cards/render_short 이름은 command catalog에 있으나 조사한 dispatch/기본 구현에서 실 executor 연결 근거 없음. convert/export_range와 PNG snapshot export는 별도 구현 | catalog 이름 나열≠지원. 현재 fresh capabilities는 실행하지 않아 실제 환경별 가용성은 미검증 |
| media-subtitles/domain/subtitles/{mod,render}.rs | CueDocument/Cue(시간·text), SRT/VTT render가 있음 | 텍스트를 새 포맷으로 출력하는 로직을 새로 복제할 필요 없음 |
| media-subtitles/application/storage.rs, infrastructure/subtitle_files.rs::publish_new | 기존 bounded UTF8 read 및 media_file_publication::publish 사용 | 기존 새파일 게시/overwrite 거부 경계를 재사용. 파일 CAS/저장 primitive 재구현0 |

기존 근거의 범위를 재사용한다.

- [proposal SDK/CLI](../../../sots/verification/application/proposals-2026-10-19.md): 실제9 모델 응답의 원문/원생성 SHA 보존·독립 technical-store scripted 수정→Accept→fresh CLI 재조회가 기록됐다. actual-nine input-store의 needs_repair와 모델 품질 승인false는 그대로다. 독립 과거 모델실험을 제품 내 inference 실행으로 바꾸지 않는다. source_sha는 호출자가 고정한 source 식별자이며 임의 외부 파일 재인증을 의미하지 않는다.
- [accepted output review wire](../../../sots/verification/application/accepted-output-review-wire-2026-10-20.md): typed SDK/client/CLI의 target/decision/read-back 검증. 기술적 판정 저장과 사용자 모델품질·미디어 적용은 다르다. 같은 guard/CAS 기능을 새 export에 복제하지 않는다.
- [A 공통 feedback](../../../sots/verification/presentation/three-task-review-feedback-a-2026-10-24.md)와 [B4 persisted 재열기](../../../sots/verification/presentation/persisted-review-reopen-b4-2026-10-24.md): 기존 read-only SDK/client/LocalHost의 실제 저장 snapshot 두 번 열기/feedback을 검증했다. B4 Card는 draft2/output없음, Highlight/DocumentCorrection은 draft3/output3이었다. 고정 snapshot 읽기이지 세 task 모두의 실시간 편집→채택→재열기를 새로 인수한 것은 아니다. 해당 미검증 범위를 “재열기 전부 미구현”으로 과장하지 않는다.
- [subtitle range CLI](../../../sots/verification/application/subtitle-range-cli-2026-10-20.md): common package를 통한 실제 SRT/VTT 구간 출력 근거. 기존 subtitle 변환/export의 기능을 교정 AI executor나 accepted-output exporter와 혼동하지 않는다.

## Unverified

새 `proposals.export_subtitles` 명칭/DTO·경로·소유권·소스 pin·시험 수는 제안이며 구현0이다. 현재 계약에 새 지원 명령을 등록한 것이 아니다. 전체제품의 새로운 correction/cards/highlights 추론을 공통SDK/CLI가 실제 모델 입력→출력→오류/취소→artifact까지 수행한 근거는 조사 범위에서 확인하지 못했다. 독립 historical actual9는 그 대체 근거가 아니다.

ASR 실제 문서 source에는 cue ID/검증 시간이 없는 사례가 있어 이 최소 exporter로 바로 SRT를 만들 수 없다. DocumentCorrection·시간 없는 cue·highlight source_refs의 미확인 시간은 Unsupported/명시 오류로 남겨야 한다. 시간을 추론·균등 배분·0으로 채우는 기능은 제안하지 않는다. 자막 timestamp 정밀도 손실은 기존 포맷/손실 확인 계약을 소비해야 하며 새 silent rounding 정책을 정하지 않는다.

export 중 proposal 최신 revision이 바뀔 때 의미를 먼저 확정해야 한다. 추천은 caller가 명시 고정한 **읽기 시점 검증된 immutable accepted snapshot**을 내보내는 것으로, publish 순간의 “항상 최신”이나 두 store 사이 원자 CAS를 주장하지 않는 것이다. 더 강한 최신성 요구가 있으면 기존 소유 session 계약을 SDK와 조율하며 inspect→publish를 임의 원자 동작으로 포장하지 않는다.

## Phases

**단일 구현 체크포인트: accepted timed-cue correction → 새 SRT/VTT artifact.** PM이 Media/SDK exact 소유 파일을 나눈 뒤 private Rust 후계로만 시작한다.

입력은 proposal_id와 expected accepted output/source identity(기존 revision·SHA), output_path, typed format/기존 loss 확인값이다. 기존 ProposalStore/AcceptanceReceipt를 읽어 기대 identity와 acceptance 존재를 확인하고 frozen SourceSnapshot::Cue + accepted Correction.edits를 소비한다. 원 cue 순서를 유지하며 cue_id별 채택 text만 바꾸고 수정되지 않은 cue는 원문을 유지한다. 시간은 원 snapshot의 검증된 값만 사용한다. DocumentCorrection·다른 task·없거나 stale acceptance·시간 미확인·범위/precision 정책 불충족은 게시 전에 닫는다. Accept를 자동 호출하지 않는다.

Media의 제안 책임은 작은 domain projection(예: domain/proposals/accepted_subtitle_projection.rs), SDK는 공통 application usecase와 typed client/schema/capabilities/CLI 동일 dispatch(예: application/proposals/export_subtitles.rs)다. 경로는 구현 인수 전 SDK/PM이 확정한다. renderer/CueDocument·SubtitleStorage.publish_new·media-file-publication은 기존 것을 소비하며 기존 packages를 편의상 고치지 않는다. 새 jobs daemon/모델 worker·generic framework·GUI 이벤트 정책은 필요 없다.

완료 조건은 새 read-consumer의 작은 Rust 시험(acceptance identity와 edited/unchanged cue·시간/문서 source 거절)과 **한 개의 fresh CLI 제품 IO 여정**이다: 기존 검증 fixture의 이미 accepted proposal을 열기→export→실제 새 SRT/VTT bytes/원 시간·수정text/출력SHA 확인→기존 inspect로 원 source/generated/draft/receipt가 불변임 확인. 기존 Accept/CAS 자체를 새로 호출·재시험하지 않는다. 기존 file publication 전체 fault suite를 반복하지 않고 이 소비자의 새 artifact binding/기존경로 오류 전달만 검증한다. 출력 재읽기는 원 source store의 최신성/동일 FileID 보장을 자동 의미하지 않으며 관측 방법을 gate에 명시한다. fmt/test/strict 및 CLI 실제 gate는 중앙 자원 배정 후에만 수행한다.

card export는 front/back artifact 형식 결정과 새로운 exporter, highlight는 text selection과 검증된 media range/실제 렌더 executor가 각각 잔여다. 이를 이번 checkpoint에 묶지 않는다. 새 AI generation executor는 revision/license/precision/provenance·취소/저장 계약 및 중앙 자원이 준비된 후 별도 연결한다. 지금 기술적 import/Accept를 새 모델생성 성공으로 표시하지 않는다.

## Rollback

지금은 문서뿐이라 코드/공유 상태를 되돌릴 대상0이다. 후계 구현 시 새 명령 capability/dispatch·projection·adapter wiring을 정확 변경 목록으로 철회하고 기존 proposal/store/CAS/원 renderer는 보존한다. 실제 출력 파일은 gate가 소유한 명시 새경로만 별도 inventory로 관리하며 기존 사용자 파일/원 media·모델·receipt를 삭제하지 않는다. shared index/commit/push는 PM 소유다.

적용 스킬: modoo 탐색의 최초 synthetic-training/runtime-delegation 결과는 목적과 맞지 않아 채택하지 않았다. 재탐색으로 적합한 `modootoday/software-qa/code-change-review-verification`을 확인했고 이미 읽은 본문·필수 verdict-table의 실제 caller/지원 주장/미검증 분리를 적용했다(body9F73865F…, ref9D9619A8…). 이는 주어진 “현재 연결 완료” 주장 대조에만 쓰며 별도 스타일 리뷰를 추가하지 않았다. 이미 읽은 `modootoday/spec-authoring/plan-authoring`의 outcome/authority/evidence/unverified/phases/rollback과 추정값 미기입 원칙으로 제안을 기록했다. Rust 이외 새 정책/검증 도구·모델 실험·새 실행0이다.
