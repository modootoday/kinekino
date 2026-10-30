---
id: plan:kinekino/20261029113512
title: owned-process 소비 배선과 기존 증거 호환
status: active
created: 2026-10-29
updated: 2026-10-29
---
# owned-process 소비 배선과 기존 증거 호환

다음 단계의 [동일 endpoint 최초 이관 exact8 후보](same-endpoint-owned-process-migration.md)를 연결한다. snapshot helper4는778c483에서 인수됐으며 이 후속은 read-only 계획이다. 전체 jobs API의13개 직접 소비자 일괄 변경 대신 명시 same-endpoint fallible 경로부터 한정하고 legacy 미이관을 추적한다. 공유 adapter의 원 absolute Deadline additive API 필요성과 기존 snapshot18 영향 회귀를 함께 검토한다.

생산자 동결19 pin `5D327CB95A39D289B76DCCC50AF7EB27B80F719199EB8A9DEFC26B74B17BBF14`의 독립 감사 중 작성한 계획이다. [정본](../../sots/runtime/owned-process-supervisor.md)과 [원 source 인계](../../../runtime/coordination/owned-process-r1/handoff.md)를 따른다. source19/원 제안/registration diff는 변경하지 않는다. 작성 회귀14(pure7·명시 Windows7)는 실행0이다. 이 문서로 helper 변경이나 실제 gate를 승인한 것은 아니다.

## 최초 snapshot 소비 이관 · 2026-10-21 PM 배정 계획

### Outcome — 목표와 현재 상태

생산자는 PM 통보 root56d9e87에서 Windows 포함 고유16·strict/fmt와 제품 호환까지 인수됐다. 위 source19/실행0은 준비 당시 이력이다. 이번 배정은 **계획만**이며 최초 snapshot 소비 helper와 직접 호출부를 owner-retaining Result로 전환할 정확 범위를 정한다. 생산자·runner·jobs helper·SDK/UI 소스·Cargo/shared/root/index는 바꾸지 않았다.

modoo.today 계획 스킬 `modootoday/spec-authoring/plan` 및 `modootoday/spec-authoring/plan-authoring`의 본문을 재사용해 기존 계획을 갱신하고 관측/제안/미검증을 분리했다. 이번 검색에 나온 sot/sot-authoring은 새 확정 불변식 작성 범위가 아니므로 채택하지 않았다. 아래 서명/시험은 구현 전 제안이다.

### Authority — 첫 소비 정확4 후보

| 경로 | 한 책임과 최소 변경 |
| --- | --- |
| `tests/support/snapshot_cli_evidence.rs` | 기존 request·파일 전후/hash/host 관측과 JSON 의미 검증 유지. OwnedCli/pipe worker/별도 kill·reap·join timeout을 생산자 owner 소비로 대체. `cli/cli_observed/cli_recorded`는 성공 값을 그대로 두고 Result를 반환하도록 전환 |
| 신규 `tests/support/owned_process_evidence.rs` | 이 helper의 path module. Windows spawn/PreparedEvidence/Supervisor 조립, raw-prefix/receipt 호환 산출, typed 실패 owner와 명시 closeout만 담당. deadline/Job/poll 상태 전이는 생산자 API에 위임. 내부 Rust 회귀를 같은 파일에 둘 수 있음 |
| `tests/snapshot_export_api_cli.rs` | 실제 CLI helper 호출9곳(반복 루프의 호출 문장 포함)을 Result 소비로 전환. 관련 fresh CLI 시험4의 최종 경계에서 실패 closeout을 명시. 픽셀/schema/context/endpoint 주입·호출횟수 정책은 유지 |
| `tests/frame_snapshot_pipeline.rs` | 기존 ignored 매체 시험의 cli_recorded 호출5곳을 Result 소비로 전환. 원 PNG/미디어 identity·crop·부분 산출물 보존 단언과 ignore 이유 유지. 이번 gate에서 실미디어 실행0 |

`tests/jobs/{fixture,playback_owner}.rs`, `tests/jobs.rs`와 그 직접 소비자는 후속 단계다. 첫 단계에 합류하지 않는다. root Cargo/lock에는 이미 등록된 owned-process dev dependency를 소비하며 test-support feature를 추가하지 않는다. 생산자 원본 수정 필요가 발견되면 이 정확4를 확장하지 않고 보고한다.

### Evidence — 직접 읽은 원 동작과 API

