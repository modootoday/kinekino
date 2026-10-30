---
id: plan:kinekino/20261029113944
title: LocalHost6 집중 gate 준비
status: active
created: 2026-10-29
updated: 2026-10-29
---
# LocalHost6 집중 gate 준비

실행 마감: [실제 정본](../../sots/verification/application/local-host-shutdown-2026-10-21.md). fixture exact1 수리 r2 B86C28B5…81DBE1/integrated6-r2 75CA764C…6BB5EC1에서 고유17pass·양strict 실제Checking/fmt0,보호snapshot1297/root1977/A7 delta[],최종owned[] 및 명시 반환을 확인했다. 원 운영 실패2/중첩과 fixture 실패1은 보존했다. 이후 아래는 준비 당시 이력이며 새 실행 배정이 아니다. CLI carrier를 섞지 않았다.

2026-10-21 계획만. SDK A7 원 실행의 명시 반환과 중앙 배정 전 실행0이다. source6 CBF11D50925E7C5FFC18D0CE5EB73711E6705C21A0622F432235055A9916F690만 대상으로 한다. 별도 CLI carrier4와 섞지 않는다. 전체 제품·viewerH·CLI4 caller 종료 통합 인수가 아니다.

## 입력·자원

중앙 지정 runner는 `D:/workspace/@myso-kr/com.kinekino.player/runtime/build-cache/resource-runner-topology/debug/resource-runner.exe`, SHA256 EEA9DEE6E8BCE8950E74A297098044EFF1154CB334D7E6FA2C4C622B4005488D다. 재사용 예정 native target은 `%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/public-p3-parent-r1/target`이다. 경로 통보는 실행 배정이 아니며 SDK 명시 반환 전 실행0을 유지한다.

실행 배정 후 정정: 위 snapshot/target 경로는 잘못된 cache 해석이므로 사용하지 않는다. **기존 target은 `%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/native`**다. 모든 Cargo 호출은 snapshot cwd에 더해 `--manifest-path %USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/public-p3-parent-r1/Cargo.toml --target-dir %USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/native`를 명시한다. CARGO_TARGET_DIR의 암시 해석에 의존하지 않는다. 첫 root cwd/잘못된 target과 두 번째 snapshot cwd/잘못된 target의 운영 실패·중첩·소유 종료는 `runtime/verification/local-host-shutdown-r1/operational-rejected.md`에 보존했다. source6 검증 결과와 합산하지 않는다. 아래 Cargo argv에도 이 manifest/target 인자를 추가한다.

합류 시 source6 각각의 원 SHA/bytes와 snapshot 합류 후 SHA/bytes/현재 LastWriteTimeUtc를 같은 입력 manifest에 기록한다. 예전 mtime을 보존해 복사하지 않는다. 실제 Cargo Checking/Compiling 로그 및 신규 executor3/server5의 정확한 시험 이름·실행 결과를 이 입력에 연결한다. 단순 fresh 캐시/짧은 실행시간/strict exit0만으로 source6가 재컴파일됐다고 주장하지 않는다. 재컴파일·정확 이름 확인이 안 되면 결과를 인수하지 않고 중앙에 보고한다. target 전체 삭제나 무단 clean으로 우회하지 않는다.

중앙이 지정한 최신 root/Meari·기존 보호 manifest를 base로 보존하고 source6 정확6만 할당 snapshot에 합성한다. source6/before3/합성후6 및 보호경로 차이를 보존한다. root live/index를 변경하지 않는다. canonical resource-runner의 현재 인수 핀과 conservative2 프로필을 확인하고, 기존 정책의 2CPU/BelowNormal/descendant Job·offline/locked/-j1/testthreads1을 따른다. runner의 전체 인자는 중앙 최신 사용 계약을 재사용하며 이 문서로 추측하지 않는다.

## 순차 Cargo target

아래는 runner가 실행할 Cargo argv 계획이다. broad `cargo test -- --list`는 하지 않는다. 기존 source에서 신규 server tests5/executor tests3과 integration tests/local_host9를 읽어 범위를 고정했다.

1. `cargo test --offline --locked -j 1 --lib infrastructure::local_host::executor::tests:: -- --test-threads=1 --nocapture` — 신규3.
2. `cargo test --offline --locked -j 1 --lib infrastructure::local_host::server::tests:: -- --test-threads=1 --nocapture` — 신규5. Windows named-pipe 및 SDK barrier, panic은 통제된 headless 범위다.
3. `cargo test --offline --locked -j 1 --test local_host -- --test-threads=1 --nocapture` — 기존9. 실제 CLI helper는 Cargo의 CARGO_BIN_EXE_player-cli를 사용하므로 그 바이너리 소스/해시도 확인한다. 불필요한 전체 integration 재빌드는 하지 않는다.
4. `cargo clippy --offline --locked -j 1 --all-targets -- -D warnings` 및 같은 명령의 `--features native-proposal-window` 구성.
5. `cargo fmt --check`.

새8+기존9는 고유17 후보이며 strict에 의해 컴파일된 시험이나 같은 시험 반복을 pass 수에 더하지 않는다. source6 compile/strict 실패가 있으면 원 입력·원 로그를 보존하고 별도 수정 pin을 인수한 후 필요한 범위만 재실행한다. native/default strict는 기능 실행 성공을 뜻하지 않는다. peer token OS 시험·GUI/audio·전체 cargo test는 이번 집중 gate에 자동 추가하지 않는다.

## 마감

각 원 runner/child handle의 실제 terminal/exit·시간·PID/프로세스 identity를 보존한다. handle timeout은 종료가 아니며 중복 시작하지 않는다. LocalHost/CLI/fixture 소유 worker와 원 하위 프로세스를 종료 근거로 정리하고 PID 재사용은 identity로 구분한다. manifest before/after·보호경로 차이·CLI artifact hash·receipt 목록·원 실패/경고를 포함해 중앙에 슬롯과 snapshot을 명시 반환한다. 단순 Drop/client Closed는 양 join receipt로 부르지 않는다. source6의 startup ready.send 실패 detach 및 caller 호환 차이는 통과 후에도 별도 미완료다.
