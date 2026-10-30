---
id: sot:kinekino/accepted-card-review-png-export
title: Accepted Card review PNG export 구현 정본
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Accepted Card review PNG export 구현 정본

2026-10-24 현재 root에 적재된 공통 Rust SDK/CLI 구현과 실제 검증 범위다. [카드 제작의 전체 제품 계획](cards.md) 중 **이미 accepted된 Cards의 한 position을 새 PNG review artifact로 내보내는 기능**만 구현한다. 자동 제작·품질 승인·템플릿 사용자 승인·플랫폼 게시 완료를 뜻하지 않는다. 초기 작성 시점의 root source45+lock1 적재·전체 회귀 진행 전 상태는 선행 이력이다. 최신 검증 반환·PM 커밋 상태는 아래를 따른다.


## 최신 canonical 검증·커밋 상태 — 2026-10-24

[canonical integration 원 SOT](../../../verification/proposals/accepted-card-canonical-integration-r1-2026-10-24.md)와 [원 slot-return](../../../../../runtime/verification/accepted-card-canonical-integration-r1/slot-return.json)을 직접 읽어 확인했다. 기본 full1163 passed/0 failed/76 ignored(default Card application5/strict inspection2 포함), default all-target strict, feature mapper exact1, 기존 SDK/CLI parity2가 통과했다. feature all-target strict는 명시 승인된 compositional reuse이며 새 전체 실행이 아니다. S62/alias4 복원·보호 반환(status=returned/pending0/writer0, carry229)이 완료됐다. 원 SOT의 정확 보호·FileID·PID reuse·legacy parity 수명 한계를 유지한다.

PM 커밋 `371ed79414698e49f6ee7444ec00d5e84a0cd1cc`(`feat(cards): export accepted review cards through shared SDK and CLI`)를 직접 확인했다. source45+lock1과 PM ignore1은 이 커밋 전 미커밋 상태에서 이후 커밋 완료로 전환됐다. 원 SOT의 미커밋 문장은 검증 반환 당시 관측이다. default CLI PDB의 미보존/해시 미관측, 09 mapper build의 fresh RAM/GPU/disk/writer 기록 누락은 원 SOT에 남긴 PM 인수 예외이며 누락을 채웠다고 주장하지 않는다. 별도 Meari cleanup5 예외도 유지한다.

아래 actual2/성공 export1+기대 오류9와 기술 검증은 그대로다. glyph·가독성·품질·템플릿 사용자 승인, 취소/timeout 지원, capability available_unverified는 승격하지 않는다. 이 완료는 accepted Card review PNG 범위에 한정하며 전체 카드 제작·다른 제품 기능 완료를 뜻하지 않는다.

## 공통 SDK·CLI·지원 상태

- application 명령: `production.export_card_review_png`. typed SDK는 `ApplicationClient::export_card_review_png(context: RequestContext, input: ExportCardReviewPng) -> Result<CardReviewArtifact, ApiError>`이며 공통 execute 경로를 호출한다. 기능 정책은 `src/application/proposals/card_export/mod.rs`, DTO/schema는 같은 모듈과 `schema.rs`에 있다. GUI 조작을 경유하지 않는다.
- SDK `PlayerSdk`는 기본 `card_export=None`. `with_card_image_export`와 사용 가능한 proposal store를 명시 주입한 경우에만 schema/capability가 `available_unverified`, 그 외 `unsupported`다. feature를 켠 사실만으로 모든 SDK 인스턴스에 port가 주입되는 것은 아니다.
- Windows CLI는 `card-review-export` feature에서 `PngCardReviewExport`를 주입하고 `--proposal-root`로 configured store를 연다. assembly/infrastructure native 모듈은 Windows+feature로 제한된다. 기본 빌드 또는 renderer/store 미구성은 지원되지 않으며 유효하지 않은 입력은 더 앞선 검증 오류가 될 수 있다. 다른 플랫폼의 실제 export 지원은 인수하지 않았다.
- 실제 사용 형식은 `player-cli.exe --input <전체 Request JSON 경로> --proposal-root <기존 proposal store> --json`. --input은 payload만이 아니라 `command/context/payload`를 포함한 Request다. `tests/accepted_card_export_cli/invocation.rs`, `src/bin/player-cli/runner.rs::execute_card_review`가 원 경로다. 이 문서에서 새 CLI 실행은 하지 않았다.