2026-10-21 helper와 직접 소비자2·producer-gate 및 owned-process의 supervisor/evidence_files/windows launch/deadline API를 읽었다. 기존 helper는 `(i32,Value)` 또는 `CliObservation`을 직접 반환하며 실패는 assert/panic이다. 원 CLI는 `--input -`와 선택 endpoint, clear_path면 PATH만 빈 값이다. 원 child wait20초(미디어 옵션45초) 뒤 kill/reap2초·I/O2초·join1초를 새로 시작하므로 합계가 wall_timeout을 넘을 수 있다. OwnedCli Drop도 최대2초 대기하며 원 자식만 kill한다. 이번 읽기는 실제 실행 측정이 아니다.

생산자 API는 `SpawnSpec`/`PreparedEvidence::create`/`spawn`의 `LaunchFailure { owner: Option<Box<WindowsFamily>>, ... }`, `Supervisor<WindowsFamily>`, `FinishOutcome::{Complete,Incomplete}`다. launch 뒤 실패와 Incomplete는 원 owner를 반환한다. Complete는 child exit와 own family Empty·receipt 기록을 확인한 결과이며 protocol JSON 성공을 뜻하지 않는다. `RawFile::open_read`는 독립 cursor/file identity를 확인한다. synchronous FS/OS 호출의 엄격한 wall-clock 종료는 보장하지 않는다.

### Unverified — 실패/증거 호환의 명시 경계

새 성공형은 `Result<(i32,Value), CliFailure>`와 `Result<CliObservation,CliFailure>` 후보로 기존 성공 필드를 유지한다. 이름을 새 try_*로 바꿔 이중 infallible wrapper를 남기는 대신 기존 세 호출 모두 fallible로 바꾸는 최소안이다. 오류는 NotSpawned/Live(owner)/ObservedTerminal(receipt)를 구분하고 원 semantic/I/O 오류·secondary cleanup/receipt 오류·evidence 경로를 함께 보존한다. raw/JSON 검증 실패 시 family가 아직 미완료면 원 Supervisor를 유지하며, 이미 Complete면 원 receipt만 유지한다. LaunchFailure의 suspended/unassigned 상태도 재spawn하지 않고 같은 owner로 마감한다.

호출부는 typed 오류를 받은 즉시 같은 원 deadline의 `closeout`을 명시적으로 소비한다. 성공 cleanup도 원 실패를 지우지 않고 시험 Err로 반환한다. 여전히 Incomplete면 최종 시험 오류에도 원 owner를 넣어 반환하며 Drop의 best effort를 종료 ACK로 표시하지 않는다. 테스트 harness의 최종 오류 drop은 마지막 안전장치일 뿐 정상 마감이 아니다. 전역 owner registry·숨은 background owner·panic 직전 owner 유실을 허용하는 wrapper는 추가하지 않는다. endpoint의 주입 LocalHost owner도 CLI 실패 분기에서 명시 마감할 수 있게 호출부 scope를 검토하며 SDK 정책은 바꾸지 않는다.

| 원 이름/필드 | 보존/정정 |
| --- | --- |
| `request.json`/request_sha256/request_timeout_ms | 원 직렬화 bytes 그대로 보존하고 finite stdin.raw와 bytes/hash 대응. Request/context를 조용히 clamp·rewrite하지 않음 |
| `stdout.bin`/`stderr.bin` | 기존 stdout MAX_RESPONSE_BYTES+1, stderr8192+1까지의 probe prefix와 원 hash/limit_exceeded 의미 유지. canonical spool은 별도 `owned/`의 stdout.raw/stderr.raw; origin/offset0/preserved bytes/complete/hash를 기록 |
| `before.json`/`after.json` | source/output/endpoint/host_port_injected/host_calls 유지. 종료 실패에도 가능한 after 관측과 원 관측 오류를 보존 |
| source.before/after `.png` 또는 `.bin`, output.before/after.png | 원 선택/관측 길이·보존 bytes/hash/complete 유지. crop·PNG CRC/픽셀·source 불변/부분파일 판단을 supervisor로 이동하지 않음 |
| `process.json` | child_pid/fixture_timeout_ms/child_path_cleared/request_timeout_ms/elapsed_ms/endpoint/request_sha256와 원 stream 관측 유지. schema_version2·원 creation identity/family/forced/cause/secondary/receipt를 additive 기록 |
| stdin_result/io_workers/kill_result | file transport는 not_applicable_file_transport/빈 worker 목록, 직접 child kill은 not_applicable. Job terminate는 owned_job_termination 별도 필드. joined/Ok(Ok(()))·자식kill 성공을 만들어 넣지 않음 |
| terminal_observed/descendants_observed | child exit와 family Empty를 별도 기록. descendants 관측은 own Job의 현재 관측이며 모든 PID 종료 이력으로 확대하지 않음. Drop 진단은 정상 receipt와 구별 |

