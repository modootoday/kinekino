---
id: plan:kinekino/20261029113709
title: CLI 종료 배선 r5 gate 계획
status: active
created: 2026-10-29
updated: 2026-10-29
---
# CLI 종료 배선 r5 gate 계획

상태: active, 2026-10-21. source-only 작성 완료/독립 감사 전이며 실행 배정은 아직 없다.

실행 마감: [실제 정본](../../sots/verification/application/local-host-cli-wiring-2026-10-21.md). 최종 r6 source11 247F238E…A67339/integrated11-r6 257FB0E2…8990EAC에서 실제26 및 r5 별도격리parent1의 고유27 근거·양strict/fmt0·보호차이[]·원identity owned[]를 마감하고 heavy/snapshot을 반환했다. r6에서 실30초를 다시 실행하지 않은 한계를 유지한다. 원 hash 입력101·strict101을 보존했다. 아래는 준비 이력이며 새 실행 배정이 아니다.

## Outcome — 결과

CLI4의 공통 종료 소비와 DTO를 Rust 회귀로 확인하고, 실제 runner 비정상 boundary 및 브라우저 없는 job EOF를 원 프로세스 handle로 인수한다. 실제 native/provider/playback 미디어·브라우저 지원이나 GUI 종료를 완료로 표시하지 않는다.

## Authority — 담당

중앙 PM이 source11 독립 읽기·최신 snapshot·heavy 슬롯과 stage/commit을 소유한다. 이 세션은 지정한 exact11과 순차 gate만 소비한다. 30초/abnormal 정책은 [ADR](../../adrs/player/adr-cli-host-shutdown-observation.md), DTO는 [wire 정본](../../sots/player/application/local-host-cli-shutdown-wire.md)을 따른다.

## Evidence — 현재 근거

r4 P2 수리는 독립 소스 인수 상태이며 실행0이다. r5 E0AA336BA8E43C131B97B4CE71A198715AAE2A40A3BFFE587A4A38D51420C310는 runner 등록1+격리 fixture 신규1만 추가한 exact11이다. parent/child ignored와 phase/--exact, 원 child handle, 45초 watchdog, 출력각64KiB 제한을 작성했다. 현재 독립 current_exe/subprocess 실행은0이다. 이전 LocalHost6 고유17는 별도 base 근거이며 이번 시험 수에 합산하지 않는다.

## Unverified — 실행 공백

타입/Clippy·실제30초·abnormal code/진단·closed-output 및 parent cleanup·실 job file owner 종료는 미검증이다. test binary의 job handler와 실제 player-cli.exe 파일 실행은 구별한다. full suite·실브라우저·실미디어·모델 실행을 자동 확대하지 않는다. child output/wait 실패는 정상 인수와 구별하고 원 로그를 남긴다.

## Phases — 정확한 실행 범위

중앙이 B15+UI7+Local6/A7/Meari98f6 최신 인수 snapshot을 배정한 뒤 보호 전체와 before11(기존6/부재5)을 동결한다. 정확11을 현재 mtime으로 합류하고 SHA/bytes/mtime·실Checking/Compiling·정확 test 이름을 연결한다. source4/r4 단독 gate와 섞지 않는다.

모든 build/test/clippy는 snapshot cwd에 더해 절대 `--manifest-path %USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/public-p3-parent-r1/Cargo.toml --target-dir %USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/native`를 명시한다. 지정 EEA9 runner/conservative2·offline/locked/-j1/testthreads1을 유지한다. broad test --list와 새 캐시 생성은 없다.

1. `cargo test ... -p kinekino-player --bin player-cli host_shutdown::tests:: -- --test-threads=1 --nocapture` — regular11 후보.
2. `cargo test ... -p kinekino-player --bin player-cli runner::tests:: -- --test-threads=1 --nocapture` — 기존3+wrapper/BrokenPipe2=regular5 후보.
3. `cargo test ... -p kinekino-player --bin player-cli runner::isolated_shutdown_tests::production_boundary_uses_isolated_children_for_timeout_panic_and_job_eof -- --exact --ignored --test-threads=1 --nocapture` — parent1이 child3case를 직렬 실행. child entry는 직접 Cargo target이나 독립 pass로 세지 않는다.
4. `cargo test ... -p kinekino-player --test local_host -- --test-threads=1 --nocapture` — 기존9.
5. `cargo test ... -p kinekino-player --test jobs actual_cli_explicit_host_and_sdk_share_durable_jobs_without_gui -- --exact --test-threads=1 --nocapture` — 기존 실제 player-cli.exe job host EOF 포함1. 기존 fixture의 stop receipt 외 새 DTO는 앞 parent case에서 구별해서 확인한다. 별도 provider/native/browser ignored 시험은 선택하지 않는다.
6. default/native-proposal-window `cargo clippy ... --all-targets -- -D warnings` 및 `cargo fmt --manifest-path ... --all -- --check`.

고유 의미 있는 시험 후보는 regular16+parent1+기존LocalHost9+기존job1=27이다. 신규 작성15/child3case/반복구성/ignored entry는 이 합계에 중복하지 않는다. 실제 시험 이름·선택 수가 다르면 원 결과로 정정한다. 원 exec handle terminal 뒤 다음 target으로 진행한다. 실패는 원 log·input pin 보존 후 별도 수리핀으로 인수한다.

## Rollback — 마감·실패 보존

실행/부하 허용 실패 시 다른 작업을 종료하지 않고 원 handle을 보존한다. 소유 PID tree는 identity로 확인하며 runner 종료가 자손 kill을 보장한다고 해석하지 않는다. parent watchdog은 직접 spawn한 Child만 kill/wait한다. 최종 source11/보호/nonoverlap/CLI artifact·receipt·원 PID identity·재사용·소유 cleanup을 마감하고 중앙에 snapshot/heavy를 명시 반환한다. root/index는 변경하지 않는다. source11 인수 실패는 live에 합류하지 않고 r4·r5 원판과 진단을 보존한다.

modoo.today plan/plan-authoring 본문의 관측/가정·결정권/완료조건/복구 분리를 적용했다.
