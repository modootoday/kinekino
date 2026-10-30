---
id: plan:kinekino/20261029113511
title: 동일 endpoint CLI harness owned-process 최초 이관 계획
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 동일 endpoint CLI harness owned-process 최초 이관 계획

최신 상태: 정확8 r4의 독립감사·중앙 gate를 완료하고 자원을 반환했다. [실제 검증 정본](../../sots/verification/runtime/same-endpoint-owned-process-2026-10-21.md)을 따른다. 아래 최초 제안·배정 전 관측은 이력으로 보존한다. legacy13/실제3category·전체 담당 goal은 완료가 아니다.

상태: active 제안, 2026-10-21. PM 배정은 읽기/계획뿐이며 새 코드·Cargo·shared·브라우저/실매체 실행0이다. [소비 배선 정본 계획](owned-process-consumer-wiring.md)의 다음 제한된 단계다. snapshot helper4의778c483 인수 동작/원 pin은 보존하고, d0d185f 동일 endpoint6의 의미를 승계한다. 전체 jobs helper 이관 완료를 이 단계로 표시하지 않는다.

## Outcome — 첫 인수 범위

`actual_three_categories_to_owned_playback_on_one_endpoint`에서 원 playback CLI host 하나와 순차 fresh CLI 요청을 owned-process의 원 handle/Job/절대 deadline으로 관측한다. ready/request/stop 및 실패 closeout에서 원 owner를 반환하고, 원 ready/route/request/hash/capture/stop·LocalHost/worker receipt 의미는 유지한다. 실제3category 실행은 별도 중앙 media slot 전까지 계속 ignored다. 첫 실행 gate는 no-media 실제 Job host와 Rust child fixture로 배선만 검증한다.

## Evidence — 직접 읽은 현재 경로

현재 Fixture::call_root는 blocking stdin.write_all+wait_with_output, Host::{start,start_provider}는 read_line, 예상 밖 ready의 child.wait, stop/kill_owned/Drop은 wait를 사용한다. NativeOwner::start는 read_line, finish는 새 observation deadline 뒤 read_to_string, Drop은10초 poll 뒤 kill/wait/read_to_string으로 원래 범위 밖 대기가 가능하다. parent exit만으로 상속 stdout의 EOF나 child family 종료를 입증하지 못한다.

provider::terminal_until은 deadline 검사 전에 default EndpointClient.get_job를 호출하므로 남은 시간보다 transport call이 길 수 있다. playback_observation의 progress/observe도 각 호출에 새30/5초 clock을 만들고 sample Vec를 계속 저장한다. same_endpoint는 category/submit마다 새 budget을 만들지만 전체 host lifetime anchor는 없다. 이 읽기는 실제 지연 측정이 아니다.

Fixture의 직접 CLI/Host 소비는 현재13개 파일에 걸친다: tests/jobs.rs, jobs/{cancel,catalog_replay,episode_capture,native_episode,provider,playback_observation,range_export,resolve_cli,same_endpoint,series}.rs, native_episode/{options,transition}.rs. 기존 API를 일괄 Result로 바꾸면 이13+fixture/playback_owner+새 공통 owner=최소16개가 된다. 첫 단계에서 이 전부를 기계적으로 바꾸는 대신 새 fallible 경로를 same_endpoint에만 연결하고 나머지는 명시 미이관으로 남긴다.

## Authority — 최초 exact8 후보

| 정확 경로 | 최소 책임 |
| --- | --- |
| `tests/support/owned_process_evidence.rs` | 원 절대 Deadline을 받는 additive `run_process_until` 경계. 기존 snapshot run_process/오류/성공 동작 유지, 두 경로가 같은 producer 실행/증거 구현을 공유 |
| 신규 `tests/jobs/owned_process.rs` | host의 Producer Supervisor/RecordCursor와 원 owner-bearing 실패 조립, EOF/ready/stop 기록. Job/프로세스 poll state machine·제품 ACK 정책은 구현하지 않음 |
| `tests/jobs.rs` | 새 경로 모듈/검증 등록만. 기존 lifecycle 비즈니스 단언/기존 ignored 경계 유지 |
| `tests/jobs/fixture.rs` | same-endpoint용 fallible finite request·canonical/legacy label export·명시 deadline 전달. 기존 Fixture::{new,named,call,call_catalog}/Host 경로는 미이관으로 유지 |
| `tests/jobs/playback_owner.rs` | 신규 BoundedNativeOwner의 원 supervised host 수명과 fallible start/stop. ready/terminal semantic validator를 기존 NativeOwner와 공유하여 복사하지 않음. 기존 NativeOwner는 다른 legacy 소비자용 미이관 상태 |
| `tests/jobs/provider.rs` | 원 endpoint를 남은 transport budget으로 조회하는 fallible terminal-until 경로. 기존 job 상태/SDK를 재사용하며 job cancel/retry 구현0 |
| `tests/jobs/playback_observation.rs` | same-endpoint용 fallible progress/capture와 원 deadline/cap. 기존 readiness/PNG/source/hash/receipt 관측 검증을 공유. legacy 소비자는 별도 유지 |
| `tests/jobs/same_endpoint.rs` | 새 bounded owner/요청/관측 경로만 소비, 전체 lifetime/ready/request/stop cap과 오류+panic+원 owner 명시 마감. categories/job/engine/filter/37요청 의미 변경0 |