생산자 raw_limit은 두 스트림 공통 한도다. 첫 소비에서는 MAX_RESPONSE_BYTES+1을 공통 spool 상한으로 쓰되 종료 시 stderr prefix8193/출력8192 초과를 기존 실패로 처리하는 안이다. 기존 read worker가 cap에서 읽기를 멈추던 조기 backpressure까지 동등하다고 주장하지 않는다. coarse spool budget은 poll 사이 overshoot/동기 FS 대기까지 hard bound하지 않는다. 서로 다른 per-stream 조기 중단이 필수라면 별도 생산자 보강을 배정받으며 원 package를 고치지 않는다.

예산은 spawn 전 하나의 절대 Deadline으로 정한다. 최소안은 wall_timeout을 전체 예산으로 해석하고 그 안의 cleanup reserve2초를 명시한다(20초→operation18초/total20초,45초→43초/45초). total<=2초는 시작 전 typed 오류로 거절한다. finish/실패 closeout이 새2초를 더하지 않는다. 기존보다 실제 operation cutoff가 짧아지는 변경을 receipt와 시험에 명시한다. API timeout_ms는 원 wire 의미를 유지하며 테스트 관측 budget으로 요청을 고치지 않는다. clear_path=true는 기존 환경을 읽어 PATH만 빈 값인 explicit environment를 전달한다. 지원되지 않는 비ASCII 환경 변수 이름은 조용히 삭제하지 않고 spawn 전 typed Unsupported로 보고한다.

### Phases — 소스 배정 후 검증 순서

1. PM이 정확4와 위 wall/cleanup·coarse stream 경계를 인수하면 before/source pin을 고정해 source-only로 작성한다. 완료 조건은 외부 package/runner/Cargo 변경0과 직접 호출14곳의 owner-retaining 오류 소비, 원 basename/필드 대응표 대조다.
2. Rust helper 회귀는 pre-spawn invalid budget/env 실패, semantic JSON 실패 뒤 원 terminal receipt, prefix 한도/원 bytes/hash, timeout/Incomplete의 owner 보존과 원·secondary 분리, cleanup deadline 비리셋을 확인한다. 생산자 state machine을 복제한 fake supervisor를 만들지 않는다. 실제 원 process seam이 필요하면 중앙 gate에서 기존 CLI만 사용하고 임의 shell/매체/브라우저0이다.
3. 중앙 배정 후 `snapshot_export_api_cli`의 기존 no-media 시험과 helper 집중 회귀를 실행한다. 고정 CLI artifact/hash와 실제 raw/process/before/after를 보존한다. frame_snapshot_pipeline은 compile/ignored metadata만 확인하며 실제 매체 pass0으로 기록한다. default/native strict와 fmt를 수행한다. 0selected/ignored는 pass로 세지 않는다.
4. 최종 exact4/before·원 package와 비중첩 경로/receipt/원 identity를 인계하고 슬롯을 반환한다. 실제 실패는 원 pin/log를 남긴 새 수정 pin에서만 수리한다. jobs 소비는 이후 별도 배정이다.

### Rollback — 원본과 실패 owner

이 계획만으로 소스는 바뀌지 않았다. 구현 실패 시 before 사본과 원 실행 로그를 보존하고 미인수 helper를 root에 적용하지 않는다. evidence는 덮어쓰거나 삭제하지 않는다. 살아 있는 실패 owner는 같은 원 closeout 경로로만 다루며 PID 검색/reopen kill·가짜 ACK·추가 deadline을 사용하지 않는다. 남은 Incomplete는 미완료로 인계하고 최종 Drop을 성공 근거로 세지 않는다.

## 정확한 소비 파일과 책임

최초 소비 변경은 `tests/support/snapshot_cli_evidence.rs`, 이후 lease 반환을 받은 `tests/jobs/fixture.rs`와 `tests/jobs/playback_owner.rs`다. 공통 raw/persistence adapter가 필요하면 `tests/support/owned_process_evidence.rs` 한 파일에 두고 세 helper가 경로 모듈로 재사용한다. 이 파일은 기존 evidence/hash/receipt 변환만 담당하며 deadline/Job/poll 상태 전이를 복제하지 않는다. `tests/jobs.rs`의 module 연결과 호출부 변경도 별도 lease 승인 뒤에만 수행한다.

