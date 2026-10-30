---
id: plan:kinekino/20261029113625
title: "r3 잔여 OS3·late2 — 읽기 계획, 실행 미배정"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# r3 잔여 OS3·late2 — 읽기 계획, 실행 미배정

현재 실제 인수는 순수7·정확 OS2(normal/nested)이며 전체 완료가 아니다. UI gate가 다음 우선이다. 원 source7336/fixed normal97829C7C(915456B)·unitFC6900DB(1740288B)·baseline child8440F120(293376B)·indexCF4DC065(3365B)는 보존한다. 전체 pin/절대경로는 첫 normal 계획074385와 nested 계획8398의 표를 그대로 사용한다. 이번 코드/Cargo/S overlay/OS/B/AI 실행0이다.

## 공통 입력·예상 실패 전달

각 별도 승인 사례의 정상 driver --test/--child/--baseline은 위 고정물로 같고 `--case <아래 정확 이름> --ignored true --total-ms 45000 --cleanup-ms 5000`을 다음 배정 후보로 한다. 작업 root는 `runtime/verification/owned-test-runner-remaining-{tag}-r1`의 새 outer/inner-temp, 실행 단위 TEMP/TMP와 baseline env3은 원 --child path/전체SHA/bytes와 정확히 같게 한다. 아직 폴더/환경을 생성·적용하지 않았다. latest root/library/lock/S/carry/graph30/alias-known6/currentbackup 및 fresh 자원을 다시 입장한다. 전체 --ignored/사례 실패 뒤 자동 다음 실행0이다.

아래는 **unit 안에서 예상 실패를 검사하는 selftest**다. inner application이 실패 상태를 보존하면 unit의 해당 시험은1pass가 될 수 있고 top normal driver도 exit0/passed=true다. 이를 inner job 성공이나 실제 정상 CLI main의 exit1 인수로 바꾸어 표시하지 않는다. outer 정확1pass·원 receipt와 inner 음성 결과/부재를 함께 대조한다. 직접 main exit1 경로는 별도 미실측이다. baseline child는 이5개에서도 guard-only이며 descendant의 추가 실제 호출은 없다.

| tag / unit의 정확 case 접두사 `tests::` | 내부 원 예산·자극 | 인수해야 할 inner 실패/보존 사실 |
|---|---|---|
| zero-selection / zero_selected_tests_is_failure_even_with_exit_zero_and_closed_family |15s/3s, 존재하지 않는 case_that_does_not_exist 선택|원 child exit0/Empty에도 selected_one=false/selection=None·passed=false·원 출력 running0/0pass·판정 오류. zero selection을 성공으로 세지 않음|
| sink-failure / sink_failure_retains_original_incomplete_owner_after_physical_close |15s/3s, fixture_pass+기존 FailingWriter|physical close/원 raw EOF와 FinishIncomplete를 구별. cause 원문 original outer sink failure, receipt_recorded=false/process sink0, 원 owner 보유·passed=false. 새 성공 sink/재시도 없음|
| timeout / timeout_stays_failure_after_original_family_is_forced_closed |**1s/300ms**, fixture_hang |**아래 정합 검토 전 실행 보류.** 원 Job 강제 종료 후 실제 exit/Empty·raw EOF와 job 실패를 구별해야 함|
| late-verdict / delayed_verdict_keeps_stored_pre_cutoff_record_and_returns_failure |15s/3s, fixture_pass+test-only 실제 file sync 뒤 원 end까지 반환 지연|durable 저장 전 first_error=null/positive remaining/verdict records0 vs 반환 Outcome late TimedOut/passed=false/records1. 물리 close/EOF는 별개|
| late-with-receipt-failure / delayed_verdict_preserves_original_receipt_failure_and_late_secondary |15s/3s, 원 receipt 실패+같은 지연 verdict|원 first error/FinishIncomplete/receipt 미기록 보존. 저장 전 JSON에는 원 receipt 오류, 반환 Outcome에는 late secondary가 추가돼도 첫 오류 불변|

원 outer receipt의 unit PID U와 fresh NEXT0으로 각 작업의 `inner-temp/outer-owned-{U}-0/evidence` 및 부모48B fixture snapshot을 결속한다. zero-selection/timeout의 기본 writer에는 원 outer-receipt.jsonl이 있지만 sink-failure/late-with-receipt-failure는 주입 writer라 **그 process receipt 파일이 생성되지 않는 것이 예상**이다. 부재/partial은 그대로 기록하고 outcome.jsonl의 원 owner receipt/cause·process_sink_records0을 원 성공 receipt 파일로 재구성하지 않는다. 모두 원 raw/verdict/snapshot·최종 top stdout/receipt를 정확 주소로 보존하고 PID/temp scan0이다.

지연2의 추가 late error/secondary는 저장 뒤 발생하므로 기존 inner durable JSON에는 없다. 고정 unit의 Rust 단언이 반환 Outcome을 검사한1pass와 저장 전 원 JSON을 구별해야 한다. fixture가 내보내지 않는 '최종 inner error JSON'을 만들어 증거로 삼지 않는다. 15초 기다림과 외부45초는 서로 다른 원 cutoff이고 sync 호출의 hard-preemption이나 whole-process 종료 보장은 아니다. source의 positive remaining 단언이 실패하면 성공 경계/예산을 완화하지 않고 중지한다.

