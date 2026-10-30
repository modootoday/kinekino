---
id: evidence:kinekino/20261029114618
title: SDK typed close progress
status: active
created: 2026-10-29
updated: 2026-10-29
---
# SDK typed close progress

SDK 단독 구현·검증을 마치고 최종 입력을 반환했다. UI/Native/generic 소스는 동결하고 새 OS 창·모델·추론·다운로드는 실행하지 않았다. [공유 계약](../../player/application/api/sdk-execution.md)의 Rust 전용 poll_close를 추가했으며 CLI JSON wire나 proposal 정책은 변경하지 않았다. UI/OS의 bounded close retry 배선 완료를 뜻하지 않는다.

## 변경 전 경쟁 재현

`runtime/quality-review/sdk-close-progress-r1`에 source-before 사본과 pin, barrier-before 시험 사본·stdout/stderr/receipt를 보존했다. barrier-before는42345ms/exit0·정확1pass/0ignored/315filtered였다. 이는 기존 경쟁을 제어 관측한 expected pass이며 수정 완료 시험으로 세지 않는다.

idle owner의 마지막 notifier가 channel 신호 뒤 barrier에서 기다리는 동안 기존 close는 NeedsInput과 worker_returned=true/worker_stopped=false/unacknowledged=0/closing=true를 반환했다. barrier를 풀고 명시 close를 재시도하면 join이 완료되지만 notifier 호출은1회 그대로다. 따라서 wake를 join ACK로 볼 수 없으며 SDK가 자동 후속 wake를 보장하지 않았음을 확인했다.

## 구현 경계

새 close.rs는 기존 control의 close 전이를 분리하고 legacy close와 poll_close가 같은 observe_close를 사용한다. 미완료 thread를 join하지 않으며 별도 observer thread·즉시 Busy wake 반복을 만들지 않는다. queue 취소·권한·예산·SDK 유스케이스는 기존 경로를 유지한다.

WorkerCloseState는 ExitPending { loop_finished }/Joined, ExecutionCloseProgress는 TerminalAckPending/WorkerExitPending/Faulted/Closed다. Faulted에는 typed kind(Terminal/WorkerJoin)와 canonical diagnostic Arc, unacknowledged, worker 상태가 있다. authority/poison 등 관측 실패는 Result::Err이며 join panic은 typed Faulted로 반복 보존하고 legacy close에는 같은 canonical Err로 반환한다. 최초 제안의 join panic outer Err 설명은 채택하지 않았다. serde wire는 추가하지 않는다.

terminal fault의 원 Request/Response는 retained_operations/peek로 비소비 조회한다. 같은 diagnostic Arc를 반복 관측하며 명시 ACK와 성공 join 전 정상 Closed를 반환하지 않는다. join_failure는 take한 JoinHandle과 별개로 보존하므로 이후 poll/close에서도 사라지거나 Closed로 바뀌지 않는다. legacy NeedsInput의 기존 details와 성공 receipt는 유지한다.

## 집중 실행

focused-first50392ms/exit0·17pass/0ignored/304filtered·본문0.06s를 확인했다. 최초11개 회귀에 변경 전 barrier1개와 신규 typed barrier·unack Joined/legacy details·fault/raw 동일 Arc·권한/8caller 동시close·실제 join panic 보존5개를 더했다. 실제 join 횟수는 cfg(test) 계수로 확인하며 production에 계수나 새 thread를 추가하지 않았다.

join panic 시험은 정상 종료한 test owner의 빈 worker slot에 소유한 failing thread를 주입했다. 원 SDK 작업 panic을 실서비스에서 관측한 증거가 아니다. 최초 worker join1회와 주입 thread join1회 이후 반복 poll/close의 추가 join0을 확인한다.

수정 후 source-current 사본·sdk-source-pins와 runner-pins를 보존했다. 사용한 native-audio-webview-r1 runner와 이전 UI frame-document-cli-r1 runner는 모두 SHA256692c22751fac7f9999f2d833571dab6605afd58e66711370494e62dd8a62c463이다. 단일 heavy·2CPU/BelowNormal/descendants job·offline/locked/-j1·단일 test thread를 유지한다.