snapshot의 직접 소비자는 `tests/snapshot_export_api_cli.rs`와 `tests/frame_snapshot_pipeline.rs`다. 성공 반환형 `(i32,Value)` 및 `CliObservation { code, response, evidence }`, `CliOptions { wall_timeout, clear_path }`, `HostObservation { injected, calls }`, `hash/evidence_call/save_json` 계약은 유지한다. `before.json/after.json`의 source/output/host_calls 관측은 원 구현에서 유지한다.

jobs의 `Fixture::{new,named,call,call_catalog,endpoint}`, `Host::{start,start_provider,stop,kill_owned}`, `NativeOwner::{start,process_id,stop,stop_chain}`의 성공 의미를 유지한다. ready/stopped event 이름, endpoint/hash, native/provider/worker joined·completed_jobs 판단은 각 기존 소비자에 남는다. supervisor에 그 JSON 정책을 넣지 않는다. `tests/jobs/native_episode.rs`, `tests/jobs/provider.rs` 및 `tests/jobs.rs` 등 start/stop 직접 호출부는 fallible 경로 전환이 필요한 범위만 명시적으로 승인받는다.

## 실패 반환의 실제 호환 경계

현재 Self/Value/()/tuple을 직접 반환하고 unwrap/assert하는 helper는 오류에 원 owner를 반환할 수 없다. 성공 값의 필드는 유지하되 `try_cli_recorded/try_cli_observed`, `Fixture::try_call*`, `Host::try_start/try_stop`, `NativeOwner::try_start/try_stop_chain` 같은 Result 경계를 먼저 배선하는 방안을 사용한다. 구체 명칭은 소비 구현 배정에서 고정한다. 기존 infallible wrapper를 실패 수용 경로로 계속 쓰며 owner-retaining이라고 주장하지 않는다.

오류는 `NotSpawned`, `Live(Supervisor<WindowsFamily>)`, `ObservedTerminal(ProcessReceipt)`를 구분한다. ready JSON parse/hash/endpoint 오류에는 아직 살아 있는 원 Supervisor를 보존한다. 원 semantic 오류를 먼저 보존한 뒤 같은 절대 budget 안에서 graceful/forced cleanup 결과를 별도로 붙인다. family 종료가 이미 관측된 뒤 terminal JSON 검증이 실패하면 그 관측과 원 semantic 오류를 반환하며 live owner를 새로 만들지 않는다. sink 실패도 원 결과와 secondary receipt 오류를 분리한다.

수용 시험의 호출부는 typed 오류를 받은 뒤 명시 cleanup/마감 경로를 소비한다. panic·Drop은 정상 종료 ACK가 아니다. 전역 owner registry나 새로운 background runtime으로 infallible signature를 억지로 유지하지 않는다. 마지막 미완료 owner를 반환할 수 없는 legacy 호출부는 미이관으로 추적하며 gate 완료에 포함하지 않는다. 불필요해진 pipe worker/OwnedCli/Host Drop wait 구현은 대체 동작 인수 후 제거한다.

## raw 이름과 stdin 대응

| 기존 증거 | 새 원 spool/호환 산출물 |
|---|---|
| snapshot request.json | 원 직렬화 bytes 보존. finite stdin.raw 준비 뒤 두 파일의 bytes/hash 대응 기록 |
| snapshot stdout.bin / stderr.bin | 기존 byte 상한의 호환 prefix. 신규 stdout.raw/stderr.raw는 별도 소유 call 하위 디렉토리의 원 spool로 보존 |
| snapshot process.json | 기존 관측 필드는 의미를 바꾸지 않고 유지. version2 owned_family/creation/forced/원·secondary 오류를 additive로 기록 |
| jobs logs/{label}.request.json / .stdout / .stderr / .exit | 기존 label 유지. 명시 owner 하위 canonical spool과 export offset/bytes/complete/hash 매핑 |
| logs/{name}-ready.stdout / {name}-stop.stdout | 같은 canonical stdout의 ready line 및 그 이후 terminal offset을 각각 보존 |
| logs/native-ready.stdout / native-stop.stdout | ready/terminal을 나눈 원 offset 유지. native-failure-stop.stdout 및 native-failure-cleanup.json도 원 오류와 cleanup 관측을 분리 |
| logs/{name}-host.stderr / native-host.stderr | canonical stderr 원문과 legacy 이름 간 복사 출처/시점/완전성을 기록 |

기존 stdout.bin 등은 원래 종료 후 산출물이므로 bounded prefix export로 유지할 수 있다. host stderr 이름을 실행 중 외부 도구가 직접 읽는 계약까지 있다면 종료 후 alias만으로 호환됐다고 하지 않는다. 현재 생산자는 PreparedEvidence::create의 고정 raw basename만 제공한다. 실행 중 legacy 경로가 반드시 원 writer여야 하는 요구가 확인되면 audit 후 별도 r2의 evidence preparer 이름 인자만 제안하며, hardlink/reparse 우회나 동시 이중 writer를 추가하지 않는다.

