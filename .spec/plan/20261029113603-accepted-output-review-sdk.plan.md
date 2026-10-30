---
id: plan:kinekino/20261029113603
title: 편집 출력 품질 판정의 SDK 최소 단계 제안
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 편집 출력 품질 판정의 SDK 최소 단계 제안

**현재 상태(2026-10-20):** 아래 내용은 초기 설계 이력이다. 현재 typed3 명령의 endpoint/schema·저장·SDK/CLI 구현은 [wire 검증 기록](../../sots/verification/application/accepted-output-review-wire-2026-10-20.md)에 따라 wire18, 실제 CLI10 및 96호출·strict 검증까지 인수했다. 아래의 “미구현/후보” 표현은 당시 제안 상태이며 현재 구현 상태를 뜻하지 않는다. 재구현하지 않고 기존 typed 명령을 UI에서 소비한다.

남은 범위는 사용자의 실제 출력 품질 판정과 그 UI 소비·시각 인수다. transport/저장 성공은 품질 승인 자체가 아니다. 외부 historical actual-nine 매체의 실제 품질 판정·완전 인수를 이 SDK/CLI gate가 입증하지 않으며, [요구와 인수 사례](edited-output-quality-decision.md) 및 위 검증 기록의 지원/미검증 한계를 유지한다.

2026-10-20 SDK 읽기 검토안. [요구와 인수 사례](edited-output-quality-decision.md), [현재 proposal API](../../sots/player/application/api/proposals.md)를 따른다. 아직 endpoint·schema 인수가 끝나지 않았으며 코드·검사·실제 판정 저장을 수행하지 않았다. worker wake 수리와 동시에 구현하지 않는다.

## 대상과 해시

첫 버전의 쓰기 대상은 현재 canonical inspect에 있는 acceptance로 제한하는 것을 제안한다. 과거 output에 대한 새 판정은 explicit historical revision reader가 생기기 전에는 지원하지 않는다. target은 proposal ID, task, accepted draft revision, output revision, source ID/revision/SHA 및 선택적 historical reference를 묶는다. 같은 output hash라도 다른 proposal/source/revision이면 다른 대상이다.

`accepted_typed_output_v1`은 receipt.output의 typed payload만 해시한다. rendered media나 exported file bytes 해시가 아니다. 별도 버전 envelope와 정렬된 object key·기존 array 순서·원문 UTF-8/줄바꿈 보존·고정 JSON 숫자/문자열 규칙을 명시해야 한다. 기존 `projects::canonical_sha256`은 현재 serde 직렬화 바이트의 hash이므로 이름만 보고 새 계약의 canonical 규칙으로 재사용하지 않는다. output hash에 reviewer/note/time을 넣지 않는다. full target identity와 output hash를 함께 비교하고 caller hash를 canonical output에서 재계산한다.

## 최소 쓰기 경계

현재 ProposalStore의 load와 별도 ReviewSession.append를 연달아 호출하면 그 사이 edit/CAS 경합이 생긴다. 두 저장소를 순차 호출하는 구현은 사용하지 않는다. 같은 proposal writer reservation을 유지한 review session port가 current state·acceptance·historical evidence를 다시 확인하고 immutable decision publication까지 소유하도록 한다. verifier의 성공은 해당 관측 경계의 증거이며 영구 유효성이나 신뢰 서명이 아니다.

제안 명령은 `proposals.inspect_output_review_target`, `proposals.record_output_decision`, `proposals.output_decisions`의 독립3개다. 이름·schema는 인수 전 후보이며 기존 `reviews.*` candidate API/bytes를 수정하지 않는다. 첫 단계는 현재 대상의 record와 immutable read-back만 맡고 project 자동 저장·과거 target 기록·export/apply는 추가하지 않는다.

중복 decision ID는 proposal 범위에서 동일 target·verdict·note·criteria identity·declared reviewer·선택 claimed time이면 기존 기록을 반환하고 불일치는 충돌시키는 안을 제안한다. 동일 요청도 현재 대상이 바뀌었다면 먼저 stale을 거부한다. 재요청 때 observed time을 새로 만들지 않는다. criteria는 version과 content identity를 함께 결합하고 바뀌면 재제시한다.

## 판정과 관측

human-edited output에 ModelCondition을 만들거나 원 candidate의 조건을 새 모델 성능처럼 복제하지 않는다. declared reviewer label/ID와 선택 claimed review time은 요청 정보이고, adapter local observed time은 저장 관측 정보다. 인증 신원·외부 신뢰 시각·모델 admission은 아니다. 과거 candidate record에 없는 필드는 absent로 남기고 파일/SHA를 재작성하지 않는다.

usable은 해당 exact output에 대한 명시 판정이며 technical acceptance·자동 적용·모델 품질 승인과 별개다. 기존 candidate automated checks/usable을 새 output 판정에 상속하지 않는다. 저장된 output decision의 존재와 현재 evidence 재검증 결과도 별개로 표시한다.

## 소스와 검증 순서

1. domain `proposals/output_review/{target,hash,decision}.rs`: exact identity·버전 해시·판정 불변식. 기존 candidate 타입과 파일을 보존한다.
2. application `proposals/output_review/`와 기존 ProposalStore의 선택적 session port: 현재 대상 제시·writer/CAS 경계·명시 unavailable·중복 계약. SDK/typed client/schema/CLI dispatch 등록은 이 계약 인수 뒤 별도 단계다.
3. infrastructure `proposals/output_reviews/`: 기존 proposal root의 writer guard를 재사용해 canonical 재검증과 immutable append/read-back을 한 소유 세션으로 제공한다. 새 root/모델 데이터/개인 미디어를 추가하지 않는다.
4. Rust 단위/저장 시험 후 fresh CLI로 요구된10사례를 검사한다. stale edit, 같은 hash의 다른 proposal, evidence 손실, 기준 변경, 동일 ID 충돌, 원 candidate bytes 보존을 포함한다. GUI와 추론을 중복 실행하지 않는다.

현재 미확정 항목은 session port의 실제 서명, 오류 wire code, exact criteria 제공 경로, optional claimed time 형식, 판정 기록 read-back 때 historical evidence를 재검증하는 범위다. 저장 결과와 관측 종류를 먼저 확정한 뒤 한 단계씩 구현한다.

## 실제 포트 대조와 좁힌 기술 제안

2026-10-20 NativeChild gate 마감 뒤 읽기 검토다. 아래 서명·경로·wire는 구현 배정을 위한 제안이며 새 지원 API로 등록한 상태가 아니다. Cargo·OS·media·추론·새 판정 저장은 수행하지 않았다.

### 확인된 현재 경계

| 실제 소스·symbol | 관측 사실 | 확장에 미치는 영향 |
|---|---|---|
| `src/application/proposals/mod.rs::ProposalStore` | `available/historical_reader/load/publish`만 있다. load·publish를 함께 소유하는 session port는 없다. | 선택적 포트를 추가해야 하며 기존 구현은 기본 미지원으로 남긴다. |
| `src/infrastructure/proposals/mod.rs::ProposalFiles::{load,publish}` | configured root의 `writer.lock`을 각각 취득한다. publish는 revision CAS와 원 source/generated/historical 불변성을 검사한다. load는 revision1과 최신의 불변 필드를 비교하고 bound evidence를 검증한다. | 같은 lock을 보유한 세션에서 public load를 다시 호출하면 lock 재취득 충돌이 난다. lock 없는 private 읽기 helper를 추출해 공유해야 한다. 현재 두 메서드의 단순 연결은 사용하지 않는다. |
| `src/infrastructure/immutable_storage_paths.rs::lock` | Windows share_mode(0), 경합은 RevisionConflict이며 rooted guard/reparse 거부가 있다. | 기존 root writer 예약을 재사용한다. 별도 lock이나 자동 retry로 CAS 정책을 바꾸지 않는다. |
| `src/domain/proposals/stored.rs::validate` | acceptance output revision은 document draft revision과 같고 accepted draft+1이며 payload와 document draft가 대응한다. automatic_application_enabled는 false다. | expected revision은 채택 전 draft가 아닌 현재 output/document revision으로 명시한다. edit하면 acceptance가 해제되므로 stale 판정이다. |
| `GeneratedResult` 및 `immutable_equal` | task/raw_text는 revision1과 최신 사이 불변이다. generic 생성에는 job/attempt/model/package ID가 없다. | 원 generated UTF-8 bytes SHA/길이를 별도 identity로 계산할 수 있다. 관측되지 않은 모델/실행 identity는 만들지 않는다. |
| `HistoricalBinding` / `VerifiedHistoricalSample` | companion SHA/bytes/item과 source/candidate bytes의 검증 연결이 있다. 공개 verified sample에는 review package SHA/item asset 관계가 없다. | companion을 package로 명명하지 않는다. 원 package lineage는 미지원/absent이며 확인할 새 포트 없이 caller 주장으로 채우지 않는다. |
| `application/model_review::{ReviewSession,record}` / `infrastructure/model_review/session.rs::append` | package/assets guard를 소유하고 별도 review root에 게시한다. 같은 decision ID는 기존에도 충돌이며 reviewer/time 필드는 없다. | accepted output의 idempotent 계약을 여기에 소급 추가하지 않는다. 이 session은 proposal writer guard를 소유하지 않는다. |
| `application/api/{client,capabilities,mod}.rs`, `proposals/schema.rs`, `bin/player-cli/runner.rs` | proposal 5명령과 candidate reviews 명령만 등록돼 있다. CLI는 공통 SDK Request를 전달한다. proposal idempotency_key/timeout은 거부되고 edit/accept만 expected_revision을 요구한다. | 새3명령·typed client·schema/capabilities/dispatch를 각각 추가해야 fresh CLI가 지원한다. 현재 CLI 지원을 주장하지 않는다. CLI 전용 정책이나 GUI 자동화는 필요 없다. |

