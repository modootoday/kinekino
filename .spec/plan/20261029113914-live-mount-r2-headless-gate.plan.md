---
id: plan:kinekino/20261029113914
title: R2 successor1 최소 headless gate — 실행 배정 전 계획
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R2 successor1 최소 headless gate — 실행 배정 전 계획

2026-10-24. 현재 배정은 문서·소스 읽기뿐이다. Cargo/S/T/GUI 실행0. PM의 별도 실행 배정과 재입장 보호 확인 전 아래 절차를 실행하지 않는다.

## 고정 입력과 재입장

R1 채택 aa88706e2ff2e1bf1e9f71ec681aa9de9dcb931a 이후 읽기 관측 root는 09fc7518407798be1480f17aa81f2efa1bdef8d3(Media A 채택)다. 실제 실행 직전 HEAD/index/dirty inventory를 다시 고정하고 새 변경을 임의 합성하지 않는다. Meari b532fe0eed06d8526b755b11bc36416070244ef7, Cargo.lock SHA256 8F2C1D56C8A2C22D36D9E57A285D727C7EE9C7D2D1160902B66CC6630B042C67를 재대조한다. 다르면 PM에 변경 계보를 인계하고 기존 pin의 실행으로 표시하지 않는다.

입력은 runtime/development/ui-live-mount-r2-successor1/source12.json SHA256 C5BD3B83776EA4D149422AC460FE19B98E526985C656C1A580B37960252ECB8E의 exact12다. delta1 53C8F7848D7D0A709E37CC721F0A650B2A47D4C6FF65F2CE931E3AFA244F4B77, private before12 8D819861BFBCCB1AB98FC844153E634FBFFBDD6DA82FEF90FB576CB1533C1DEE, predecessors 9455A8DA10EED0BD3E93626DC2CE5B0A9BB2F35E9B330DABB3E1878CCEDF0109. 원 R2 raw12와 원 R1/repair1–4는 불변이며 successor는 fixture leaf1만 상속하고 다른11은 동일하다. before12는 private checkpoint의 preimage이며 현재 canonical/S preimage가 아니다.

exact12 경로의 정본은 위 source12 manifest다. viewer_host의 mount_card.rs/live_entry.rs/live_entry/tests.rs/cards.rs/mod.rs/return_viewing.rs/mount_proposal.rs/mount_proposal/tests.rs, player_shell/document의 product_frame.rs/product_frame/tests.rs, player_shell_host/observation의 retained.rs/retained/tests.rs에 한정한다. 현재 canonical R1을 predecessor로 사용하는 기존10/new2 구성이며 실제 존재·hash를 다시 확인한다. 불일치 파일을 덮어쓰거나 범위를 늘리지 않는다.

## 공유 소스·산출물 보호

S=%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/public-p3-parent-r1, T=%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/native. 실행 직전 현재 S 전체 inventory와 exact12 bytes/mtime/부재, 채택 R1·SDK/product·Media A 입력을 별도 보존한다. 이전 gate의 S snapshot을 현재 baseline으로 간주하거나 Media A를 되돌리지 않는다. root tracked/library/index, lock, fixed95의 실제 inventory/hash/mtime, alias4 hash/FileID/mtime, known cache2와 T exe inventory를 새 보호 근거로 기록한다. fixed95가 다르면 actual inventory 차이를 PM에 먼저 인계한다. known2의 과거 귀속 미해결 상태를 유지하며 all-cache-protected로 승격하지 않는다.

S exact12만 명시 적용하고 canonical은 변경하지 않는다. 임시 신규 파일·기존 파일을 구별해 반환 시 기존 bytes/mtime를 복구하고 신규 파일은 검증된 S 내부 exact 경로만 정리한다. 공용 신규 fixture/Media 파일을 삭제하지 않는다. cache 산출물 변화는 소스 복구와 별개로 전후 hash와 producer를 남긴다. alias 실행·touch·clean·cold target·전체 소스 복사0.

## 최소 실행 순서와 실제 카운트

PM sole-heavy 배정, writer0와 fresh RAM16GiB/VRAM3GiB/C12GiB 확인 후 원 resource-runner conservative2(CPU2/BelowNormal/상속 Job)를 사용한다. runner SHA256 EEA9DEE6E8BCE8950E74A297098044EFF1154CB334D7E6FA2C4C622B4005488D. 각 호출 stdout/stderr/terminal receipt를 독립 보관한다. Cargo cwd S, target T, locked/offline/jobs1와 기존 단일 thread 환경을 유지한다.

