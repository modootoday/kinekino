---
id: evidence:kinekino/20261029113741
title: 제품 Native UI helper 4단계 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제품 Native UI helper 4단계 검증

UI4 검증·인계 checkpoint. 증거 경로는 `runtime/quality-review/product-native-window-helper-r1/`다. [UI 계약](../../player/presentation/product-native-window-helper.md), [계획](../../../plans/player/presentation/product-native-window-helper.md)을 함께 읽는다. main/Cargo/SDK 입력8/Native316/Core/CSS를 변경하지 않았다.

## 현재 관측

| 실행 | 결과 | receipt elapsed |
| --- | --- | --- |
| check-first | 실패: ReviewScope/PluginScope 혼동2곳, AtomicU64 deprecated 경고 | 원 로그 보존 |
| check-r2 | 최소 수리 후 check 성공 | 9687ms |
| product-first | actual SDK headless 8 통과 | 137485ms |
| consumer-r2 | 기존 consumer 전체 + helper8 실패0, 기존 OS9 ignored | 462587ms |
| root-on | product lib366 통과/5 ignored, workspace lib 총414 통과 | 102587ms |
| product-r3 | 새 OS test support import 누락 컴파일 실패, 수리 | 3204ms |
| product-r4 | headless9 통과/1 실패: context 오류 기대값 Unsupported가 실제 InvalidArgument와 불일치. OS2 ignored | 54341ms |
| product-r5 | direct SDK code/details 대조 후 headless10 통과. OS2 ignored | 53405ms |
| os-close | 실제 product helper 정상 frame/close와 SDK join, exact1 통과 | 2039ms |
| os-attach | 실패: deferred Native attach를 consumer preflight attach로 잘못 예상 | 1534ms |
| os-attach-r2 | Native Startup::Attach 원 Host 오류·recovery token·view notice0·SDK join·Unconfirmed 유지, exact1 통과 | 18806ms |
| consumer-strict | all-targets feature-on clippy -D warnings 성공 | 50978ms |
| root-on-strict | 신규 cleanup의 collapsible_if 경고1로 실패. 동일 조건 let-chain으로만 정리 | 10801ms |
| root-on-strict-r2 | all-targets feature-on clippy -D warnings 성공 | 32040ms |
| root-off | 기존 jobs actual_cli_explicit_host_and_sdk_share_durable_jobs_without_gui의 forced-submit read_response client_deadline timeout으로 실패. execution_unknown/실제exit10 vs 기대0 | 326273ms |
| jobs-r2 | 새 독립 namespace+기존 JOB_EVIDENCE_DIR 보존으로 exact1 통과, 정책/timeout 변경 없음 | 10127ms |
| root-off-r2 | 기본 구성 전체440 통과/0 실패/23 ignored | 151151ms |
| consumer-off | feature-off 전체96 통과/0 실패/9 ignored. 부분 포함 fixture의 poll_shutdown 미사용 경고1 | 496570ms |
| consumer-off-ports | factory20 통과, initial 부분 실패 시험은 receipt 보관 위치를 단일 필드로 가정해 실패 | 61120ms |
| consumer-off-ports-r2 | controller와 동일한 unregistered/factory-parent 조회로 수정. 영향3 target43 통과/0 실패/9 ignored | 54858ms |
| consumer-off-strict | all-targets feature-off clippy -D warnings 성공. 경고 숨김 없음 | 41909ms |
| canonical-final | 최종 canonical product SDK10+factory20 통과. worker가 받은 context 전체 및 direct SDK 오류 code/details 일치 | 143376ms |
| root-fmt | cargo fmt --all --check 성공 | 2797ms |
| consumer-fmt | consumer cargo fmt --all --check 성공 | 3182ms |

context 기대값과 deferred attach 기대 단계 수정은 테스트의 잘못된 가정을 바로잡았으며 SDK/Native 정책 변경으로 우회하지 않았다. 원 실패 로그는 덮어쓰지 않는다. canonical-final에서는 실제 SDK client seam에서 worker가 받은 context 전체를 JSON 값으로 관측해 원 요청과 동등함을 확인했다. direct SDK와 동일 code/details를 보존하고 business failure의 typed Arc를 명시 종료 수락 후에도 유지했다.