### 선택적 세션 포트 제안

기존 ProposalStore에 다음 기본 메서드를 더하는 안이다. 반환 객체는 UI나 OS authority가 아니며 메서드 내부 writer guard 수명을 가진다. 실제 구현 시 object safety와 lifetime은 Rust 검사로 확정한다.

```rust
fn open_output_review<'a>(
    &'a self,
    proposal_id: &str,
) -> Result<Option<Box<dyn OutputReviewSession + 'a>>, ApiError> {
    Ok(None)
}

fn output_decisions(
    &self,
    proposal_id: &str,
) -> Result<OutputDecisionHistory, ApiError> {
    Err(/* Unsupported: output decision reader unavailable */)
}

trait OutputReviewSession {
    fn current(&self) -> &StoredProposal;
    fn verify_current_evidence(&self) -> Result<(), ApiError>;
    fn append(
        &mut self,
        decision: &PreparedOutputDecision,
    ) -> Result<StoredOutputDecision, ApiError>;
}
```

`open_output_review`는 기존 root writer lock을 취득하고 private 최신+revision1 읽기/불변 검증을 수행한 상태를 소유한다. application은 같은 세션에서 state.validate→current acceptance·expected output revision→target/hash/generated/source/historical identity→trusted criteria 대조→evidence 재검증 순서로 확인한다. `append`도 선택 target과 session snapshot의 결합을 대조하고 게시 직전에 bound evidence를 재검증한 뒤 불변 판정을 게시한다. guard는 결과 파일 재읽기·digest 대조까지 유지한다. 세션을 유지한 동안 public load/publish를 호출하지 않는다. 내부 helper는 예를 들어 `read_current_locked`와 `append_output_decision_locked`처럼 guard-held 계약을 명시하고 guard 참조를 요구한다. 기존 load는 lock 취득 뒤 같은 읽기 helper를 호출하도록 추출한다. 기존 publish의 CAS/불변 검증도 lock 취득과 guard-held 본문을 분리하되 새 결정 append가 기존 proposal revision publish를 호출하거나 revision을 증가시키지는 않는다. 이 추출은 infrastructure 단계에만 배정하고 순수 domain 첫7 범위에 넣지 않는다.

writer guard가 보호하는 것은 이 root를 사용하는 proposal edit/accept/publication이다. 외부 historical 파일 전체나 verifier 프로세스가 같은 lock을 쓰는 것은 아니다. evidence 재검증은 게시 직전의 검증 관측이며 외부 파일의 영구 불변성이나 모든 저장소의 전역 트랜잭션을 증명하지 않는다. 필요한 외부 asset guard가 공개되지 않은 현재 포트로 그보다 강한 원자성을 주장하지 않는다.

판정 저장은 같은 configured proposal root 아래 별도 `output-decisions/p-{proposal_id}/d-{decision_id}/` namespace를 제안한다. 기존 `p-{proposal_id}` 안에 새 디렉터리를 넣으면 현재 revisions parser의 `r-N` 전용 검사를 깨므로 피한다. 새 configured root나 기존 StoredProposal/AcceptanceReceipt의 필드를 추가하지 않는다. root writer guard가 이 namespace도 보호한다. decision body는 schema/version+내용 해시로 불변 게시하고 `StoredOutputDecision`은 기록 SHA와 typed body를 반환한다.

동일 ID 재요청도 현재 target·criteria/evidence 검증부터 수행한다. 그 다음 기존 요청 identity를 비교해 같으면 원 기록/observed time을 반환하고 다르면 RevisionConflict다. observed time은 중복 비교 입력에서 제외하고 최초 게시에만 부여한다. 사전 validation/CAS/evidence 실패는 publication0이다. 디스크 I/O가 게시 중 실패한 경우에는 파일이 없다고 단정하지 않고 원 오류·잔존 경로/commit 관측을 보존한다. 불완전 게시나 digest 불일치는 수동 복구가 필요한 NeedsInput으로 읽기를 차단하며 자동 덮어쓰기·자동 삭제하지 않는다. 실제 atomic publication helper와 crash 경계는 저장 단계에서 별도 검사한다.

### hash·criteria·시각·read-back 제안

- output hash v1은 version envelope와 receipt.output을 대상으로 compact UTF-8 JSON, object key의 명시 정렬, array 순서 유지, 문자열 무정규화/원문 줄바꿈 보존을 사용한다. 현재 typed payload는 문자열·배열·object와 DecimalU64 문자열을 사용하므로 JSON float canonicalization을 도입하지 않는다. JSON number가 생기는 미래 schema는 새 hash version을 요구한다. RFC canonicalization 준수라는 이름을 붙이지 않는다. generated identity는 원 raw_text UTF-8 bytes의 SHA/길이와 task를 함께 결합한다.
- criteria는 caller 임의 문구/해시가 아니라 Rust에 포함된 task별 versioned catalog의 전체 typed 내용(`criteria_id/version/task/purpose/clauses`)을 inspect 응답에 제시하고 content hash를 계산하는 안이다. 저장 시 catalog에서 재취득해 전체 identity를 대조하며 decision에 사용한 전체 내용을 보존한다. correction 의미 보존/허용 편집, cards supporting cue/중복/길이, highlights 선정 목적/근거를 분리한다. 현재 proposal에는 사용자별 선정 목적이 없으므로 첫 버전은 명시한 고정 purpose만 지원하고 임의 목적별 승인으로 확대하지 않는다. task별 정확한 기준 문구·version은 구현 전 PM/AI 인수가 남는다. 자동검사 결과나 candidate 기준을 그대로 복제하지 않는다.
- target kind는 항상 `accepted_output`으로 고정하고 모델 sample 집계 제외를 명시한다. accepted만으로 human-edited라고 이름 붙이지 않는다. raw generated와 output typed 비교나 edit 호출은 실제 사람의 편집 증명이 아니므로 이 단계에는 human provenance를 생성하지 않는다. 원 candidate 모델 조건·sample_review_passed를 상속하지 않는다.
- declared reviewer는 label/선택 ID를 요청 그대로 보존한다. claimed time은 선택적 DecimalU64 Unix milliseconds, observed time은 adapter가 최초 게시 시 읽은 local system Unix milliseconds와 `local_system_unix_ms` 출처를 저장하는 안이다. clock이 epoch 이전이거나 범위를 벗어나면 게시 전에 명시 실패한다. 둘은 인증 신원·신뢰 시각·monotonic 완료 시간이 아니다. 도메인은 OS clock을 읽지 않는다. 새 후보 API의 시간 규칙만 고정하며 legacy 시간/검토자를 발명하지 않는다.
- immutable read-back은 proposal 최신 상태나 외부 evidence가 없어져도 저장된 decision의 body/digest/identity 검사를 수행해 기록을 반환하는 별도 reader다. `OutputDecisionHistory { records, current_evidence: not_checked }`처럼 기록과 현재 미관측 상태를 명시적으로 분리하는 새 응답을 제안한다. 이 응답에 현재 evidence_verified=true를 저장하거나 유추하지 않는다. 현재 target inspect는 기존 canonical bound 검증을 수행하므로 evidence 누락 때 원 오류를 낸다. 저장 기록 읽기와 현재 재검증은 별도 호출로 구분한다. project capture/reopen에 새 결정을 자동 포함하지 않는다.