`schema` 요청의 payload는 `{"command":"production.export_card_review_png"}` 한 필드만 허용한다. schema의 verification 문자열은 현재 `source_candidate_not_actual_CLI_verified`, capability는 `not_actual_CLI_verified`로 보수적으로 남아 있다. 아래 actual 근거를 이유로 이를 `supported`로 고쳐 쓰거나 품질 지원으로 승격하지 않는다.

## 요청과 immutable accepted target

Request context는 공통 API v1 규격을 따른다. `expected_revision`, `idempotency_key`, `timeout_ms`는 이 동기 명령에서 null/미설정만 허용된다. non-null이면 `unsupported`이며 CAS·재시도·취소·wall deadline 계약을 제공하지 않는다. generic Request 유효성 검사가 먼저 수행된다.

`ExportCardReviewPng`와 `TemplateSelection`은 unknown field를 거부한다. payload의 필수 필드는 다음과 같다.

| 필드 | 실제 계약 |
|---|---|
| target | `OutputReviewTarget`, kind=`accepted_output`, schema_version=`1`, task는 실제 accepted Cards여야 한다. proposal/draft/output revision 및 typed-output/source/generated-raw 해시와 optional historical binding을 함께 고정한다. |
| position | 0부터 시작하는 `DecimalU64` JSON 문자열. 플랫폼 index 변환과 accepted cards 범위 검증을 수행한다. |
| frame_path / frame_sha256 | 명시적 절대 로컬 PNG 경로와 SHA256. root adapter는 Windows Disk/VerbatimDisk prefix, canonical regular file, 최대 바이트, 실제 SHA를 확인한다. URL/UNC 입력으로 대체하지 않는다. |
| template | id/version/declaration_sha256 모두 명시. renderer DTO를 임의 보내는 계약이 아니라 root product declaration과 정확히 대조하는 선택이다. |
| output_path | frame과 다른 명시적 절대 로컬 .png 경로. 부모는 기존 canonical local disk 디렉터리여야 하며 새 파일만 게시한다. |
| review_artifact_only | true를 명시해야 한다. false는 `needs_input`; 품질 미승인 결과임을 인정하는 입력이다. |

`OutputReviewTarget` 필드는 `schema_version, kind, proposal_id, task, draft_revision, output_revision, hash_format, output_sha256, source_id, source_revision, source_sha256, generated_raw_sha256, generated_raw_bytes, historical`다. `hash_format=accepted_typed_output_v1`; revision/bytes는 decimal 문자열이다. 구조 검증은 draft+1=output revision 등의 조건을 확인하고 `validate_against`는 저장된 current acceptance에서 다시 만든 target과 전체 값의 동일성을 확인한다. 단순 proposal ID 조회나 PNG hash로 accepted typed-output hash를 대체하지 않는다.

application은 accepted receipt의 `Cards`에서 선택한 front/back/source_refs를 빌려 renderer에 전달한다. 새 Accept/CAS나 proposal 수정은 수행하지 않는다. 현재 상태와 다른 target은 `revision_conflict`, Cards가 아닌 accepted payload는 `unsupported`다. target 검증은 저장된 typed identity의 결합을 입증하며 실제 방송 프레임 PTS·원문 진실성·사용 권리를 인증하지 않는다.

## 프레임·제품 템플릿·폰트·검사

제품 선언은 `src/assembly/card_export/product_template.rs` 한 책임에서 관리한다. 현재 선택은 `card.frame-two-slot`, version=`0.1.0-proposal.1`, render version=`card.frame-two-slot/0.1.0-proposal.1`. declaration SHA256은 **해당 Rust 선언 파일의 compile-time include_bytes 전체 바이트** 해시 `5f4778d66b7fa9c15221033487e46e2e000f2ccbb7a054e33554cf6a2931bc81`이다. 이것은 renderer가 별도로 계산하는 template DTO hash나 전체 렌더 환경 fingerprint와 다르다.

출력 proposal은1080×1080, 배경RGB11/12/14, frame(64,64,952,536) contain/center, front(96,640,888,160), back(96,832,888,160), font40/line56/center다. 이 선언은 `ProposedNotUserApproved`이며 새 사용자 승인으로 간주하지 않는다. layout bounds를 확인하며 자동 축소·ellipsis·텍스트 crop으로 overflow를 숨기지 않는다.

`src/infrastructure/card_export/assets.rs`는 기존 `prepare_product_styles`의 공통 VerifiedAssets와 font license metadata를 소비하고 frame PNG를 추가한다. Pretendard1.3.9 핀은 다음과 같다.

