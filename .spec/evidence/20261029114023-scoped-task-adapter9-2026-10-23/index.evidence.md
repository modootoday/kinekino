---
id: evidence:kinekino/20261029114023
title: 공통 scoped task adapter9 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공통 scoped task adapter9 검증

## 입력

`execution-leases.md`의 scoped adapter9 단독 heavy 배정에 따라 root `72c8132a01ca4168fdec8a0678121b67b4224b45`, Meari `b532fe0eed06d8526b755b11bc36416070244ef7`, lock SHA256 `E469A7204B9DC2AC82AC729E36FD8AC41449E1C382E9167A28A00A45C45789A2`를 사용했다. 기존 S/T와 runner SHA256 `EEA9DEE6E8BCE8950E74A297098044EFF1154CB334D7E6FA2C4C622B4005488D`를 재사용했다.

채택된 task3는 baseline으로 동기화했다. 이전 S5,169개에서 mod1 변경·신규2만 적용해 baseline5,171개가 됐다. SDK10은 그대로 유지했다. source9 SHA256 `F227A690B28A7A2C9A96F9DFBDB2E66131B120B8B571A08270A5995EC20F97F9`의 기존5/신규4만 추가했다. before9 SHA256 `B5192A130FA58043465322286573D29668AF9C2ABFA7294B18194F5F660D90D2`의 preimage5/부재4를 canonical과 S에서 재확인했다. before9의 과거 canonical 필드는 동결 당시 기록이며, 현재 mod preimage는 task3의 `66B1677E…`로 실제 확인했다.

R1/제품13/Media/HTTP 후보, 창·수동 시각 검토·VM/DPI·모델 실행은 포함하지 않았다. canonical/index 편집·stage/commit/push는 수행하지 않았다.

## 산출물과 실제 등록

원 증거: `runtime/verification/ui-scoped-task-adapters-r1/`. `04-executable-artifacts.json`은 Cargo JSON의 실제 library producer를 기록한다. target `kinekino_player`, `src/lib.rs`, profile.test=true, features `native-observer-test-support`와 `native-proposal-window`, fresh=false다.

고정 `bin/scoped9-tests-r1.exe`: 173,670,912B, SHA256 `5D30E6F597C4E249DA2C80D3D7AD63D150FD785225CD87637BA0343BAD900039`. 종료 후 producer와 고정본 해시도 일치했다. `05-list` 실제 등록1,097개, `06-ignored-list` 제외54개다. `selected23-registration.json`의 영향23개는 모두 제외 시험이 아니다. 목록 개수를 통과 개수로 계산하지 않았다.

## 검증 범위와 결과

`runner-receipts.json`의 9개 실행은 모두 terminal/exit0, CPU2/BelowNormal(16384)/원 Job 상속이다. Cargo locked/offline/j1 및 thread1, 입장·build/test 직전 fresh RAM/VRAM/C 여유를 기록했다.

- `01-fmt`: 전체 cargo fmt --check 통과.
- `02-strict`: `cargo clippy -p kinekino-player --all-targets --features native-observer-test-support -- -D warnings` 통과.
- `03-build`: 같은 feature의 `cargo test -p kinekino-player --lib --no-run --message-format=json` 통과.
- 고정 BIN `07-common8`: 신규 공통 binder8 통과.
- 고정 BIN `08-task-bridge4`: 신규 bridge 정책4 통과.
- Cargo `09-card-lease7`: `cargo test -p kinekino-player --lib --features native-observer-test-support presentation::web_ui_adapter::proposals::scoped_panel::tests:: -- --test-threads=1`의 기존7 통과. 같은 producer를 사용했다.
- 고정 BIN `10-original-bridge4`: 기존 scoped bridge4 통과.

고유23 passed / 0 failed / 선택된 ignored0이다. source 수정·추가 수리는 없었다. whole library/workspace/CLI/doc/ignored 실행을 추가하지 않았으며 선행 task3 전체1031 결과를 이번 후보의 전체 시험으로 표시하지 않는다.