### 오류·wire 경계 제안

| 조건 | 기존 ErrorCode로 좁힌 제안 |
|---|---|
| 형식/ID/필수 note·reviewer·시간 불량 | InvalidArgument |
| adapter/session 미지원, hash/schema version 미지원 | Unsupported |
| proposal/명시 decision 부재 | NotFound |
| 현재 acceptance 부재, incomplete publication/corrupt digest | NeedsInput + 명시 remediation |
| expected output revision·target/source/generated/hash/criteria 불일치, 동일 ID 다른 요청, writer 경합 | RevisionConflict |
| 제한된 history/body 용량 초과 | ResourceExhausted |
| verifier/파일권한/실행/시각/I/O 오류 | 원 adapter 오류 분류를 보존하며 품질 verdict로 변환하지 않음 |

새 wire 이름은 후보3개를 유지한다. inspect target/list stored decisions는 expected_revision/idempotency_key/timeout_ms를 허용하지 않는 독립 계약을 제안한다. record만 expected_revision=현재 output revision을 필수로 하고 decision_id가 도메인 중복 identity를 맡는다. transport idempotency_key/timeout_ms는 계속 미지원이다. 이 처리를 기존 proposals.execute의 일괄 context 검사에 억지로 넣지 않고 새 output_review dispatch가 명시적으로 담당하도록 한다. legacy proposals/reviews 요청·응답 bytes/schema는 변경하지 않는다.

### 첫 구현 exact 범위 제안 — 배정 전

첫 단계는 순수 domain 계약7파일로 한정하는 안이다. 신규 `src/domain/proposals/output_review/{mod.rs,target.rs,hash.rs,criteria.rs,decision.rs,tests.rs}`와 기존 `src/domain/proposals/mod.rs`의 최소 등록만 변경한다. target kind/receipt binding/generated hash, 고정 hash version, criteria 전체 내용 identity, declared/observed 메타데이터와 verdict 불변식을 맡는다. wire/clock/storage/application/CLI 변경은0이며 endpoint 지원 완료로 표시하지 않는다. 기준 문구 인수가 없으면 criteria 내용은 versioned 값 객체와 불변식까지만 구현하고 승인 문구를 지어내지 않는다.

다음 application 단계는 `src/application/proposals/output_review/{mod.rs,ports.rs,tests.rs}`와 기존 `application/proposals/mod.rs`의 선택적 기본 포트다. 그 후 infrastructure 세션/reader/publication과 기존 ProposalFiles private helper 추출, 마지막 SDK typed client/schema/capabilities/dispatch/fresh CLI10사례를 별도 exact 범위로 배정한다. 새 port를 기존 store에 덧붙일 때 기존 fake adapter가 default 미지원으로 컴파일되는 회귀가 필요하다. 한 번에 미확정 전체 wire를 구현하지 않는다.

현재 기술적으로 좁힌 것은 writer reservation/namespace/현재 target CAS/중복 ID/시간 출처/기록과 재검증 분리다. 남은 인수는 task별 criteria 정확 문구·고정 purpose, endpoint/DTO/schema version의 최종 명칭과 제한치, 단계별 exact 배정이다. 원 review package lineage는 기존 검증 포트가 제공하지 않으므로 첫 버전 absent라는 지원 경계로 유지한다. native parent-ready/surface와 전체 SDK/CLI 목표는 후속에서 계속 유지한다.

## 첫 domain7 소스 준비 상태

PM 배정에 따라 위 순수 domain exact7을 작성했다. 아직 Cargo/typecheck/회귀 실행은 하지 않았다. 기존 StoredProposal/candidate review DTO·wire·저장소·CLI·dependency는 변경하지 않았다. criteria 값 객체는 schema/version/task/purpose와 clause ID/검토방법/기대내용 전체를 보존하고 전체 내용 해시로 결합한다. 제품 승인 criteria 문구는 이 단계에 포함하지 않는다.

PM이 [AI criteria 초안](../../sots/player/application/accepted-output-review-criteria.md)의 제한된 텍스트 근거·의미·가독성 purpose를 첫 catalog 설계안으로 인수했다. 이는 전체 기능/미디어/사용자 목적별 품질 목표 축소나 실제 사용자 품질 승인이 아니다. 정확 문구의 Rust catalog 이식은 application 후속 배정으로 남긴다. 첫 단계에는 clause별 outcome wire를 추가하지 않고 필수 note로 문제 위치·미확인 사항을 남기는 계약을 유지한다. 필수 note가 의미 있게 작성됐다는 사실을 domain 문자열 검사만으로 증명하지 않는다.

새 단위 회귀8은 세 task의 accepted-only/model 집계 제외, 고정 hash envelope/문자열 bytes/array order/숫자 거부, 같은 output의 다른 proposal/source/generated 교환, acceptance 해제·hash version·historical 교환, criteria 전체 내용/version/task/purpose와 중복 clause, declared/observed·동일 요청 비교, 불량 메타데이터/자동적용·집계 flag 거부, 기존 proposal 직렬화의 비변경을 다룬다. 합성 receipt와 criteria를 사용하며 외부 evidence 검증·실제 품질 승인·legacy 파일 전체 byte 회귀를 실행한 것으로 표시하지 않는다. 이후 focused Rust 실행과 strict/fmt는 별도 heavy 슬롯에서 수행한다.

PM 소스 검토 뒤 r1 사본/pin `0C26B0B78D60CB6A27D85B37901CA6E852AC3FC8EF08F44D3E330C05665F56AE`를 보존하고 exact7의 r2로 독립 저장 검증을 보강했다. `OutputDecision::validate_stored`는 현재 proposal/evidence 없이 record schema·target ID/SHA/version/revision 관계·criteria 전체내용 identity/task·note/reviewer·scope/자동적용/모델집계 불변식을 검사한다. target과 input도 각각 순수 stored 검증을 제공한다. prepare/current binding 검증은 이 검사 후 실제 state와의 결합을 확인한다. payload/source bytes가 없는 stored 검증으로 원본 해시를 재계산하거나 현재 evidence verified를 주장하지 않는다. scope/clock은 닫힌 enum이라 알 수 없는 값은 deserialize에서 거부된다.

회귀는 총10개가 됐다. 추가2개는 state가 소멸한 뒤 typed record 독립 readback과 current acceptance 부재의 별도 실패, deserialize된 record의 bool/criteria/target SHA·revision·ID/reviewer/note/schema 변조 및 scope/clock 교환 거부다. r2 역시 source/직접 rustfmt 검사만 수행했으며 Cargo/typecheck/회귀 실행은 미배정이다.

`creating_domain_decision_preserves_existing_proposal_and_candidate_contract_bytes`라는 현재 시험 이름은 실제 단언보다 넓다. 이 시험의 byte 단언은 **동일한 in-memory StoredProposal의 전후 직렬화가 같음**만 뜻한다. candidate 관련 단언은 새 decision JSON에 candidate/model 필드가 없다는 범위이며 기존 candidate schema·파일·SHA·CLI 전체 호환을 입증하지 않는다. 해당 호환은 후속 기존 reviews schema/명령 및 저장 read-back 회귀에서 확인해야 한다. 지금 소스 동결을 유지하므로 시험 이름 변경도 별도 배정 전에는 하지 않는다.

## 다음 application optional session의 exact 후보

현재는 읽기 제안이며 구현 배정이 아니다. domain7 r2를 전제로 application의 최소 **exact4**를 제안한다.

- 기존 `src/application/proposals/mod.rs`: `pub mod output_review` 등록과 ProposalStore의 optional session/독립 기록 reader 기본 메서드만 추가한다. 기존 create/inspect/edit/accept/execute 및 wire dispatch는 변경하지 않는다.
- 신규 `src/application/proposals/output_review/mod.rs`: application 포트/메타데이터 re-export와 미지원 error 생성, 세션 취득의 작은 경계만 둔다. catalog·clock·명령 DTO/dispatch는 이 단계에 넣지 않는다.
- 신규 `src/application/proposals/output_review/ports.rs`: object-safe `OutputReviewSession`, 준비 입력/저장 기록 wrapper/독립 history·current_evidence:not_checked 값 계약을 둔다. `PreparedOutputDecision`은 domain `OutputDecisionInput`과 전체 `CriteriaDocument`를 보존하며 observed time은 caller가 채우지 않는다. 실제 adapter가 최초 게시 시 observed time을 부여하고 domain prepare를 호출하는 책임을 명시한다. guard-held snapshot lifetime은 세션 객체가 맡는다.
- 신규 `src/application/proposals/output_review/tests.rs`: 기본 미지원과 trait-object/lifetime/세션 소유 경계 회귀만 작성한다. 실제 filesystem/writer/CAS 지원은 가짜 세션으로 입증했다고 표시하지 않는다.

