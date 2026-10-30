---
id: evidence:kinekino/20261029114419
title: r137 실제 관측의 evidence 봉인 및 byte 재검증
status: active
created: 2026-10-29
updated: 2026-10-29
tags: [application]
---
# r137 실제 관측의 evidence 봉인 및 byte 재검증

## 원본과 정본 봉인 구분

[실제 실행 관측](native-gui-endpoint-controls-2026-10-19.md)의 runtime 원본은 `runtime/native-gui-endpoint/r137-native-cli-20261019-a/`다. 기존 실행 trace·fixture·report·기존 seal을 덮어쓰거나 보완하지 않았다. source와 종료된 PID/endpoint도 재사용하지 않았다.

새 정본 복사본은 `.spec/evidence/player/native-gui-endpoint/r137-native-cli-20261019-a/`다. Rust 도구 `tools/native-gui-evidence`가 fresh staging에 복사하고 검증한 뒤 final 경로로 게시했다. **91개 manifest entry + manifest/sidecar 2개**다.

- `run/`: runtime 원본74파일 그대로. fixture/report/CLI identity/initial status, 21 request·response·exit의63파일, independent native snapshot7개다.
- `binaries/`: GUI와 LocalHost가 포함된 player, mpv, CLI candidate, test-harness candidate4개.
- `support-source/`: 봉인 시점의 기존 Rust test/support5파일. 이후 prospective hardening 전 snapshot이며 과거 exact compilation input으로 선언하지 않는다.
- `seal-tool-source/`: 봉인을 만든 독립 Rust tool source/Cargo manifest/lock/README8파일.

각 입력은 read-only share lock을 잡고 SHA256/bytes를 계산해 복사한 뒤 source 재독해와 저장 copy의 hash/bytes까지 비교했다. 복사본만 read-only attribute를 설정했고 원본 runtime/binary/source의 attribute를 바꾸지 않았다. manifest는 그 자체를 entry에 넣지 않아 self-reference를 피했다. read-only attribute·digest는 WORM 저장소나 전자서명이 아니며 외부 digest pin이 신뢰 anchor다.

## 고정 manifest anchor

[manifest.json](../../../evidence/player/native-gui-endpoint/r137-native-cli-20261019-a/manifest.json): **34,644 bytes**.

```text
4ff92be2286c187e943b90b3a9662822c7b207e3e382637a3c7e44c9d815e71f
```

`manifest.sha256` sidecar와 위 **외부 고정 SHA**를 사용한 저장 바이트 재검증이 통과했다. 중앙 checker는 위 digest를 pin하고 manifest91개 entry의 bytes/SHA를 검사할 수 있다. sidecar만 읽는 verify는 자기 일관성 검사이며 외부 진위를 증명하지 않는다.

## 실행파일 연결과 과거 귀속 한계

| 봉인 파일 | bytes | 봉인 시점 SHA256 |
|---|---:|---|
| binaries/server-gui.exe | 71880192 | `2334de3a87cfd4aff77d7d5f70bc5a1c447290a13c3a359c220154ee38562780` |
| binaries/mpv.exe | 121051136 | `f8bf7e8fd8d6696e35441c55091498060e4594f724a132fd6c46579341bc0790` |
| binaries/cli-candidate.exe | 2844672 | `ff933c8d4f6db7160e6039e79619e6983bed8dc7fafb493afd922f9c8d3d591b` |
| binaries/test-harness-candidate.exe | 2035712 | `94ac1060f27770f4d572421cbfb9bc781455569ed502cac22aaa1ecef169edfb` |

manifest의 binary_links는 GUI/server·mpv의 `run/fixture.json` executable path와 CLI의 `run/cli-executable.json.path`를 각각 해당 봉인 파일의 실제 SHA/bytes에 연결한다. GUI/mpv는 당시 process/image path를 검증한 review-r137 경로에 남아 있던 파일이다. CLI는 mutable SDK target의 현재 candidate다. test harness는 당시 tool output의 invocation path에 남아 있던 candidate다.

