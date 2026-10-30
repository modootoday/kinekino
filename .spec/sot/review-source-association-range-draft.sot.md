---
id: sot:kinekino/review-source-association-range-draft
title: Review source association·사용자 range draft 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Review source association·사용자 range draft 계약

## Verified 요청·응답 후계 — 2026-10-27

`reviews.association.revise`는 선택적 `timeline_evidence:{asset_id,sha256}`를 받는다. 생략/null은 기존 Unverified revise이고, 참조를 지정하면 기존 association의 expected_revision을 대상으로 VerifyAssociation을 실행한다. create는 evidence 참조를 거절한다. 호출자 timeline_status/record 필드는 받지 않는다. 신뢰된 assembly만 ReviewSourceService.with_timeline_evidence로 resolver를 연결한다. 기본 CLI의 review root 구성은 이를 자동 추론하거나 외부 JSON으로 주입하지 않는다.

새 Verified 전이는 보유한 실제 evidence에서 mapping record를 만들어 단일 envelope에 저장하고 publication 직전에 control/case/media/proof를 재검증한다. 저장 wire의 timeline_status는 `{"verified":TimelineMappingRecord}`이며 evidence_refs는 해당 참조 하나다. 이는 보관된 검증 기록으로, live playback 권한이나 사용자 AI 품질 승인이 아니다. playback_ready=false를 유지한다. request의 요약을 권위로 받아들이는 Create/Revise/combined create는 거절한다.

schema/capabilities의 timeline_verification_available은 readable/writable/resolver가 모두 구성됐을 때만 true다. false여도 과거 Verified 기록의 읽기와 writable 구성의 exact committed replay는 가능하다. replay는 새 근거 입장보다 먼저 처리하며 case custody는 요구한다. 읽기 전용 구성은 mutation/replay write 명령을 기존대로 거절한다. [작업 근거](../../../verification/application/review-verified-publication-20261027.md)에서 합성 authority·실 파일 저장·실제 봉인 미디어·정상 CLI 근거를 구분한다.

2026-10-26 설계 기준선과 F1 구현 후계. 소유: SDK(domain/application/DTO/저장/API/CLI), Media(provenance/원자료), UI(typed 표시·입력). 현재5명령은 공통SDK/CLI에 등록됐으며 아래단계별 원근거로 실제지원·미검증을구분한다. 구현 입력은 seal-r2 B47E9913…의 격리 후계이며 원본 수정 없이 채택 차이를 별도 manifest로 관리한다. 격리9pass는 실제 저장소·제품 CLI·Verified PTS·사용자 품질 인수가 아니다.

## 범위와 단계

후속 실행2026-10-26: 실제별도process의reservation/pre-link/post-publication중단·복구와경합3시험이통과했다.[process근거](../../../verification/application/review-source-process-recovery-20261026.md)의20pass/1internal-helper ignored·fmt/strict를따른다. RD08의diskfull/permission/root교체/전원손실이나실제CLI ACK유실전수완료는아니다.

후속 실행2026-10-26: 최신 정상CLI의별도21프로세스로5명령/저장재열기/replay/주요오류/readonly를검증했고integration2시험/fmt/strict·명시정상bins를통과했다.[정상CLI근거](../../../verification/application/review-source-normal-cli-20261026.md). 아래정상CLI미실행은당시상태다. process강제중단/경합·schema전수·최신190과다른custody/Verified·AI/GUI는계속남는다.

후속 구현2026-10-26: 아래5명령을 API/DTO/schema/capability/PlayerSdk와 CLI 별도 user root에 등록했다. [SDK·CLI 연결](../../../verification/application/review-source-sdk-cli-20261026.md)의 실제파일 SDK17영향시험·CLI parser6·fmt/strict 통과. 정상CLI 실제5명령/최종190·bins는 미실행이며 아래 계획/미등록은 당시 상태다. 다른custody/Verified·실AI/GUI는 전체범위에 유지한다.

후속 구현 2026-10-26: 기존 package asset의 ReviewPackageMedia와 실제 case/media/user transaction 연결 시험을 추가했다. [미디어 custody 근거](../../../verification/application/review-source-media-custody-20261026.md)의 영향17시험·fmt·strict가 통과했다. 이는 package custody의 format admission 범위이며 다른 미디어 저장소·실제9 연결·decode/Verified PTS·SDK/CLI 등록을 대체하지 않는다. 아래 과거 MediaResolver 미연결은 당시 기록이다.

후속 구현 2026-10-26: 기존 ReviewStore/ReviewSession을 보유하는 ReviewCases가 package/item/source request membership을 검증하고 원 증거 guard를 CaseLease 수명까지 유지한다. [실제 파일 검증](../../../verification/application/review-source-case-custody-20261026.md)의 영향17시험·fmt·strict는 통과했다. MediaResolver 실연결·process 복구·SDK/CLI·Verified는 별도 미완료다. 아래 이전 단계의 미연결 문구는 당시 상태다.

