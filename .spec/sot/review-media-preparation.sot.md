---
id: sot:kinekino/review-media-preparation
title: 원 결과 보존형 로컬 미디어 준비
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 원 결과 보존형 로컬 미디어 준비

## 공통 SDK/CLI 연결 후계

R72 구현 후계의 명령은 `reviews.media.prepare`와 `reviews.media.inspect`다. payload는 `{base_case:{package_sha256,item_id,source_request_sha256},selection_id}`만 받는다. 임의 경로·asset metadata·clock/quality flags는 받지 않는다. 두 명령은 expected_revision/idempotency_key를 거절하며 deterministic immutable package identity를 사용한다. prepare만 timeout_ms(1~120000)를 받고 기본30000ms의 publication 전 checkpoint에 적용한다. inspect는 timeout context도 거절한다. 발행 후 취소를 rollback으로 표현하지 않는다. strong_cancel_available은false로 유지하고 실행중취소 전체 인수로 주장하지 않는다.

typed client는 `prepare_review_media`/`inspect_review_media`이며 요청 case·selection ID와 응답 relation/case의 대응, 새 package hash, unverified/품질false 불변조건을 재검사한다. receipt는 selection_id, 새case, 원case/candidate/media 관계, replayed를 반환한다. inspect는 원 case와 고정 선택 metadata에서 결정적인 package를 다시 계산하고 기존 ReviewFiles로 exact content-addressed package와 모든 objects를 검증한다. 이미 발행된 package를 읽는 데 외부 선택 경로가 계속 존재할 필요는 없다. 원본 검증은 계속 요구한다.

`ReviewSourceService.with_media_preparation` opt-in 포트만 지원한다. 기존 source service만 있거나 기본SDK면 두 명령은 Unsupported다. 읽기 전용 포트는 inspect만 지원하며 prepare를 거절한다. discovery100은 기능완료100을 뜻하지 않고 기존98 기록은 해당 원입력의 역사다.

정상 CLI의 설정은 `--review-root ABS --user-review-root ABS --review-media-config ABS_JSON --review-media-config-sha256 SHA`다. 설정 file은 기존 RetainedReviewStartupSettings의64KiB·SHA·root/file guard로 보유한다. strict JSON은 destination_root, max_total_bytes, selections(ID→{path,asset})만 가진다. 두 설정flag는 함께 필요하고 local review/user root가 없거나 remote endpoint와 섞으면 거절한다. 이 명시적 로컬 선택 설정은 historical/media clock 권위가 아니다. 새 모델/네트워크/decoder는 준비·조회에서 실행하지 않는다.

위는 구현 계약이며 정상CLI 실제 실행·구성별 발견100·SDK/CLI parity·취소/저장 전체 검증은 해당 원 실행 근거로 별도 기록한다. 하단 초기 API/CLI 미연결 표기는 당시 단계의 역사다.

2026-10-28 중앙 PM. SDK application의 `review_source::media_preparation::prepare`는 exact 원 package bytes와 ReviewCase, 명시 media metadata를 받아 결정적인 새 package body와 `review-media-selection/1` 관계를 만든다. infrastructure의 `timeline_evidence::publication::preparation::prepare_local_media`는 신뢰된 Rust 조립 전용 경로이며 아직 공개 CLI 명령·지원 capability가 아니다.

원 case의 package/item/source-request를 검증하고 원 candidate 전체의 canonical JSON SHA를 관계에 보존한다. 미디어는 기존 asset ID/SHA/bytes/MIME 계약과1GiB 한도를 사용한다. 원 item source/range/baseline/candidate output·model 조건·기존 checks·criteria·known issues와 나머지 items는 그대로 유지하고, 선택 item에 별도 NotRun check와 media/relation/base-package assets만 추가한다. 원 package hash·후속 package hash는 다르며 원 판정/proposal을 후속으로 자동 이동하지 않는다.

선택 관계는 `explicit_local_selection`, historical_media_verified=false, timeline_verified=false, quality_approved=false다. source metadata를 보유한 것이 역사 실행과 같은 원본/clock을 검증했다는 뜻은 아니다. derived package 자체에 새 hash를 넣어 순환 identity를 만들지 않고, 반환된 새 case와 package 안 relation이 결합된다. 기존 시간축 publisher는 이 준비 패키지를 새로운 base로 사용할 수 있으나 반드시 새 producer와 원 retained media를 입장시켜야 한다.

로컬 path는 Deserialize를 구현하지 않은 trusted assembly 설정이다. request의 임의 path/URL/hash를 권한으로 받지 않는다. source parent/root/file과 원 ReviewFiles를 보유하고 대상은 원 evidence·source parent·명시 excluded roots와 분리한다. content hash는 bounded cooperative checkpoint로 확인한다. 원 complete objects를 기존 objects namespace로 복사하고, 전체 budget preflight 뒤 기존 timeline writer를 독점한다. object 부분 복사 실패는 orphan으로 보존하며 동일 hash의 다른 내용은 거절한다. 새 package의 create-only final publication이 유일한 commit이다. 동일 입력은 결정적인 같은 body를 만들어 exact replay하며 기존 package 내용을 덮어쓰지 않는다.

한도는 기존 package512KiB/asset1GiB/전체4GiB 이하다. 미디어 decode·timeline·playback·historical provenance·사용자 품질 검증은 수행하지 않는다. 시간 없는 correction과 cue 선언 정보는 기존 상태를 유지한다. source metadata·본문·관계의 저장/읽기 검증과 실제 미디어 producer 인수는 분리한다.

검증은 원본/후보/판정 불변·새 package custody·exact replay·wrong case/media·budget·선행 취소·기존 partial object 보존부터 진행한다. 실제9·공통 SDK command/CLI·실제producer·GUI 연결과 publication 중단/권한/전원손실 전체 행렬은 별도 미완료다. 기존 WIP·소유권·원 실패와 보류를 유지한다.
