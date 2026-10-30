---
id: sot:kinekino/api
title: 제안 초안과 명시적 채택 API
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 제안 초안과 명시적 채택 API

공통 Rust domain/application 계약이다. GUI와 CLI는 같은 유스케이스를 호출한다. `reviews.*`의 표본 품질 판정은 채택 권한이나 모델 전체 승인으로 사용하지 않는다.

## 값과 불변성

`SourceSnapshot`은 source_id, source_revision(십진 문자열), source_sha256, cues(cue_id/text/검증된 선택적 start_us/end_us)를 가진다. 시간은 원문에서 확인된 경우에만 공급하며 빈 시간은 채택 후에도 빈 값이다. `SourceRef`는 cue_id만 가리키고 저장된 snapshot의 범위 밖 참조는 거부한다.

`GeneratedResult`는 task(correction/cards/highlights), raw_text를 원문 그대로 가진다. 수입 시 JSON을 엄격한 task별 `ProposalPayload`로 해석한다. correction은 edits(cue_id,text), cards는 cards(front,back,source_refs), highlights는 highlights(title,source_refs)다. 모델이 출력한 임의 timestamp는 허용하지 않는다. 실패/다른 schema/없는 참조는 typed proposal로 위장하지 않고 `needs_repair`와 import_error를 반환한다. raw_text와 source snapshot은 수정할 수 없다.

`ProposalDocument` schema_version="1"은 proposal_id, source, generated, draft_revision(십진 문자열), draft(선택적 typed payload), import_error를 가진다. 정상 초안은 `editable`, 해석 실패는 `needs_repair`, 채택 후에도 원본 document와 이전 초안은 보존한다. 유효한 사용자 수정본은 기존 모델 품질 판정과 독립적으로 채택할 수 있다.

## 명령과 typed client

| 명령 | 입력 | 출력/typed client |
| --- | --- | --- |
| proposals.create | proposal_id, source, generated | ProposalDocument / create_proposal |
| proposals.inspect | proposal_id | StoredProposal { document: ProposalDocument, acceptance: AcceptanceReceipt 또는 null } / inspect_proposal |
| proposals.edit | proposal_id, draft | ProposalDocument / edit_proposal |
| proposals.accept | proposal_id, target_source_revision | AcceptanceReceipt / accept_proposal |

edit/accept는 공통 RequestContext.expected_revision 필수다. 저장된 최신 draft_revision과 불일치하면 revision_conflict. edit는 task/sourceRefs/중복 cue 수정/빈 텍스트를 검증한 뒤 새 revision만 추가한다. accept는 최신 draft와 target_source_revision을 함께 검증하고 별도 output revision과 receipt를 하나의 불변 publication으로 추가한다. 유효한 draft가 없으면 needs_input이며 수동 수정 안내를 반환한다. 자동 채택 경로는 없다.

`AcceptanceReceipt`는 proposal_id, draft_revision, source_id/source_revision/source_sha256, output_revision, output(typed payload), automatic_application_enabled=false를 가진다. correction의 출력도 원문 파일을 덮어쓰지 않는 새 revision이다. 시간 기반 렌더링/프로젝트 덮어쓰기/모델 승인과 구분한다.

응답 소실 후 같은 expected_revision과 target_source_revision으로 accept를 재호출하면 해당 receipt가 현재 저장 상태인 동안 기존 receipt를 반환한다. 새 출력은 만들지 않는다. 이후 edit가 저장됐으면 stale 재호출은 revision_conflict다. 일반 idempotency_key/timeout_ms는 이 명령에서 지원하지 않는다. 현재 출력은 불변 typed JSON revision과 receipt이며 실제 SRT/카드 렌더 파일이나 기존 프로젝트 수정 지원을 뜻하지 않는다.

## 저장과 조립

inspect는 저장된 현재 draft와 현재 acceptance를 함께 읽는다. 재시작·재조회 시 revision 숫자로 채택 여부를 추정하지 않는다. edit 이후 acceptance는 null이며 이전 receipt는 이전 불변 revision에 보존한다. create/edit 응답은 ProposalDocument, inspect 응답만 StoredProposal 읽기 모델이다.

Application ProposalStore port가 원자적인 revision 비교와 불변 publication을 요구한다. Rust 파일 adapter는 명시적 `--proposal-root ABSOLUTE_DIRECTORY`에서만 활성화한다. 기본 backend는 unavailable. 저장 root/경로는 웹 메시지가 선택할 수 없다. CLI의 원격 endpoint와 로컬 root를 동시에 지정하지 않는다. presentation은 ApplicationClient의 typed 메서드를 사용하며 직접 파일/플랫폼 API를 호출하지 않는다.