미사용 경고는 lint 허용이나 public 범위 확대로 숨기지 않았다. handoff 자체 종료와 등록 전/후 실패 최종 정리를 실제 worker로 검사하는 단위시험2개를 추가했다. 동일 prepared/raw Arc와 worker inspect1, SDK join·미ACK0, 최종 정리 시작 후 recover_for_retry 거부를 확인했다. 등록 전 receipt는 failure.close_progress, 등록 후는 factory.parent_close_progress에 보존되며 이는 기존 controller의 canonical 조회 경로와 같다. 해당 시험의 최초 단일 필드 기대 실패도 보존한다. 기존 factory/bridge 실패 시험은 poll_cleanup을 직접 거친 뒤 legacy retry의 멱등성을 검사하도록 보강했다.

root-off 원 timeout의 원인은 확정하지 않았다. 같은 uncertain 요청을 재전송하지 않고 새 fixture를 실행했다. 첫 실행의 임시 fixture는 기존 tests/jobs/fixture.rs Drop이 삭제하므로 원 publication/storage 상태를 사후 확인할 근거는 남지 않았다. 원 stdout/stderr/receipt와 가계 종료 관측은 보존하며 publication 성공/실패 어느 쪽도 추정하지 않는다. jobs-r2의 독립 fixture 원 request/response/CLI SHA/실제 store는 runtime의 jobs-r2 하위에 보존한다. 그 통과를 원 uncertain 요청의 결과로 해석하지 않는다. 기본 전체 gate 재실행은 별도 root-off-r2다.

read_response timeout은 frame 쓰기 완료 이후 응답 deadline을 넘긴 관측이다. dispatch/admit/publication 여부는 불명이다. source blob 존재도 admit 전 prepare의 결과일 수 있어 성공 증거가 아니다. `jobs-fixture` request_id만으로는 상관키가 충분하지 않으며 향후 재현에는 독립 endpoint/label, journal submission key/hash/job_id/revision/attempt, publication receipt와 host phase trace를 함께 대조해야 한다. SDK transport/job 정책은 이번 UI 단계에서 변경하지 않았다.

정상 OS 시험은 실제 Native runner의 result Ok와 Closed runtime, SDK Closed(worker_stopped/미ACK0)를 함께 확인했다. frame은 checked CPU submission이며 OS compositor의 픽셀/시각 승인이 아니다. attach 거부는 context.rs의 Plain::run schedule 일치 검사에서 configure_shell 전 실패한다. attach_with_observer는 동일 runtime Rc를 token/job에 clone하고 runner는 token.runtime을 outcome.runtime으로 보존한다. controller/runtime identity 검사를 통과한 같은 bridge의 SDK join을 관측하고 recovery token 자체도 반환 carrier에 남긴다. token의 Native 권한이 정상 teardown됐다고 주장하지 않으며 SDKQuiescent Unconfirmed를 유지한다. 새 loop/worker는 생성하지 않는다.

OS 원 실행 뒤 production cleanup에는 동일 조건의 중첩 if→let-chain lint 정리만 있었고, 나머지 후속 변경은 종료 포트 시험 보강이다. 최종 canonical headless30은 이 최종 production 소스를 사용했다. OS2를 최종 source pin의 새 OS 재실행으로 표시하지 않으며 불필요한 GUI 반복은 추가하지 않았다.

## 핀·보호 범위와 미완료 게이트

변경21 final-source-inventory SHA256 `535EC6B64BF0737C3C19384620E4EBD856FE982DCD963A77610330FEFBC855A9`, 고정 module/CSS10 inventory `58E4C98A8ED0A2FD62EA72C1CFAB57ACBF89A8CAB485689C793A64338CCA818B`와 사본을 보존한다. 최종 직접 재해시에서 변경21/SDK8/Native316 차이는0이며 assembly/main/Cargo·consumer Cargo·generic5 보호 관측도 별도 파일에 남긴다. 단일 runner/2CPU/BelowNormal/Job/offline locked -j1/테스트 thread1을 사용했다. 각 원 exec 종료/terminal receipt/가계[]를 확인한 뒤 다음 Cargo를 실행했고 최종 소유[]를 인계한다. root-on 광범위 실행 이후 추가된 initial 포트 시험2개는 off 부분 포함 경로2개에서 실제 실행했다. 그 복제 실행을 canonical root 전체 재실행이라고 표시하지 않는다.

product CSS/VerifiedAssets의 실제 main 연결, renderer CSS/폰트 지원, 전체 승인 시안13 및 물리 입력·VM/DPI는 완료하지 않았다. 기능 fixture의 CSS empty/assets None과 사용자 관측 흰 창의 동일성은 확인되지 않았다. 프로그램 중간 기능 화면을 최종 제품 디자인으로 표시하지 않는다.