구체 회귀 후보는 (1) 기존 형태의 `LegacyStore`가 load/publish만 구현해도 컴파일되고 `open_output_review` 기본값 None, (2) 독립 기록 reader 기본 Unsupported, (3) 세션 미지원 경로에서 load/publish/evidence 호출0, (4) UnavailableProposalStore와 `&dyn ProposalStore`에서도 같은 지원 경계, (5) borrow된 fake session이 store lifetime 안에서만 생성되고 session Drop 전까지 원 snapshot/owner를 유지하는 관측이다. 이것은 실제 lock 재진입/CAS/crash 회귀의 대체가 아니다.

현존 ProposalStore 구현은 ProposalFiles/UnavailableProposalStore, application projects의 Proposals fake, ai_provenance의 MemoryStore, bridge tests의 Files2종/RefreshFailureOnce다. 이 exact4에서 이들 구현을 편집하거나 기본 미지원을 지원으로 바꾸지 않는다. 이후 root 검사와 별도 bridge 컴파일/기존 회귀로 default 메서드 추가가 기존 consumer에 미치는 영향을 확인한다. 지금 그 검사들을 실행하지 않았다.

task별 catalog의 정확 문구 이식·clock adapter·실제 세션/immutable publication·SDK/schema/CLI는 각각 후속 배정이다. application exact4가 끝나도 새 endpoint와 저장 기능은 미지원 상태다. 현재 Meari heavy 진행 중이므로 SDK Cargo 실행은 기다린다.

PM의 별도 배정 후 application exact4 소스를 작성하고 직접 rustfmt 검사를 통과했다. optional method는 `Box<dyn OutputReviewSession + '_>`로 store borrow를 유지하며 trait object 경로를 가진다. 기본 adapter는 session None/reader Unsupported이며 session 취득 helper는 load/publish/historical_reader를 호출하지 않는다. 독립 history는 current_evidence:NotChecked를 명시한다. trait 계약과 borrow된 fake session 회귀는 실제 filesystem lock/CAS/crash 보장이 아니다.

application 회귀5와 domain 회귀10은 아직 Cargo/typecheck/실행하지 않았다. 별도 슬롯 검증 후보는 root lib의 `domain::proposals::output_review::tests` 정확10과 `application::proposals::output_review::tests` 정확5를 각각 실행하고 root all-target strict/fmt를 수행하는 것이다. 기존 구현의 기본 미지원 호환은 root application의 projects/ai_provenance 회귀와 bridge consumer 컴파일을 후속 범위에 맞춰 확인한다. 지금 bridge나 다른 producer 소스는 변경하지 않았다. domain7 r2의 동결 원본·핀과 시험 이름 불일치 추적은 그대로 유지한다.

## 실제 파일 어댑터의 책임·범위 후보

다음은 SDK11 동결 중 읽기 검토이며 새 코드 배정이 아니다. 기존 `ProposalFiles::read/revisions/load/publish`, rooted storage guard, `media_file_publication::publish`와 proposal 파일 회귀를 대조했다. domain/application 의미 판정과 파일 경로·직렬화·게시를 한 session에 모으지 않는다. 아래 단계들은 각 입력이 인수된 뒤 별도 exact 배정한다.

### 선행 application provider 후보 exact5

실제 adapter가 caller 기준을 신뢰하지 않도록 trusted criteria 제공과 clock 출처를 먼저 정해야 한다. 신규 `src/application/proposals/output_review/catalog.rs`는 PM 인수된 좁은 텍스트 purpose의 세 task 기준 전체 내용·version을 제공하고, 신규 `clock.rs`는 `OutputReviewClock::observe_record_time() -> Result<ObservedRecordTime, ApiError>` 포트만 둔다. 기존 `output_review/mod.rs`의 최소 등록과 `tests.rs`의 하위 시험 등록, 신규 `tests/providers.rs`를 합쳐 exact5 후보이다. 기준·시각 요청 DTO나 wire 지원은 추가하지 않는다. domain에 OS clock이나 문구 catalog를 넣지 않는다. clock 오류와 criteria 재제시·내용 hash 회귀는 이 단계의 책임이다. 기존 domain7/app4 원본은 별도 인수핀으로 계속 보존한다.

### infrastructure 후보 exact12

| 경로 (`src/infrastructure/proposals/` 기준) | 단일 책임 |
|---|---|
| 기존 `mod.rs` | 하위 등록, ProposalFiles의 provider 조립과 optional trait 구현 위임. 기존 load/publish 외부 결과·bytes 계약 유지. |
| 신규 `reservation.rs` | 기존 root `writer.lock`과 rooted guard를 보유하는 private WriterReservation. 생성 경로를 제한하고 root 연관을 기록해 다른 store의 guard 교환을 거부. |
| 신규 `state_read.rs` | 기존 revisions/read/current+revision1 불변 검사를 추출. `read_current_locked(&WriterReservation, id)`로 호출하며 외부 evidence 재검증 여부는 명시 인수/별도 함수로 분리. public load의 lock 재취득 없이 session에서 재사용. 기존 publish CAS가 쓰던 읽기 검증도 공유하되 proposal revision publication 정책은 변경하지 않음. |
| 신규 `output_reviews/mod.rs` | open session/독립 history reader의 adapter 진입·조립. optional 지원 여부를 실제 provider 준비 상태와 결합. |
| 신규 `output_reviews/session.rs` | reservation+borrowed store+canonical snapshot+trusted providers 수명. 준비 입력/현재 revision/target/기준/evidence를 application/domain 함수로 대조하고 publication에 위임. 경로 구축·JSON 해시·파일 쓰기 구현은 넣지 않음. |
| 신규 `output_reviews/paths.rs` | 같은 configured root의 output-decisions/p-/d- 경로, ID·경로·reparse/ordinary file guard, 없음을 안전하게 관측. 기존 p-/r- namespace는 건드리지 않음. |
| 신규 `output_reviews/records.rs` | 제한된 record/history 읽기, raw body digest·파일명·decision/proposal/ID 결합·역직렬화·domain validate_stored. 반환 순서와 불완전 publication 탐지. 현재 proposal/evidence를 읽지 않음. |
| 신규 `output_reviews/publication.rs` | 사전 직렬화/크기 제한 후 새 d-ID 예약, 불변 파일 게시·sync·원파일 readback·잔존 오류. 정책 verdict·clock·자동 재시도는 없음. 기존 media-file-publication Rust API 사용을 우선하고 해당 공통 패키지는 이 범위에서 수정하지 않음. |
| 신규 `output_reviews/clock.rs` | application clock 포트의 실제 SystemTime/UNIX_EPOCH→checked u64 milliseconds와 LocalSystemUnixMs 구현. epoch 이전/overflow는 오류. |
| 신규 `output_reviews/tests.rs` | 공통 Rust 임시 root/fixture와 정상·중복·변조·독립 readback 검사, 하위 시험 등록. |
| 신규 `output_reviews/tests/recovery.rs` | 예약 후 crash 단계·불완전 파일·게시 후 오류·복구 요구의 증거 보존. |
| 신규 `output_reviews/tests/concurrency.rs` | 실제 두 ProposalFiles handle과 기존 writer.lock을 사용하는 session/CAS 경합. |

이 exact12는 후보이며 임의 파일 추가를 허용하는 포괄 범위가 아니다. 파일 분할이나 기존 tests.rs 편집이 추가로 필요하면 경로를 먼저 인계한다. application provider와 SDK11 검증이 선행돼야 하며, 저장 adapter가 지원돼도 새 SDK wire/CLI 명령 등록은 별도 단계다. 기존 candidate reviews adapter/bytes·CLI는 이 범위에서 편집하지 않는다.

### 예약에서 게시까지