구현진행갱신2026-10-26: 공통domain/review_source의독립revision/초안binding·strictwire·한도규칙을작성하고영향5시험/fmt/strict를통과했다. [검증근거](../../../verification/application/review-source-domain-20261026.md)에입력pin·원r2와매핑차이·미완료를분리했다. 이는아래계획API의등록/실저장/Verified/CLI지원완료가아니다. store/CaseResolver/MediaLease·RD01~RD12및GUI사용자품질은계속구현대상이다.

목표는 원 모델 결과를 유지하면서 같은 원미디어와의 연결을 명시하고 사용자 구간 초안을 독립 저장·수정·재열기하는 것이다. **독립 association 생성/조회/새 revision 및 draft read/save와 Verified evidence 전이 모두 전체 요구에 유지**한다. r2의 최초 draft와 association 결합 출판은 호환 입력일 뿐 전체 요구를 대체하지 않는다.

독립 association의 최초 create와 최초 draft 결합 create는 같은 store transaction으로 동작한다. 기존 association에 draft를 추가할 때는 exact association revision을 참조한다. draft 수정 중 association binding을 몰래 바꾸지 않는다. 다른 영상으로 재연결하려면 명시 association revision 생성 후 별도 새 draft ID로 작성하고 과거 draft는 원 binding으로 조회한다. user draft/모델 원문/sample decision/accepted output은 서로 별개다. actual9 package·objects·현재 decisions/proposals는 자동 덮어쓰지 않는다.

## typed 값

| 값 | 필드·검증 |
|---|---|
| ReviewCase | package_sha256/item_id/source_request_sha256. 서버 CaseResolver가 실제 package item·request asset membership과 hash를 확인. caller SHA만 신뢰하지 않음 |
| MediaReference | 기존 custody asset_id/sha256, resolver가 bytes/media_type/content identity와 원 guard를 확인. arbitrary path/URL/command·새 asset registry 금지 |
| AssociationRevision | association_id, case, media, revision, timeline_status, evidence_refs, note(UTF8 최대8192bytes). 기존 r2 request_sha256은 명시 DTO 매핑. case는 association 수명 동안 불변 |
| TimelineStatus | Unverified 또는 Verified{evidence_sha256,timebase,origin,mapping_revision}. Verified는 검증 어댑터가 만드는 opaque proof에서만 생성. caller의 enum/숫자 직렬화만으로 승인 불가 |
| RangeDraft | draft_id, association_id/association_revision, draft_revision, authored_by=User, ranges, note. 모델 raw/quality verdict를 이 객체에 저장하지 않음 |
| UserRange | [start_us,end_us), 정수 마이크로초 start<end, 끝≤검증 duration일 때만 duration 검증완료. r2의 최대64개 유지, 순서 보존·겹침 자동 병합/정렬 금지 |
| Save context | mutating request의 idempotency_key, expected_revision(create는 없음/update는 현재 revision). operation timeout과 transport deadline은 별개. read는 revision/key/timeout mutation 필드를 거절 |

JSON의 시간/revision은 기존 DecimalU64 십진문자열, strict deny_unknown_fields. r2 ID는 ASCII alphanumeric/_/- 최대128bytes를 유지하며 공통 request_id의 Unicode128 codepoint 규칙과 혼동하지 않는다. note는 UTF8 최대8192bytes, ranges 최대64개, user note를 HTML/실행 입력으로 해석하지 않는다. 빈 ranges는 유효한 빈 사용자 draft이며 원 item.range나 issue_ranges 허용 조건을 바꾸지 않는다. overflow/음수/float/잘못된 hash는 InvalidArgument다.

Unverified에서도 유효한 수동 입력은 저장할 수 있으나 playback_ready=false다. duration unknown을 ‘길이 내’ 검증완료로 표시하지 않는다. Verified가 되어도 decode/seek/capture/export capability가 별도로 없으면 playback_ready=false다. Verified는 시간 대응의 기술 근거이지 모델 추천 정확성·사용자 채택·사용권·품질 승인이 아니다.

## 계획 API 표면

application진행2026-10-26:공통mutation/read흐름과단일transaction/lease/writer/guard port를구현했고최종10영향시험/fmt/strict통과.[근거](../../../verification/application/review-source-application-20261026.md)의mock범위이며아래명령/schema/capability/CLI등록이나실디스크복구완료는아니다.

아래5명령은F1후계의COMMANDS/schema/capability/SDK/CLI에등록됐다. 기본SDK의adapter미주입·readonly·타host/GUI연결·Verified미지원은구분한다. 이름/계약변경시이정본과추적표도함께갱신하고기존미지원상태를지원으로덮어쓰지않는다.

