---
id: evidence:kinekino/20261029114408
title: 실제 시간축 proof를 원 worker 종료까지 보유 — 2026-10-27
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 시간축 proof를 원 worker 종료까지 보유 — 2026-10-27

중앙 PM. [scoped worker 계약](../../player/application/review-scoped-proof-worker.md)의 영향·실제 검증이다. modoo.today code-change-review-verification/test-fixture-and-mock-build의 근거·부분 인수 구분을 적용한다. 공개 재생 open/close의 완료 근거가 아니다.

## 입력·실행

runtime/verification/source375-scoped-proof-worker-input-r1/SOURCE.md SHA256=EBFFE08604DC068A5518AFC8E5E666AE6A6F48C0762D49D77F1DB1B38D18083F. inventoryA302880F9F71F9CC1EB032CE9C3B99FEDAA470C83C0D4D41179CCFC6D72518F0/2176파일. 309 초기strict·310 no-run·311 최종strict/fmt 통과 후 입력을 고정했다. 기존 SOURCE6381DE6F의190/정상bins는 이전 checkpoint이며 이 변경의 전체 회귀로 승계하지 않는다.

312 실제 CPU media/정상CLI 시험 정확1 통과(시험7.33초). 봉인 Sprite의 case/media/evidence와 기존 tool pin으로 실제 resolver를 worker thread 내부에 만들었다. non-Send proof와 resolver는 launch.run의 동기 stack에 유지했고 입장/child 생성 직전/실행 loop에서 총49회 check_current를 수행했다. CLI seek17/25.5초·동일 engine status 관측 및 종료 후 CLI NotFound를 확인했다.

원 PID35128/creation134375121482151996, child exit0·family Empty·IPC 종료·비강제·first_error 없음. 준비 범위는 연결 관측 시 살아 있고 원 run 반환 후 끝났음을 확인했다. 원 source/case/tool/witness/test/CLI producer guard의 전후 불변을 확인했다. 313 기존 prepare failure exact1도 통과해 child 미생성·첫 오류 보존·worker 종료 경로를 유지했다.

실제 출력 runtime/verification/review-scoped-proof-worker-actual-datalab-r1/RETURN.json SHA256=2ACDF13A79C7E8E1CABF9752996F10DBB6C0446766F8A7330EC134BDDB1F1988. 원 worker 근거는 runtime/build-cache/tmp/review-worker-27436-1793038543908302800이며 WITNESS.json SHA256=F482A993AAF3802186609E8DC892026F6E1A3AD9AFEE362999F5E30440518A5E다. 원 INPUT/PROCESS/OBSERVATIONS 및 stdout/stderr는 그대로 보존한다. 정상 CLI witness SHA256=35DB09B76252CA6C69E525DEEE51540FA96BC7C3422D9175D9C6F15B163895DA.

314 전체 입력 후속 감사는 종료0으로2176파일의 content/identity 동일을 확인했다. 검사 중 소스 변경은 없었다.

## 한계·후속

retained_timeline_custody=true는 실제 raw 근거의 수명 확인이다. 저장 association의 exact revision/mapping을 선택해 open한 public service가 아니므로 verified_timeline=false·actual9_binding=false·quality_approved=false·GUI 인수false를 유지한다. engine-time을 frame 완료로 바꾸지 않는다. worker guard가 어떤 proof와 input을 결합할지는 trusted assembly 책임이며 공개 service에서 exact binding을 검증해야 한다.

공개 SDK/CLI open/status/close, command별 즉시 권한 재확인, open/close idempotency·취소·재시작·late revocation/panic 전체 행렬은 미완료다. 모델·다운로드·수동 GUI·WARP·push는 재개하지 않았다. 기존 headless CPU 기술 시험만 수행했고 heavy1/C cache/jobs1/CPU2/BelowNormal·기존 WIP/소유권을 유지했다.