생성 원문 수입 실패 시 UI는 needs_repair/import_error와 원문을 보여 주고 사용자가 유효한 draft를 편집·저장한 뒤 명시적으로 accept하도록 한다. 실제 9개 출력은 evidence source로만 읽으며 실패 JSON을 정상 결과로 표시하지 않는다. 기능 구현 및 실제 자료 통합 검증은 별도 관측 근거로 기록한다.

## 2026-10-20 세 기능의 SDK/CLI 인수 시나리오

현재 공개 source를 대조한 인수 계약이다. 아래 문서 v2의 준비 당시 미완료 표현과 실제 실행 상태를 구별한다. historical import·fresh inspect9개와 correction/cards/highlights 각1개 technical edit/accept/retry의 기존 실행 범위는 [제품 실행 정본](../../../verification/application/historical-provenance-product-2026-10-20.md)을 따른다. 이번 문서 보완에서 명령을 새로 실행하거나 사람 품질 verdict를 저장하지 않았다.

공개 명령은 `proposals.create/inspect/edit/accept/import_historical`이며 typed client는 각각 `create_proposal/inspect_proposal/edit_proposal/accept_proposal/import_historical_proposal`이다. historical import 입력은 `proposal_id,source_id,source_revision,reference {sha256,bytes,item_id}`이고 raw/source는 pinned verifier에서 얻는다. `StoredProposal.historical`은 document/acceptance와 함께 보존되며 bound document version3, binding version1이다. 일반 cue/document version1/2와 구분한다. 자세한 검증·설정은 [historical 계약](historical-provenance.md)을 따른다.

### 공통 실행 순서와 저장 증거

CLI는 `player-cli --input REQUEST_FILE --json --proposal-root ABS_DIR`의 full Request(`command`,`payload`,`context`)로 같은 SDK를 호출한다. historical import/bound 조회에는 명시 verifier 설정도 필요하다. 새 명령·자체 CLI 우회 정책을 만들지 않는다. `schema proposals.edit`와 capabilities는 현재 입력/adapter 지원 확인용이며 실제 수정 성공 증거를 대신하지 않는다.

| 단계 | 명령·입력 | SDK 기대 결과와 확인할 대상 |
|---|---|---|
| 비교 대상 확보 | `reviews.inspect {package_sha256}`와 `reviews.read_text_asset {package_sha256,asset_sha256}` 또는 canonical `proposals.inspect {proposal_id}` | package candidate와 `document.source/generated.raw_text`를 구분해 읽음. review 명령은 별도 명시 review root 필요. source는 human gold가 아니며 historical companion SHA는 package SHA가 아님 |
| 기존 결과 수입 | `proposals.import_historical` 위 입력, expected_revision 없음 | 검증된 원 source/raw를 가진 StoredProposal. 실제9의 NeedsRepair·원 raw 유지. generic create의 typed parse 실패도 성공 응답의 NeedsRepair이며 모델 실행 실패/정상 편집본으로 바꾸지 않음 |
| 사람 수정 저장 | `proposals.edit {proposal_id,draft}` + context.expected_revision=조회한 draft_revision | typed/참조 검증 후 새 draft revision, acceptance=null. 미저장 working draft와 구분. generated/source/historical 불변 |
| 명시 technical 채택 | `proposals.accept {proposal_id,target_source_revision}` + context.expected_revision=수정 후 revision | receipt.draft_revision은 채택한 draft, output_revision은 다음 revision. output이 선택 draft와 일치하고 자동 적용 false. 예시 revision1→edit2→accept output3은 시나리오이며 고정 값으로 하드코딩하지 않음 |
| fresh read-back·재요청 | 새 CLI `proposals.inspect`, 동일 accept 요청 재전송 | 저장된 receipt와 output·불변 원본 확인. 현재 receipt가 남아 있으면 같은 accept는 같은 receipt 반환. 이후 edit로 receipt가 해제되면 옛 요청은 stale 거부. revision 숫자만으로 채택을 추정하지 않음 |

### 기능별 수정 payload와 실패 경계

아래 C·R·U는 canonical inspect에서 얻은 cue ID·DocumentTextRef·사용자 편집 문자열의 상징값이다. raw candidate에서 임의 참조를 추출하거나 번호를 cue ID로 만들어 사용하지 않는다.

