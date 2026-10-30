---
id: evidence:kinekino/20261029114705
title: F1 정상 CLI 실제5명령 종단 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# F1 정상 CLI 실제5명령 종단 검증

2026-10-26. 앞선 [SDK/CLI 연결](review-source-sdk-cli-20261026.md) 후 최신 정상 player-cli.exe를 직접 실행했다. 별도 합성 package/media와 user root만 사용하며 실제9/raw/decision·타 WIP는 변경하지 않았다. synthetic video bytes는 custody/명령 저장 검증용이며 decode/PTS/품질 승인 근거가 아니다.

## 실행과 증거

runtime/verification/source375-datalab-b0-r1-logs/121-review-source-normal-cli: 최초 종단1pass, exit0/44338ms.122-review-source-normal-cli-final은 owned child RAII와 readonly 회복 시험을 추가해 최종2pass/0fail/0ignored, exit0/29971ms, 시험21.18s였다. Cargo integration test의 CARGO_BIN_EXE_player-cli가 가리키는 정상 exe를 실행했다. Cargo test 자체의 libtest exe와 실제 호출된 CLI exe를 구분한다.

최종 fixture는 runtime/build-cache/tmp/review-source-cli-8796-0 및 -1이다. 첫 fixture의19회·두 번째2회, 총21개 별도 CLI 프로세스가 실행됐다. 각 요청 JSON/stdout JSON/stderr/process.json에 원 PID/exit/입출력 경로/CLI SHA를 보존했다. 각 호출 전후 CLI SHA를 비교하고 원 package·두 asset SHA·원 decisions0/cases없음을 확인했다. child stdin은 null, 출력은 파일, 기한30s다. helper가 소유한 child만 timeout 시 종료/wait하며 RAII에서도 잔여 child를 회수한다. 최종 프로세스 조회에서 프로젝트 Cargo/rustc/runner/player-cli 잔여0이었다.

123 fmt --check exit0/2720ms,124 root all-targets/native-proposal-window strict exit0/1031ms.125 명시 cargo build --locked --offline -j1 -p kinekino-player --features native-proposal-window --bins --message-format=json exit0/556ms. 두 compiler-artifact는 canonical C native/debug 경로·fresh=true·profile.test=false다. GUI는 실행하지 않았다.

| 정상 파일 | SHA256 |
|---|---|
| runtime/build-cache/native/debug/player-cli.exe |050E47FA9A97784D3D699EB2FAED1FC5797F80CD4CFA60F5F4C0A7B3F9890833|
| runtime/build-cache/native/debug/player.exe |C3D5E8C8CB679BF57F6FC4C47F09C2B62B38D328127C671B1ACDA370E74C7EA8|

실제 호출별 process.json의 CLI050e SHA와 후속 정상build 파일050E SHA가 같다. 과거8A 기준본의 bin SHA/190 성공과 새 결과를 구분하며 원 기록을 보존한다. 이번 추가 source는 앞선 F1 후계와 아래 신규 Rust 시험4파일이며 전체 source closure의 신규 pin/전체190 재검증은 아직 아니다.

## 입증한 동작

- association create→새 프로세스 exact replay→동일key 변경note RevisionConflict→revise(expected1) revision2→새key stale expected1 거절→inspect current2.
- draft save는 과거association1에 명시 연결→새 프로세스 replay→read에서 원association1 유지, media_check=not_checked/playback_ready=false→mutation context를 붙인 read 거절→draft expected1 update2→재프로세스 read에서 수정note 확인.
-5명령 schema supported 조회·capability strong_cancel=false, caller Verified/unknownfield InvalidArgument. schema query 성공은 독립 JSON Schema validator 전수검증을 대신하지 않는다.
- 빈 user root의 readonly read는 NotFound이며 directory/file 생성0. 후속 명시 association writer는 같은 root에 사용자 namespace를 만들고 성공했다. 원 evidence는 끝까지 불변이다.

모두 locked/offline/jobs1/CPU2/BelowNormal/C native/tmp·기존 toolchain. Root/Meari lock과 원 Media seal SHA는 기존 pin 그대로, root index/stage/commit/push0. 실제 process 강제중단/출판중ACK유실·경합/전원손실은 이 정상종료 시험과 다르며 미완료다.

## 시험 source SHA256

| 파일 | SHA256 |
|---|---|
| tests/review_source_cli.rs |CB75A8195D9ABF73993EE0F1C22DAA166B756C2DBF6416AE740D0ECE645C4C0B|
| tests/review_source_cli/fixture.rs |289D73D72420A80284DF6754BF588E310BD0F45CD4DA04F00C90D9FC8B150A55|
| tests/review_source_cli/process.rs |C788139399D6E0D5B7DBD0C7A1AA112A72E06B2D169738E0D51307C8419F17EB|
| tests/review_source_cli/readonly.rs |BDD7C77DBD4F2CD04BC11C62FFB38512290F7A604BD10955738EE953DE525073|

스킬: 앞서 읽은 modootoday/software-qa/test-fixture-and-mock-build SKILL.md SHA a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100의 독립 fixture·실자원 해제/후속 획득·mock과 실제의 구분을 적용했다. 실제 CLI와 파일시스템으로 기능을 확인했고 합성 media의 품질·타임라인을 승인하지 않았다.

다음은 process 경계의 강제중단/경합과 schema 정합성, 최신 입력 pin을 고정한 전체190 회귀다. 다른 custody/실제9 원미디어 연결·Verified/AI/GUI/시각과 다운로드/제작 잔여를 전체목표에 유지하며 추론/다운로드/수동GUI/WARP/push 보류를 해제하지 않는다.