| 계획 명령 | 입력 / 반환 / mutation 조건 |
|---|---|
| reviews.association.inspect | case+association_id+optional revision → exact revision/current revision·기술 readiness. no write |
| reviews.association.create | case+association_id+media+note/evidence references → revision1, key 필수/expected 없음. case/media 검증 후 Unverified publication |
| reviews.association.revise | case+association_id+media 또는 기존 검증 evidence reference → 다음 revision, expected/key 필수. 과거 draft 자동 이동 없음 |
| reviews.range_draft.read | case+draft_id+optional revision → draft+exact association/readiness·원 저장 receipt. 현재 media 부재는 저장된 draft를 숨기지 않고 상태로 표시 |
| reviews.range_draft.save | draft+exact association binding, expected/key → SavedDraft receipt. 신규 association 결합 생성은 명시 create_association 입력만 허용하고 동일 transaction에 출판 |

Verified 전이는 association.revise에서 **이미 기존 custody가 보유한 evidence reference**를 받아 adapter가 검증한다. 값/경로를 입력받아 자체 PTS를 만들어내지 않는다. 검증 어댑터가 없으면 해당 transition Unsupported, 원영상/근거가 미제공이면 NeedsInput, 참조 손상/identity 불일치는 NotFound/RevisionConflict로 구별한다. media import/registry·다운로드·추론을 자동 실행하지 않는다.

제품 client는 ApplicationClientHandle/기존 sdk_execution로 연결한다. UI는 local filesystem을 직접 읽거나 draft store를 복제하지 않는다. Web은 같은 권한 있는 typed SDK transport와 resource grant가 없으면 Unsupported다. capability는 read/write/verified_mapping/playback readiness를 분리하며 command별 available 조건을 공개한다. read-only store는 read만, writable store지만 media resolver가 없으면 새 연결/저장은 Unsupported다. 이미 committed인 exact replay 조회는 이후 media unavailable/cancel로 사라지지 않는다.

## transaction·CAS·replay

구현진행2026-10-26:Windows별도user root의단일envelope 파일어댑터와OS예약·checked publication·readonly/reopen/replay를구현했다.[실제파일검증](../../../verification/application/review-source-file-store-20261026.md)의6실파일시험/최종16영향시험·fmt·strict범위까지만입증했다. CaseResolver/실Media custody·프로세스중단·SDK/CLI·Verified/GUI연결은미완료다. 현재adapter상한은case4096 transaction/각262144bytes이며새출판만거절하고원key/record를만료시키지않는다. 변경시명시migration/admission계약을갱신한다.

namespace는 기존 ReviewStore 원본과 분리한 user-review 저장 root의 case scope다. root는 trusted assembly가 지정하고 request는 path를 받지 않는다. 새로운 미디어 registry나 모델 원본 저장소를 만들지 않는다. case scope 아래 association/draft별 immutable revision 기록과 transaction envelope를 저장한다. writer는 해당 case의 원 directory/file guards와 배타 reservation을 유지한다. canonical JSON 입력·mutation 종류·case/object ID·expected revision·key의 equality를 보존한다.

단일 transaction envelope에는 schema_version, 원 request identity, association revision(필요한 경우), draft revision(필요한 경우), stable receipt를 함께 넣는다. 원자 commit의 권위는 **한 envelope의 완료 publication**이며 별도 receipt 파일/인덱스 갱신은 권위가 아니다. 기존 rooted paths·bounded decode·writer reservation·no-overwrite new_file/file publication 원리를 재사용하되 generic helper를 multi-file transaction이라고 부르지 않는다. 파생 index는 재생성 가능해야 하고 단독 존재로 완료를 판단하지 않는다.

처리 순서: 입력/한도 검증 → 검증된 case membership → 기존 committed exact replay 확인 → control checkpoint → media/evidence lease retain → expected revision/CAS·참조/길이 검증 → 원 writer 안에서 cancel/lease 재확인 → immutable envelope publication → readback → receipt. 같은 key+동일 요청은 최초 receipt, 같은 key+다른 요청은 RevisionConflict. replay는 새 revision을 만들지 않으며 현재 media 가용성·후속 취소보다 우선한다. 권한/case 검증을 건너뛰는 replay는 허용하지 않는다.

create는 identity 없음/next=1, update는 exact current+1(checked overflow)다. 경쟁 두 writer는 하나만 출판하며 나머지는 conflict. association 재연결·timeline 전이는 draft CAS와 별개로 revision을 증가시키고, draft가 참조한 과거 association은 유지한다. key는 case+operation+object scope로 보관하며 현재 설계에는 자동 만료가 없다. 보관 제한이 필요하면 명시적 migration/거절을 추가하고 committed key를 조용히 재사용하지 않는다.

