---
id: plan:kinekino/20261029113615
title: host17 통합·실제 CLI endpoint gate 계획
status: active
created: 2026-10-29
updated: 2026-10-29
---
# host17 통합·실제 CLI endpoint gate 계획

실행 마감: [실제 검증](../../sots/verification/application/manual-skip-host-2026-10-20.md). 최종 source17-r5 C6184933…9048 + 별도 자막 fixture1 7A477285…70E8을 합친 integrated18 D08F697E…00F7, 고유46pass·CLI37·양 strict/fmt0·보호615 차이0. 최초 경고/정합성 수리는 관측 실패로 과장하지 않고 별도 핀을 보존했다. PID79112의 새 conhost 재사용을 정정·분리한 후 현재 owned[] 및 명시 반환을 확인했다. PM root commit43e6d9276b912992d818661dd0d4d25eaa634b5c/푸시0, 다음 capture/shell 소유 중으로 이 문서는 재실행 지시가 아니다.

2026-10-20 준비본이다. 현재 Meari가 heavy/snapshot을 소유하므로 이 문서는 **읽기·계획만**이며 적용·Cargo·CLI 실행을 지시한 것으로 취급하지 않는다. 소스/manifest는 변경하지 않는다. PM이 반환을 확인해 명시 배정한 뒤 아래 단계를 직렬 수행한다.

## 1. 입력과 적용 절차

- core 정책: PM commit `eb626726757d7297e8fe82e7c11b74e8f3173853`, frozen r3 `9C1BE6D389B41E9E23D089E49058F88CC6A383A068AC53C8A301641069EF15FE`. PNG `4fa9396fcf509abf8bf0e6b893b8a128fc8a7103` 및 SDK surface a611을 보존한다. Meari gate에서 새로 인수된 입력이 있으면 중앙이 지정한 현재 입력 핀을 추가 기록하며 예전 producer로 되돌리지 않는다.
- 최종 host17-r3: `runtime/coordination/manual-skip-host-r3/source17.json`, SHA `203D85C372ED61FB2F4A9C90976FD6C543C0FF8DCA100CC48E0F7B233D96E4B3`. 기존 r2 C5E8…3AE6는 보존하며 변경은 tests/manual_skip_cli.rs의 evaluate/override 회귀1건 추가뿐이다. test-only.diff SHA `5C39C2E5AFE7D831DEF4F1121C7FE45C63C4396029F9DAFB1FAA47248B070EE5`.
- shared6 patch: 같은 디렉터리의 `shared-changes.patch`, SHA `349FC80D5377474C61010A7556D515AA1185E5157E96C902EA97C7688726493D`.
- core2 extension patch: `core-extension.patch`, SHA `BCAD36EBED474ED482E7FE9DF2366C639AC24C0953FBA7E22A25BA374804BC9F`. 등록1줄과 응답3 Deserialize만 추가하며 r3 정책을 교체하지 않는다.
- 개발 checkout: `%USERPROFILE%/.codex/worktrees/subtitle-range-export/com.kinekino.player`. 실제 결합 실행 root는 중앙 소유 `%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/public-p3-parent-r1`이다. 개발 checkout 전체를 덮어쓰는 방식은 금지한다.

배정 직전에 current root HEAD·snapshot 입력/등록을 재확인하고 source17 manifest/patch/hash를 대조한다. 17경로 각각 before bytes/hash/size 또는 absent 상태를 새 증거 디렉터리에 보존한다. 신규9는 frozen bytes만 복사하고 기존8은 **shared6/core2 patch로만** 합류한다. 이 두 patch는 eb62672에서 읽기 apply-check가0이었지만, 실제 시점에도 다시 check해야 한다. conflict 또는 예상 밖 drift면 원 자료를 보존하고 중앙과 조율하며 전체 파일 복사로 해결하지 않는다.

기존8의 실제 합류 후 해시가 원 source17 전체 파일 해시와 다르면 다른 인수 변경·줄바꿈 여부를 설명한 새 integrated17 pin을 만든다. 원 핀은 보존하며 동일 해시를 만들기 위해 PNG/SDK/Meari/core 내용을 삭제하지 않는다. 신규9는 원 SHA와 일치해야 한다. 적용17의 mtime을 갱신하고 actual source/patch/input 명세를 고정한 뒤 컴파일한다.

보호 목록은 현 snapshot의 기존 보존 목록에 PNG17·core11·SDK/producer·자막13과 root Cargo/lock/main/제품 식별/기존 helper를 합쳐 만든다. 이번17 의도 변경만 별도 예외로 분리해 중복 없이 실제 보호 수를 기록한다. 이전 core gate의609를 고정 숫자로 재사용하지 않는다. root live/index/stage/commit은 PM만 소유한다. 우선 실행 snapshot만 결합하며 live 반영은 중앙 별도 배정이다.