1. 같은 root의 WriterReservation 취득 후 최신+원본 불변 읽기와 필요한 bound evidence 검증. accepted target가 없으면 NeedsInput, expected output revision/전체 identity가 다르면 RevisionConflict. 새로운 helper를 통해서만 읽으며 public load/publish를 호출하지 않는다.
2. trusted catalog의 task/purpose/내용 identity와 prepared 요청을 비교하고 domain input을 재검증한다. catalog 교환·현재 수정은 게시 전에 거부한다. target·criteria·evidence 재확인 없이 동일 ID의 예전 성공을 반환하지 않는다.
3. bounded history를 검사하고 같은 ID가 완전한 기록으로 존재하면 저장된 요청 전체와 비교한다. 동일하면 최초 body/SHA/observed time 그대로 반환하며 clock 재호출0·추가 게시0. 다르면 RevisionConflict. history가 가득 차도 기존 동일 요청 재조회는 허용한다.
4. 신규 ID만 adapter clock을 관측하고 domain OutputDecision을 준비·검증한다. body bytes/크기·SHA를 계산한 뒤에만 namespace/d-ID를 생성한다. 기록 파일 SHA는 실제 저장 body bytes이며 typed output hash/criteria hash와 서로 다른 identity다.
5. immutable publication과 같은 reservation 내 원파일 재읽기/digest+구조 검증 후 반환한다. 검증 후 namespace 생성 전에 evidence를 다시 확인하고 그 외부 관측의 한계를 기록한다. 다른 proposal writer는 이 예약에 막히지만 외부 evidence 전체를 이 lock이 잠그는 것은 아니다.

bounded history는 proposal별 최대256개 decision directory, record body 최대1MiB를 첫 후보로 둔다. domain의 최대32개 clauses·각 문구 제한 및 JSON escape 팽창을 포함한 직렬화 bytes를 사전 검사한다. 초과는 ResourceExhausted이며 새 d-ID 예약0이다. 정확 제한치는 파일 단계 인수에서 확정한다. 읽기는 결정 ID의 byte 정렬로 고정하고 시간순/최신 승인 의미를 만들지 않는다. 유효 ID인데 namespace가 없으면 독립 history는 빈 records+NotChecked를 반환한다. 기존 proposal 부재를 확인하려고 최신 state를 열지 않으며, 앞서 제안한 일반 NotFound 분류는 명시한 개별 기록 조회나 current target 조회에 적용한다.

### crash·복구와 독립 read-back

기존 proposal publication은 r-N 디렉터리 예약 후 crash가 나면 incomplete state로 읽기를 차단한다. 새 판정도 d-ID 예약 후 같은 보존 원칙을 따른다. 단, 새 namespace만의 incomplete 때문에 기존 proposal revision 읽기를 깨뜨리지는 않는다. decision history/해당 append는 NeedsInput과 `recover_output_decision_publication` 같은 전용 remediation을 반환한다. 복구는 수동 절차 후 재검증이며 자동 삭제/덮어쓰기/성공 추정은 없다.

`media_file_publication::publish`는 temp write+sync 뒤 hard_link로 새 파일을 게시하고 temp 제거를 시도한다. 파일을 게시했지만 temp cleanup이 실패했거나 프로세스가 멈춘 상태는 단일 정상 파일과 구분해야 한다. directory가 비어 있음, temp만 있음, 최종 파일 부분 bytes, 여러 파일/추가 파일, SHA 불일치, 올바른 SHA지만 typed schema/criteria/target 불량은 모두 완전 기록 성공으로 읽지 않는다. hard_link 성공 뒤 readback/permission 작업 오류가 발생하면 원 오류와 `publication_may_exist`/phase/잔존 경로 관측을 보존하고 새 ID 자동 retry를 하지 않는다. 완전한 기존 기록이라면 다음 동일 요청은 그 원 record로 수렴할 수 있다. power loss 후 filesystem durability나 모든 crash 위치를 fault injection만으로 보증했다고 하지 않는다.

독립 history는 reservation을 취득해 publication과 경합을 정리하되 proposal snapshot/historical reader/catalog/clock을 요구하지 않는다. 저장 body digest와 `validate_stored`만 검사하고 current_evidence:NotChecked를 반환한다. source/evidence가 사라진 뒤에도 기록이 완전하면 원 identity와 판정을 읽는다. 현재 대상/기준 재검증은 별도 session/current inspect 경로이며 오류를 history 없음이나 새 성공으로 바꾸지 않는다. catalog가 바뀌어도 과거 criteria_document를 현재 문구로 덮지 않는다.

### 실제 저장 검증 후보

| 경계 | 필수 관측 |
|---|---|
| 기존 load/publish helper 추출 | 기존 proposal 정상/수정/CAS/historical 변조 회귀 유지. root writer 예약 중 private 읽기는 성공하고 public 두 번째 writer는 원 RevisionConflict. 재진입·중복 lock 취득0. |
| 실제 current CAS | 두 handle에서 target 선택 후 edit/accept 교체 → 이전 prepared append 거부/publication0. session 보유 중 별도 edit는 writer 경합으로 거부, drop 뒤 재시도 edit 성공. |
| 같은 ID의 동일 요청/충돌 | 정상 append→fresh adapter readback→동일 요청은 body/SHA/time 동일·clock1회. note/reviewer/claimedtime/target/criteria/verdict 중 하나가 다르면 기존 body 불변. stale 현재 target면 동일 요청도 먼저 거부. |
| 역직렬화 변조 | 같은 크기 body 변조는 파일 SHA에서 거부. 파일 SHA까지 다시 맞춘 불량 schema/bool/criteria/target는 domain stored 검증에서 거부. 모든 해시를 일관되게 재작성한 공격의 인증 방어는 주장하지 않음. |
| evidence·catalog 손실 | 제시 후 저장 전 bound evidence 누락/동일크기변조·criteria 교환 → 원 오류와 새 decision0. 저장 뒤 evidence 손실 → history는 원 기록+NotChecked, 새 session/current 검증은 실패. |
| crash 단계/복구 | d-ID 예약 직후/partial write/temp만/게시 직후 readback 오류를 보존하고 incomplete 목록을 정상 기록으로 숨기지 않음. retry는 overwrite0, 수동 복구 요구·원 오류/phase 보존. |
| 제한/경로 | history256/body1MiB 경계, invalid ID/reparse/다른 root guard 교환·unexpected entry 거부. 용량 사전 실패는 예약0. |
| 원 candidate 보존 | 명시한 legacy package/object/decision fixture의 전체 bytes/SHA before/after, 기존 candidate schema/SDK 읽기 유지. 이 파일 회귀로 CLI 전체 호환을 주장하지 않고 fresh CLI 단계에서 기존 reviews 호출을 별도 확인. |

이 단계도 verdict의 실제 의미 품질을 합성 입력으로 승인하지 않는다. 실제 filesystem 보증은 위 Rust 저장 회귀의 관측 범위로만 보고한다. 모델 성공집계/미디어 적용/자동 저장/apply/export를 추가하지 않는다.

## Provider5 소스와 합집합14의 검증 대기

PM 배정으로 provider exact5를 작성하고 직접 rustfmt --check를 통과했다. catalog는 인수된 criteria 문서의 세 task purpose와 task별4 clauses를 그대로 보존한다. clause 제목은 how_to_review, 해당 전체 문단은 expected에 넣고 안정 ID를 부여했다. caller의 self-consistent criteria hash/문구도 현재 compiled catalog 전체 identity와 다르면 RevisionConflict로 재제시한다. clock은 Send+Sync application 포트뿐이며 SystemTime 구현·게시·wire는 없다.

provider5 원본/pin은 `runtime/coordination/sdk-output-review-providers5-r1/`, pin JSON SHA256은 `5D3BD4603A0606F576534234225440EA70EB76405E4C17DBB73F4B49045FE1D2`다. domain7 r2와 application4 r1의 원본/pin은 보존했다. 기존11 중 application output_review/mod.rs와 tests.rs 두 경로만 provider 등록으로 변경했으며 domain7과 app의 나머지2 경로는 원 핀과 동일하다. 신규3과 중첩2를 합친 정확14의 독립 원본/pin은 `runtime/coordination/sdk-output-review-combined14-r1/`, pin JSON SHA256 `5A7FC3696BA87F150DDD29BA35E68632EC42B1E44BF832083F501140D7BBBF63`이다. 실제 union 경로 수14를 직접 확인했다. 기준 문서 SHA256은 `B13FD2BFFF656DD1E42F5640228D08DDD478988F1E5902947C7674D56A38BBB0`이다.