예약/부분 record·손상·hash mismatch·ambiguous publication은 보존 후 읽기를 fail-closed하고 NeedsInput/원 I/O 오류+publication_may_exist/phase/recovery로 알린다. 최신 revision이 비어 있다고 자동 되돌리거나 같은 ID를 덮어쓰지 않는다. 복구는 원 receipt/envelope 확인 후 같은 key 조회로 수행하며 새 key로 재실행하지 않는다. power-loss·NTFS flush 보장은 프로세스 중단 복구 시험과 구별하며 실제 보장을 측정하기 전 완전 내구성을 주장하지 않는다.

## 오류·취소·기한

기존 ErrorCode를 재사용한다: Invalid→InvalidArgument, NotFound→NotFound, Unsupported→Unsupported, Conflict→RevisionConflict, pre-publish Cancelled→Cancelled, I/O permission→PermissionDenied/일반 저장오류→EngineFailed. ExecutionUnknown은 새 ErrorCode 이름을 발명하지 않고 원 timeout/engine failure code와 details.delivery=execution_unknown, publication_may_exist=true, automatic_retry=false, 같은-key 조회 안내를 보존한다. 취소 후 no-write는 publication 전 관측 근거가 있을 때만 보장한다. publication 뒤 완료를 Cancelled로 바꾸지 않는다.

기존 sdk_execution의 running cancel이 request만 표시하는 한계를 유지한다. 새 mutating usecase는 cooperative control checkpoint를 durable port까지 연결해야 하며 그 전에는 strong_cancel_available=false다. CLI/GUI는 cancel 요청 접수와 실제 저장 전 중단을 별도로 표시한다. receipt는 source case/object/revision·content hash·key·readiness·publication 상태를 담고 로컬 비밀 경로/서명 URL은 넣지 않는다.

## 필수 인수 (작성 기준, 실행 아님)

내부 TimelineEvidenceResolver의 package 기반 본체와 지명된 producer 묶음의 신뢰 경계는 [producer bundle v1](../timeline-producer-bundle.md)을 따른다. 신뢰된 Rust 조립의 원 bundle 입장 없이 caller JSON의 hash로 권한을 만들지 않는다. 현재 wire Verified와 playback readiness는 아직 지원하지 않는다.

case별 timeline 증거 입력은 RetainedReviewEvidence가 원 ReviewSession을 보유한 상태에서 선택 item의 source/candidate/baseline/자동검사 asset ID와 SHA를 대조해 읽는다. 같은 package의 다른 item에만 있는 asset은 이 입력 경로에서 사용하지 않는다. handle의 입력 custody와 실제 source/tool/process/clock 교차검증 권한은 구별한다. [case 증거 resolver](../../../verification/application/review-case-evidence-resolver-20261026.md)를 따른다.

resolver의 내부 관측 읽기는 기존 ReviewSession의 read_evidence_asset(asset_id,sha256,max_bytes)를 사용한다. 선언된 ID/SHA/bytes와 고정 root의 원 asset을 대조하고 session guard를 유지하며 최대64MiB로 제한한다. 기존 provider의 기본 구현은 Unsupported다. 새 wire 명령이나 임의 경로 권한은 만들지 않고, 공개 text 읽기는 기존64KiB/UTF-8 계약을 유지한다. binary 읽기 자체는 source/tool/process provenance나 Verified 전이를 입증하지 않는다. [내부 읽기 검증](../../../verification/application/review-package-evidence-reader-20261026.md)을 따른다.

RD01 wrong package/item/request/hash·foreign custody·path injection no-write. RD02 독립 association create/read/revise와 과거 draft binding 보존. RD03 결합 create+draft의 단일 envelope·readback/reopen. RD04 동일key exact replay/변경입력 conflict와 media unavailable 이후 replay. RD05 stale CAS·동시writer·overflow. RD06 invalid/empty/64초과range·UTF8 byte한도·검증 duration초과·duration unknown 표시. RD07 queued/pre-publish cancel no-publication, running Requested≠cancelled, publish뒤ACK유실 same-key조회. RD08 실제 filesystem의 예약/부분파일/중단/손상/disk full/permission/root replacement·원본무변경. RD09 사용자 초안과 sample decision/accept/output 독립. RD10 검증PTS없는 replay/export 거절과 caller가 만든 Verified 거절. RD11 실제 content/timebase/mapping proof·동일 SDK seek/capture 관측 후 readiness. RD12 command/schema/capability/CLI 및 thin GUI binding 동등성. 기존 mock9는 이 행렬 전체를 입증하지 않는다.

연결: [상위 후계 계획](../../../../plans/player/ai-review-source-association-goal-2026-10-25.md), [수명·저장](../execution-storage-boundaries.md), [모델 검토](model-review-contract.md), [시간 provenance](../../../verification/ai/highlight-media-provenance-gap-2026-10-25.md).