## 2. 실행 환경과 명령

현재 준비 기준 runner는 `runtime/quality-review/resource-runner-build4-r3/final-bin/resource-runner.exe`, SHA `EEA9DEE6E8BCE8950E74A297098044EFF1154CB334D7E6FA2C4C622B4005488D`다. 직접 `--profile conservative2`, 기존 native cache, BelowNormal/2CPU/descendants, offline/locked/-j1/testthreads1을 사용한다. build4/-j2나 새 cache·target·dependency download는 추가하지 않는다.

아래 경로는 새 gate 시도용이다. 이미 있으면 이전 자료를 덮지 말고 새 시도 경로를 배정한다. 환경변수는 해당 실행 shell마다 전달한다.

```powershell
$taskRoot='%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/public-p3-parent-r1'
$taskRunner='D:/workspace/@myso-kr/com.kinekino.player/runtime/quality-review/resource-runner-build4-r3/final-bin/resource-runner.exe'
$taskEvidence='D:/workspace/@myso-kr/com.kinekino.player/runtime/verification/manual-skip-host-r3-r1'
Set-Location -LiteralPath $taskRoot
New-Item -ItemType Directory -Path $taskEvidence
$env:CARGO_TARGET_DIR='%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/native'
$env:RUST_TEST_THREADS='1'
$env:MANUAL_SKIP_EVIDENCE_DIR="$taskEvidence/host-cli"
$env:SUBTITLE_RANGE_EVIDENCE_DIR="$taskEvidence/subtitle-range"
$env:SDK_CAPABILITY_EVIDENCE_DIR="$taskEvidence/capability"
$env:SOURCE_SNAPSHOT_CLI_EVIDENCE_ROOT="$taskEvidence/png-cli"
New-Item -ItemType Directory -Path $env:MANUAL_SKIP_EVIDENCE_DIR
New-Item -ItemType Directory -Path $env:SUBTITLE_RANGE_EVIDENCE_DIR
```

capability 디렉터리는 기존 harness가 스스로 create_dir하므로 미리 만들지 않는다. 인수된 PNG snapshot_export_api_cli의 SOURCE_SNAPSHOT_CLI_EVIDENCE_ROOT는 절대 경로를 요구하며 harness가 root/create_dir_all과 고유 run/call 디렉터리를 만든다. 위 새 png-cli 경로로 지정해 기존 PNG gate 자료에 섞지 않는다. 잘못된 환경으로 원 자료를 덮지 않는다.

각 단계의 원 handle·stdout/stderr·receipt terminal/exit를 확인한 후에만 다음을 시작한다. 첫 실패는 그 자리에서 보존하고 뒤 명령을 자동 진행하지 않는다. 새 수리에는 exact scope·별도 핀을 남긴다.

