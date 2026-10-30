---
id: evidence:kinekino/20261029114531
title: 원 worker 제어 gate의 IPC 직전 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 worker 제어 gate의 IPC 직전 검증

2026-10-27 중앙 PM. [제어 큐 계약](../../player/application/review-worker-command-gate.md)의 원본·instance·proof/close 경계를 검증한다. 기존 modoo.today code-change-review-verification/test-fixture-and-mock-build의 전달 전 실패/미확정·합성/실제 구분을 적용했다.

## 입력·영향 시험

runtime/verification/source375-review-worker-command-input-r1/SOURCE.md SHA256=54AE1E3C5D7CF25926906639F7798B3D5B2CF7106B158E1C8B65EC393A893B1E, inventory78EC880FF70485CF66F62CCD920EBD13F8F67F7FBF48C39F53A2B3D7A5AAE4FB/2185파일. 348 strict·349 영향7시험·350 actual no-run·fmt check 통과 뒤 입력을 고정했다. 최초347은 collapsible_if lint 실패였고 원 로그/해당 run 소스를 review-worker-control-preimage-r1/run-first-strict-failure.rs에 보존한 뒤 수정했다. 기존 pre-change worker/run/actual 소스 사본도 같은 디렉터리에 유지한다.

합성 mailbox5시험은 queued caller 만료 뒤 IPC 미전달, foreign instance/proof 무효화 거절, proof 검사 뒤 close가 먼저 확정된 interleaving, 단일 enqueue/unobserved receipt, sending 단계의 응답 단절을 execution_unknown으로 보존하는 경우다. 기존 status/prepare failure2를 포함해349 총7pass/0fail/0ignored였다. 이 fixture를 실제 엔진 전달 근거로 대체하지 않는다.

## 실제 CPU runtime·정상 CLI

351 정확 actual1 통과/outer0/8950ms(시험8.16초). 실제 Sprite proof를 원 worker stack에 보유하고 ReviewWorkerControl을 공통 SDK PlaybackControlPort로 주입했다. 정상CLI6process로17/25.5초 seek와 각 source/instance clock 및 종료 뒤 status NotFound를 확인했다.26초는 실제 CLI NeedsInput/not_sent로 거절했다. 다른 instance control은 SDK RevisionConflict, 종료 뒤 보유한 control reader는 NotFound/not_sent였다.

원 PID37852/creation134375172463818482, exit0/Job Empty/IPC 종료/비강제/first_error 없음. intent guard는3회(범위 밖1+정상 seek2), custody 검사는41회였다. 수락 receipt의 completion=unobserved를 실제 same-instance clock 관측과 구분하고 clock100ms 허용을 frame-exact로 승격하지 않는다. 원 proof는 run 반환/물리 종료까지 유지됐다.

- runtime/verification/review-worker-command-actual-datalab-r1/RETURN.json SHA256=4FF62F0FBFF1D23D2C83F15C2F33D22ED3132BBA17E8C6ABCC0C45E09E7EAEC9.
- 원 근거 runtime/build-cache/tmp/review-worker-22048-1793043641019153300의 WITNESS.json SHA256=0A1BE4EE0E01BC559AFD6FA1D4B622FB492815537E98219650BC53FE69D62F93. INPUT/PROCESS/OBSERVATIONS와 원 stdout/stderr/CLI PID 영수증을 보존한다.
- 정상CLI witness SHA256=B6E424251DA1E32B0285C0BE05496FC5295AF3F9EBB0CF48AF5C928C2879D6C7. 시험/CLI/tool/nomination/source/WITNESS/OBSERVATIONS guard의 전후 동일을 확인했다.

352 전체 입력 후속 감사가 종료0으로2185파일 content/identity 동일을 확인했다. 코드 편집은 freeze 이후 중단했다. SOURCEFECF의190/명시bins는 이전 통합 checkpoint로 유지하며 이 후속의 전체 회귀로 승계하지 않는다.

## 남은 범위

공개 command 목록96과 기본 지원 상태는 바꾸지 않았다. 제품 SessionRef/SessionOwner의 제어 요청·기본 host/fresh scope 조립·범위 play 정책·재시작/실패/취소 전체 행렬·이 변경의190/명시bins·AI/GUI는 남는다. 이 실제 gate 시험은 trusted host에 주입한 포트의 seek 범위이며 모든 control action의 기능 인수나 제품 UI 준비가 아니다. verified_timeline/actual9_binding/quality_approved/GUI인수는false다.

새 모델/캡처/다운로드/수동 GUI/WARP/push는0이다. 기존 headless CPU 미디어 실행만 수행했고 C 문맥별 cache·heavy1/jobs1/CPU2/BelowNormal/locked/offline·기존 소유권/WIP를 유지했다.