| 기능 | 공개 draft 구조와 정상 인수 | 현재 구현된 거부 | 현재 자동 보장이 없는 의미 기준 |
|---|---|---|---|
| 자막 교정 | cue source: `{task:correction,edits:[{cue_id:C,text:U}]}`. 실제 ASR document source: `{task:document_correction,edits:[{source_ref:R,text:U}]}`. R은 source_asset_sha256/outer_pointer/extraction/inner_pointer 전체를 source에서 얻음 | 없는/중복 cue ID, 다른 source asset·pointer/extraction의 R, 중복 R, 빈 edits/text, source kind·task 불일치 거부 | source의 모든 문장을 수정할 필요는 없으며 부분 edits도 유효. 전 문장 보존·허용 구두점/대소문자·ASR 오인식 교정·원음 정확도는 구조 통과로 입증되지 않음. 실제 SRT 출력/자막 적용도 별도 |
| 카드 문구 | `{task:cards,cards:[{front:U,back:U,source_refs:[C]}]}`. 원문 cue와 앞/뒷면 문장을 대조해 저장·채택 | 빈 cards/front/back/source_refs, 중복 refs, 없는 cue, 다른 task, 4096개 초과 항목/refs 거부 | ID 존재는 문장 의미 근거의 증거가 아님. supporting cue 최소성·추출/요약 목적·자연스러움·한문장·반복/제품 길이 기준은 별도. typed receipt는 카드 렌더/파일 export가 아님 |
| 하이라이트 | `{task:highlights,highlights:[{title:U,source_refs:[C]}]}`. 참조 cue의 텍스트 근거와 편집 목적을 사람이 확인 | 빈 highlights/title/refs, 중복 refs, 없는 cue, 다른 task, 4096개 초과 항목/refs 거부. start/end 등 미등록 payload 필드는 거부 | 현재 title+cue refs이며 모델 ranges/reason의 자동 의미 변환 아님. 최대2범위·시간 연속성·흥미도·음성 쉼·PTS·실제 skip는 이 validator의 보장 아님. 텍스트 선택을 미디어 구간 승인으로 승격하지 않음 |

텍스트는 현재 UTF-8 bytes 기준 최대65,536·공백만/빈 값/NUL 거부이고 각 edits/cards/highlights는1..4096이다. typed 유효성과 사람이 쓰기 적합한 길이를 구분한다. 실제 task 요청의 더 좁은 조건을 만족하지 않아도 현재 구조 검사가 허용할 수 있으므로 “검사 통과=요청 제약 모두 충족”으로 표시하지 않는다.

### 기대 실패와 미구현 계약

| 조건 | 현재 결과 / 검증 한계 |
|---|---|
| expected_revision 누락·parse 불가, create/import/inspect에 expected_revision, proposal idempotency_key/timeout_ms, unknown payload 필드 | application `invalid_argument`. startup의 잘못된 root/설정 오류와 이 wire 오류를 혼합하지 않음 |
| draft_revision stale 또는 target_source_revision 불일치 | `revision_conflict`, 새 draft/output/decision 게시 금지. 자동 최신화·재시도 후 채택 금지 |
| 유효 draft 없는 NeedsRepair 상태에서 accept | `needs_input`, remediation=`edit_proposal_draft`. 잘린 raw를 수선하거나 빈 정상 draft로 우회하지 않음 |
| 없는 source ref·잘못된 payload/task/중복·범위 초과 | `invalid_argument`; 기존 draft/acceptance/raw/source 보존. source에 있는 ref라도 의미가 맞지 않으면 이 오류를 기대할 수 없고 별도 품질 검토 대상 |
| backend 또는 historical reader 미설정 | 해당 경로 `unsupported`. 설정 없는 bound 상태를 일반 proposal로 강등하지 않음 |
| companion/source/candidate 누락·변조, binding 교환·저장 recovery 이상 | pinned reader/store의 원 오류와 cleanup_complete를 보존하고 성공/추가 publication0. 모든 경우를 not_found 하나로 고정하지 않음. 실제 tamper accept 재요청의 engine_failed도 실행 정본 범위로 보존 |
| 편집 output의 품질 decision·실제 apply/export | **현재 누락/별도 후속**. `reviews.record_decision`은 원 package/item candidate 대상으로만 사용. technical receipt나 원 verdict를 수정본 품질 승인으로 전이하지 않음 |

우선 필요한 SDK 결정은 [편집 output 판정 후속 계획](../../../../plans/player/edited-output-quality-decision.md)의 exact output target/hash/version·관측/게시 CAS·legacy candidate 호환·declared reviewer/local observed time이다. task별 의미 기준과 실제 자막/카드/미디어 apply/export는 따로 인수해야 한다. 새 endpoint 이름·DTO·오류 code는 이 문서에서 대신 확정하지 않는다.