새 provider 회귀4는 세 task의 전체4clauses+내용 identity/readback, caller 문구·purpose·version·task·ID 교환의 trusted catalog 거부, local observed clock 포트의 선언정보 부재, clock 원 오류/phase/remediation/request ID 보존이다. 포트 fake만으로 OS clock·실제 publication·동일 retry clock1회 보장을 입증하지 않는다. 기준 문구를 읽었다는 사실이나 catalog 회귀는 실제 사용자 품질 승인이 아니다.

UI의 원실행/strict/명시 반환과 고정 Meari1df·최종UIr5 보호핀을 인수한 뒤에만 별도 snapshot revision에 exact14를 overlay할 계획이다. 실행 중 snapshot 변경0이며 현재 Cargo 슬롯은 미배정이다. 기본 계획은 다음과 같다.

1. root default lib에서 `cargo test -p kinekino-player --offline --locked -j 1 --lib output_review:: -- --test-threads=1`로 union19를 한 번 실행한다. full test name을 보존해 domain10/application기존5/provider4를 분리 확인하고 총19를 중복 합산하지 않는다.
2. 기존 store consumer의 동작은 `application::projects::tests` 2개와 `application::ai_provenance::integration_tests` 1개를 root default에서 별도 집중 실행한다. 새 판정 시험19와 기존 호환3을 합쳐 새 품질 성공22라고 세지 않는다.
3. root default all-target strict와 native-proposal-window feature-on all-target strict로 optional ProposalStore·typed client/GUI consumer를 두 문맥에서 검사한다. native feature에서 같은19를 반복 실행할 필요는 현재 없으며 컴파일 범위와 동작 실행을 구분한다. cargo fmt --check를 별도 기록한다.
4. bridge는 기존 `original_review_sdk`, `document_repair_sdk`, `native_document_sdk` test target을 `cargo test --manifest-path tools/player-ui-sdk-bridge/Cargo.toml --offline --locked -j 1 --no-run`에 각 `--test`로 한정하는 컴파일 호환 후보이다. bridge root의 native feature 전달이 필요한지 PM 범위에서 확정하고 동일 입력핀을 쓴다. no-run은 기존 실제CLI/파일/렌더 회귀 통과를 뜻하지 않으며 ignored historical 실제 실행을 활성화하지 않는다. 별도 캐시와 단일 heavy owner 순서를 지킨다.

각 원실행 handle/receipt·cwd·runner 제한과 before/final14·보호 base delta·PID 종료를 보존한다. UI Inspector의 통과/실패 사실을 SDK 필터 결과로 바꾸지 않는다. gate는 배정 후 실행하며 실패가 생기면 원 입력/결과를 보존하고 최소 exact 수리를 별도 revision으로 인계한다.

### union14 실행 마감

PM 슬롯 배정 후 동결14 그대로 새19(10+5+4), 기존projects2/provenance1, root default/native all-target strict/fmt, bridge native 기존3target no-run을 통과했다. source수리0·최종보호delta0·원실행7 terminal0/14PID부재를 확인하고 heavy/snapshot을 반환했다. [실행 정본](../../sots/verification/application/accepted-output-review-contracts-2026-10-20.md)을 따른다. 위 실행 전 대기 서술은 당시 계획 기록이며 현재 재실행 권한이 아니다. 실제 저장/clock/CLI 지원·사용자 품질 승인과 구별한다.

### infrastructure12 소스 준비 — 미실행

검증된14는 커밋 `c49425c81f6dede4556e34cc8dd42281431b69de`로 분리했고 원핀을 유지한다. 후속 PM 배정에 따라 위 exact12를 작성·직접 rustfmt 검사했으며 Cargo/typecheck/실제 저장 회귀는 아직 실행하지 않았다. 새 파일 경로나 dependency·wire/CLI 변경은 없다.

구현은 root 연관 WriterReservation과 guard-held state 읽기, lifetime session, rooted decision paths, 제한된 raw body/digest·domain stored 검증 reader, 새 ID의 불변 publication/readback, 실제 local SystemTime adapter로 분리했다. 기존 proposal revision 형식/직렬화·CAS 정책은 유지하며 읽기 helper를 공유한다. 현재target/trustedcatalog/evidence→기존ID 동일request 반환→신규clock→불변게시 순서이며 중복 retry는 clock을 다시 읽지 않는다. 독립 history는 current proposal/evidence/catalog/clock 없이 NotChecked로 읽는다. readback은 방금 게시한 body의 예상 SHA와 typed decision 전체까지 대조한다.

새 회귀16은 정상/fresh history/원 proposal bytes와 retry clock1회, ID 충돌·self-consistent caller criteria 거부, clock 실패 예약0, proposal 소실 뒤 독립 history·정렬, capacity256/새ID거부, historical evidence 누락·동일크기 raw 교환·동일IDretry 재검증, 명시 legacy fixture bytes 비변경, invalid ID/unknown entry, 실제 두handle writer 경합·stale current·다른root guard, empty/partial/temp/extra publication 복구, digest 및 재해시된 typed bool 변조, 예약/게시 후 fault injection, bytebound, 방금 게시한 body의 교환 거부를 다룬다. fixture/실패 checkpoint는 Rust 시험용이며 OS crash/power-loss 보증이나 실제 품질 승인이 아니다. legacy fixture byte 회귀는 candidate schema/CLI 전체 호환을 대체하지 않는다.

검증 후보는 새 `infrastructure::proposals::output_reviews::tests` 정확16과 기존 `infrastructure::proposals::tests`(historical 포함) 회귀, root default/native all-target strict/fmt다. 기존SDK14 회귀와 bridge 컴파일을 반복할지는 helper 추출/타입변경의 실제 영향에 맞춰 별도 배정한다. 현재 heavy는 Meari이므로 이 source를 snapshot에 복사하거나 Cargo를 실행하지 않는다. 판정 저장 어댑터는 소스만 준비된 상태이며 새 endpoint/CLI 지원은 여전히 없다.

#### 동결12 이후 저장 gate 목록 — 실행 미배정

원 infrastructure12 r1 pin JSON SHA256은 `C323E584DE32280253E6B8CF185C0ACC62A7FAC190998F5C7E3D03A2C8F74FA9`이며 `runtime/coordination/sdk-output-review-infrastructure12-r1/`에 source/paths를 보존했다. 독립 읽기 검토와 PM 검토 후 확정 수리가 필요하면 이 원본을 덮지 않고 exact12 안의 r2로 분리한다. 추가 경로는 먼저 인계한다.

| 검사 후보 | 확인할 수·범위 |
|---|---|
| root default lib `infrastructure::proposals::output_reviews::tests` | 신규16. 실제 두handle writer·currentCAS·저장/readback·보존오류와 한계를 시험명별 확인. |
| root default lib `infrastructure::proposals::tests` | 기존6 + historical1 = 7. state_read 추출 전후 load/publish 원 source/generated/receipt·CAS1winner·partial/tamper·문서 source 보존·원 historical 결합을 검증. 신규16과 별도 집계. |
| root default lib `application::proposals::output_review::tests::providers` | 기존4를 trusted catalog/clock port 호환으로 재실행할 후보. clock port 오류는 epoch/overflow 실제 OS 상태를 재현한 결과가 아님. 신규 성공표본으로 중복 합산0. |
| integration `--test proposals` | 기존1 actual CLI/typed port·proposal schema/capabilities·revision 지원 계약. 새 output-review CLI 명령 지원을 뜻하지 않음. |
| integration `--test model_review absent_backend_schema_context_and_missing_material_are_explicit` | 기존 candidate schema/지원 경계1. candidate wire를 output decision으로 바꾸지 않았음을 기존 contract에서 관측. |
| integration `--test model_review actual_cli_inspects_records_and_reads_same_immutable_decisions_as_shared_sdk` | 기존 candidate SDK/CLI record/readback1. 신규 fixture byte 비변경 시험의 제한을 이 기존 실제 경로와 구분. |
| root default/native all-target strict + fmt | helper/새 adapter가 두 feature 문맥의 기존 consumer에 미치는 컴파일·lint·형식 영향. |

기존 `historical_provenance_cli::actual_nine...`는 explicit verifier/offline actual-nine 전용 ignored 실행이며 이 저장 gate가 자동 활성화하지 않는다. bridge의 기존3target은 SDK14 gate에서 native no-run으로 이미 컴파일했고, 이번에는 application trait 서명이 그대로이므로 반복 no-run은 새 실패/실제 영향이 있을 때 PM이 별도 지정한다. 전체 suite 통과나 모든 candidate bytes의 전역 무변경을 작은 focused gate로 주장하지 않는다.

