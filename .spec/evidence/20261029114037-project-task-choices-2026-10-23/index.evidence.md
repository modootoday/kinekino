---
id: evidence:kinekino/20261029114037
title: 명시 task 선택 source3 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 명시 task 선택 source3 검증

## 입력과 범위

실행 배정은 `.spec/plans/player/execution-leases.md`의 task3 단독 heavy다. root `8a7c0d65d48b26353cf84cc4a8cc181cd79c8c3d`, Meari `b532fe0eed06d8526b755b11bc36416070244ef7`, Cargo.lock SHA256 `E469A7204B9DC2AC82AC729E36FD8AC41449E1C382E9167A28A00A45C45789A2`를 사용했다. 채택된 SDK10을 제거하지 않았다.

`runtime/development/ui-project-task-choices-r1/source3.json` SHA256 `7058FDCF49482C3DDC0F589768F3332E652D97A653967C3B0D827D44ED1E7AA3`의 기존1/신규2만 기존 S에 추가했다. before3 SHA256 `B6A91BB34014C92C9EAB0648A922E5CC97CA6534C1BE3CDE48EB192A16A513FA`의 canonical/S preimage와 신규 부재를 확인했다. scoped9/R1/제품13/Media 후보는 합성하지 않았다. canonical 소스·index 편집, stage/commit/push, 창·수동 시각 검토는 수행하지 않았다.

## 실제 결과

원 증거는 `runtime/verification/ui-project-task-choices-r1/`에 있다. `runner-receipts.json`의 7개 실행은 모두 terminal/exit0, CPU2/BelowNormal(16384)/Job 상속이다. Cargo는 locked/offline/j1, 관련 thread 수1을 사용했다. 입장 및 build/test 직전 RAM/VRAM/C 여유를 기록했다.

- `01-fmt`: 전체 `cargo fmt --check` 통과.
- `02-strict`: `cargo clippy -p kinekino-player --all-targets --features native-observer-test-support -- -D warnings` 통과.
- `03-build`: 같은 feature의 `cargo test -p kinekino-player --lib --no-run --message-format=json` 통과.
- `05-list`: 고정 바이너리의 실제 목록 1,085개. 새 task 선택6 및 기존 Card 선택4 등록을 확인했다.
- `06-task3`: 새6 모두 통과. 원 identity/두 번째 항목/cue·문서 참조/명시 선택/stale key/캡처와 작업 초안 분리/repair 입력 부재/인덱스 변환을 검증했다.
- `07-card`: 기존 Card4 모두 통과.
- `08-full-lib`: 1,031 passed / 0 failed / 54 ignored, 107.89초. PM이 10개 영향 시험으로 충분하다고 알리기 전에 계획 문구에 따라 시작된 실행이다. 원 실행을 종료까지 보존했으며 추가 반복하지 않았다.

위 전체 시험은 root library 범위다. 전체 workspace/CLI 통합/doc/ignored 시험의 이번 실행으로 표시하지 않는다. stderr의 의도된 오류 fixture와 closeout 원문도 그대로 보존했다. 원 실행 세션은 모두 종료와 출력 EOF를 회수했다.

`04-executable-artifacts.json`은 Cargo producer를 기록한다. target `kinekino_player`, `src/lib.rs`, profile.test=true, features `native-proposal-window` + `native-observer-test-support`, fresh=false다. `bin/task3-tests-r1.exe`는 173,413,376B, SHA256 `73B3AF01C09D318286C9F495E2B3C926D802CF78B4DDFCBF82DA1A6710594AF2`이며 종료 후 producer와 고정본 해시를 다시 확인했다. 기존 root alias는 실행 근거로 사용하지 않았다.

## 보호와 반환

SDK 반환 당시 S5,096개는 입장 시 bytes/mtime 차이0이었다. source3를 시험 후 정확히 복원했다. 기존 S5,096개 bytes/mtime 차이0, SDK10 유지다. 신규73개는 `S-new73.json`에 남긴 기존 시험 store 산출물이며 기존 파일을 대체하지 않았다. Card4 원 child25004와 전체 시험 child66308의 `ui-scoped-card-store`, `ui-current-card-mount-store`, `ui-job-details-stores`에 귀속한다. lock 파일 존재를 열린 writer로 해석하지 않는다. 원 프로세스는 종료했다.

root tracked1,346/library493/fixed81/index2의 bytes 차이0이며 기록된 mtime도 차이0이다. root alias4는 입장 당시의 원 bytes/현재 FileID/mtime 차이0이다. 이 alias들의 과거 FileID 복원이나 현재 소스 실행을 주장하지 않는다.

T exe573개 중 변경1개는 위 Cargo producer다. 기존 producer CA074982는 `preimage/`에도 별도 보존했고 과거 고정본은 불변이다. 나머지572개 bytes/mtime 차이0이다. 과거 미귀속 knowncache2는 원 해시 그대로지만 귀속 미해결이며 `all_cache_protected=false`를 유지한다. 별도 고정본과 캐시 producer는 task3 입력의 결과다. S를 이전 bytes/mtime로 돌려놓았으므로 이 캐시를 복원된 S의 재빌드 증거로 사용하지 않는다.

`final-process-observation.json`은 빈 배열이며 담당 원 세션/프로세스/handle/writer/cleanup 대기0으로 반환한다. source3 manifest/raw는 불변이다. PM의 exact3 채택을 요청하며 직접 stage/commit하지 않는다.

## 남은 경계

이는 연결 전 typed 선택 projection이다. 표시 key는 권한이 아니며 같은 generation의 독립 프로젝트에서 재생성될 수 있다. 호출자가 원 owner/scope와 매 frame의 fresh generation을 검증해야 한다. fresh inspect가 캡처를 수정 권한으로 승격하지 않는다. repair 입력·Viewer route·교정/Highlight 편집·명시 채택·실제 화면·시각적 일치 인수는 별도 후속이다.

## 스킬 적용

앞서 modoo.today에서 검색하고 본문을 읽은 `modootoday-free/design-delivery-plus/frontend-first-journey`를 원 typed port/공통 선택 구조와 미지원 repair의 명시적 경계에 적용했다. `modootoday/software-qa/test-fixture-and-mock-build`를 원 domain fixture 검증, synthetic와 실제 실행 구별, 원 오류/owner/정리 보존에 적용했다. PowerShell은 파일·해시·시각·프로세스 메타데이터와 Cargo 실행에만 사용했으며 새 정책/시험은 Rust다.