worker가 없어졌는데 `io_workers=joined`나 `stdin_result=Ok(Ok(()))`를 만들어 쓰지 않는다. legacy key는 not_applicable_file_transport 또는 빈 worker 목록으로 사실을 남기고 schema version을 명시한다. 기존 kill_result는 직접 child kill 의미이므로 Job terminate 성공으로 채우지 않는다. 새 owned_job_termination 필드에 별도로 기록한다. terminal_observed는 child exit 관측이고 family Empty와 구분한다. descendants_observed를 모든 PID의 종료 역사로 확대하지 않는다.

현재 snapshot stdout prefix는 MAX_RESPONSE_BYTES, stderr prefix는8192다. 생산자 r1의 raw_limit는 양쪽에 공통인 한도다. 이것만으로 서로 다른 두 기존 cap을 동일하게 이관했다고 주장하지 않는다. consumer prefix/오류 검증은 원 한도를 유지하며 per-stream 조기 중단까지 필요하면 audit 후 OutputLimits 또는 evidence cap의 최소 보강을 검토한다. source19를 계획 작성 중 수정하지 않는다.

## budget·환경·host EOF

one-shot CLI는 원 request를 finite file stdin으로 전달해 blocking write worker를 없앤다. host는 write하지 않는 control pipe의 부모 writer만 보유하고 stop에서 EOF를 요청한다. user endpoint나 임의 PID로 다른 host를 찾아 종료하지 않는다. 동일 host owner가 ready에서 얻은 endpoint만 다음 호출에 사용한다.

scenario/host의 절대 lifetime budget은 spawn 전 생성해 owner에 고정한다. request마다 원 남은 operation budget, 명시 request.context.timeout_ms, CliOptions.wall_timeout의 최솟값으로 cap을 정한다. cleanup reserve를 제외한 값만 요청에 넘긴다. stop_chain의 observation_budget과 기존 stop10초는 원 deadline을 단축하는 cap이며 새10초를 덧붙이지 않는다. 기존에 lifetime 한도가 없는 jobs 생성자는 새 명시 budget 입력을 받아야 하며 임의의 성공 기본값으로 숨기지 않는다.

RequestContext.timeout_ms는 Option<u64>, TransportOptions.timeout은 Duration이고 transport validator는10ms~30초를 허용한다. 남은 시간이 최소10ms 미만이면 시작하지 않으며 budget을 늘리려고 반올림하지 않는다. 원 Request를 조용히 변경하지 않는다. timeout clamp가 wire에 반영되어야 하면 원 request bytes와 effective request bytes/hash를 둘 다 보존하고 기존 API validator를 사용한다. PNG/SDK 정책이나 timeout validation을 helper에 복제하지 않는다.

clear_path=true의 의미도 유지한다. Windows SpawnSpec.environment=None은 상속이므로 PATH만 제거하려면 부모 환경을 명시적으로 읽어 기존 값은 보존하고 PATH=""를 적용한 환경을 전달해야 한다. explicit environment r1은 ASCII 변수명만 지원하므로 비ASCII 변수가 있으면 이를 조용히 누락하지 않고 typed Unsupported로 남기거나 별도 작은 보강을 감사한다. shell/PATH executable 탐색은 하지 않는다.

## 최소 Cargo와 검증 순서

root Cargo 등록은 이미 보존한 미적용 patch의 workspace members/default-members + dev-dependency owned-process path 하나다. test-support feature를 root dev-dependency에 무조건 켜지 않는다. 자체 Rust child/family gate만 `-p owned-process --features test-support`로 명시 실행한다. 기존 fixed windows0.62.2 외 버전·runtime 추가0, root lock은 중앙 offline Cargo 생성 결과로만 인수한다. 최초 소비 gate에는 runner source/lock 변경이 필요하지 않다.

producer 독립 감사/수리 pin → package pure/명시 Windows gate → snapshot public 결과·raw/hash/환경/timeout 호환 → jobs lease 반환 후 fake/no-media helper → default/native compile·strict/fmt → 별도 실제 동일 endpoint3category gate 순서다. 0selected/ignored를 실행 통과로 세지 않는다. source19, 기존 helper 원문, root Cargo/lock 및 실제 실행 binary/원 receipt를 각 단계별로 고정한다. 현재 이 계획에서는 어떤 배선도 적용하지 않았다.
