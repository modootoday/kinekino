---
id: evidence:kinekino/20261029113529
title: 원 forced-success 실제 OS·직접 CLI 차등
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 forced-success 실제 OS·직접 CLI 차등

2026-10-24 PM 최상단 heavy lease로 원 GATE-PLAN983403FB의 **OS selftest red/green 각1, 직접 CLI red/green 각1 총4회**를 실행했다. 모두 예상과 일치했다. red의 성공 오판을 직접 CLI로 실제 재현했고 green은 동일 물리 종료에서 실패를 반환했다. 이 결과는 이 고정 감독 도구의 범위이며 제품/AI 기능 전체 완료나 C2 실행 성공이 아니다.

근거는 `runtime/verification/owned-test-runner-forced-success-os-r1/`의 원 raw/receipt/input/resource, execution-summary4, evidence-files다. 고정 source는 red0F7A/green310535, producer는 이전 타입 gate의 red unit5BE0A123/normal0F8B9F15, green unit8BA73689/normal02DD2851이다. 전체 SHA·bytes·FileID는 input6-pins와 이전 fixed2/Cargo JSON에 연결한다. baseline child8440F120/293376B 및 execution-baseline CF4DC065/3365B를 fresh SHA/bytes/mtime/FI로 확인했다. index가 자기인증 compilation proof인 것은 아니며 원 source23/Cargo/lock/graph 기록을 외부 출처로 유지했다.

rootabb63c8/Mearif85/lock8F, 기존 S6081/T589/carry142 DB258A10와 과거 예외를 보존했다. Cargo/overlay/build/새target/clean/download0이다. 각 호출마다 새 case/evidence/inner-temp와 OWNED_TEST_BASELINE_CHILD/SHA/BYTES를 사용했고 original runnerEEA9의 CPU2/mask3/BelowNormal/descendant Job, fresh RAM16GiB/GPU3072MiB/C12GiB/writer0을 확인했다. 실제 resource 값은 각 resource.json이다. 재시도·추가 kill·예산 확대·Abort0이다.

## 실제 4회

| 호출 | runner exit / elapsed | 관측 결과 |
| --- | --- | --- |
| 01-red-os |1 /23573ms| 의도한 회귀 단언 실패. A exit101/Empty/nonforced; B는 아래 동일 조건을 충족한 뒤 first_error=None라 tests.rs:255:38의 Option::unwrap(None)에서 정확 panic |
| 02-green-os |0 /23571ms| A exact1pass/exit0/Empty/nonforced. B의 같은 물리 조건에서 원 TimedOut kind/정확 메시지·passed=false 단언을 모두 통과 |
| 03-red-direct |0 /12497ms| 최종 stdout **passed=true**, first_error=null. 강제 종료가 필요한 family를 성공으로 오판하는 실제 반례 |
| 04-green-direct |1 /12100ms| 최종 stdout **passed=false**, first_error=`selected test family required forced termination`. 같은 물리 Complete를 시험 성공으로 수락하지 않음 |

4개의 runner receipt는 terminal=true이며 exit1 두 개는 각각 의도된 red assertion 실패와 green CLI 실패다. 이를 4개의 exit0/pass로 합산하지 않는다. OS selftest2와 direct CLI2의 분모도 분리한다. red OS의 panic만으로 passed=true를 추론하지 않고 03의 최종 stdout/exit0를 별도 실제 근거로 사용했다.

4회 모두 선택된 fixture B는 **prior libtest1pass/exit0, SDK receipt Complete/forced=true/family Empty, selected_one=true, stdout_eof=true/stderr_eof=true, process_sink_records1**였다. direct의 최종 verdict_sink_records1, OS B의 durable snapshot records0은 최종 verdict 저장 전 snapshot이므로 구분했다. 01 B remaining2667ms,02 B2680ms,03 최종2549ms,04 최종2911ms로 원15s/cleanup3s의 cleanup reserve 안에서 종료 관측을 마쳤다. 최종 전체 end 만료로 분류한 것은 아니다.

OS outer는 원45s/cleanup5s, regression 내부 B 요청은 원15s/cleanup3s이며 direct도 원15s/cleanup3s다. fresh Supervisor::new→finish에서 Complete까지 갈 수 있는 forced 원인은 operation cutoff 분기뿐이라는 이전 소스 판정과 대응한다. 일반 forced receipt를 timeout으로 해석하는 SDK 계약을 추가하지 않았다. Incomplete가 없었으며 원 SDK Complete 의미는 물리 종료 그대로다. green ErrorKind::TimedOut은 02의 같은 fixed test 내 메모리 단언으로 확인했고 direct JSON은 kind 필드 없이 정확 first_error 문자열을 보존한다.