소스 대조: `src/application/{proposals/mod,proposals/schema,ai_provenance/import,api/client,model_review/mod}.rs`, `src/domain/proposals/{mod,validation,document_source}.rs`, `src/bin/player-cli/runner.rs`, `tests/historical_provenance_cli.rs`. 기존 실제3task gate는 technical fixture이며 사람 품질 승인0이다. modoo.today `modootoday-free/ai-quality-plus/llm-output-eval-suite` 본문의 task별 기준·알려진 실패 분리를 적용했고 출처/선택 판단은 [인수표](../../infrastructure/inference/research-followups/model-quality-final-review-readiness.md)에 둔다. 새 모델 조사·코드·Cargo·브라우저·Pages·원자료 복사·추론/다운로드는0이다.

## 문서 참조 source v2 (source 구현·검증 진행)

실제 correction 입력은 cue ID/시간이 없는 ASR 문자열 배열이다. [실제 source 매핑](../../infrastructure/inference/research-followups/text-r3-source-mapping.md)을 따르며 번호를 가짜 cue ID로 만들지 않는다. source에는 원래 시간값이 있는 cards/highlights와 구분하고 해당 값을 생성 timeline으로 승격하지 않는다.

기존 cue source JSON은 그대로 읽고 쓴다. Rust의 SourceSnapshot을 untagged enum으로 확장해 Cue(CueSourceSnapshot, 기존 wire 그대로)와 Document(DocumentSourceSnapshot)를 구별한다. 문서 source는 schema_version="2", source_id/source_revision/source_sha256, asset_utf8, selector { outer_pointer, extraction }, texts [{ inner_pointer, text }]를 가진다. 기존 cue-only ProposalDocument의 schema_version="1"과 저장 bytes/경로를 유지하고 문서 source를 가진 신규 ProposalDocument만 "2"를 쓴다. 기존 source를 자동 이관·덮어쓰기하지 않는다.

DocumentTextRef는 source_asset_sha256 + outer_pointer + 명시 extraction 규칙 + inner_pointer를 묶는다. 실제 ASR 배열 위치 /0.. /5를 참조하며 cue_id/time을 만들지 않는다. outer JSON /messages/0/content, 고정 task 지시문/경계, suffix 전체 decoding의 정확한 escaping 단계는 실제 bytes를 확인한 버전 규칙으로 고정한다. '[' 검색, 임의 escape 수선, newline 정규화, 잘린 candidate 복구를 하지 않는다. 문서 bytes SHA와 selector로 추출된 실제 text가 snapshot/ref에 일치해야 한다.

구현 profile은 `asr_request_after_no_think_lf_v1`이다. 실제 Rust decode 근거에 따라 outer JSON 1회, 고정 ASR 지시문과 단일 U+000A 줄바꿈 `/no_think` 경계, suffix 전체 JSON 배열 1회를 검증한다. literal backslash+n 경계나 별도 JSON-string-body 재해석으로 fallback하지 않는다. 원문 문장 내부 literal backslash+n은 그대로 보존한다. 직접 SDK 수입도 64KiB 상한을 JSON parse 전에 확인하고 SHA2=0.10.9 순수 Rust로 원문 bytes를 검증한다(기존 잠금 package 직접 선언, 새 다운로드 없음).

문서 수정 draft는 ProposalPayload::DocumentCorrection { edits: [{ source_ref: DocumentTextRef, text }] }이며 wire task="document_correction", 기능 task()는 Correction이다. 기존 task="correction"의 cue edits와 섞지 않는다. 생성 raw task/source는 유지하고 원래 응답이 이 typed 구조와 다르면 needs_repair다. 사용자 수정본만 구조·참조·revision 검증 후 명시적으로 채택한다. 명령 4개와 inspect 읽기 모델, CAS·receipt 정책은 유지한다. 구현·기존 v1 store 읽기·실제9 수입·전체 gate·GUI 인수는 후속 증거 없이는 완료로 표시하지 않는다.

2026-10-27 후속: [subtitles.edit_cues](../analysis/cue-text-editing.md)는 cue snapshot correction 전체 초안 교체를 이 문서의 proposals.edit 정책으로 위임한다. 별도 source/revision 저장 정책을 만들지 않으며 typed client/CLI가 같은 application을 사용한다. 시각 편집·ASR 문서 교정과 구분한다.
