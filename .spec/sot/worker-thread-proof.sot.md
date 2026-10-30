---
id: sot:kinekino/worker-thread-proof
title: 원 worker thread의 proof 수명 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 원 worker thread의 proof 수명 계약

2026-10-27 중앙 PM. 제품용 source-bound 재생 open/close를 위한 기반이며 공개 API 완료가 아니다. [Verified 저장](api/review-source-draft-contract.md)의 plain record를 live 권한으로 사용하지 않는다.

ReviewPlaybackWorker.spawn_scoped는 trusted Rust assembly의 closure를 원 worker thread에서 호출한다. closure 안에서 case/media/evidence resolver와 non-Send RetainedTimelineEvidence를 생성한 뒤 ReviewWorkerLaunch.run(input, check_custody)을 동기 호출한다. closure의 원 proof와 resolver는 run의 원 child exit·원 Job Empty·IPC join·terminal 기록이 끝나 반환할 때까지 같은 stack에 남는다. unsafe Send나 caller JSON capability를 도입하지 않는다.

run은 입장과 child 생성 직전 및 실행 loop에서 check_custody를 호출한다. 실패는 새 실행 전이면 child 미생성 오류, 실행 중이면 기존 closing/원 Supervisor cleanup으로 이어진다. closing 뒤 추가 제어는 기존 connected 하향 및 control 제거 정책을 따른다. cleanup 동안 proof는 해제하지 않는다. raw 범위 계획·저장 Verified·engine clock은 frame 실제 완료나 사용자 AI 품질 승인이 아니다.

기존 spawn(prepare)는 추가 timeline 권한 없이 같은 scoped 경로를 사용하는 호환 wrapper로 유지한다. 이 저수준 trusted 경로를 제품의 Unverified 재생 지원으로 광고하지 않는다. 공개 제품 service는 exact association revision·stored mapping과 새 retained proof 일치·명시 runtime 허가를 결합해야 한다. command별 즉시 권한 재확인과 open/close idempotency·실패/취소/재시작 인수도 후속이다.

panic의 전체 cleanup 행렬은 아직 검증하지 않았다. owner_thread_ended를 physical exit로 바꾸지 않는 기존 경계를 유지한다. 수동 GUI·추론·다운로드·WARP·push 보류 및 heavy1/C cache 문맥을 유지한다. 실제 범위는 새 입력/원 witness·원 PID/Job 종료 근거와 함께 기록한다.
