---
id: sot:kinekino/generic-review-package-and-immutable-sample-decision-v1
title: Generic review package and immutable sample decision v1
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Generic review package and immutable sample decision v1

2026-10-26 후속 사용자 source association/range draft는 [독립 계약](review-source-draft-contract.md)으로 설계했다. 현재 reviews의 sample decision API와 분리하며 새 명령은 아직 제품 미등록이다. 실제9의 range=null/NeedsRepair를 변경하지 않는다. 아래 초기 ‘세 명령’ 문구와 별개로 현재 read_text_asset도 source/schema에 존재하며, 정확한 조건은 [host 표면](host-support-boundaries.md)과 실제 handler가 권위다.

사용자 품질 승인 경계(2026-10-24): 실제 제품 GUI의 사용자 테스트에서 사용자가 검증·승인한다. 이 API/CLI의 판정 저장 성공이나 sample_review_passed는 계약상의 표본 상태이며 그 자체로 실제 사용자 GUI 인수·전체 모델/기능 승인 근거가 아니다. 개발·자동 시험의 판정 이력과 실제 사용자 판정 이력을 구별하며, 공통 저장 계약은 유지한다. 상위 [모델 검토 정책](../model-quality-review.md)을 따른다.

상위 정책은 [모델 검토](../model-quality-review.md)다. 검토 자료가 없으면 `not_found`/`prepare_review_package`이며 사용자 승인 대기가 아니다. GUI/main에는 아직 주입하지 않았다. 공통 SDK `with_review_store(ReviewFiles::open(root)?)`와 CLI의 명시 `--review-root ABSOLUTE_DIRECTORY`가 같은 application 유스케이스를 사용한다. 미주입은 세 명령 모두 unsupported다.

## 명령과 저장 배치

`reviews.inspect`: `{"package_sha256":"<64 lowercase hex>"}`.

`reviews.record_decision`: `{"package_sha256":"<sha>","item_id":"sample-001","decision_id":"review-001","verdict":"deferred","note":"구체적 이유/관찰","issue_ranges":[]}`.

`reviews.decisions`: `{"package_sha256":"<sha>","item_id":"sample-001"}`.

공통 Request의 context/api_version/request_id를 사용한다. expected_revision/idempotency_key/timeout_ms는 이 동기 유스케이스에서 unsupported다. --endpoint와 --review-root를 함께 선택하지 않는다. --endpoint인 경우 host에 실제로 주입된 adapter만 따른다. --review-root가 없으면 CLI도 기본 unavailable이다.

Root는 사전에 존재하는 local directory이며 하위 packages/objects/decisions 디렉터리를 준비한다. package는 **실제 JSON 파일 바이트 SHA256**로 `packages/<sha>.json`, asset은 실제 바이트 SHA256로 `objects/<sha>`에 둔다. package JSON 자체에 자기 SHA를 넣지 않는다. source/result/검사 evidence 모두 동일 객체 배치를 사용한다. domain에는 HF/파일 경로나 OS 타입이 없고 asset_id/sha256/bytes/media_type만 있다. media_type은 생산자 메타데이터이며 hash 검증이 codec/의미 품질 검증을 대신하지 않는다.

package schema는 `schema reviews.inspect` 결과의 `review_package_schema`로 조회한다. 완전한 예시는 [package.example.json](../../../../artifacts/quality-review/contracts-v1/package.example.json)에 있다. 예시의 0/1/2 SHA는 placeholder이며 실제 asset 없이 실행 가능한 evidence로 간주하지 않는다.

## Package 형태

최상위: schema_version="1", package_id, items(1..64). **task는 item마다 고정**하여 하나의 package에 서로 다른 task/model을 담는다.

item: item_id, task, source(asset), range(start_us/end_us decimal u64 문자열 또는 null), baseline(result 또는 null), candidate(result), criteria(1..32), known_issues(0..64). range/baseline 생략도 null과 같다. range=null은 명시적 non-temporal 표본이며 issue_ranges는 빈 배열만 허용한다. 검증되지 않은 timestamp를 발명해 채우지 않는다.

result: condition, assets(1..16), automated_checks(1..64), measured_resources(string map). condition은 model_id/revision/precision/engine_id/engine_revision/input_conditions(string map)이다. revision/engine_revision에 main/master/latest/HEAD/unknown은 거부한다. 이는 생산자의 revision metadata를 고정하는 것이며 외부 저장소 조회·서명 검증은 수행하지 않는다. input_conditions/measured_resources는1..32 entries, 각 value1024bytes·전체key+value4096bytes 이하다. 측정하지 않은 자원은 status=not_measured처럼 명시한다.

check: check_id, outcome=passed/failed/not_run, summary, evidence(asset list). criterion: criterion_id, **how_to_review**, **expected**. 원본/기준선/후보를 어떻게 비교하고 무엇을 확인할지 생산자가 제공한다. string/배열/구간·ID 및 package524288bytes/asset1GiB·총unique assets4GiB 상한을 application/domain/adapter가 검사한다.

