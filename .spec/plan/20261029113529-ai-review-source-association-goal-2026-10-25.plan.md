---
id: plan:kinekino/20261029113529
title: AI 실제 표본 검토 goal 및 source association 후계
status: active
created: 2026-10-29
updated: 2026-10-29
---
# AI 실제 표본 검토 goal 및 source association 후계

2026-10-26 중앙 설계 후속: [association/draft typed·저장·API 계약](../../sots/player/application/api/review-source-draft-contract.md)을 추가했다. 독립 association 생성/조회/새 revision과 draft read/save·Verified 전이를 전체 목표로 유지하고, r2의 결합 출판/Unverified-only는 구현 checkpoint로 분리한다. canonical 등록·durable adapter·CLI·실미디어·사용자 GUI는 아직 미완료다. 아래 옛 D 실행 recipe는 역사이며 현재 DATALAB은 [C NVMe 결정](../../adrs/player/adr-datalab-build-cache-2026-10-26.md)을 따른다. 원 seal-r2/소스/실패·mock9 증거는 변경하지 않았다.

2026-10-25 사용자 재정의 요청과 중앙 PM의 actual9 source-only 배정을 반영한다. 목표는 기존 모델·엔진·실제 표본 provenance를 보존하면서 공통 Rust SDK/CLI의 검토·수정·오류·취소·저장을 완성하고 실제 제품 GUI의 사용자 품질 검토까지 연결하는 것이다. 이 문서의 당면 범위는 highlights3의 영상 연결과 사용자 구간 draft 계약 공백을 SDK 담당에게 인계하는 일이다. 전체 품질 검토 완료를 의미하지 않는다.

Goal API는 사용자 reset 통보 뒤에도 이전 blocked objective를 반환했고 create_goal은 unfinished goal로 거부했다. 이전 목표를 허위 complete 처리하지 않았다. 이 문서를 재정의안의 로컬 정본으로 삼고 중앙에 API 불일치를 알렸다.

## 현재 관측과 불변 경계

- [기존 provenance 감사](../../sots/verification/ai/highlight-media-provenance-gap-2026-10-25.md)의 실제9 package `659b3cb6757259d8e770f6d3c96c4a77dafbab43c6ea540bcc7e0c58e4eb0197` 및 raw/candidate/판정은 수정하지 않는다. 모든 item range=null을 유지한다.
- `src/application/model_review/mod.rs`의 ReviewSession은 package/read_text_asset/append/decisions만 제공한다. source association과 사용자 시간 draft 계약은 없다.
- `src/domain/model_review/decision.rs`는 item.range가 없으면 비어 있지 않은 issue_ranges를 거부한다. issue_ranges를 구간 작성용으로 전용하지 않는다.
- `src/application/proposals/source_import.rs`는 요청 cue 시각을 검사하되 SourceCue.start_us/end_us를 None으로 보존한다. 요청 선언값을 검증된 영상 PTS로 승격하지 않는다.
- Sintel portrait는 개발자 선택 클립이다. Sprite는 다른 표본이다. 둘을 actual9 모델 선택 결과로 연결하지 않는다. 정확한 원영상 현존 경로와 영상 PTS 연결은 미검증이다.

## 제안 typed 계약 — 아직 미구현

별도 `ReviewSourceAssociation`은 association_id, package_sha256, item_id, source_request_sha256, media_asset_id, media_sha256, evidence_refs, revision을 가진다. media_asset_id는 SDK의 등록된 로컬 미디어 식별자이며 브라우저 임의 경로/URL이나 실행 명령이 아니다. 패키지 membership과 요청 hash를 먼저 검증한다. 경로만으로 같은 영상이라고 판단하지 않는다. 현재 등록 계약이 부족하면 Unsupported를 반환하고 가짜 playback source를 생성하지 않는다.

타임라인 근거는 `Unverified`와 `Verified { evidence_sha256, timebase, origin, mapping_revision }`의 명시 enum으로 분리한다. Verified는 실제 원영상 identity와 bounded PTS 관측 근거를 검증하는 어댑터만 생성한다. 모델 출력에 없는 시간을 이 상태 전이로 생성하지 않는다. 선언 cue 시간은 별도 provenance로만 남긴다.

`ReviewRangeDraft`는 draft_id, association_id, association_revision, draft_revision, authored_by=User, ranges와 note를 가진다. range는 DecimalU64 마이크로초 start<end이며 원영상 기준이다. 사용자 작성 draft와 모델 원본은 독립이다. 미검증 타임라인에서도 입력 자체를 저장할 수 있으나 playback_ready=false로 표시한다. 재생·내보내기 승인과 품질 판정을 자동 생성하지 않는다. 검증된 duration 경계를 넘는 범위는 거부한다. duration 미확인은 검증 완료로 표시하지 않는다.

