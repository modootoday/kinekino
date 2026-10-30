---
id: evidence:kinekino/20261029114239
title: Review host 종료·원 owner 물리 완료
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Review host 종료·원 owner 물리 완료

후속380 감사는2192파일 content/identity 동일·종료0/40293ms다. 두 실제 PID32584(creation134375200331613554)/31292(creation134375200419749072)의 원 PROCESS.json은 exit0/forced=false/family_empty/ipc_exited=true/first_error=null이다. PROCESS.json의 owner_thread_ended=false는 원 thread 마지막 파일 기록 시점이며, 이후 application 관측의 owner_finished=true와 구분한다. 마지막 owner 완료는 RETURN/HOST-SHUTDOWN의 후속 관측으로 입증한다.

2026-10-27 중앙 PM. common ReviewPlaybackSessions의 begin_close/observe_shutdown을 추가했다. open/control과 같은 lock에서 새 입장을 차단하고 원 owner 전체에 close를 요청한다. 기존 key replay/status/close는 보존한다. shutdown 관측은 case/media 권한이 사라져도 원 owner를 직접 확인하며 child가 있었다면 Job Empty/IPC 종료까지 요구한다. SDK 전체 host 종료 명령은 추가하지 않았다. caller는 최초 close 오류를 후속 성공으로 덮지 않아야 한다.

입력 runtime/verification/source375-review-host-shutdown-input-r1/SOURCE.md SHA256 `EB637CA274C97793B7B85541047F5766A90073F9EC9AB56C475DE4DB795632B8`,inventory `915821CB36BCB164CE78CC4457FBAD13FC5120490FB62E37BE40DD109A4AC219`,2192파일. preimage는 runtime/verification/review-host-shutdown-preimage-r1에 보존했다. 기존3940 영향/2088 통합/원375 overlay는 구별한다.

376 공통17시험·377 all-target native strict·378 no-run·379 actual1(24.97초)·fmt --check가 통과했다. 합성3신규는 빈 host/새 요청 차단/원 replay/child 물리 gate/권한 상실 후 cleanup/close 오류·관측 유실 시 owner 보존을 검증했다. 원 로그/launcher receipts는 runtime/verification/source375-datalab-b0-r1-logs의 각 번호다.

실제 결과 runtime/verification/review-host-shutdown-actual-datalab-r1/RETURN.json SHA256 `ABBC088B8CFB02D2E09A924C47E9BFC077FDCCB5656E9BF75693AF7B63D21F18`. 실제 Verified owner 정상 close·Unverified child미생성 뒤 같은 fixed Sprite/mpv의 두 번째 미디어 owner(instance2)를 직렬 연결했다. begin_close 두 번 뒤 새 open=NotFound/control=not_sent·기존 key 조회 보존을 확인했다. 총3owner/physically_finished3/complete=true. 마지막 owner는 process_created/physical_closed/family_empty/ipc_exited/owner_finished=true·failure=null이었다. HOST-SHUTDOWN.json과 각 runtime 원 증거를 보존한다.

이 시험은 application host 수명 종료이며 별도 OS 프로세스 stdin EOF/transport 오류/강제 종료·실제 CLI serve-review 경로 인수가 아니다. 기본 제품 host 설정·trusted admission 공급·transport/dependency 종료 연결은 여전히 남는다. 이 입력의 전체190/명시 정상bins는 미실행이다. actual9 연결/AI품질/GUI시각 완료로 확대하지 않는다.

기존 modoo.today QA/cache/shared-checkout 적용 판단을 유지해 합성·실제·수락·물리 종료를 분리했다. 정확한 출처·본문/참조SHA는 선행 worker 제어/공통 조립 검증 문서에서 유지한다. C native/tmp·locked/offline/jobs1·CPU2/BelowNormal/heavy1 유지. 전역설정·기존소유권/WIP·원실패를 보존하며 수동GUI·추론·다운로드·WARP·push0이다.
