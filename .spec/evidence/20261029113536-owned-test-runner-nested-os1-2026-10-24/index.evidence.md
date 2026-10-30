---
id: evidence:kinekino/20261029113536
title: r3 nested OS1 — 실제1 인수
status: active
created: 2026-10-29
updated: 2026-10-29
---
# r3 nested OS1 — 실제1 인수

PM 계획8398D375의 exact1 배정으로 root `c351b5d6e87120d6455944e10ff74bc134bd3171`/Meari b532/root lock8F2C에서 실행했다. 고정 r3 normal97829C7C/915456B·unitFC6900DB/1740288B·baseline child8440F120/293376B·외부 indexCF4DC065/3365B를 그대로 사용했다. 전체 SHA/경로/FileID는 `runtime/verification/owned-test-runner-nested-os1-r1/input4-pins.json`에 있다. Cargo/S overlay/소스 수리/다른 OS/지연/B/evidence15/AI0이다.

기존 runner EEA9/conservative2/CPU2/BelowNormal/descendant Job와 매실행 fresh RAM16GiB/GPU3GiB/C12GiB/writer0, carry122/graph30/alias-known6의 사전 pin을 확인했다. 새 작업의 실행 단위 TEMP/TMP·child env3만 명시했고 전역 환경을 바꾸지 않았다. 외부45s/cleanup5s, A application15s/3s, baseline spawn3s/1s 원 cutoff는 그대로였다. 자동 재시도/새 budget/이름별 kill/새 owner0이다.

## 실제 원 사실

`tests::nested_family_is_observed_by_same_outer_owner`는 exact1pass/0fail/0ignored(원 stdout 2.48s), 입력 자극 `tests::fixture_nested_family`도1pass(2.06s)다. 후자를 독립 인수로 더하지 않는다. 원 runner는 normal driver PID34072의 terminal exit0/13572ms를 기록했고 최종 stdout은 passed=true/ownership=complete/physical_closed=true/first_error=null/secondary=[]였다. 이 시간은 이번 관측이지 성능 상한이 아니다.

| 원 process/Job 층 | PID / creation_filetime | 원 receipt |
|---|---|---|
| A: OS1 unit |37944 / 134372621758646712|exit0, Empty, forced=false|
| B: 같은 unit의 fixture_nested_family |88812 / 134372621869986815|exit0, Empty, forced=false|
| C: 고정 baseline child의 descendant mode |90488 / 134372621870266135|**실제 exit124**, Empty, forced=true|

C의 실제 executable은 고정 `baseline-owned-process-child-r1/bin/owned-process-test-child.exe`와 대응했다. 원 C stdout은17B의 완전한 `descendant:63780\n`이며, 고정 부모의 spawn 성공 기록을 확보했다. 기존 첫 normal OS1의 guard-only와 달리 이번에는 baseline parent 실제 호출1과 descendant spawn 기록1이 있다. 이를 OS 인수2개로 합산하지 않는다.

forced=true/exit124는 원 C Job의 의도된 종료 자극이다. 상위 A/B의 정상0·nonforced와 구별하고 원 child 실패를0으로 덮지 않았다. baseline Supervisor.finish는 operation2s 후 원 Job 종료 요청, end3s 전 child exit·Empty·원 receipt 저장이 확인되면 forced Complete를 반환할 수 있다. 이번 fixture가 그 경로를 통과했다. D PID63780의 독립 creation timestamp/개별 Job readback/exit code는 fixture가 내보내지 않아 **미관측**이다. 고정 코드의 breakaway 없는 상속·원 C assignment/readback 경로와 원 family accounting을 그 독립 증거로 승격하지 않는다.

A receipt→정확 `inner-temp/outer-owned-37944-0/evidence`의 B receipt→정확 `inner-temp/outer-nested-88812/receipt.jsonl`의 C receipt를 결속했다. 후자에는 별도 outcome.jsonl이 없다. A/B 원 Rust outcome의 stdout/stderr EOF=true4와 C의 원 exit+Empty 뒤 정확 파일 EOF 사후 관측2(stdout17B/stderr0B)를 구분했다. C fixture에 없는 SDK EOF flag를 발명하지 않았다. 사후 두 root FileID도 기록했으나 생성 시 독립 custody 증명은 아니다. 다른 temp를 scan하거나 PID를 새로 열지 않았다.

outer·middle의 원 raw/receipt/verdict8, middle snapshot48B(compiled_baseline_proof=false), C 원 receipt/raw3 등 총12개를 SHA/bytes/mtime/FileID와 함께 보존했다. stdin_eof_requested는 요청 사실이며 읽기 ACK가 아니다. durable verdict는 저장 전 관측, 최종 normal stdout은 저장 완료 후 결과다. 외부 index 파일 pin은 공급 bytes 검증이지 자체 컴파일 인증이 아니다.

## 보호·반환·잔여

root1377/library493/index·manifest·lock4/S5856/T exe581/fixed122/protected6/input4/graph30의 검사 범위 bytes/mtime와 protected/input/graph FileID 차0, S/T 신규0이다. currentknown2 원 bytes 사본 일치와 carry122 원30CB4B86을 유지했다. 과거 cache4709 producer 미귀속2/all_cache_protected=false도 유지한다. 원 tool session67858 terminal0, 최종 관련 process/session/handle/pending0이며 추가 실행 없이 슬롯을 반환했다. canonical/stage/commit/push0이다.

나머지 OS3(timeout/빈 선택/receipt failure), 지연2, D 독립 identity·Job 관측, B IPC/evidence custody15와 제품 미디어·AI 기능·품질은 미완료다. 원 미종료 owner 보존/같은 handle 후속 관측/동기 I/O 비선점·whole-process hard deadline 미보장 경계도 그대로다. 이번 정상 nested 경로로 그 실패 경로를 통과했다고 주장하지 않는다. 다음 UI gate 우선순위를 따른다.

원자료는 `runtime/verification/owned-test-runner-nested-os1-r1/`의 admission·execution-input·01-nested stdout/stderr/receipt·정확 두 inner root·observed-evidence12·C-raw-post-terminal-observation·actual-observation·protection-summary·final-process-observation·slot-return이다. modoo.today test-fixture-and-mock-build 본문 SHA `a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100`(추가 필수 참조 없음)의 원 자극/실제 호출·Job/EOF·원 실패·미관측 분리를 적용했다. 전체 Media 목표는 미완료다.