## 최종 실행과 반환

| 원 증거 prefix | 관측 결과 |
| --- | --- |
| barrier-before |42345ms/exit0·정확1pass/0ignored, 변경 전 경쟁 관측 |
| focused-first |50392ms/exit0·17pass/0ignored/304filtered, 본문0.06s |
| strict-first |60656ms/exit0, all-targets Clippy -D warnings |
| root-full |401449ms/exit0·전체426pass/0fail/23ignored, rootlib316pass/5ignored |
| ui-consumer |409776ms/exit0·83pass/0fail/0ignored, 고정 consumer 입력의 호환 회귀 |
| fmt-check |1451ms/exit0, product scoped cargo fmt --check |

각 원 exec session의 terminal과 receipt/stdout을 확인했다. 집중 gate 후의 변경은 같은 의미의 if-let 문법 정리와 fmt이며 그 최종 소스는 strict/root 전체/consumer가 검증했다. sdk-source-pins.json과 마지막 sdk-source-final.json은 모두 SHA2566F8EA80821549619B4219E465E1FFC2FAF67EA0E1440E600B30215FA86CE7044로 같아 최종 입력 이후 변경이 없음을 확인했다.

SDK/application parent·async assembly·manifest source·root Cargo/lock 포함 final-source-inventory.json의 최종 SHA256은 E1959DC775A0EBA92619F83AB26EB48672367298CD5441D80D66B265DA298BF5다. UI16 파일의 실제 재hash는5b6515510b154805e5d8d199fdf778f3d439486a0fd6f7a15113def151d17a2f, Native 입력은8cf2ab2f6eb431a6138288f6475be73c7506fecb096cc3dde481d8df4fed4739, generic5는f469a872ece03c8c41ab4f2a534658a97cba0e94de2a7aa2227c7f05cdea8d2b로 인수 값과 동일했다. 각 source-final JSON과 원본 inventory를 분리 보존했다.

manifest-final.json은 기존 UI 인수의 final-manifest-observation.json과 SHA2563E00F0BE8EBEA4AB4F473B17E3B077228E92BE9BE0BD15B636C45A7AB6DEA293로 같았다. dependency·manifest 파일과 lock 변경은 없다. provider의 include_bytes 기반 부분 source identity는 SDK types/control 변경으로 달라질 수 있으며, 이 값이나 변경 없는 manifest 파일을 전체 schema/provider attestation으로 주장하지 않는다. 신규 close.rs를 포함한 정확 SDK 입력은 위 최종 inventory로 구별한다.

기존 actual CLI73명령의 원 evidence/binary pin은 S1 이력으로 보존했다. 이번 변경은 Rust 내부 종료 관측이며 CLI wire/command·proposal 정책 변경이 없어73을 다시 실행하지 않았다. 현재 root 전체의 실제 CLI/typed port·capability parity 회귀를 영향 근거로 사용하며 PM이 이 범위를 인수했다. 과거73을 새 바이너리의 실행으로 승격하지 않는다.

final-owned.json의 cargo/rustc/link/resource-runner 이름 기준 프로세스 관측은[]였다. UI consumer의 장기 링크 중에는 같은 handle을 유지했고 timeout으로 중단·재시작하지 않았다. UI assertion/입력 수정이나 cache 삭제·우선순위 변경·새 관측 도구는 없었다. 단일 heavy 슬롯을 PM에 반환한다.

UI/OS가 WorkerExitPending에 미래 bounded 재시도를 배선하는 작업은 후속이다. TerminalAckPending/Faulted는 실제 원 operation/fault를 보존하며 자동 무한 timer·무조건 ACK 대상이 아니다. 실제 OS wake 자동 close 진행·시각 일치·강한 S2 in-flight 취소/게시·전체 제품 goal은 완료하지 않았다.