shared adapter1을 포함하는 이유는 현재 run_process가 Duration만 받아 내부에서 새 Deadline을 만들기 때문이다. 원 remaining을 Duration으로 계산해 넘기는 wrapper만으로는 계산/호출 사이 지연만큼 원 절대 끝을 늘리지 않았다고 입증할 수 없다. additive API가 원 Deadline.constrained_by를 직접 사용해야 한다. 기존4 파일 중 이 shared1만 수정 후보이며 before/원 source pin을 보존하고 기존 snapshot18을 영향 회귀로 다시 인수한다. 단순 복사·독립 supervisor/오류 state machine은 만들지 않는다. producer package/runner/Cargo/lock 편집은 이8에 없다.

## Signatures/ownership — 구현 전 후보

- shared `run_process_until(..., original: Deadline, request_cap: Duration, ...) -> Result<ProcessOutput,Box<CliFailure>>`: 기존 finite stdin/Supervisor/원 raw/failure를 재사용하고 새 전체 예산을 열지 않음. 원 run_process는 종전20/45·cleanup2 동작을 유지하는 호환 진입.
- `BoundedNativeOwner::try_start(..., original: Deadline, ready_cap: Duration) -> Result<Self,ScenarioFailure>`: 원 Supervisor/RecordCursor를 소유. endpoint는 ready JSON·원 hash/identity·child 생존/family 관측을 확인한 뒤만 공개.
- `Fixture::try_call_until(..., endpoint:&str, original:Deadline, request_cap:Duration, expected_exit:i32) -> Result<Value,Box<CliFailure>>`: 첫 범위는 원 endpoint만. 로컬 --job-root/--catalog-root replay와 기존 call_catalog 전체는 후속이다.
- `BoundedNativeOwner::try_stop_chain(self,...,stop_cap:Duration) -> Result<OwnedHostReceipt,ScenarioFailure>`: EOF 요청/원 family finish와 기존 product stop 검증을 분리. 원 NativeOwner stop_chain의 completed_jobs/LocalHost joined/native/provider/pending-transition 의미를 유지.
- ScenarioFailure는 원 request 실패와 host 실패를 **두 역할의 고정 소유 필드**로 보유한다. 전역/숨은 owner registry는 없다. NotSpawned/Live(original Supervisor)/ObservedTerminal을 혼동하지 않고 primary protocol/API/assertion 오류와 secondary cleanup/receipt 오류를 함께 보존한다.

현재 same_endpoint의 catch_unwind→host.stop_chain 정리 순서를 활용한다. closure가 typed Err를 반환하면 현재 request owner를 같은 deadline으로 먼저 closeout하고, 이어 원 main host에 EOF/stop을 요청한다. assertion panic도 원 payload와 실패 증거를 유지한 채 이 마감 경로를 거친다. 둘 다 Incomplete일 수 있으며 최종 Err가 두 원 owner를 모두 보존해야 한다. 성공 tuple로 바꾸거나 resume_unwind/Drop만으로 미완료 owner를 잃고 정상 종료라 표시하지 않는다. Fixture 임시 디렉터리는 실패·미완료 owner가 남으면 삭제하지 않는다.

## Deadline — 하나의 원 lifetime과 단계 cap

host spawn 전에 명시 lifetime/cleanup reserve로 원 Deadline을 생성한다. ready/request/job observation/progress/stop은 이 값과 단계 cap의 min만 사용한다. 기존 SAME_ENDPOINT_JOB_EXECUTION_MS는 제품 job 실행 입력이며 테스트 lifetime으로 바꾸지 않는다. 원 Request/context도 조용히 clamp하지 않는다. 실제 gate의 lifetime/ready/request/stop/cleanup 수치는 중앙이 고정한 입력으로 남기고 부족한 budget을 자동 연장하지 않는다.

ready는 PreparedEvidence(None)의 non-inheritable 부모 writer를 보유하고 Supervisor::wait_stdout_record/RecordCursor로 newline까지 bounded 관측한다. EOF/partial/malformed/과대 ready/원 hash mismatch·query failure는 원 owner-bearing 오류다. ready line만 맞고 child가 이미 exit했거나 family가 Unknown이면 live host 성공으로 표시하지 않는다.

request는 같은 Scenario Deadline과 request cap을 shared adapter에 직접 전달한다. parent와 client는 별개의 own family이며 client timeout이 이미 admission된 host job의 취소/미실행을 뜻하지 않는다. main host owner는 그대로 마감한다. EOF stop은 새10초를 더하지 않고 원 lifetime과 stop cap을 constrain한다. 강제 own Job 종료는 product graceful/ACK receipt와 별도다.

