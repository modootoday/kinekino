---
id: evidence:kinekino/20261029114505
title: 별도 host 프로세스 EOF·재시작 실제 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 별도 host 프로세스 EOF·재시작 실제 검증

후속401 감사는2195파일 content/identity 동일·종료0/39333ms다. hostPID35920/13388 부재와 index 비어 있음을 확인했다.

2026-10-27 중앙 PM. 새 개발 코드는 Rust 시험 fixture에만 추가했다. 제품 구현/설정·trusted admission 권위는 변경하지 않았다. source375 private overlay/원 실패/이전 통합을 구별한다.

입력 runtime/verification/source375-review-process-host-input-r1/SOURCE.md SHA256 `CE23E8A8746B4A7DC8174131A10C8ECAF6277D7E6B0ACA2F37F27956F7838448`; inventory `D266FE90267333C468B62AF4D8535CC16C531A7E929E9C0BDFD56192640EE65F`,2195파일. 이전E28 inventory와 git diff --no-index 대조 시 actual main/sessions fixture2변경·process_hosts fixture1추가뿐이며 제품 소스·Cargo/lock/도구·자산 행은 동일했다. 이를 이번 입력의 전체190 재실행으로 계산하지 않는다. preimage는 runtime/verification/review-process-host-preimage-r1이다.

397/398 strict·399 no-run·400 actual1(22.78초)·fmt --check 통과. stdout/stderr/원 launcher receipts는 source375-datalab-b0-r1-logs 각 번호다. 실제 결과 runtime/verification/review-process-host-actual-datalab-r1/PROCESS-HOST-RETURN.json SHA256 `A2F5B731C934485CF1D5443CEC097733FF63D844184750D789632B73F68FCCBB`.

원 test producer `D1FEEEBE6D79F106D53FDB128E00BAFB6DC0AF787CEE483A6FC8CE09DFD8D7E7`를 두 별도 OS 프로세스로 실행했다. trusted Rust fixture가 고정 봉인 NOMINATION SHA와 도구 pins로 조립하며 임의 JSON을 product admission으로 승격하지 않는다. 정상CLI producer `2FFEC1A5591587E9938076049DB3042D2F94DEF40D29F7AAAC89B3C018B1DD60`의20요청을 각각 원 process receipt로 보존했다.

첫 hostPID35920은 실제 Verified 재생 연결 중 stdin EOF로 종료했고 원 pipe/SDK Joined·owner1물리완료·host exit0/forced=false를 기록했다. 같은 user-review/process-runtime root로 두 번째 hostPID13388을 시작하고 이전 session status=RevisionConflict를 실제CLI로 관측했다. 동일 open key/ordinal1은 새 scope와 새 session을 만들었다. 두 번째도 연결 중 EOF→Joined/owner1물리완료/exit0/비강제였다. 각 READY/STOPPED/PROCESS 및 원 stdout/stderr를 보존한다.

두 mpv PROCESS 기록은 engine_instance_id1을 각각 재사용했지만 host scope는 달랐다. 모두 exit0/forced=false/Job Empty/IPC 종료/first_error=null이다. 원 PROCESS의 owner_thread_ended=false는 마지막 기록 시점이며 STOPPED의 후속 owner 물리완료 관측과 구분한다. 두 미디어는 직렬 실행했다. forced cleanup fixture 분기는 실행되지 않았다.

이 시험의 RETURN.json은 선행 publication-only/no-decode 범위다. 뒤이어 PROCESS-HOST-WITNESS/PROCESS-HOST-RETURN에 두 host·두 실제 decode를 별도로 기록했으므로 publication receipt의 new_decode_execution=false를 전체 시험의 미실행으로 해석하지 않는다.

이 결과는 별도 프로세스에서 shared product assembly를 사용한 fixture다. 제품 CLI serve 설정/승인 입력 공급, stdin read/write 실패·강제 crash/재시작 전체 행렬·GUI/AI품질은 미완료다. actual9_binding/quality_approved=false. 이전E28의190/정상bins는 해당 원 입력의 통합 근거로 보존한다. 기존 modoo.today QA/cache/shared-checkout 적용 출처·판단을 유지하며 C native/tmp/locked/offline/jobs1/CPU2/BelowNormal/heavy1·원 소유권/WIP/보류를 유지했다. 수동GUI·추론·다운로드·WARP·push0.