표의 test 수는 소스에서 확인한 기대값이며 실제 test log와 다르면 조사한다. 모든 Cargo는 PM의 다음 heavy/snapshot 배정 뒤 표준 runner/offline locked-j1/threads1로 순차 실행한다. 새16/기존7/provider4/기존CLI3의 범위와 반복을 분리하고 합성 저장 시험을 사람 품질 승인·전체 CLI 지원으로 집계하지 않는다.

#### infrastructure12 저장 gate 마감

별도 배정 후 r1의 clock 가시성 E0603을 exact12 한 줄로 수리한 r2에서 실제 저장16·기존proposal/historical7·기존schema/CLI3과 양feature all-target strict/fmt를 통과했다. r1 실패·원source/pin을 보존했고 r2 pin은 `48CA503F0417B5A8E2298254E36DB49DA8E8588B961B300F73BA02D14610DE0C`이다. [저장 실행 정본](../../sots/verification/application/accepted-output-review-storage-2026-10-20.md)에 검사·한계·실행9/PID18 종료/보호delta0·명시 반환을 기록했다. 신규 wire/CLI는 여전히 미지원이며 후속 배정을 따른다.

## 공통 SDK/wire/CLI 연결의 exact18 제안 — 소스 배정 전

저장12는 커밋 `cd6ec8117c092a49298241d7a25c54fbd2145301`로 분리했다. 다음은 기존 계획의 후보3명령을 공통 SDK에 연결하는 정확 범위 제안이다. 현재 새 wire 코드/Cargo/제품 UI 변경은 없다.

### DTO와 명령 경계

| 명령 | payload / context | 응답 제안과 검사 |
|---|---|---|
| `proposals.inspect_output_review_target` | `{proposal_id}`. expected_revision/idempotency_key/timeout_ms는 미지원 | `OutputReviewInspection {schema_version, state:StoredProposal, target:OutputReviewTarget, criteria_document, criteria_identity, evidence_observation}`. 한 canonical session/writer 예약 안에서 같은 state의 target+trusted criteria를 계산하고 bound 검증을 확인한 뒤 clone해 반환. state의 source/generated/draft/acceptance로 원자료 비교가 가능하며 서로 다른 load 결과를 섞지 않음. |
| `proposals.record_output_decision` | `{proposal_id, decision:OutputDecisionInput}`. expected_revision=현재 output/document revision 필수. decision_id는 proposal 범위의 도메인 중복키. transport idempotency_key/timeout_ms 미지원 | `StoredOutputDecision {decision_sha256, decision}`. route proposal_id와 decision.target.proposal_id 일치, context revision과 target output/current revision 대조→trusted criteria→PreparedOutputDecision→같은 session append/readback. caller observed time 필드는 없고 unknown 필드로 거부. |
| `proposals.output_decisions` | `{proposal_id}`. expected_revision/idempotency_key/timeout_ms 미지원 | `OutputDecisionHistory {records, current_evidence:"not_checked"}`. 현재 proposal/evidence/catalog/clock 없이 원 stored body/digest/metadata를 반환. 정상 빈 namespace는 빈 records이며 원 proposal을 새로 읽지 않음. |

inspection의 evidence_observation은 문자열 enum `not_historically_bound` 또는 `historical_binding_reverified_in_session`처럼 현재 반환 경계를 명시하는 새 DTO 값으로 제안한다. 후자는 실제 session.verify_current_evidence 성공 후에만 생성한다. 전자는 historical binding이 없다는 뜻이며 source의 독립 정답/모델 품질/외부 사실을 검증했다는 뜻이 아니다. 저장 record에 verified=true를 넣지 않는다. 관측 시간·인증 신원·지속 유효성을 새로 주장하지 않는다. schema와 typed DTO에 이 scope를 명시한다.

record에서 missing/malformed expected_revision은 InvalidArgument, unsupported context는 Unsupported, route/target·revision/hash/source/generated/historical/criteria 교환과 동일ID 다른요청은 RevisionConflict를 제안한다. 현재 revision이 맞지만 acceptance가 없으면 NeedsInput이다. 원 adapter의 evidence/권한/I/O/복구 오류는 그대로 보존한다. schema/hash version 미지원은 domain의 일반 mismatch와 섞지 않고 application이 먼저 Unsupported로 분류한다. 사용자 note는 필수이나 문구가 실제 검토를 증명한다고 표시하지 않는다.

### 기본 미지원과 legacy 보존

ProposalStore에 `output_review_available() -> bool { false }` 기본 capability를 추가하고 검증된 ProposalFiles만 true를 반환하는 안이다. optional None인 기존 store를 단순 available=true 때문에 지원으로 표시하지 않는다. capability query는 세션을 열거나 load/evidence를 호출하지 않는다. 실행은 capability 확인 후 optional session None/reader Unsupported도 그대로 거부한다.

global COMMANDS에는 새3개를 추가하되 기존 proposals::is_command/schema/execute와 reviews schema/body는 그대로 둔다. API dispatch는 새 output_review 명령/schema 분기를 기존 proposal 분기 앞에 별도로 둔다. capabilities에는 새 명령의 status와 별도 `output_reviews` 객체만 추가한다. 기존 `proposals` 객체/기존 명령 schema·응답 필드는 보존하지만 global capabilities 전체 bytes는 새 항목 때문에 달라지며 전체 byte 동일을 주장하지 않는다. persisted StoredProposal/AcceptanceReceipt/candidate 파일을 migration하지 않는다.

CLI runner는 현재 공통 Request→configured SDK execute 경로를 이미 사용하므로 새 명령별 정책 분기를 만들지 않는다. 기존 --proposal-root/명시 historical verifier 설정을 그대로 사용하며 help에 새 지원·제한을 기술한다. GUI 조작이나 추가 host/root/자동 retry는 없다. SDK typed client도 같은 Request DTO/결과를 사용하는 얇은3 wrapper다.

### 필요한 작은 adapter 계약 조정

현재 file session open이 acceptance 부재를 먼저 NeedsInput으로 거부한다. edit 후 옛 revision으로 record하면 이 검사 때문에 current CAS를 먼저 분류할 수 없다. 해결안은 session을 canonical structural state+writer reservation의 소유로 유지하고, open에서 acceptance 필수 검사만 제거하는 것이다. bound evidence 검증은 유지한다. inspect 유스케이스는 target 구성 때 acceptance를 요구하고, record는 현재 document revision과 expected를 먼저 비교한 뒤 acceptance/target를 요구한다. append의 기존 full target/criteria/evidence 검증은 유지하므로 no-acceptance 상태가 판정 성공으로 바뀌지는 않는다.

이 조정에 따라 기존 concurrency 회귀의 "edit 뒤 session open 실패" 단언을 "session은 구조 state를 소유하지만 acceptance 없음; 새 유스케이스 inspect/record는 명시 실패"로 바꾸는 명시 수정이 필요하다. 원16 결과/핀은 보존한다. 오류 메시지 전체를 잡아 일괄 RevisionConflict로 바꾸는 우회는 사용하지 않는다.

### 정확18파일 책임 후보

| 경로 | 책임 |
|---|---|
| `src/application/proposals/mod.rs` | optional 지원 capability 기본 false 추가만. 기존5명령 정책 유지. |
| `src/application/proposals/output_review/mod.rs` | 새 모듈 등록/re-export/is_command 및 wire entry 위임. |
| `src/application/proposals/output_review/ports.rs` | 새 stored/history wire의 serde+snake_case 정의. 기존 저장 body의 domain schema는 변경하지 않음. |
| 신규 `src/application/proposals/output_review/types.rs` | 세 요청과 inspection/evidence observation DTO, unknown 필드 거부. |
| 신규 `src/application/proposals/output_review/usecases.rs` | 동일 session inspect/current CAS/prepare/append/history·context validation/dispatch. I/O·clock·catalog 문구는 복제하지 않음. |
| 신규 `src/application/proposals/output_review/schema.rs` | 새3 schema/capability 주석과 타입. 기존 source/payload/state schema는 공개 legacy schema helper를 참조하며 수정하지 않음. |
| `src/application/proposals/output_review/tests.rs` | 하위 wire/schema 시험 등록. |
| 신규 `src/application/proposals/output_review/tests/wire.rs` | optional unsupported·context/CAS·typed dispatch·observation 책임 회귀. |
| 신규 `src/application/proposals/output_review/tests/schema.rs` | DTO/schema/context/지원 표시와 기존 명령 경계 회귀. |
| `src/application/api/mod.rs` | 새3명령/schema 분기와 별도 capabilities 위임. |
| `src/application/api/client.rs` | 공통 typed client3 wrapper. |
| `src/application/api/capabilities.rs` | 새3명령 discovery 목록 추가. |
| `src/infrastructure/proposals/mod.rs` | 파일 adapter의 새 capability=true. |
| `src/infrastructure/proposals/output_reviews/session.rs` | open의 acceptance 필수 검사만 usecase로 이동. |
| `src/infrastructure/proposals/output_reviews/tests/concurrency.rs` | 위 session 단언의 책임 이동을 정확히 반영. |
| `src/bin/player-cli/runner.rs` | help에 새3명령/지원 제한·명시 root·quality/technical 차이 기술. 정책 dispatch 추가0. |
| 신규 `tests/output_review_cli.rs` | fresh CLI 인수10사례. |
| 신규 `tests/output_review_cli/fixture.rs` | Rust 임시 root/프로세스 I/O·요청/응답/exit·파일hash/수량 관측 지원. 제품 정책 복제0. |