## 원 family·identity·raw

| 호출 | SDK 원 A(회귀 unit) | SDK 원 B(fixture unit) | 별도 baseline.stdout marker |
| --- | --- | --- | --- |
|01|PID61264 /creation134372799201684490|PID89924 /creation134372799312680366|descendant:85748|
|02|PID34128 /creation134372799901170046|PID66344 /creation134372800012765229|descendant:61768|
|03|없음(직접 B)|PID61076 /creation134372800387813026|descendant:29600|
|04|없음(직접 B)|PID67800 /creation134372800800854923|descendant:91428|

01/02 내부 B evidence는 각 `inner-temp/outer-owned-<A PID>-0/evidence`, fixture marker는 각 `inner-temp/outer-forced-zero-<B PID>/baseline.stdout`다. direct는 outer의 원 receipt/raw와 같은 marker 주소를 연결한다. B exe는 해당 stage unit의 guarded canonical path/hash이며 identity creation은 원 SDK receipt에 있다. normal PID는 각 runner child로64604/62020/34372/12100이며 normal의 독립 creation identity는 runner가 출력하지 않아 새로 주장하지 않는다.

baseline parent C는 고정844 `parent-exit`이고 fixture가 원 Child.wait exit0를 단언한 뒤 1pass로 돌아왔다. C exit0 근거는 이 단언+prior1pass이며 별도 C receipt/creation은 없다. hang D는 C의 same-exe 자손 marker로 관측됐다. source의 breakaway 없는 일반 Command와 원 Job 상속, SDK의 원 Job Empty/force receipt를 함께 근거로 삼는다. C/D 개별 Job readback·D 생성시각/개별 exitcode는 출력되지 않았고 입증하지 않는다. marker만으로 EOF나 원 family 종료를 판정하지 않았다.

B가 반환한 뒤 잔존 family의 종료는 원 Supervisor의 같은 Job/원 cutoff에서 관측됐으며 외부 resource-runner 종료/운영자 강제 정리로 대체하지 않았다. B의 SDK EOF 두 flag가 원 raw를 대상으로 관측됐고 A 두 건도 별도 EOF/receipt를 남겼다. 네 fixture raw는 정확1pass/0fail/0ignored이며 baseline marker는 별도 파일로 리다이렉트해 libtest 행에 섞이지 않았다. stdin_eof_requested는 요청 사실이며 application ACK로 표시하지 않는다.

## 보호와 반환

입력6 fresh SHA/bytes/mtime/FI 전후 불변이다. source/index/lock 명시9 fresh SHA/bytes/mtime도 이전 타입 gate 기준과 일치하고, baseline graph30 fixed 사본은 원 manifest SHA/bytes/mtime/FI와 전부 일치했다. S6081/T589/carry142/protected6/기존 producer4는 retained SHA+fresh stat/기록FI 대조 범위에서 변동0이다. cache 전수 fresh SHA/신규 파일 전수 부재를 주장하지 않는다. source2 overlay/원복 자체가 없으며 기존 r2 복원 상태를 유지했다. historical alias/known2 및 all_cache_protected=false는 그대로다.

원 실행에서 나온 파일54개를 `evidence-files.json` SHA256 `ECA628E76A237B218F318D12FBCA40B280C99D950A41B381CBCE2F72D6F06D24`로 고정했다. 이 파일들의 FileID는 post-terminal 읽기 시점 관측이며 생성 순간 독립 custody를 새로 증명하지 않는다. snapshot fixture48B의 compiled_baseline_proof=false를 외부 production baseline provenance로 바꾸지 않았다. 원 raw와 저장 snapshot은 덮어쓰지 않았다.

기록된 runner/normal/A/B/marker PID 숫자 조회는 모두 부재, writer0이다. C original PID와 D creation identity를 모르므로 이 숫자 조회를 모든 개별 프로세스 custody 증명으로 확대하지 않는다. 주 종료 근거는 원 SDK Job Empty+child exit+raw EOF다. 추가 PID kill/reopen·deadline 갱신0이다. canonical/index/stage/commit/push/GUI/AI/worker0, carry142는 기존 manifest 그대로 반환한다.

원 문제의 actual 반례와 green 거절은 이 frozen tool branch에서 확인됐다. **C2r3 source5696BCB8/expected23은 여전히 타입·등록·실행0**이며 원 producer를 재사용해 인수하지 않는다. 다음 private C2 gate는 별도 중앙 배정이다. 기존 modoo QA review-verification/fixture 본문의 supplied claim·원 first cause·mock/실제·물리/성공 분리 원칙으로 판정했다.