EndpointClient poll은 매 호출 남은 operation budget과 기존 transport cap의 min으로 만든다.10ms 미만이면 호출하지 않고 실패하며 반올림/새 timeout으로 연장하지 않는다. 기존 &dyn ApplicationClient의 임의 동기 호출·FS/OS를 강제로 선점한다고 주장하지 않는다. 이 첫 endpoint 경로만 transport cap을 정확히 전달한다. progress 관측 sample 수/serialized evidence bytes도 명시 cap을 두되 원 instance/artifact/time/영상크기·오디오/PNG 판정을 복제하지 않는다.

## Raw/receipt compatibility

원 logs/{label}.request.json·.stdout·.stderr·.exit, native-ready.stdout/native-stop.stdout, native-host.stderr, native-failure-stop.stdout/native-failure-cleanup.json 및 route/categories/host/outcome/PNG 파일명을 유지한다. canonical spool은 owner별 하위 디렉터리에 두고 원 name과 origin/offset/bytes/hash/complete를 연결한다. RecordCursor의 ready offset 뒤 terminal 원 bytes를 export하며 stdout을 read_to_string EOF로 무제한 기다리지 않는다.

stdout ready/terminal record는 기존 MAX_RESPONSE_BYTES를 각각 상한 후보로, host canonical stream은 두 record와 separator를 포함한 별도 명시 상한으로 산정한다. stderr prefix/전체 spool 크기도 구별하고 overshoot/동기 FS의 hard wall-clock 한계를 숨기지 않는다. raw 상한의 정확 수치는 코드 동결/실제 직렬화 시험에서 인수한다.

현재 stderr legacy 파일은 실제 child writer였으므로 canonical-only로 바꾼 뒤 종료 후 복사만으로 실시간 tail 호환이라고 하지 않는다. 첫안은 ready/마감의 legacy 보존 사본과 canonical 실시간 경로·시점/complete를 명시한다. live legacy writer 자체가 필수라는 소비 계약이 확인되면 producer evidence basename API 같은 추가 경로가 필요하므로 이8을 조용히 확대하지 않고 PM에 보고한다. hardlink/reparse/동시 이중 writer로 우회하지 않는다.

원 child exit/family Empty/forced/creation identity/예산·원/secondary 오류를 새 owned-family metadata에 기록한다. supervisor 파일 transport에는 가짜 stdin worker/stream joined/직접 child kill 성공을 쓰지 않는다. 반면 제품 stop JSON의 local_host_shutdown·native IPC/stdout/stderr worker·provider browser worker·completed jobs는 **실제 제품 원 필드**이므로 삭제하거나 supervisor family Empty로 대신하지 않는다.

## Phases/verification — 실제 검증 계획

1. PM이 정확8과 legacy 미이관/공유1 영향 범위를 배정하면 before/원 snapshot helper pin을 보존하고 독립 staging에서 구현·Rust 회귀를 동결한다. 승인 전 소스 편집0이다.
2. Rust seam 회귀는 ready 없음/부분 newline/잘못된 JSON/hash/child exit+live descendant, ready/request/stop의 같은 deadline, expected nonzero/무출력·과대 spool/원 stderr, semantic 오류+familyUnknown·receipt/raw쓰기 실패, request+host 두 Incomplete·panic 마감·강제 종료의 무ACK를 포함한다. 원 producer Supervisor/ProcessFamily를 재사용하고 state machine을 복제하지 않는다.
3. 중앙 슬롯에서는 기존 owned-process-test-child의 고정 Rust fixture artifact와 --serve-jobs/no-media actual CLI로 launch→ready→finite request→EOF/stop→child+family/원 metadata를 검증한다. fixture binary가 없으면 producer test-support executable을 명시 build하는 별도 실행 입력을 먼저 배정하며 root dev dependency feature를 몰래 켜지 않는다. 임의 PID reopen/사용자 앱 kill0이다.
4. shared adapter 영향으로 기존 snapshot_export_api_cli18과 새 bounded harness 필터를 실행하고, jobs의 기존 no-media 실제 lifecycle/관련 변환·범위 회귀를 필요한 만큼 인수한다. 실제 선택 이름/수는 소스 작성 후 고정한다. same_endpoint/native/provider 실제 ignored는 compile/no-run/metadata만, 브라우저·매체 실행0이다. default/native all-target strict·fmt와 artifact/원 exec receipt/PID identity/보호 마감 후 슬롯을 반환한다.
5. 이후에만 중앙이 기존 pin의 브라우저/mpv/미디어·3category/37요청 실제 시나리오를 별도 배정한다. helper 배선 통과를 실제 광고차단/재생/모델 지원으로 확대하지 않는다.

## Rollback/Unverified

원 helper4·legacy jobs 경로/actual ignored source를 before에 보존하고 신규 bounded 연결만 분리한다. 원 실패/evidence/미완료 owner는 삭제하지 않는다. first8의 성공을 아직 unbounded인 Host/NativeOwner/legacy13개 소비 전체 완료로 표시하지 않는다. 실제 elapsed/전체 API typecheck/회귀 수·raw cap/실미디어/브라우저 실행은 지금0이다. modoo.today 기존 plan/plan-authoring 및 code-change-review-verification의 책임 경계·실측/미검증·원 오류 보존/복구 구분을 적용했다. 새 비 Rust 개발·검증 로직은 없다.
