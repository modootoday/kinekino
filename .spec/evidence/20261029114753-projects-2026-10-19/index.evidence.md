---
id: evidence:kinekino/20261029114753
title: 프로젝트 저장 기능 검증 기록
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 프로젝트 저장 기능 검증 기록

상태: 2026-10-19 공통 Rust SDK/API·CLI 저장 구현과 아래 기능 검증 완료. GUI의 저장 API 연결·typed 상태 표시·입력 UX는 후속이며 전체 제품 완료가 아니다. 기능 검증은 SDK/CLI, GUI 검토는 UI/UX라는 최신 사용자 기준을 따른다. [기능 계약](../../player/application/api/projects.md)과 [인수 계획](../../../plans/player/project-persistence-acceptance.md)을 따른다.

## 소스 범위

domain/projects는 revision·선택 locator·captured StoredProposal·별도 working_draft·도출 상태·canonical SHA·크기 제한을 소유한다. application/projects는 live capture·saved capture 재사용·비어 있지 않은 문맥·ProjectStore port·CAS를 소유하며 API Request에 의존하지 않는다. api/projects와 project_schema가 dispatch/schema 경계를 담당한다. infrastructure/projects는 한 writer lock·불변 hash 파일·전체 이력 손상 검증을 소유한다. 역사 검증 시 한 record씩 읽어 모든 snapshot을 메모리에 누적하지 않는다.

기존 StoredProposal 값과 검증은 domain/proposals로 옮기고 application/proposals의 공개 경로를 reexport로 유지했다. valid_id/valid_sha256는 domain/identifiers가 소유하고 기존 model_review 경로는 reexport한다. 저장 wire는 바꾸지 않는다. 두 저장 adapter의 동일 rooted path/파일/lock 검증은 infrastructure/immutable_storage_paths에서 공유한다. root·main·CLI는 명시 project-root를 조립하고 GUI/CLI가 같은 typed ApplicationClient를 사용할 수 있다.

## 실행 원문과 실패

근거 경로는 `runtime/quality-review/projects-sdk-r1/`이다. 최초 all-targets check는 새 테스트의 미등록 tempfile 참조 E0433으로 exit101이었다. `logs/check-initial.log`를 그대로 보존했다. 새 dependency 대신 기존 Rust Temp fixture 방식으로 수정했고 check-r2와 실제 CLI 통합 테스트 추가 후 check-r3가 exit0이다. 전체 workspace test/clippy/fmt/bin 검사는 진행 중이다. source-at-audit는 그 검사 시작 시점 사본이며 전체 compiler-input attestation은 아니다.

새 통합 테스트는 기본 합성 proposal과 환경으로 지정하는 보존 실제9 proposal 입력을 구분한다. create→새 CLI inspect→typed client inspect→proposal backend 없는 saved 재저장→stale CAS 거부→과거 조회→잘못된 capture SHA·빈 문맥 거부→최신 torn publication의 현재/과거 조회 거부→원 proposal 불변을 검사하도록 작성했다. 테스트 작성 자체는 통과 근거가 아니다.

## 최초 전체 검사와 보강

최초 workspace 전체 테스트는 **163 pass/9 ignored**로 통과했다. 새 CLI/typed client 통합 테스트도 통과했으며 이는 기본 합성 fixture 실행이다. 최초 strict Clippy는 `tests/projects.rs`의 불필요한 unwrap_or_else 한 건으로 exit101이었고 `logs/clippy-initial.log`를 보존했다. unwrap_or로 수정한 뒤 `clippy-r2.log`가 exit0이다. hash 변조·추가 파일·이력 gap과 aggregate record/working 크기 테스트를 보강하고, 실제 fixture 실행 시 명시 CLI 경로·SHA pre/copy/post 검사도 추가했다. 이 최종 소스의 전체 test/build는 별도 logs/test-final.log와 build-final.log에서 진행 중이다. source-final이 해당 소스 사본이다.

## 최종 고정 실행과 실제 자료

최종 원문 합산은 **165 pass/9 ignored**다. fmt --all --check, strict workspace all-targets Clippy, bins/examples build 모두 exit0이며 최종 build는0.64초다. 제외9개를 성공으로 세지 않는다. 위의 초기 실패와 중간163개 실행은 당시 기록으로 보존한다. final logs와 source-final이 최신 checkpoint다.

고정 GUI SHA `084679d8f7012abcc66875e392927fe638001632f64935c3adab258581f52e23`, CLI `34b67069241213437fb3c276c84935cb33aa95a22f8f4059012919414d1bb115`, importer `f3f83c973bdb4257ef253ce0cf8d9a5908334c5a816b58a25a874b709599964f`다. final-bin과 pins.txt에 보존한다. UI 후속 변경을 이 실행본에 소급하지 않는다.

보존된 실제9 technical-store를 입력으로 고정 CLI SHA pre/copy/post 검사 후 프로젝트 통합 테스트를 추가 실행해3.79초/exit0을 확인했다. actual-nine-r1에는 captured source/raw/accepted 결과·별도 working buffer·canonical SHA를 보존했다. create/restart inspect/backend 없는 saved save/history/final restart/schema는0, stale save는5, 잘못된 saved SHA와 빈 문맥은2, torn 최신 revision의 현재·과거 조회는3이다. 각 request/stdout/stderr/exit와 테스트 원문을 보존했다. 원 proposal 9개의 canonical SHA는 전후 동일하다. 이 시험의 projects 저장소에는 의도적 손상이 있으므로 정상 UI 자료로 사용하지 않는다.

정상 ui-store는 별도 고정 CLI create→save→inspect를 새로 실행해 각각0으로 생성했다. project_id는 actual-work, 최신2/과거1이며 실제9 snapshot과 작업 buffer를 포함한다. 손상 자료를 자동 복구하거나 과거 상태로 대체한 것이 아니다. UI 담당에게 실행본·정상 store·typed client 계약과 presentation 소스 작업권을 인계했다. 실제 GUI 저장/재열기/충돌 성공은 아직 주장하지 않는다.

새 고정 CLI의 옛 S1/S2 proposal 읽기는0/0이며 저장 schema1/2와 receipt3를 유지했다. 기본 backend의 payload 없는 projects.create는 unsupported/exit3이다. 명시 root의 capabilities와 create/inspect/save schema를 보존했다. Cargo.lock은 분리 checkpoint와 동일하며 새 registry dependency·모델 추론·다운로드는 없다.

## 유지하는 한계

기존 문단의 GUI 저장/재열기/충돌 인수 표현은 최신 사용자 지시에 따라 위 SDK/CLI 기능 결과의 표시·입력·API 연결 UX로 해석한다. GUI에서 같은 기능을 반복 검증하도록 요구하지 않는다. 과거 GUI 행위 원문은 해당 실행본의 연결·UX 관측으로 보존한다.

locator를 열거나 미디어·URL·프로그램을 실행하지 않는다. 별도 root의 원자 transaction·현재 native media/PTS binding·project-bound acceptance 최신성은 미검증이다. 기존 project_current_revision_verified=false를 유지한다. GUI 저장·재시작·충돌 인수와 전체 시각 검토는 후속 고정 실행본에서 확인한다. 새 모델 추론·다운로드·벤치마크는 실행하지 않는다.
