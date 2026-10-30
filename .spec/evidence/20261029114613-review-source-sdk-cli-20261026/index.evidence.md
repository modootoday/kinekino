---
id: evidence:kinekino/20261029114613
title: F1 SDK·schema·CLI 연결 checkpoint
status: active
created: 2026-10-29
updated: 2026-10-29
---
# F1 SDK·schema·CLI 연결 checkpoint

2026-10-26. reviews.association.inspect/create/revise 및 reviews.range_draft.read/save를 공통 ReviewSourceService·DTO·schema/capability·PlayerSdk와 CLI에 연결했다. 정상 CLI exe의 실제5명령 인수는 아직 미실행이다.

## 계약

association write payload는 case/association_id/media/note다. create는 expected revision 없음, revise는 필수이며 다음 revision을 계산한다. mutating context의 idempotency_key는 필수다. draft save는 case/draft/optional create_association을 받으며 원 exact binding/revision 검증을 재사용한다. association payload의 caller Verified/evidence/unknown field는 거절한다.

read는 case/object ID/optional revision을 받고 mutation용 revision/key/timeout context를 거절한다. 반환은 media_check=not_checked·playback_ready=false·automatic_application_enabled=false다. mutation은 원 receipt와 readiness false를 반환한다. timeout 기본30000/최대120000ms를 publication guard까지 전달하며 strong_cancel_available=false다. publication 후 불확실성/same-key 조회 정책을 유지한다.

schema는 strict request/response/context와 transaction receipt를 제공한다. capability는 adapter 연결/readonly/writable을 구분하고 timeline/strong cancel 미지원을 표시한다. PlayerSdk의 기존 builder4곳이 새 adapter를 보존하며 default는5명령 unsupported다.

Windows open_api는 기존 rooted guard와 canonical 경로 대소문자 정규화로 evidence/user root 동일·상하위 포함을 거절한다. CLI --user-review-root는 --review-root를 요구하고 endpoint/중복 조합을 거절한다. 두 root는 기존 일반 로컬 절대 디렉터리여야 한다. 읽기는 readonly로 namespace/기록을 생성하지 않는다. 다른 요청은 명시 user root의 write-enabled namespace를 준비하므로 capabilities/schema도 cases namespace를 만들 수 있다고 help에 표시했다. 일반 in-process CLI assembly이며 native host/GUI 신규 연결 완료가 아니다.

## 실제 검증

원로그/receipt: runtime/verification/source375-datalab-b0-r1-logs.

- 115 command-tests:17pass/0fail/0ignored/1412filtered, exit0/67624ms. 실제파일 fixture의5명령 처리·create/draft replay·context/unknown Verified/timeout 상한 거절·readonly 쓰기거절·과거 association binding.
- 116 sdk-tests:17pass/0fail/0ignored/1412filtered, exit0/67522ms. 기존 custody 시험1을 확장했다. root 동일/포함 거절 후 cases생성0, 실제 PlayerSdk read,5schema supported 조회, capability write/strong_cancel false, default unsupported. 별도 JSON Schema validator의 전수 검증은 아직 아니다.
- 117 root all-targets/native-proposal-window strict exit0/25437ms.
- 118 CLI runner::tests:6pass/0fail/27filtered, exit0/61427ms. 신규 user root 정상/단독·중복·endpoint 거절 및 기존 파서 회귀. libtest exe이며 정상 CLI 실제5명령 실행이 아니다.
- 119 fmt --check exit0/2431ms.120 최종 root all-targets/native-proposal-window strict exit0/1126ms. 기존 paint deprecation은 원로그 보존.

locked/offline/jobs1/CPU2/BelowNormal/C native/tmp·기존 toolchain. root index0·heavy 종료를 확인했다. 두 lock/원 Media seal SHA는 기존 pin 불변이다. actual9/raw/decision·타 WIP·추론/다운로드/수동GUI/WARP/push 보류 유지. 스킬은 [case custody](review-source-case-custody-20261026.md)의 modootoday/software-qa/test-fixture-and-mock-build 본문/출처/자원 재획득 원칙을 이어 적용했다.

## 주요 코드 SHA256

| 파일 | SHA256 |
|---|---|
| src/application/review_source/api.rs |648B0C19EC91B779B8E3D80F50ADE4C681D2D9B03F39C7106B37F07980123566|
| src/application/review_source/schema.rs |B3AEB22213F463B8F17E443132A2D77720DF61575AEA72F5978CA0814364782C|
| src/application/api/mod.rs |CEBDD0A3E3AA8F49537E5A5DD0D4F59A5A94D265A0CA7CE596470D81571E0C8D|
| src/application/api/capabilities.rs |033801A9BE465A5381611891F0904D0A48C4343675784D3F5F87DF7E5D0F474E|
| src/bin/player-cli/runner.rs |8395C41DEEB4182F947AACCEB16B91063B606EFD3726257F0E44A14D0D21C4D6|
| src/infrastructure/review_source/mod.rs |F316C8DBD226BC79110D41899E215C1FD8CCE15BCC7E2267F751AB03D45F9491|
| src/infrastructure/review_source/case_tests.rs |B947B1B41D3345FAB8663363E4B2C93A860A2FFA62138EEDFE842FF0282A01A3|

다음 gate: 최신 normal CLI 실제5명령·오류/저장/재열기·schema 정합성과 process 경계 복구/경합, 최종 입력의 전체190/정상2bins. 다른 custody/실제9 원미디어 연결·Verified PTS·AI 사용자품질·GUI/시각·제작 잔여를 유지한다. 이전8A의190/bins를 현재 source 성공으로 승계하지 않는다.