1. cargo fmt --check, cargo clippy --all-targets --features native-observer-test-support,native-proposal-window -- -D warnings. 기존 gate와 동일 locked/offline/jobs1 옵션을 사용한다. 실패하면 원 로그를 남기고 exact12 밖 수정·자동 repair·범위 확대 없이 PM에 반환한다.
2. cargo test --lib --features native-observer-test-support,native-proposal-window --no-run --message-format=json의 실제 kinekino_player library profile.test artifact를 선택한다. producer source/feature/target와 fresh 여부, exe hash/크기를 기록한다. 기본 player.exe/과거 alias를 대신 실행하지 않는다. 해당 fixed exe의 --list와 --list --ignored로 정확한 시험 이름·ignore 상태를 고정한다.
3. R2 신규14를 exact 이름으로 각각 직렬 실행한다: product_frame 신규3, retained 신규3, live_entry R2 신규4, mount_proposal 신규4. 각 manifest raw의 #[test]와 predecessor diff로 목록을 확정한다. ignored는 literal --ignored --exact --test-threads=1, 일반 시험은 --exact --test-threads=1. exit0만 세지 않고 원 test result의 1 passed/0 failed/0 ignored를 요구한다. 누락·중복·0 passed는 실제 통과로 세지 않는다.
4. 동일 producer에서 R1 원8을 별도 영향 회귀로 실행한다. 이어 R1 final23-results.json 중 나머지 기존15(원 Card callbacks2/close6/captured guard2/Jobs5)를 exact 이름·ignore 상태로 실행한다. 기존 product_frame/retained/mount_card/Viewing return의 직접 영향 시험도 목록 대조로 중복 제거해 exact 실행한다. 새14·R1회귀8·기존영향은 별도 집계하며 총수는 list/diff 확정 전에 단정하지 않는다. whole-lib/workspace/CLI/다른 ignored/OS window 실행은 이번 최소 범위에 포함하지 않는다.

SDK를 쓰는 각 fixture는 원 setup/body/cleanup/오류 carrier, LocalHost/ExecutionOwner/endpoint-first close/terminal ACK·release를 유지한다. 원 bounded supervisor와 원 deadline을 재사용하며 새 deadline·worker·재시도로 실패를 숨기지 않는다. 실패 시 원 receipt/owner/최초 오류와 cleanup 결과를 함께 보존한다. fixture compile/actual 오류가 생기면 미검증 delta를 만들어 독립 동결·PM 읽기 검토 후 별도 배정한다.

## 반환과 판정 한계

마지막 terminal/EOF, owned process/session/handle/writer/cleanup0을 확인하고 현재 baseline 기준 S·root·lock·library·index·fixed/alias/known2/T 보호 차이를 기록한다. 변경 cache/새 산출물은 producer와 귀속을 분리한다. source 복구 후 cache가 곧 복구 source의 rebuild 증거라는 주장을 하지 않는다. .spec/sots/verification/presentation에 원 로그·실제 시험 결과·미실행 범위와 연결된 정본을 작성하고 PM에게 명시 slot 반환한다. stage/commit/push0.

이 gate는 headless typed 연결·오류·소유권 검증이다. [추가 fixture11 계획](live-mount-r2-fixture-gates.md)의 실제 Some/entry→activation→새 Viewing proof/원 notifier 양성은 별도 미배정이고 합산0이다. R3/R4·사용자 화면 진입·OS compositor·키보드/IME·13화면 시각 인수·DPI/VM은 완료로 표시하지 않는다. 사용자 demo를 종료하지 않는다.

적용 스킬: modoo.today `modootoday-free/design-delivery-plus/frontend-first-journey`의 원 typed port·공용 컴포넌트·미지원 표시 원칙, `modootoday/software-qa/test-fixture-and-mock-build`의 원 owner/setup/body/cleanup/error provenance와 model/actual 구분을 적용한다. 앞서 읽은 본문에 따라 테스트 수·exit0와 실제 통과를 분리하며 새 비 Rust 검증 로직을 도입하지 않는다.
