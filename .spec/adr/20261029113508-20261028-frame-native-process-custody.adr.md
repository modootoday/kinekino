---
id: adr:kinekino/20261029113508
title: 프레임 native 프로세스의 원 family 보유
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 프레임 native 프로세스의 원 family 보유

상태: private 후보에 구현 중. Windows 대상이며 전체 인수 전이다.

R47의 실제 후손 fixture에서2초 기한 호출이30.04초 뒤 반환했다. 부모만 종료하고 reader thread를 join하는 기존 helper로는 후손이 잡은 파이프의 종료를 보장할 수 없었다. 원 실패는 별도 보존한다.

기존 owned-process의 suspended 생성·Job 할당·resume·원 handle 관측·cleanup/receipt 계약을 소비한다. 공통 패키지나 다른 담당의 source owner 구현은 변경하지 않는다. 출력은 같은 패키지의 RawFile/PreparedEvidence로 받으며, stdout/stderr 상한을 별도로 검사한다. 이 전송에는 pipe EOF를 관측했다고 표시하지 않는다.

원 Supervisor는 새 private Session과 같은 SDK 호출 stack에 남는다. SDK execution/LocalHost의 기존 worker 보유가 이 호출을 계속 소유한다. 미완료 정리를 ApiError로 직렬화하며 owner를 버리거나 별도 registry·분리 thread로 옮기지 않는다. 정상 동기 FrameCapturePort는 원 family의 물리 종료와 최종 receipt 확인 후에만 반환한다.

작업 cutoff는 호출자가 전달한 원 Instant다. 준비/spawn 전에2초 cleanup reserve를 추가한 두 cutoff를 고정하고, 후속 단계는 연장하지 않는다. 출력 상한/관측 오류는 기존 cleanup cutoff를 단축해 동일 family를 종료한다. 생성 단계의 TimedOut도 Timeout으로 보존한다.

최종 receipt 기록은 자동 반복하지 않는다. 기록 실패는 원 오류와 owner를 pending으로 유지하며 완료/취소 ACK를 반환하지 않는다. 이 상태의 명시적 복구 경로는 아직 미구현이다. 동기 직접 호출도 pending 동안 반환하지 않으므로 이 조건의 전체 응답 기한을 보장하지 않는다.

실행 중 SDK 취소, 활성 요청 중 EOF, panic/receipt/termination fault 전수 및 비Windows 경로는 별도 미검증/미구현이다. 일반적인 오류 반환과 비상 Drop containment를 물리 종료 receipt로 혼동하지 않는다. 새 GUI·추론·다운로드·WARP·push를 재개하는 결정이 아니다.