제안 use case는 association inspect/create, range draft read/save이다. 저장은 expected revision CAS, 재요청 idempotency key, atomic publication과 재조회 계약을 요구한다. 취소 전 publication은 no-write, publication 후 응답 유실은 execution_unknown과 같은 key 재조회로 해소한다. 오류 이름과 transport 표면은 SDK 담당이 기존 ApiError/ExecutionHandle 계약에 맞춰 확정한다. 새 명령이 현재 지원된다고 문서화하지 않는다.

## 소유와 순서

SDK 담당: domain/model_review 아래 association/range_draft 책임 분리, application/model_review use case·ports·schema, infrastructure/model_review 저장 어댑터, application/api client·commands·schema/capability와 CLI 노출을 함께 소유한다. 기존 decision 정책과 proposal accepted-output 판정을 변경하지 않는다. 재생 담당은 등록 로컬 미디어 identity와 seek/clock/close 연결의 지원 여부를 확정한다.

UI 담당: SDK/CLI 실제 검증 후 기존 raw_review_state/raw_review_panel 및 typed host에 association 선택·사용자 구간 draft·미검증 상태·저장/오류/이력을 연결한다. 제품 정책을 TSX/GUI에 구현하지 않는다. 중앙 PM이 통합·빌드 슬롯과 root stage/commit을 배정한다.

## 검증 계획과 현재 실행 상태

Rust 계약 시험은 원 actual9를 읽기 전용 fixture로 사용하고 write는 독립 임시 저장소로 격리한다. wrong package/item/request/media hash, stale revision, 잘못된 범위, 미검증 타임라인 재생 거부, 취소 전후 publication, 동일 key 재조회, reopen 이력과 원본 불변을 포함한다. 원 fixture의 range=null과 기존 issue_ranges 거부 assertion은 유지한다. CLI로 성공/오류/취소/저장을 먼저 확인한 뒤 GUI 연결을 검토한다.

이번 작업은 소스와 작은 문서 읽기 및 계약 설계만 수행했다. Cargo fmt/test/Clippy, 실제 미디어 PTS/재생, GUI, 성능 시간은 실행·측정하지 않았다. 신규 추론·다운로드·worker 빌드와 Cargo/GUI는 중앙 배정 전 보류한다. 향후 빌드·임시 파일은 D runtime/build-cache만 사용한다. 코드 구현과 사용자 품질 승인은 미완료다.

## 스킬 적용 근거

modoo.today `modootoday/software-qa/test-fixture-and-mock-build` 본문을 읽었다(기존 고정 본문 SHA256 `a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100`, 부속 참조 없음). 입력 미제공 사항을 추정하지 않는 원칙, 실제 계약에서 성공/오류를 도출하는 원칙, 공유 fixture 읽기 전용·쓰기 격리, mock으로 실제 서비스/시간 동작을 입증하지 않는 원칙을 위 검증 계획에 적용했다. 현재 검색의 published-cli-quickstart 결과는 onboarding 검증용이므로 이 계약 설계에 채택하지 않았다. 타 언어 예제를 도입하지 않았다.

## 중앙 구현 분산 배정 후 checkpoint

`runtime/development/ai-review-source-association-r1/`에 격리 Rust domain/application port·use case와 9개 의미 시험을 작성했다. 상세 지원 경계·SDK/CLI 후크·기존 asset 계약 조사와 남은 실검증은 해당 `HANDOFF.md`에 기록했다. canonical SDK/API/CLI와 actual9는 무변경이다. association은 user draft와 함께 출판하는 값이며 독립 생성/변경·Verified PTS·실디스크 어댑터는 미구현이다. Cargo/fmt/Clippy/시험은 중앙 자원 보류로 실행하지 않았다. 빌드 출력은 D runtime/build-cache/native, TMP/TEMP는 D runtime/build-cache/tmp만 허용한다. 다음은 중앙 슬롯의 격리 소스 검증과 SDK 소유자의 DTO/저장·등록 연결이다.

후속 제한 배정: 직접 `rustfmt --edition 2024 --check src/lib.rs` 최초 exit1(포맷), 격리 모듈에 직접 rustfmt 적용 exit0, 재check exit0. Cargo/컴파일/시험/Clippy는 실행하지 않았다. `source-seal.json`으로 포맷 이후 소스와 HANDOFF 고정 목록·해시를 기록한다. 실제 기능 성공이나 품질 승인 근거로 계상하지 않는다.

중앙 단독 Cargo 슬롯 후속: fmt0→첫test101(E0308 mock MutexGuard 참조) 원 실패 보존→격리 tests.rs 지역 guard 변수 분리→fmt0/test9pass/strict Clippy0. 원 근거 `runtime/verification/ai-review-source-association-r1/`. conservative2 CPU2/BelowNormal/jobs1/offline·D native 전용 namespace 및 D tmp 사용. 보호26 metadata/root HEAD/index/추적 WIP 동일, writers0로 슬롯 반환. 새 namespace12,031,667B. source-seal-r2.json이 수리 후 소스·Cargo.lock·인계를 고정한다. 원seal은 보존한다. 실제 디스크 어댑터/API·CLI 연결·실제 미디어/GUI·사용자 품질 승인은 미완료다.