## timeout 읽기 발견 — 실행 전 설계 정합 필요

frozen r3 tests.rs의 timeout 시험은 !passed와 함께 `owns_original()` 및 `Ownership::FinishIncomplete(forced)`만 요구한다. 그러나 원 baseline Supervisor.finish는 operation cutoff에서 원 Job을 force한 뒤 end 전에 child exit+Empty/receipt 저장이 관측되면 **Complete(forced=true)**를 반환한다. 이번 nested C 실제도 그 반환이 가능한 원 API 경로를 입증했다. r3 application은 Complete의 nonzero exit를 원 first error `selected libtest exited nonzero`로 보존해 passed=false로 만들지만 owns_original=false다. 따라서 올바른 실패 전달·정상 강제 회수 경로에서도 현재 timeout 시험의 추가 ownership 단언은 실패할 수 있다. 이는 읽기 발견이며 timeout 실제 실패를 이미 관측했다는 뜻은 아니다.

별도 기존 tests.rs leaf 설계 조율에서 물리적 Complete와 논리 job 실패를 구별하고 Complete/Incomplete 양쪽 원 cause/receipt/forced/exit/EOF를 엄격히 검사해야 한다. 무조건 bool 허용/단언 제거·Supervisor 정책 변경으로 해결하지 않는다. 고정 r3를 느리게 만들어 우연히 Incomplete를 얻거나 새 budget/재시도로 기존 시험을 통과시키지 않는다. private 새 revision/검토·타입/고정 BIN이 필요하면 그 권한부터 다시 배정받는다. 또한 원1s에는 pin/evidence 준비도 포함돼 spawn 전 소진이 가능하다. 그 경우 force/cleanup 시험에 입장하지 못한 사실이며 timeout force 인수로 세지 않고 원 오류·증거를 보존한다. 지금 원 code/BIN/예산 수정0이다.

### PM 정합 확인 뒤의 최소 test-leaf 수리 제안 — 아직 코드 미배정

후속 후보는 **기존 `tools/owned-test-runner/src/tests.rs` exact1**이다. application/production 정책·main/launcher/SDK·기존 budget은 변경하지 않는다. `Outcome`을 먼저 기존 retained에 보유한 뒤 소유 variant를 명시 match하고, 각 분기별 엄격한 사실을 검사한다. 단순 `Complete || Incomplete` 허용이 아니다.

- **Complete 분기:** Done의 원 receipt identity/forced=true/child_exit=Some(nonzero)/family Empty, 원 stdout/stderr EOF·physical_closed, selected_one 없음·passed=false·nonzero 원 first error를 함께 검사한다. 이 variant는 원 owner가 이미 완료됐으므로 owns_original=false가 맞다. forced=false/exit0/EOF 없음/빈 first error/가짜 selection은 거절한다. 이를 Timeout ErrorKind라고 만들어내지 않는다.
- **FinishIncomplete 분기:** 그 원 carrier/owner identity와 원 cause(Deadline/TimedOut인지)를 대조하고, 원 forced·후속 physical close/EOF·passed=false·첫 오류를 보존한다. 예상 밖 query/receipt/termination 오류는 이번 timeout 회수 성공으로 삼지 않는다. `receipt_recorded`는 원 SDK 문서상 '한 sink call의 Ok'일 뿐 partial/final 구분이 아니다. true인 partial 원 파일을 임의 false로 바꾸거나 final ACK로 승격하지 않고, 원 stored receipt와 같은 owner의 후속 close 사실을 분리한다. 미완료·원 first error/secondary/receipt error는 원문 그대로 남기며 새 sink·owner·cleanup retry0이다.
- **Err/LaunchIncomplete/미관측 분기:** 실제 spawn/원 identity·receipt가 없거나 prelaunch pin/prep가1s를 소비했다면 '강제 회수 미입장'으로 실패한다. 목적이 다른 launch failure를 정상 timeout으로 허용하지 않는다. 시험 pass/skip로 바꾸거나 더 긴 예산으로 자동 재실행하지 않는다.

필요 순수 branch 시험은 test-only 관측 adapter/명시 사실 validator에 한정한3개를 제안한다: (1) Complete/nonzero/forced의 음성 job과 잘못된 force/exit/EOF/selection 조합 거절, (2) 원 Deadline cause·identity/부분 receipt와 후속 physical close 구분 및 잘못된 cause/first-error 변경 거절, (3) never-launched·미관측을 어느 성공 분기로도 분류하지 않음. 원 opaque Completion/owner를 private literal·unsafe·가짜 성공 sink로 생성하지 않는다. 실제 adapter는 원 Outcome을 빌려 읽고, 모델 입력의 통과는 OS 검증으로 세지 않는다. 정확 시험 이름/새 수·코드는 별도 승인 때 동결한다.

