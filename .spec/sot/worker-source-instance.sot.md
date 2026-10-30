---
id: sot:kinekino/worker-source-instance
title: 원 worker의 source/instance 제어 큐
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 원 worker의 source/instance 제어 큐

2026-10-27 중앙 PM. [proof 수명](review-scoped-proof-worker.md)을 유지하며 IPC 전달 직전 권한을 확인하는 기반이다. 일반 queue admission을 native 실행 완료나 frame-exact 근거로 승격하지 않는다.

ReviewPlaybackWorker.scoped_control은 유효한 case/media/원 engine instance에 묶인 ReviewWorkerControl을 만든다. 이 포트는 공통 PlaybackControlPort를 구현하고 raw JSON/path/program을 받지 않는다. 실제 authority는 생성자가 아니라 원 worker가 보유한 input identity·동일 control instance·원 proof 및 command별 guard다. worker thread 밖으로 non-Send proof를 옮기지 않는다.

bounded64 queue의 요청은 fixed2초 응답 예산과 원 runtime deadline을 따른다. 원 worker는 source/instance를 대조하고 source/custody/intent guard를 검사한 뒤, close gate와 직렬화한 작은 critical section에서 IPC enqueue를 수행한다. 한 poll은 최대64개만 처리해 cleanup을 무한히 미루지 않는다. 종료 중에는 proof 검사를 반복하지 않고 남은 큐 요청을 not_sent로 거절한다. close와 마지막 enqueue를 같은 gate로 직렬화한다.

정상 phase는 queued→sending→finished다. 호출자는 기다리기를 끝낼 때 queued에서만 CAS로 not_sent를 확정하고, 늦게 worker가 재개해도 IPC에 들어가지 않는다. worker는 IPC enqueue 자체가 거절된 경우에도 not_sent를 기록한다. sending/finished 중 응답 유실은 execution_unknown이며 자동 재시도하지 않는다. 성공 receipt는 기존 ControlReceipt의 accepted/unobserved·원 engine instance이며 적용/완료 ACK가 아니다. source/instance/guard/close 실패의 전달 전 오류는 not_sent를 보존한다.

run_with_control_guard의 callback은 원 worker stack의 proof를 검사한다. 범위의 source-time point 검사는 명시 mapping coverage만 사용하고 전체 영상 duration으로 확장하지 않는다. 실제 시험은 Sprite17~25.5초의 seek point(양 경계 포함)와26초 거절을 확인한다. 이는 range 전체의 play 경계 정지·decode frame 일치나 사용자 AI 품질 승인과 다르다.

기존 raw control/guard 없는 run wrapper는 trusted 저수준 호환 경로다. 후속 SessionOwner/SessionRef 제어는 공개 reviews.playback.control로 연결했고 관측 coverage 기반 Play/Pause 정책과 실제CLI35를 검증했다. [후속 근거](../../verification/application/review-session-control-cli-20261027.md). 공개 명령은97이지만 기본 host는 manager 미주입 Unsupported다. 제품 기본 host 조립과 재시작/취소 전체 인수는 별도이며 frame 일치·사용자 AI품질을 주장하지 않는다. 수동 GUI·추론·다운로드·WARP·push 및 기존 소유권/WIP/heavy1/C cache 정책을 유지한다.