```powershell
& $taskRunner --profile conservative2 "$taskEvidence/core-host.stdout" "$taskEvidence/core-host.stderr" "$taskEvidence/core-host.receipt.json" cargo test --manifest-path "$taskRoot/Cargo.toml" -p kinekino-player --lib skip_segments:: --offline --locked -j1 -- --test-threads=1
& $taskRunner --profile conservative2 "$taskEvidence/build-cli.stdout" "$taskEvidence/build-cli.stderr" "$taskEvidence/build-cli.receipt.json" cargo build --manifest-path "$taskRoot/Cargo.toml" -p kinekino-player --bin player-cli --offline --locked -j1
$taskCli="$env:CARGO_TARGET_DIR/debug/player-cli.exe"
Get-FileHash -LiteralPath $taskCli -Algorithm SHA256 | ConvertTo-Json | Set-Content -LiteralPath "$taskEvidence/cli-pre.json" -Encoding utf8
& $taskRunner --profile conservative2 "$taskEvidence/host-cli.stdout" "$taskEvidence/host-cli.stderr" "$taskEvidence/host-cli.receipt.json" cargo test --manifest-path "$taskRoot/Cargo.toml" -p kinekino-player --test manual_skip_cli --offline --locked -j1 -- --test-threads=1 --nocapture
& $taskRunner --profile conservative2 "$taskEvidence/png.stdout" "$taskEvidence/png.stderr" "$taskEvidence/png.receipt.json" cargo test --manifest-path "$taskRoot/Cargo.toml" -p kinekino-player --test snapshot_export_api_cli --offline --locked -j1 -- --test-threads=1 --nocapture
& $taskRunner --profile conservative2 "$taskEvidence/subtitles.stdout" "$taskEvidence/subtitles.stderr" "$taskEvidence/subtitles.receipt.json" cargo test --manifest-path "$taskRoot/Cargo.toml" -p kinekino-player --test subtitle_conversion --test subtitle_range_cli --offline --locked -j1 -- --test-threads=1
& $taskRunner --profile conservative2 "$taskEvidence/parity.stdout" "$taskEvidence/parity.stderr" "$taskEvidence/parity.receipt.json" cargo test --manifest-path "$taskRoot/Cargo.toml" -p kinekino-player --test sdk_capability_parity --offline --locked -j1 -- --test-threads=1
& $taskRunner --profile conservative2 "$taskEvidence/strict-default.stdout" "$taskEvidence/strict-default.stderr" "$taskEvidence/strict-default.receipt.json" cargo clippy --manifest-path "$taskRoot/Cargo.toml" -p kinekino-player --offline --locked -j1 --all-targets -- -D warnings
& $taskRunner --profile conservative2 "$taskEvidence/strict-native.stdout" "$taskEvidence/strict-native.stderr" "$taskEvidence/strict-native.receipt.json" cargo clippy --manifest-path "$taskRoot/Cargo.toml" -p kinekino-player --features native-proposal-window --offline --locked -j1 --all-targets -- -D warnings
& $taskRunner --profile conservative2 "$taskEvidence/fmt.stdout" "$taskEvidence/fmt.stderr" "$taskEvidence/fmt.receipt.json" cargo fmt --manifest-path "$taskRoot/Cargo.toml" --check
Get-FileHash -LiteralPath $taskCli -Algorithm SHA256 | ConvertTo-Json | Set-Content -LiteralPath "$taskEvidence/cli-post.json" -Encoding utf8
```

위 명령은 단계 목록이다. 각 원 receipt를 확인하며 직렬 진행하고, 앞 단계 실패 시 뒤 명령을 실행하지 않는다.

최종 CLI pre/post 및 host-cli 호출 JSON의 cli_sha256이 같아야 한다. Cargo test가 동일 경로 executable을 재작성했다면 내용 해시를 비교하고 실제 각 binary identity를 원 증거와 연결한다. 최종 검증본을 별도 artifact/에 보존한다면 새 복사본 SHA/bytes와 source input을 기록하며 release 배포본이라고 표시하지 않는다.

## 3. 예상 실행 수와 회귀 범위

- lib skip_segments 필터: 기존 core15 + registry5 = 예상20, 0-test 결과 불인수. 이전 코어15와 새20을 누적35개로 더하지 않는다.
- manual_skip_cli:4시험, 정상 완료 시 fresh CLI37호출. 기존 동일 owner11/unknown fixture7/owner close·새 generation4에 evaluate/override15를 추가했다. typed inspect SDK 호출은 추가 CLI로 세지 않는다.
- snapshot_export_api_cli: 기존7. 실제 PNG typed client·schema/context·endpoint·모든 기존 reconstructing builder의 **PNG port 보존**을 회귀한다. skip registry의 모든 builder 이전을 직접 계측하는 시험은 아니다.
- subtitle_conversion: 기존9 active +1 외부 corpus ignored, subtitle_range_cli:4. `-p kinekino-player`를 명시해 workspace의 동명 plugin target을 함께 선택하지 않는다.
- sdk_capability_parity:2. default CLI-equivalent PNG port 주입과 output-review의 valid/malformed fixture 수리는 최신 인수본을 보존한다. 새로운 skip 기본 unavailable/schema·unsupported 응답도 전수 비교되지만 이는 실제 host 주입 긍정 경로를 대신하지 않는다.

위 예상 전체 active 고유 수는46(20+4+7+9+4+2)이며 실제 stdout 수가 정본이다. ignored corpus1 및 이전 다른 gate의 pass를 합산하지 않는다. PNG backend14나 전체 capture/media 회귀는 이번 SDK owner 연결 변경에서 반복하지 않는다. 새 실패·관련 의심이 생기면 중앙과 좁은 추가 범위를 정한다. strict는 양 feature의 모든 target 컴파일/경고를 검사하며 시험을 실제 실행한 수로 세지 않는다.

## 4. 실제 host 주입·재시작 증거의 경계

