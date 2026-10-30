---
id: evidence:kinekino/20261029113543
title: timeout 후계 타입·기본10 실제 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# timeout 후계 타입·기본10 실제 검증

2026-10-24 중앙 단독 heavy 배정으로 frozen timeout source8 `003D22E2EB275D6B089308851A70475530AE3DF28F91C84F6BC2D2C7A6EF6CB7`만 검증했다. source1 `6F66F68D…`, 부모 r3 source8 `7336E34F…`, GATE-PLAN `AC1FD94A…`, expected20 `EF3A50A5…`의 실행이다. 원시 근거는 workspace root `runtime/verification/owned-test-runner-timeout-r1/`에 보존한다. forced-success source2 `1080B107…` 혼합0이다.

입장 root `688a99cbab9a8d371a33d0d3716c34a52f2949b7`, Meari `b532fe0eed06d8526b755b11bc36416070244ef7`, root lock `8F2C1D56C8A2C22D36D9E57A285D727C7EE9C7D2D1160902B66CC6630B042C67`, standalone tool lock `0B2562E8D295AEE5602407228C3A5A21DBD5E478299A685CE94780FADE49226B`를 확인했다. UI D 커밋을 S에 동기화하지 않았다. SDK/제품 소스의 동일 producer라고 주장하지 않는다.

S는 기존 public-p3-parent-r1, T는 기존 native를 사용했다. 원 resource-runner SHA `EEA9DEE6E8BCE8950E74A297098044EFF1154CB334D7E6FA2C4C622B4005488D`, conservative2 CPU2/mask3/BelowNormal/descendant Job을 15개 receipt에서 관측했다. 입장 가용 RAM 28,604,264,448B/GPU 6205MiB/C 여유105,632,460,800B, writer0이며 strict/no-run 직전에도 별도 admission을 기록했다. Cargo는 offline/locked/j1, 시험 thread1이다. 새 target/clean/deps/normal 링크0이다.

## 실제 결과

- `cargo fmt --manifest-path tools/owned-test-runner/Cargo.toml --check` pass.
- `cargo clippy --offline --locked --manifest-path tools/owned-test-runner/Cargo.toml --all-targets -j1 -- -D warnings` pass.
- `cargo test --offline --locked --manifest-path tools/owned-test-runner/Cargo.toml --bin owned-test-runner --no-run --message-format=json -j1` pass. compiler-artifact target/bin/profile.test=true/fresh=false를 보존했다.
- 새 fixed unit SHA256 `221F02C51242BD0EC18F8EBDF9CF9CC4FE97273850B00768B71B0236D3E45EE5`, 1,817,088B, FileID `0x00000000000000000004000000024f2d`. Cargo JSON·artifact 목록·fixed-unit.json이 출처다. 원 FC6900 시험 결과를 승격하지 않았다.
- 새 unit 실제 list20/ignored10/default10이 expected와 일치했다. ignored는 등록 대조만 했다.
- 신규 순수3 `timeout_complete_requires_original_forced_nonzero_and_job_failure`, `timeout_incomplete_preserves_cause_and_stored_partial_vs_closed_owner`, `timeout_never_launched_and_unobserved_facts_are_not_acceptance`부터 기존 영향7까지 각각 exact로 실행해 총10pass다. 각 stdout은1pass/0fail/0ignored이고 actual10.json이 이름과 원 receipt를 연결한다. 15개 runner receipt 모두 terminal=true/exit0이다. 첫 실패나 수리·재시도0이다.

이 결과는 receipt 모델과 검증 코드의 타입·정책 회귀 근거다. 실제 timeout OS의 admission/force/exit/EOF/저장 receipt 관측은0이며 원1s/cleanup300ms 시험은 실행하지 않았다. 새 forced+exit0 OS, late 실제 지연, baseline child, B IPC, GUI, AI/ORT/model/다운로드/worker 빌드0이다. production source7이 변하지 않았으므로 원 normal978는 baseline control로 보호했으며 새 후보 normal로 재명명하지 않았다. 기존 OS2는 기존 r3 근거로 유지한다.

## 입력 복원·보호·producer

입장 전 S application.rs/tests.rs는 이전 r2 before SHA/bytes/mtime와 정확히 일치했다. 부모r3 application baseline1 + timeout tests1만 overlay했다. standalone tool 다른6은 source8과 일치했다. 완료 후 두 파일을 이번 preimage bytes/mtime로 복원했으며 restored2.json의 matches2=true다. 따라서 현재 복원된 S가 새 T unit producer와 같다고 표시하지 않는다.

root tracked1383/library493/index·lock6/S6021/carry133/protected6 모두 SHA/bytes/mtime 불변이며 해당 manifest의 기존 FileID도 불변이다. S 새 파일0, T 기존exe588 중 변경1/신규exe0이다. 변경한 `native/debug/deps/owned_test_runner-810aae3316a0ab46.exe`는 원FC6900→새221F02C5, compiler-artifact의 원 executable과 일치하는 이번 unit producer다. UI/제품 alias 직접 덮어쓰기0이다. baseline graph30 고정 사본 SHA도 모두 일치했으며 이 추가 감사는 새 FileID custody를 주장하지 않는다.

기존 fixed133에 새 unit1을 더한 carry134 SHA256 `C8DF774B74A1532A50431E4D23DC6086213CF23792DA2E355230A99EF16E211E`를 반환한다. historical alias4 FileID 예외 및 cacheunknown2 보호 한계는 유지하며 all_cache_protected=false다. 기록된 launcher/child PID의 종료 후 조회는 모두 부재이고 writer0이며, 실제 SDK 원 owner를 다루는 OS 시험이 아니므로 owner/handle 계측을 새로 증명하지 않는다.

modoo.today `modootoday/software-qa/test-fixture-and-mock-build` 본문 SHA `a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100`의 모델/실제 자원 분리와 최초 오류 보존 원칙을 기존 HANDOFF/시험 설계에서 적용했다. 본문·필수 verdict-table을 읽은 `modootoday/software-qa/code-change-review-verification`의 소스/관측 주장 구분을 결과 판정에도 적용했다. root canonical/index/stage/commit/push 변경0이며 후속 실행은 중앙의 별도 배정이다.
