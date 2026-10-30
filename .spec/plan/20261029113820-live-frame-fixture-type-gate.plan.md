---
id: plan:kinekino/20261029113820
title: Live frame fixture12 — 다음 타입 gate 제안
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Live frame fixture12 — 다음 타입 gate 제안

2026-10-24. 실행 배정 전 계획이다. source12 **5245F605D60D7DA5DDFD859707A9DCDFB6E135B022AE93F2CB3940B51F586A69**, before12 E2FAF3ACD161E7D76913C258C109840EB70F549ADA042C69906A32E81CDAE79A, authored13 D7A5250E22EC1CEAC6CC074F160EF0243E3106EDE070E60F6600736A87F3D073을 runtime/development/ui-live-frame-fixture-r1에서 소비한다. 현재 작성13/leaf12 fmt·check 통과/타입·목록·actual0이며 raw는 동결 유지한다.

## 입장과 보호

- PM의 유일 heavy 명시 배정·writer0와 fresh RAM16GiB/VRAM3GiB/C12GiB를 확인한다. root는 최신 채택값을 다시 고정한다(현재 dd27fd6). Mearib532/lock8F2C 및 기존10/new2 before12의 canonical·S hash/부재를 대조한다. b942 원 preimage를 임의 새 root로 바꾸지 않는다. 다른 변경/불일치 시 자료를 보존해 재조율한다.
- 기존 S=…/CodexBuildCache/project-9dc296c4/public-p3-parent-r1, T=…/project-9dc296c4/native를 사용한다. 중앙 예고 S5622+Media 신규9, T exe581, fixed106은 **예상값**이며 실제 inventory로 확정한다. Media 잔존 신규파일·SDK/R2 채택 입력을 삭제하거나 전체 복사하지 않는다. S exact12의 기존 bytes/UTC mtime/부재 및 전체 baseline을 보존하고 그12만 overlay한다.
- root/library/index/lock, fixed106, alias4 hash·mtime·FileID, T exe를 보호한다. known2는 현재 실제 bytes를 **별도 backup**하고 hash/mtime/FileID·동일 파일 관계도 기록한다. 과거2D2094 및 R2 입장 대비10B560 변경의 producer 미귀속/보호 실패는 별도 이력이며 해결로 표시하지 않는다. cache producer 변화는 원 기록으로 인계하고 임의 복원·mtime 재작성·clean/touch/cold target0.

## T: 타입·실제 목록

모든 호출은 원 resource-runner conservative2(SHA EEA9DEE6…)/CPU2/BelowNormal/descendant Job, cwd S/target T, offline/locked/jobs1 및 RAYON/RUST_TEST/OMP/MKL thread1이다. stdout/stderr/terminal receipt를 각각 남긴다.

```text
cargo fmt --check
cargo clippy -p kinekino-player --locked --offline -j 1 --all-targets --features native-observer-test-support,native-proposal-window -- -D warnings
cargo test -p kinekino-player --locked --offline -j 1 --lib --features native-observer-test-support,native-proposal-window --no-run --message-format=json
```

CargoJSON의 kinekino_player/lib/src/lib.rs/profile.test producer만 별도 고정·hash한다. alias를 대신 실행하지 않는다. 그 BIN을 원 runner로 `--list`, `--list --ignored` 실행해 authored13의 정확한 이름과 expected ignored10/nonignored3을 대조한다. macro child8도 실제 목록 전에는 작성 수다. 첫 실패는 원 로그/owner를 보존하고 자동 수리·범위 확대 없이 별도 exact 수리를 제안한다.

## P/H/W: 실행 범위는 별도 배정

- **P 일반3:** NoMedia static source opt-in/foreign anchor2, notifier delegate 모델1. 각각 BIN의 정확 이름과 `--exact --test-threads=1`로 실행한다.
- **H headless SDK 모델2:** live_entry 및 mount_proposal의 proof-error carrier. 원 SDK/LocalHost/Rig/cleanup을 쓰며 literal `--ignored --exact --test-threads=1`이다. 실제 commit+주입 Err 모델이지 실제 Meari Busy/Some 인수가 아니다.
- **W Native child8:** Card/Highlight/Cue/Document 양성4, 실제 arm borrow/typed notifier fault2, window-backed proof-error 모델2를 각각 분리한다. authored13의 exact/child label을 원 NoMedia parent_run에 전달하는 시험 BIN 호출만 한다. 원 WindowsFamily/Supervisor20s+cleanup2s·EOF/familyEmpty/forcedfalse/inner schema를 유지한다. 새 launcher/worker/shutdown loop/timeout 연장0. OS 창·테스트 표면/자원은 이 타입 계획으로 자동 배정되지 않는다. DPI/VM·가상 모니터 보류도 자동 해제하지 않는다.

각 실행의 실제 stdout `1 passed/0 failed/0 ignored`와 원 receipt를 확인한다. exit0·일반 호출로 skip된 ignored·모델·실제 양성을 합산하지 않는다. 전체 library/workspace/CLI/그 외 ignored 반복은 제안하지 않는다. 변경한 원 NoMedia 기본/Jobs 및 직접 mount/retained 회귀는 실제 list와 독립 읽기의 영향 지적을 바탕으로 exact 최소 목록을 별도 배정한다.

입력은 원 source callback enqueue→controller commit→current projection/실제 frame→projected value와 editable raw buffer 비교까지다. OS 키/IME/bitmap 인수가 아니다. entry 관측 뒤 입력 epoch가 바뀌므로 old proof 재인증0. return은 typed host 요청→새 Viewing proof/Some이며 **실제 source navigation·R3 기본 도달성0**. notifier 계수는 원 inspect continuation을 제외한 arm 호출/시도/Ok/오류를 구분하고 별도 observation retry wake도 분리한다.

## 반환

실행 성공/실패와 무관하게 원 endpoint/SDK/Native owner의 cleanup/미완료·EOF·session/handle/process/writer를 구분한다. S exact12만 원 bytes/mtime/부재로 복원하고 신규 store/Media 파일은 귀속별 보존한다. root/index/library/fixed/alias/known2/T 차이를 기록해 예외도 공개하며 복원 후 cache를 rebuild 증거로 쓰지 않는다. 원 실패·고정 BIN·manifest와 검증 정본을 연결해 PM에 명시 반환한다. canonical/stage/commit/push0, 사용자 demo 종료0.

이미 읽은 modoo.today test-fixture-and-mock-build의 원 owner/setup/body/cleanup/error·source/model/actual 구분과 frontend-first-journey의 typed 공통 경계·도달성 구분을 적용한다. 이 문서는 실행 권한이나 타입·실제 통과 근거가 아니다.
