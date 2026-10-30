---
id: sot:kinekino/native-proposal-launch
title: Native proposal launch 입력 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Native proposal launch 입력 계약

2026-10-20 제품 등록 3단계. 구현은 `src/application/launch/native_proposal.rs`와 `native_proposal/options.rs`, Windows adapter `src/infrastructure/native_proposal_request.rs`다. [조립 계획](../../../plans/player/sdk-proposal-window-startup.md)의 후속 UI helper/main 연결은 아직 미구현이다.

## 입력

`--native-proposal-request <absolute-file>`와 명시 `--proposal-root <absolute-directory>`를 함께 사용한다. 두 옵션은 =형도 지원하며 중복·빈 값·상대 경로·누락을 거부한다. Native request와 --ui-demo/--smoke-test/--start-url(공백형/=형)의 혼합도 거부한다. 기존 proposal_root의 공통 파서를 재사용한다.

`native_proposal_options`는 구문과 충돌 검사 이후 Windows native-proposal-window 빌드에서만 options를 반환하고, 나머지 빌드는 typed Unsupported를 반환한다. 3단계의 feature-on 실행 미연결 staged guard는 5단계에서 main Native 분기·owned outcome 처리와 함께 해제했다. 현재 desktop_mode는 feature-on NativeProposal 모드를 반환하며 main은 worker 전 파일 decode/스타일 준비 후 기존 client를 단일 Native backend로 전달한다. feature-off Unsupported와 eframe fallback 금지는 유지한다. [제품 실행 계약](native-proposal-execution.md)을 함께 따른다.

요청은 UTF-8 JSON 한 문서이며 원 바이트 한도 65536이다. `decode_native_proposal_request`는 한도를 먼저 검사하고 Deserialize 및 기존 RequestContext::validate를 사용한다. schema_version은 문자열 "1"이며 다른 버전은 Unsupported다. context 전체 필드(request_id/api_version/expected_revision/idempotency_key/timeout_ms)를 그대로 보존한다. validate 성공을 SDK inspect의 해당 context 지원 성공으로 해석하지 않는다.

selection에는 proposal_id, document_revision, source_id, source_revision, source_sha256, target이 모두 필요하다. 두 revision은 기존 DecimalU64의 십진 문자열이며 full DocumentTextRef/SourceExtraction도 기존 domain wire를 재사용한다. target은 다음 형태다.

| kind | 명시 데이터 |
| --- | --- |
| card / highlight | index: u32 |
| cue_correction | index: u32, cue_id |
| document_correction | index: u32, source_ref: 전체 DocumentTextRef |
| document_repair | selection: {kind: "unselected"} 또는 {kind: "selected", source_ref: 전체 DocumentTextRef} |

index는 `checked_item_index`에서 usize로 checked conversion한다. repair는 index None이며 첫 항목을 선택하는 의미가 아니다. application에 UI 타입을 넣지 않는다. 실제 ID/revision/SHA 일치, target membership 및 ref의 snapshot 존재 여부는 기존 SDK inspect와 UI project_initial 정책에서 판정한다. 입력 JSON으로 TSX/inventory/executable/shell을 선택할 필드는 없다.

모든 DTO 계층과 tagged variant는 unknown/중복 필드를 거부한다. 미선택 repair는 빈 struct variant `Unselected {}`다. 초기 단위 variant는 Serde가 추가 source_ref를 허용하는 실제 결함이 있었으며 이를 최소 수리하고 실패 단언을 유지했다. Selected와 Unselected 또는 생략을 서로 대체하지 않는다.

## 파일 adapter

adapter는 바이트 읽기만 담당한다. immutable_storage_paths::root(parent)의 canonical 경로와 ancestor handle guard들을 읽기 종료까지 보존하고 file(root.join(file_name))을 연다. 기존 ordinary/local/non-reparse 제한과 공유 모드를 재사용한다. Read::take(65537)와 실제 읽은 바이트 수로 한도를 검사하며 metadata 길이만 신뢰하지 않는다. JSON/버전/context 정책은 application decode에 남긴다. lock·디렉터리 생성·내용 쓰기는 하지 않는다.

## 후속 UI와 소유 경계

DTO는 DocumentRepair Selected를 온전히 지원한다. 3단계 당시 InitialPanelStartup::prepare의 Some 거부 공백은 후속 UI4 공통 correction projection에서 보완됐으며 DTO→initial/live의 같은 fullref를 사용한다. None/첫 항목 fallback이나 DTO 지원 축소를 하지 않는다. production Correction/helper/cleanup의 정확 계약은 [UI helper](../presentation/product-native-window-helper.md), main 연결은 [제품 실행](native-proposal-execution.md)을 따른다. 과거 3단계 검증 기록은 당시 checkpoint 범위로 유지한다.

## 검증

원 실패/수리 로그·receipt·사본·diff·핀은 `runtime/quality-review/product-native-launch-input-r1/`에 보존한다. 최초 focused-off는 12 통과/1 실패(exit101,58481ms)로 unknown 단위 variant 결함을 재현했다. 수리 후 focused-off-r2는 15 통과/0 실패(37171ms), 기존 launch 전체는 14 통과(1449ms), feature-on focused도 15 통과(77317ms)다. focused의 15는 신규 DTO/options/adapter 14개와 기존 proposal_manifest 회귀 1개이며 중복 실행을 새 테스트 수로 합산하지 않는다.

unknown/중복 JSON, revision 정밀도·u32 범위, 필수 tagged repair, context 보존/검증, 정확 64 KiB·초과·잘못된 UTF-8, 옵션 충돌과 양 feature 상태의 no-fallback을 검사했다. Windows 실제 junction을 Rust DeviceIoControl fixture로 만들어 ancestor/final reparse 거부를 확인했고 ordinary file·exclusive writer·missing/relative/parent traversal도 검사했다. shell/GUI 실행은 없다.

root cargo fmt --check는 1712ms exit0, cargo clippy --offline --locked -j1 --all-targets -- -D warnings는 26420ms exit0이다. 이번 단계는 영향 launch/adapter 회귀를 실행했으며 과거 전체 test/CLI73/OS9 결과를 새 실행으로 주장하지 않는다. 모든 Cargo는 기존 C target/오프라인 locked -j1/단일 테스트 thread/2CPU BelowNormal runner로 직렬 실행했다.

최종 변경 8파일 inventory SHA256은 `B2A69A908163CC3282E36848A67FA25716FDD3221A2C1EFD14B68F64BFB11CAE`다. UI2단계 11파일 관측 inventory는 `215B94167313293829674568B504154EC4349A27A67DE9E92F446D9DF4D73332`로 기준과 동일하다. Cargo.toml/Cargo.lock/main/assembly startup은 before/after 핀으로 동결을 별도 관측한다. 제품 전체 완료나 실제 Native 창 실행 성공으로 해석하지 않는다.