| 시험 | 실제 gate 실행 시 입증할 내용 | 입증하지 않는 내용 |
| --- | --- | --- |
| registry5 | Rust 소유 registry/core 상태, open 재사용/close/다른 generation, Mutex busy·예산·stale·typed dispatch, counted seek | OS endpoint·실제 native engine·실프로세스 restart·동시성 race |
| fresh_cli_processes_share_only_the_same_live_host_owned_session | 실제 LocalHost IPC에 같은 SDK를 주입하고 여러 실제 CLI 프로세스가 같은 session/revision/pass를 유지, 원 submit1·Unobserved·취소 flag | `player-cli --serve-native` 프로세스를 실제 시작하거나 실제 mpv seek/landing |
| unknown_native_like_host_edits_state_but_never_advertises_or_submits_seek | 실제 endpoint에 **합성 Unknown 포트**를 주입, Auto 편집과 skip Unsupported/submit0·schema 일치 | EpisodePlaybackSessions native adapter나 OS seekability를 실제 관측 |
| explicit_close_and_new_host_generation_invalidate_old_sessions | 한 test process 안에서 실제 LocalHost owner flag/종료와 새 LocalHost, 서로 다른 **고정 fixture generation**·old session 거절 | native host subprocess 재시작, 새 skip OS GUID factory 실행·entropy, in-flight 실행 철회/원 SDK worker join |
| evaluate_and_override_wire_preserve_canonical_state_and_reject_stale_revision | 같은 endpoint/session/binding의 evaluate·manual_seek override·stale expected_revision 거절·원 상태 불변·명시 new pass·cancel 후 canonical inspect와 typed SDK 일치, 합성 submit0 | 실제 수동 seek 실행·native 위치 변화·자동 host tick |

모든 builder의 skip registry 이전은 읽기 근거이며 전부 직접 계측한 것으로 확대하지 않는다. evaluate/override wire 공백은 r3에서 시험을 작성해 보완했지만 실제 실행 전이므로 해결된 실행 근거로 표시하지 않는다(core/service 동작 회귀와 구분). 원 지원 범위를 늘리는 타이머·durable 저장·복구·AI 검출·자동 검출 신뢰도는 없다. 같은 CLI 바이너리의 다른 endpoint나 endpoint 없는 새 SDK가 세션을 이어받는다고 주장하지 않는다.

## 5. 원 증거·handle·소유 프로세스 마감

host-cli JSON37개는 endpoint·Request·Response·exit·CLI SHA를 create_new로 기록한다. helper는 각 실제 CLI의 PID/elapsed/IO 상태를 Debug receipt로 eprintln하므로 `--nocapture` 원 stdout/stderr를 보존한다. JSON 파일만으로 CLI PID·별도 host subprocess가 기록됐다고 주장하지 않는다. LocalHost는 테스트 process 안의 실제 IPC/thread owner이며 별도 host PID를 발명하지 않는다.

각 runner의 원 launcher/child PID·profile·admission·terminal·exit와 unified exec handle을 보존한다. CLI helper가 기록한37개 원 PID도 모아 종료 후 부재를 확인한다. 기존 helper의 finish는 blocking recv/join이고 잠재적인 I/O/worker 오류나 hang의 모든 정리를 입증한 코드는 아니다. 정상 반환 전에는 완료·owned[]·슬롯 반환을 표시하지 않는다. hang이면 원 handle을 그대로 유지하고 원 로그/PID/프로세스 경로를 PM에 알리며 동일 시험을 중복 시작하지 않는다. 중단이 필요하면 소유가 확인된 원 실행만 중앙 절차로 회수하고 실패/실행 미확정으로 남긴다. unrelated 프로세스·endpoint를 종료하지 않는다.

정상 마감 조건은 모든 원 receipt terminal, 원 runner/child/관측 CLI PID 부재, 실제 helper 반환 및 해당 host owner drop·새 admission 거절 결과, source17/integrated 핀·보호 목록 차이0, CLI hash 동일이다. 원 SDK executor thread의 joined 증거를 따로 수집하지 않았으면 join 완료라고 말하지 않는다. 실제 query after owner close는 Unsupported이지만 이미 접수한 seek 취소 증거가 아니다.

`final-source.json`, `final-protected-differences.json`, `final-receipts.json`, `final-owned.json`을 실제 관측으로 남기고 정본·남은 한계를 갱신한 뒤 heavy/snapshot 소유권을 명시 반환한다. root stage/commit은 PM에게 인계하며 user push 요청은 없다. 이 계획을 쓴 현재 시점에는 실행·snapshot 변경0이다.

스킬 적용: 앞서 읽은 modoo.today requirement-to-testable-stories의 수명·의존·경계 분모와 code-change-review-verification의 source/실제 실행/미검증 분리를 적용했다. 계획은 기존 Rust harness/runner 실행과 파일 메타데이터 기록만 지정하며 새 비Rust 검증 정책이나 새 helper를 만들지 않는다.