| 폰트 | SHA256 |
|---|---|
| Regular | 6d0af5258997aec7354a6e340fc2325ba321c410ca48b3af858c8c3d6e92a324 |
| SemiBold | 5e1c548732af70873103066c16e1369b9a8a871f0b38c321a1d5bc73e43cea2d |
| JP Regular fallback | b14698ed6d82737f45e918d079bca6fc498a107bcb094b56c5229dbddf3246d3 |

OFL license2는 공통 자산 계약으로 소비하고 결과 provenance에 남긴다. 공통 manifest의 hash/license 존재와 renderer의 font delivery 관측은 실제 glyph 선택·전체 글리프 coverage·가독성 검증을 대신하지 않는다. 시스템 fallback도 통제 완료를 주장하지 않는다.

`StrictPng` port는 frame과 생성 output 모두 root 공통 snapshot PNG parser에 연결한다. 완전한 static PNG, CRC/trailing-data/APNG 및 공유 resource 제한을 검사한다. 공유 상한은64MiB 및3840×2160 pixels; renderer text budget은131072bytes다. 원 parser `ApiError`는 단계(frame/output) 진단만 별도 stderr로 기록하고 code/message/retryable/details/remediation/request_id 원인을 변경하지 않은 채 반환한다. 별도 caller pixel budget 검사는 공통 parser 뒤에 수행되므로 더 작은 임의 caller limit의 preallocation 보장으로 확대하지 않는다.

## 새 파일 게시와 결과

렌더·PNG 검사·바이트 제한이 모두 통과한 뒤 application이 SHA/bytes/receipt를 구성하고 `CardImagePort::publish_new`를 호출한다. root 구현은 `media_file_publication::publish`를 사용한다. 기존 부모 안의 create_new 임시 파일에 write_all/sync_all 후 새 output에 hard_link하고 temp 제거를 시도한다. 기존 output을 덮어쓰지 않으며 충돌은 `revision_conflict`다. 실패 시 final artifact 성공 receipt를 반환하지 않는다. temp 제거는 best-effort이며 모든 I/O 실패에서 temp가 반드시 없어진다는 보장은 아니다. 이 명령은 deadline 없는 publish 경로를 사용한다.

`CardReviewArtifact`는 target/position/source_refs, output_path/output_sha256/output_bytes, provenance와 다음 고정 상태를 반환한다.

- state=`review_artifact`, overwrite=false, quality_approved=false, template_approved=false.
- cancellation_supported=false, timeout_supported=false.
- limitations: `glyph_coverage_unmeasured`, `ink_overflow_unmeasured`, `host_font_fallback_uncontrolled`, `wall_deadline_not_provided`, `frame_placement_not_pixel_verified`, `template_approval_not_checked`, `cancellation_not_provided`.

provenance는 frame path/hash/size, canvas, template 선택/DTO hash/render version/status, manifest hash, font3/license2/delivery, front/back layout 관측, source_refs와 한계를 보존한다. `source_pts_authenticated=false`, `frame_rights_verified=false`. fixture의 제한된 정확 픽셀 샘플이 통과해도 일반 frame placement/glyph/ink 검증 한계 enum은 제거하지 않는다.

## 오류: 원 코드와 원 응답 대조

아래는 root actual의 원10개 요청·응답·status와 대조한 사례다. exit 값은 이 CLI 실제 실행의 값이며 새로운 코드 매핑을 정의하지 않는다.

| 원 case | 결과 / ApiError code | CLI exit |
|---|---|---:|
| new-png | 성공 review_artifact | 0 |
| existing-output | revision_conflict: Output already exists; choose a new path | 5 |
| target-pin | revision_conflict: Accepted output target changed | 5 |
| position | invalid_argument: Card position is outside the accepted output | 2 |
| frame-pin | revision_conflict: Frame differs from the explicitly pinned SHA256 | 5 |
| template-pin | revision_conflict: Explicit template selection does not match the product proposal declaration | 5 |
| review-ack | needs_input: 명시적 unapproved review artifact 확인 필요 | 3 |
| inspection-failure | invalid_argument: complete, CRC-valid static PNG with no trailing data 필요 | 2 |
| layout-failure | needs_input: Card review render failed: LayoutOverflow(Front) | 3 |
| deadline-unsupported | unsupported: no CAS, retry, cancellation or wall deadline contract | 3 |

