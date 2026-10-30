---
id: evidence:kinekino/20261029114833
title: 로컬 review runtime의 사전 Job 소유권
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 로컬 review runtime의 사전 Job 소유권

최종 영향 검사: 229 제품 및 owned-process(test-support 포함) all-targets clippy -D warnings terminal=true/exit0, 42096ms. fmt --all --check 통과. dependency blitz-paint의 기존 deprecated 경고는 남아 있으며 담당 코드를 수정하지 않았다. stage된 변경 없음. 전체190/정상 bins의 이번 변경 재검증 및 실제 media backend 결합은 미완료다.

2026-10-27 중앙 PM. 기존 episode/C 담당의 Mpv/HostScope 경로는 변경하지 않았다. 공통 Rust owned-process에 선택적 사전 Job 경로를 추가했다.

## 구현 및 보장

PreparedFamilyJob::reserve(JobBudget)는 실제 unnamed Windows Job을 생성한다. 양수 committed-memory 한도와 현재 프로세스/시스템 할당의 부분집합인 affinity를 요구한다. KILL_ON_JOB_CLOSE·JOB_MEMORY·AFFINITY와 실제 수치를 QueryInformationJobObject로 다시 확인하고 family Empty를 검사한다. public raw Job handle·Default·Deserialize는 없다.

spawn_prepared는 바로 그 Job을 move하고 기존 suspended create→원 image/creation identity→assign/membership readback→resume 경로로 연결한다. 기존 spawn/spawn_faulted 호출은 종전의 Job 생성을 유지한다. post-create 실패는 원 WindowsFamily/Job carrier를 반환하고 pre-create 실패는 child가 없는 상태로 종료한다. 기존 cleanup 규칙을 변경하지 않았다.

JobMemoryLimit는 committed-memory 한도이며 RSS/working set나 물리 RAM 예약으로 주장하지 않는다. 이에 application ReviewRuntimeProfile의 필드도 max_job_committed_bytes로 명확히 했다. 이 경로만으로 중앙 heavy permit·고정 tool·source custody·미디어 runtime 입장이 완성되지는 않는다.

## 실행 근거

추가 강화: 할당 후 resume 전에 같은 budget을 OS에서 재조회한다. 실패하면 AssignedSuspended 원 owner를 기존 failure carrier로 반환한다. 228 실제 family9시험을 재실행해 모두 통과했다. 227 제품 admission3시험도 통과했다. 이후 변경은 이 Job 재조회이며 제품/owned-process all-target strict는229로 별도 확인한다.

- 223 owned-process: unit6, polled cleanup9, supervisor9, compile-fail doc2 통과. 당시 실제 Windows family8시험은 ignored로 남았다.
- 224 명시 배정 actual Windows family9/9 통과. 새 시험은 사전 Job budget→같은 child owner에서 OS budget 재조회→ready→stdin EOF→exit0/family Empty/forced=false를 확인했다. 기존 argument roundtrip·partial startup·assignment/termination/query failure·sentinel 분리·late resume 및 timeout 시험도 통과했다.
- 새 actual fixture/원 stdout/stderr는 runtime/build-cache/tmp/owned-family-25288-13에 보존한다. 실제 media engine나 GUI를 실행하지 않았다.
- 225 strict는 새 테스트의 manual_isolate_lowest_one lint로 실패했다. 기록을 보존하고 isolate_lowest_one으로 수정하여 226 owned-process all-targets clippy -D warnings 통과.

이 기록은 Source396 선택190/정상 bins나 E710 봉인 publisher checkpoint를 갱신하지 않는다. 다음 whole input freeze/회귀는 실제 미디어 backend 통합 뒤 별도 수행한다. WARP·추론·다운로드·수동 GUI·push 보류와 C cache/heavy1/CPU2를 유지했다.

기존 적용 modoo.today test-fixture-and-mock-build/code-change-review-verification의 fixture/실제·강제종료/정상종료·child exit/family Empty 구분을 적용했다. [custody 근거](review-playback-object-custody-20261027.md), [원 runtime 설계](../../player/application/review-playback-runtime-admission.md)에 연결한다.