**당시 실행 전 executable SHA가 기록되지 않았으므로 네 파일 모두 과거 실행 byte identity의 암호학적 증명이 아니다.** 특히 mutable CLI를 지금 해시했다고 과거 실행 binary로 재귀속하지 않는다. 현재 source snapshot도 과거 컴파일 입력으로 귀속하지 않는다. 파일 수정 시각은 참고 metadata이며 역사적 digest를 대체하지 않는다. `historical_cli_execution_digest_proven=false`, 각 binary link `historical_run_digest_recorded=false`가 이 한계를 보존한다. 실제 CLI receipt/native property 관측 PASS를 취소하거나, missing historical digest가 해소됐다고 확대 주장하지 않는다.

## 저장 바이트만 검사하는 Rust 모드

repository root에서 실행한다. 이 도구에는 process launch/GUI/IPC/network/inference 모드가 없다.

```text
cargo fmt --manifest-path tools/native-gui-evidence/Cargo.toml --check
cargo test --locked --offline --manifest-path tools/native-gui-evidence/Cargo.toml --target-dir runtime/parallel-build/sdk-evidence -j 1
cargo clippy --locked --offline --manifest-path tools/native-gui-evidence/Cargo.toml --all-targets --target-dir runtime/parallel-build/sdk-evidence -j 1 -- -D warnings
cargo run --locked --offline --manifest-path tools/native-gui-evidence/Cargo.toml --target-dir runtime/parallel-build/sdk-evidence -j 1 -- verify --manifest-sha256 4ff92be2286c187e943b90b3a9662822c7b207e3e382637a3c7e44c9d815e71f
```

verify는 봉인 디렉터리만 읽는다. 원본 runtime·실행파일·source·PID가 없어도 된다. 모든 entry의 SHA/bytes, inventory 누락·추가·중복/unsafe relative path, fixture/report, 21 request/response correlation·exit0·stderr 없음, accepted/unobserved engine1 및 독립 native 상태·복원값을 검사한다. GUI/CLI/mpv를 실행하지 않는다. 잘못된 외부 SHA(64개0)를 준 negative gate도 **exit1 / external manifest SHA256 mismatch**로 거부됐다.

최초 seal 명령은 다음이었고 이미 final 경로가 존재하므로 반복하면 거부한다. 재봉인·덮어쓰기 명령으로 사용하지 않는다.

```text
cargo run --locked --offline --manifest-path tools/native-gui-evidence/Cargo.toml --target-dir runtime/parallel-build/sdk-evidence -j 1 -- seal
```

## 다음 실제 실행용 test hardening

봉인 후 live test만 prospective하게 강화했다. `tests/native_gui_endpoint/executables.rs`는 GUI/host·CLI·mpv·현재 test harness의 SHA256/bytes를 **첫 native 관측/CLI/control 전에** 기록하고 fresh run의 binaries에 복사한다. source-before/source-after/copy의 digest와 bytes를 비교하고 read-share lock을 전체 test 동안 유지한다. CLI는 `CARGO_BIN_EXE` 원본 대신 **fresh evidence run 안의 복사본만 실행**한다. GUI/mpv/test harness는 인계/실행 중인 원본 image와 동일한 파일을 preflight에 snapshot하는 것이며 GUI/test loader 시작 전 해시라고 과장하지 않는다.

`executables-before.json`은 original/copied/executed path와 SHA/bytes를 기록한다. 복원 시도 후 `executables-after.json`으로 원본·복사본 digest 재검증 결과를 남긴다. 기존 r137 raw/seal의 old source5파일과 이 prospective 변경을 구분한다. **새 GUI 요청·실제 시험 재실행은 하지 않았다.** 다음 fresh owned lease·실행 승인과 end-to-end 적용 검증은 root가 후속 조율한다. 이미 종료된 r137 fixture를 재사용하지 않는다.

## 검사 결과

- 독립 seal tool: fmt check/test2 passed/clippy all-targets-Dwarnings exit0.
- root test: `cargo fmt --check`, `cargo test --locked --offline --target-dir runtime/parallel-build/sdk -j 1 --test native_gui_endpoint`, clippy all-targets-Dwarnings exit0. prospective SHA256 known-vector **1 passed / actual GUI1 ignored**, process·GUI·IPC 호출 없음.
- 현재 업무에서는 기존 full IPC fixture suite·r137 GUI endpoint·AI를 재실행하지 않았다. 앞선 실제121tests/ignored gate 관측은 기존 SOT의 당시 결과로 유지한다.

main/presentation·AI tool·기존 seal·runtime 원본은 수정하지 않았다. root가 별도로 갱신 중인 SDK contract 문서도 이 업무에서 편집하지 않았다.