예산 선택은 다음 두 목적을 분리한다. **A:** 기존1s/300ms를 유지한 최소 기대 정합 수리로 원 prelaunch 소진/실제 force를 그대로 분류하며, 입장 실패도 유효한 미완료 결과로 남긴다. **B:** 원1s 사례를 보존한 별도 'force 뒤 Complete/Incomplete job 실패' OS 사례를 새로 제안한다면 후보3s/1s 등 목적 전용 예산을 별도 승인한다. 기존 nested의 baseline3s/1s에서 강제 회수가 실제 관측됐다는 참고만 있으며, unit의 pin/prep 비용까지 같은 시간에 끝난다는 보장은 없다. 따라서3s를 기존 시험에 덮어쓰거나 관측 없는 성공 여유로 정하지 않는다. 실제 준비·spawn·force·receipt 각 원 cutoff와 첫 실패를 기록해야 하며 현재 어느 budget 변경도 승인되지 않았다.

**별도 application 읽기 경계:** 현재 primary는 Complete의 nonzero만 오류로 만들고 passed는 forced=false를 직접 요구하지 않는다. parent unit exit0+정확1pass/EOF+원 end 전인데 잔존 family를 force해 Complete(forced=true)가 된 조합은 passed=true로 남을 수 있다. 현재 OS2의 A/B는 nonforced라 그 조합 actual0이다. 이는 이번 fixture_hang/nonzero test-leaf 기대 정합 수리와 다른 검토 사항이며, test-only 수리로 application 성공 정책을 고쳤다고 표시하지 않는다. PM 지시대로 application/제품 정책 수정0을 유지한다.

계약 층을 구분한다. **원 SDK가 허용하는 사실**은 `packages/owned-process/src/application/supervisor.rs::Supervisor::finish`의 forced Complete다. 이는 원 물리 종료/receipt 완료이며 application job 성공 정의가 아니다. **현재 고정 도구가 구현한 성공**은 `tools/owned-test-runner/src/application.rs::execute_inner`의 Complete/nonzero primary 분기와 `Outcome::passed`의 조건식으로, forced+exit0을 명시 거절하지 않는다. **실패 요구 후보**는 outer watchdog이 잔존 family를 종료해야 했다면 '원 시험 전체가 예산 안에 끝남'으로 인수하지 않는 strict gate다. 과거 B gate의 'timeout 뒤 물리 종료는 시험 성공 아님' 취지와 대응하지만, Request에는 expected_forced/허용 정책이 없으므로 이 요구를 현재 확정 application 계약으로 소급하거나 이미 고쳤다고 말하지 않는다. PM이 외부 gate 성공 계약을 먼저 확정해야 한다.

nested C forcedtrue는 `tests.rs::fixture_nested_family`가 내부에서 명시한 종료 자극이며, 그 자극을 끝낸 A/B는 nonforced였다. 이는 outer 일반 normal driver의 forced 성공 허용 근거도, 기존 OS2 인수의 반증도 아니다.

실제 반례 fixture의 **별도 source-only 제안**은 기존 tests.rs 한 leaf의 전용 ignored 자극이다. 원 application Job 아래에서 고정 child8440의 `parent-exit`를 breakaway 없이 실행하고 부모 C의 원 종료와 descendant PID 기록을 확인하되, 그 stdout은 별도 고정 파일로 보관해 unit의 정확1pass 출력과 섞지 않는다. 새 private Job을 만들어 D를 먼저 종료하거나 새 cleanup owner를 두지 않고, fixture unit 자체는 정상0/1pass로 끝내 D를 기존 application Job에 남긴다. 그러면 원 SDK가 parent exit0/family Active→원 cutoff force→Complete(forced/Empty)를 관측하는지, 도구 passed가 어떻게 나오는지 재현할 수 있다. D PID를 재오픈/kill하지 않는다.

이 재현에는 **새 fixture가 포함된 별도 unit revision·타입/등록·고정 BIN·중앙 예산/slot**이 필요하다. frozen unit17에 없는 case를 호출하거나 기존 nested C 결과만 대체해 증명할 수 없다. 권한 후보는 재현 단계에서 기존 tests.rs exact1, 성공 계약 수정까지 필요하다고 결정된 경우에만 application.rs+tests.rs exact2를 다시 제안하는 것이다. 코드/Cargo/실행/정책 수정은 현재0이다. 모델/source 경로 설명을 실제 재현으로 세지 않으며, 계약이 실패를 요구한다면 별도 red regression/최소 수리 인수로 진행해야 한다.

첫 오류/launch/query/receipt/주소/EOF 불일치·미종료에서 다음 사례를 멈춘다. 같은 원 normal driver/session/handle·Incomplete/LaunchFailure를 유지하고 poll/cleanup도 원 cutoff로 한다. 나중의 physical close는 원 job 오류/receipt 미기록을 성공으로 바꾸지 않는다. 실패한 sink 재시도/새 owner/PID 재오픈/일괄 kill0이다. 각 승인 사례 뒤 원 보호 감사·SOT·명시 반환이 필요하다. 기본 제안 순서는 zero-selection→sink-failure, timeout 정합 후 별도, late2도 별도다. modoo fixture skill의 기대 음성 결과/실제 소유·원 오류·미입증 구분 원칙을 적용했다.
