---
id: sot:kinekino/cli-host-wire
title: CLI host 종료 관측 wire 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# CLI host 종료 관측 wire 계약

2026-10-21 중앙 채택 shape. 구현은 `runtime/development/local-host-cli-wiring-r3`의 source-only 후보이며 아직 Cargo/실제 subprocess 인수 전이다. 공용 application API Response나 LocalHost transport wire를 확장하는 계약이 아니라 **명시 host CLI stdout의 additive wrapper**다. ready 기존 shape는 유지한다.

최신 후보는 별도 r4(source10 917F0927CC962CD700C8713F9152ED0DED703CCCCAD7D45A79EB98C5B79B090A)다. r3의 stderr eprintln! panic P2를 fallible Write 주입 경계로 수리했으며 DTO shape·원 Response·성공 wire는 불변이다. 원 r3를 보존했고 r4도 작성13/실행0 상태다.

검증 후속: 최종 r6 source11 247F238EA0C0129EA8BF0268C1BAE9E6E3BCD417BA7163512F0555DBA6A67339에서 실제26/양strictfmt 및 r5 별도 실제 격리 parent1을 인수했다. 위 r4는 당시 이력이다. 실제 제품 job EOF 원 JSON과 timeout/panic의 첫 오류/상태 분리는 [검증 정본](../../verification/application/local-host-cli-wiring-2026-10-21.md)을 따른다. DTO shape는 그대로이며 native/provider/playback 전 외부환경의 실제 실행을 주장하지 않는다.

## 정확 DTO

`src/bin/player-cli/host_shutdown/carrier.rs::LocalHostShutdownView`가 원 LocalHostShutdownStatus를 직접 변환한다. 각 상태를 원 enum에서 매핑하며 합성 transport_joined bool은 없다. 실제 join을 발급하는 capability가 아니라 진단 view다.

```json
{
  "type":"object",
  "additionalProperties":false,
  "required":["pipe","sdk","dispatch","dispatch_poisoned"],
  "properties":{
    "pipe":{"enum":["pending","joined","panicked"]},
    "sdk":{"enum":["pending","joined","panicked"]},
    "dispatch":{"enum":["idle","accepted","running","completed",null]},
    "dispatch_poisoned":{"type":"boolean"}
  }
}
```

dispatch=None이면 dispatch=null/dispatch_poisoned=true이며 그 외는 false다. completed는 SDK 함수 반환(잡힌 panic 반환 포함)이며 저장·재생 등 업무 성공이 아니다. poison은 미확정이며 idle/completed로 바꾸지 않는다. 양 joined는 LocalHost pipe/SDK의 정상 join만 뜻하며 하위 host 정리와 별도다.

## 성공·일반 오류·비정상 경계

- 성공 `native_host_stopped`, `playback_host_stopped`, `provider_host_stopped`, `job_host_stopped`에 `local_host_shutdown` 객체를 추가한다. 기존 event/receipt/해당 host_lifetime_ended 필드를 유지한다. 원 양 정상 join 및 해당 dependency 기존 shutdown 성공, 첫 I/O 오류 없음인 경우만 출력한다.
- LocalHost가 생성돼 양 join을 관측한 뒤 I/O/dependency/최종 출력 오류가 있으면 기존 Response::failure를 `src/bin/player-cli/runner.rs::HostFailure`의 Serialize flatten으로 감싸고 root에 local_host_shutdown만 추가한다. 원 Response의 error/details와 필드를 덮어쓰지 않는다. LocalHost 생성 전 setup 실패는 local_host_shutdown을 생략한다. 추가 dependency 오류는 별도 stderr 진단이며 첫 오류를 대체하지 않는다.
- timeout/panic은 일반 stopped가 아니라 아래 별도 진단이다. owner와 dependency를 runner stack에 유지한 상태에서 nonzero abnormal process exit한다. stdout 실패 시 같은 진단을 stderr에 best-effort 출력하며 성공으로 fallback하지 않는다.

```json
{
  "event":"host_shutdown_incomplete",
  "host":"native-host",
  "reason":"ObservationBudgetExpired",
  "first_error":null,
  "local_host_shutdown":{
    "pipe":"joined","sdk":"pending","dispatch":"running","dispatch_poisoned":false
  },
  "budget_expired":true,
  "cleanup_complete":false,
  "sdk_cancellation_confirmed":false
}
```

host는 native-host/playback-host/provider-host/job-host다. reason은 ObservationBudgetExpired 또는 WorkerPanicked다. first_error는 원 ApiError 또는 null이며 join 진단과 분리한다. 위 상태는 형식 예시이며 실측 결과가 아니다. stopped 이벤트나 host_lifetime_ended=true는 출력하지 않는다. 원 오류가 있으면 그 exit mapping을 사용하고, 없으면 기존 EngineFailed를 사용한다.

## 소비·검증 경계

SDK same-endpoint harness는 위 필드만 소비하며 snapshot status나 합성 bool로 원 join을 추측하지 않는다. 원 response와 wrapper의 전필드 동등성, setup의 필드 생략, DTO의 각 join/dispatch poison 매핑은 Rust 회귀 source로 작성한다. 실제 EOF/실패 subprocess·30초 정책·exit code·소유 cleanup 인수는 별도 중앙 슬롯 전까지 미검증이다. 첫 오류 serialization/output이 실패해도 원 오류 객체와 code를 유지하고 정상 종료를 반환하지 않는다.