| 시험 범위 | 개수 | 실제 fixture와 입증 경계 |
|---|---:|---|
| 신규 공통 binder | 8 | 원 Card SDK/store를 재사용해 실제 typed Highlight/Cue/Document create·inspect 및 원 save/별도 accept를 호출한다. 즉시 registration lease로 target/ref/labels-before-registration/오류·partial owner·명시 cleanup을 확인한다. 새 async worker lifecycle의 인수는 아니다. |
| 신규 task bridge | 4 | 최소 headless Host 그래프·원 NativeDocumentHandle·주입 commit port다. provider 필요성, suffix/scope 거부, 원 commit 실패 carrier와 provider 보존을 확인한다. 실제 교정 panel render·OS checked frame·사용자 입력은 아니다. |
| 기존 Card immediate | 3 | 원 Card wrapper의 stale/scope 거부, registration 실패 보존, 원 SDK edit/save/별도 accept/reopen 회귀다. |
| 기존 Card async lease | 1 | 원 ExecutionOwner/Handle·async output review·save/accept/terminal·quiescent unbind 뒤 원 execution 재연결과 worker 종료를 확인한다. 새 typed task async 경로로 확대하지 않는다. |
| 기존 current inspection | 3 | 원 ExecutionOwner와 inspect/release, 오류 및 rejected pending consumer 보존 회귀다. captured에서 live 권한 발급을 입증하지 않는다. |
| 기존 scoped bridge | 4 | 원 headless Native renderer의 opaque local callback/capacity resume/거부/commit 실패·scope 소유 회귀다. OS window 또는 Some checked-frame 관측은 아니다. |

## 보호와 반환

source9를 시험 후 baseline으로 정확히 복원했다. S 기존5,171개 bytes/mtime 차이0이며 task3와 SDK10은 유지했다. 신규107개는 `S-new.json`의 기존 scoped Card fixture store 산출물이다. 원 child52272(54개), 81200(21개), Cargo 시험 child67496(24개), 65208(8개)로 구별하여 보존했다. lock 파일 존재는 열린 writer를 의미하지 않는다.

root tracked1,348/library493/fixed82/index2의 bytes와 기록된 mtime 차이0이다. alias4도 입장 당시 SHA/현재 FileID/mtime 차이0이다. alias는 현재 소스 실행 증거로 사용하지 않았다.

T exe573개 중 변경1개는 원 Cargo producer `kinekino_player-4f59447537afc4e4.exe`이며 위 JSON/고정본으로 귀속한다. 이전73B3AF 고정본도 fixed82 안에서 보존했다. 나머지572개 bytes/mtime 차이0이다. knowncache2의 과거 SHA는 유지됐으나 귀속 미해결이므로 all_cache_protected=false다. 복원된 S를 현재 producer가 다시 빌드했다는 주장은 하지 않는다.

모든 원 실행 세션의 종료와 출력 EOF를 회수했다. `final-process-observation.json`은 빈 배열이며 담당 process/session/handle/writer/cleanup 대기0으로 명시 반환한다. raw/source9/before9는 불변이다. PM exact9 채택을 요청하며 직접 stage/commit하지 않는다.

## 남은 작업과 스킬

이 gate는 연결 전 공통 adapter/호환 wrapper와 선택된 영향 회귀다. 새 typed task의 실제 async worker·실제 panel source/입력 연결·Some checked frame·Viewer route는 후속 R1/R2 범위이며 인수하지 않았다. captured/current-inspect identity 일치 또는 ACK는 편집 권한이나 제안 채택이 아니다. repair는 이 entry에서 Unsupported이며 기존 독립 repair 기능을 변경하지 않았다.

앞서 modoo.today에서 탐색하고 본문을 읽은 `modootoday-free/design-delivery-plus/frontend-first-journey`를 원 typed port·공통 컴포넌트·미지원 repair 경계에 적용했다. `modootoday/software-qa/test-fixture-and-mock-build`를 원 SDK fixture/setup·원 parent/session·오류/partial/cleanup 보존 및 synthetic/실제 실행 분리에 적용했다. PowerShell은 파일·해시·시각·프로세스 메타데이터와 Cargo 실행에만 사용했다. 새 제품 정책·시험 구현은 Rust다.