baseline=null은 package inspect를 허용하지만 review_ready=false다. 이 경우 usable이라고 기록해도 sample_review_passed는 false다. criteria 누락·빈 검토방법·invalid range·중복item/criterion/check ID는 package 거부다. root 실제 텍스트9개 표본은 각 원래 task를 item.task에, 각 실제 모델·revision·precision·engine 조건을 candidate.condition에 담는다. 기준선이 원문/사람참조면 그 사실과 고정 reference revision을 baseline.condition에 명시한다.

## 판정 의미

verdict는 usable(사용 가능), needs_changes(수정 필요), deferred(판단 보류)다. note는 빈 문자열을 허용하지 않는다. issue_ranges는 temporal item의 fixed range 안이어야 하고 non-temporal item은 빈 배열만 허용한다. 판정이 참조하는 package SHA·item ID에서 package_id/task/candidate condition을 가져오므로 caller가 다른 model condition을 판정에 주입할 수 없다. 원문/ASR draft baseline은 source-reference 등의 정직한 ID와 실제 고정 asset revision·UTF8 precision·reader revision으로 기록할 수 있으며 인간 gold나 모델 출력으로 가정하지 않는다.

응답은 decision_sha256와 decision이다. decision은 원래 request 필드에 schema_version/package_id/**item.task**/candidate_condition/review_ready/automated_prerequisites_passed/sample_review_passed/automatic_application_enabled=false/scope=sample_only를 더한다. sample_review_passed는 baseline 존재 + **baseline/candidate의 선언된 자동 검사 모두 passed** + verdict=usable일 때만 true다. deferred·needs_changes·not_run/failed·baseline 없음은 통과하지 않는다. 자동 검사 성공은 주관적 판정을 대체하지 않는다.

이는 표본 판정이며 **모델 전체 채택·서명된 사용자 승인·자동 적용 활성화가 아니다**. 이후 기능별 채택/자동 적용 정책은 별도다. 현재 GUI 검토 화면·재생/비교 연결·실제 root package 수집·사용자 의미 판정은 남아 있다. synthetic test 판단을 실제 사용자 승인으로 보고하지 않는다.

## 고정 텍스트 증거 조회

`reviews.read_text_asset` / `ApplicationClient::read_review_text_asset(context, ReadReviewTextAsset)`는 `{package_sha256, asset_sha256}`만 받는다. `ReviewTextAsset` 응답은 package_sha256/asset_sha256/bytes(decimal u64 문자열)/media_type/utf8이다. 임의 파일 경로·URL·프로그램은 입력하지 않는다. package에 참조되지 않은 SHA는 실제 object 파일이 있어도 not_found다. 전체 package와 참조 assets를 SHA/bytes 검증하고 guard를 보유한 session에서 읽는다.

64KiB 이하의 text/* 또는 application/json만 지원하며 media_type 매개변수(`text/plain; charset=utf-8`)를 허용한다. strict UTF-8로 읽고, JSON 파싱이나 HTML 실행/렌더링은 하지 않는다. 잘린 JSON도 원문 검토 대상으로 반환할 수 있다. 용량 초과/지원 외 미디어/같은 SHA의 상충 media type은 unsupported, 비UTF8은 needs_input, 변조는 revision_conflict다. 미제공 backend는 unsupported다. 이 조회는 판정/사용자 승인/자동 적용을 변경하지 않는다.

## 파일 어댑터 경계

패키지와 assets는 hash/bytes를 읽어 검증하고 session 동안 파일·디렉터리 guard를 보유한다. Windows local disk root만 제공하며 root/하위의 reparse·symlink·원격UNC·parent traversal을 거부한다. 미제공 플랫폼/adapter를 supported로 꾸미지 않는다. CLI JSON에서 임의 경로·프로그램을 선택하지 않는다.

decision identity는 한 번만 생성한다. 실제 내부 ledger는 `decisions/<package-sha>/i-<item-ID UTF8hex>/d-<decision-ID UTF8hex>/<decision-sha>.json`이다. ID를 경로로 직접 쓰지 않아 DOS device/점/alias를 회피한다. 한 item당32개, 한 decision64KiB 상한이다. 동일 decision ID 재사용은 revision_conflict이며 변경은 새 ID로 기록한다. 새 디렉터리+내용-addressed no-overwrite publish로 저장하고 read-only를 설정한다. 파일내용/이름 hash 불일치·다른조건의 forged pass flags·잘못된 기록은 읽을 때 거부한다.

한 item의 append는 delete-on-close transaction lock으로 직렬화하고 busy는 resource_exhausted다. 중단으로 생긴 incomplete record는 통과/빈 성공으로 처리하지 않고 needs_input으로 보고한다. pending/기존 기록을 자동 삭제하거나 덮어쓰지 않는다. read-only/SHA는 동일사용자가 모든 기록·이름을 악의적으로 바꾸는 상황에 대한 WORM/전자서명이 아니며 반환된 digest/별도 근거 봉인이 신뢰 anchor다. listing은 decision ID hex 순서의 immutable history이며 시간순/최신 판정 자동선택·집계승인을 하지 않는다.