이 사례 밖에서도 입력 문법·경로/position 오류는 invalid_argument, 미구성 renderer/store·비Cards/APNG 등은 unsupported, 자원 상한은 resource_exhausted, layout unavailable/overflow는 needs_input, 기타 renderer 실패는 engine_failed로 원 코드가 처리한다. 파일 I/O는 publication 공통 mapper의 not_found/permission_denied/revision_conflict/engine_failed 등을 그대로 사용한다. 임의 새 오류·저장 정책은 추가하지 않는다. 취소/timeout을 지원하는 것처럼 문서화하지 않으며 외부 test runner의 제한시간은 제품 API 계약이 아니다.

## 실제 검증과 남은 범위

[renderer actual-r2 원 SOT](../../../verification/proposals/accepted-card-renderer-actual-r2-2026-10-24.md): exact ignored1 선택/통과, 나머지24 filtered. 8×8 합성 프레임의320×320 **메모리 버퍼**에서 exact red/yellow, contain margin, font delivery/layout/ink/selection/limitations를 확인했다. 이 실행은 PNG 파일을 저장하지 않았고 최종 PNG hash도 원 stdout에 노출되지 않았다. 원 r4 실패는 보존됐다.

[root export actual-r1 원 SOT](../../../verification/proposals/accepted-card-export-actual-r1-2026-10-24.md): exact ignored1 선택/통과, 나머지8 filtered. **두 actual 실행 전체에서 root CLI 요청은10개=새 PNG 성공1+기대 오류9**다. 각 요청은 nonforced/Empty로 종료했다. 원 fixture는 `runtime/verification/accepted-card-export-actual-r1/fixture/`의 request/stdout/status와 PNG, before/after 관측을 유지한다.

저장된 `review.png`는36847bytes, SHA256=`98f1395550589d060180be9ed30e841a6371f9a0cf68ee85c64daf336e2b13ac`,1080×1080 RGBA8이다. 독립 Rust decode로 exact red/yellow·contain margin·양쪽 일부 ink·폰트3/OFL2와 handler delivery를 확인했다. 입력 frame은8×8/112bytes, SHA256=`61ebb9c7891bb3fbee74911953bc9e17b9b44aa99fc9a847b2b1b75d58316a33`. target card-plain/draft1/output2/position0/source_refs[cue-1]과 PNG hash는 별도 identity다.

원 actual은 충돌 시 기존 PNG bytes, 입력 frame bytes, 두 logical accepted snapshot 및 store 파일5개의 SHA/길이/mtime 불변과 실패 output 부재를 검사했다. setup에서 새 합성 store를 만들고 accept한 것과 export 자체가 store를 변경하지 않은 것을 구분한다. 기존 CLI69D6F5AC…는 승인된 type-r2 산출물을 재사용했고 r3 root 테스트는 새 fixed92F5B8F7…이었다. 재사용 CLI를 r3에서 재빌드했다고 주장하지 않는다.

**미인수:** 실제 방송 frame PTS·권리, 모든 glyph/정확한 철자·가독성/ink coverage, 사용자 template 승인, 생산 품질, 수동 GUI/시각 인수. 초기 작성 시점에는 두 exact actual 통과·root 정본 적재 후 전체 regression 전이었으며, 이후 canonical 회귀·반환 완료는 최상단 최신 상태와 원 SOT를 따른다. 새 추론·다운로드·worker build·GUI 우회는 이 기능 검증의 일부가 아니다.

## 정본 책임·스킬 적용

SDK가 application/schema/typed client/CLI 조합·기능 검증을, Media가 독립 renderer와 이 구현 정본을, 제품 UI가 미승인 template/UI 선언을, PM이 자원·공개 경계·shared index를 소유한다. 최신 갱신은 PM 배정의 이 구현 정본과 두 계획 문서에 한정하며 source/Cargo/.gitignore/index 및 다른 문서는 변경하지 않았다. 새 compile/render/CLI/추론 실행0.

이전에 탐색·본문/필수 참조를 읽은 modoo `software-qa/test-fixture-and-mock-build`(정확 이름 `modootoday/software-qa/test-fixture-and-mock-build`, body SHA a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100)와 `modootoday/software-qa/code-change-review-verification`(body9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B, references/verdict-table.md9D9619A829AE24D73645EF5F7C8126C8257CAEDDAC80800632866257008AD2A9)를 재사용했다. 원 source→schema/port/CLI→원응답을 대조하고 synthetic/actual/quality 경계를 구분하는 데 적용했다. 명령 오류·저장 정책을 새로 설계하지 않았다.