새 dependency/Cargo 변경·제품UI·기존 candidate reviews 소스 수정은 없다. 18 경로 외 helper가 필요하면 먼저 인계한다. 이 표는 구현 배정 전 제안이며 검증된14/12 원사본과 혼합하지 않는다.

### fresh CLI10 인수 사례 제안

모든 fresh 명령은 별도 player-cli 프로세스/같은 rooted store를 사용한다. 합성 입력이며 실제 사용자 usable 판정이 아니다. 정상 판정의 fixture verdict는 deferred를 기본으로 두고 usable 의미 승인을 생성하지 않는다.

1. 세 task의 create/accept→inspect target→record→fresh history에서 exact target/criteria 전체내용/SHA/verdict/note 일치와 automatic=false/modelcount=false 확인. cue correction과 document correction의 task 대응은 domain/application 회귀와 구분한다.
2. 동일 output hash의 다른 proposal에 route/target를 교환해 거부하고 decision0·원record 불변 확인. hash 동일만으로 대상 동일 처리0.
3. inspect 후 edit로 acceptance 해제, 옛 expected revision record는 RevisionConflict. 새 inspect는 NeedsInput. 재accept 뒤 옛 target를 latest로 재연결하지 않음.
4. criteria version/hash/purpose/task를 각각 교환하고 caller가 전체 문구 hash를 다시 맞춰도 compiled catalog와 다르면 거부. 현재 기준 재조회·전체내용 보존.
5. target output hash/payload publication 교환·미지원 hash version을 거부. 새로운 decision 수량과 원 bytes 불변 확인.
6. source revision/SHA/generated identity/companion reference 교환을 거부. companion과 package SHA를 바꾸어 부르지 않음.
7. 성공 inspect 뒤 canonical proposal publication 파일을 누락/동일크기로 변조해 record를 거부하고 저장된 history는 원기록+NotChecked로 읽기. 이 fresh CLI 사례는 **canonical publication 손실** 관측이다. 외부 historical 파일이 실제로 사라지는 end-to-end 재현은 명시 pinned verifier가 필요한 별도 gate이며 SDK 저장 회귀의 fake evidence 손실 결과와 구분한다. ignored actual-nine을 자동 실행하지 않는다.
8. 동일 decision ID/전체 동일 요청의 fresh retry는 원 SHA/body/observed time, 추가publication0. note/verdict/reviewer/claimedtime을 바꾼 같은 ID는 conflict. current target가 달라졌으면 동일 요청도 먼저 stale 거부.
9. declared reviewer label/ID와 claimed time, adapter observed local time/clock 출처가 fresh history에서 구분되고 caller observed 필드 주입은 unknown field로 거부. local time을 인증·신뢰시각으로 표시0.
10. configured/미configured capability·schema와 context expected 필수/transport 지원거부를 확인하고 기존 proposals/reviews schema/응답 비교를 별도 수행. candidate schema와 sample_only 기록 bytes는 migration되지 않으며 새 output target는 모델 sample 성공으로 합산0.

새 CLI10과 기존 회귀·가짜 session·실제 파일16은 각각 관측 범위를 기록한다. source·metadata·잘못된 입력의 여러 변형을 한 사례로 묶더라도 로그의 실제 호출/시험 수를 남긴다. 최초 gate는 새 application/wire/schema 회귀+freshCLI10, 기존 저장16/7·기존CLI/schema3, 양feature strict/fmt를 제안하며 실제 heavy 순서와 정확 기대 수는 source 준비 후 별도 배정한다.

### wire18 소스 준비 — Cargo 미실행

PM 배정에 따라 위 정확18을 작성하고 직접 rustfmt --check를 통과했다. `runtime/coordination/sdk-output-review-wire18-r1/{paths.txt,pins.json,source/}`에 보존했고 pin JSON SHA256은 `55E7D2D73DBF24CF021490E49B6A8BBBB12AAA30EDA9FA5565C73A42499127EF`이다. source-only 상태이며 컴파일·unit·fresh CLI 실행·실제 사용자 판정 저장은 아직 수행하지 않았다. root index·제품UI·추론/미디어/actual-nine 변경은0이다.

구현은 승인된3명령·동일 canonical session inspection·record current output CAS·독립 NotChecked history, 기본 false capability/파일 adapter true, 별도 새 schema/capability 객체·typed client3 wrapper·기존 CLI Request 경로+help를 연결했다. 세션 open은 구조 state/evidence/guard를 유지하며 acceptance 검사를 usecase로 이동했다. 기존 concurrency 회귀는 edit 후 옛 prepared append 거부, inspect NeedsInput, 옛 expected record RevisionConflict, 현재 revision이나 acceptance 없는 record NeedsInput, 원 기록 불변을 확인하도록 보강했다. 이 회귀를 단순 session open 성공으로 대체하지 않았다.

새 unit 기대 수는 wire4/schema2=6이다. 기존 application 회귀5/provider4는 그대로이며 기존 app 전체 필터를 실행하면15(기존9+새6)다. domain10도 반복할 경우 기존표본으로 분리한다. 새 fresh CLI suite는10시험이며 정상시험 안에서 세 task를 다루고 여러 실패변형을 각 시험에 묶었다. `OUTPUT_REVIEW_EVIDENCE_DIR`를 명시하면 case별 store와 호출별 request/response/exit/rooted 메타데이터를 Rust fixture가 남긴다. CLI는 매 호출 별도 프로세스이며 완료 때 실제 호출 수를 출력한다. gate에서는 별도 새 evidence 디렉터리와 `--nocapture --test-threads=1`로 실제 수를 보존한다.

최소 실행 제안은 root lib `application::proposals::output_review::tests::wire` 정확4 및 `...::schema` 정확2, integration `--test output_review_cli` 정확10, 변경된 session/concurrency를 포함한 저장16과 기존proposal7, 기존proposals CLI/schema1+candidate schema/CLI2, root default/native all-target strict/fmt다. 기존 provider4·domain10·bridge no-run 재실행은 새 실패/실제 타입영향이 있을 때 범위를 별도 지정한다. 기존 schema/응답 필드는 유지하지만 global command/capability 새 항목으로 전체 discovery bytes는 바뀐다. compilation/실행 검증 전 새 명령의 실제 지원 성공이나 fresh 영속 성공으로 표시하지 않는다.

### wire18 실행 마감

별도 슬롯에서 위 unit6·fresh CLI10(실제96호출)·저장16/기존7·기존CLI/schema3·양feature all-target strict/fmt를 통과했다. r2는 fixture 실행 경로 metadata만 추가했고 r3는 unused test import2줄만 제거했다. 제품/시험 단언은 같으며 원r1/r2·경고·성공로그를 보존했다. 최종 r3 pin은 `3EBC2F908DBAD95BFD6E499C5808A774E5120C94A72AFF8C98B79763E70EBE7C`다. [wire 실행 정본](../../sots/verification/application/accepted-output-review-wire-2026-10-20.md)에 CARGO_BIN_EXE/바이너리 SHA·호출별request/response/exit·실행11/PID22종료·보호차이0와 명시 반환을 기록했다. configured file store에서 새3명령은 이 범위로 지원하지만 실제 사용자 품질 승인·전체제품·미디어·과거 target 재검토를 뜻하지 않는다. 위 미실행 서술은 당시 단계 기록이며 재실행 권한이 아니다.
