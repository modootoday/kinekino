---
id: sot:kinekino/endpointclient
title: EndpointClient의 전송 전 취소 경계
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# EndpointClient의 전송 전 취소 경계

현재 LocalHost wire는 단일 Request/Response이며 원격 operation ID·retained terminal 조회·cancel/ACK 프로토콜을 제공하지 않는다. 연결 종료/클라이언트 취소를 서버 실행 취소로 해석하지 않는다. 원 server Executor의 accepted/running/completed 및 close 정책은 유지한다.

EndpointClient의 ApplicationClient.execute_controlled는 같은 프로세스 SDK execution의 원 control을 전송 전 checkpoint에 연결한다. 공통 trait의 control은 Sync이고, 기존 전용 transport thread를 scoped spawn/join하여 호출 동안만 빌린다. 별도 observer thread나 새 polling timer·store·job을 만들지 않는다. PlayerSdk의 직접 usecase control은 기존 별도 타입 계약을 유지한다.

순서는 caller identity 검사·Request 검증 → control checkpoint → bounded encode → 기존 pipe open/retry 루프 checkpoint → 기존 server identity 검증 → 마지막 pre-write control checkpoint → transmitted=true → frame write/read/기존 acknowledgement다. 기존 busy pipe의5ms retry 대기와 transport deadline을 그대로 사용한다. 관측된 pre-write 취소는 Cancelled/delivery=not_sent/automatic_retry=false다. 첫 frame byte를 쓰기 전이라도 마지막 checkpoint를 이미 지난 경합은 전송 가능 범위로 보수적으로 취급한다. 즉시 중단이나 hard preemption을 보장하지 않는다.

transmitted 이후에는 control을 확인하지 않는다. 실제 server 응답을 보존하고 송수신 실패/기한 만료는 기존 execution_unknown을 유지한다. 요청을 재전송하거나 늦은 취소를 거짓 Cancelled 성공으로 바꾸지 않는다. SDK execution의 cancel 반환은 Requested이며, 실제 terminal과 delivery 상태를 조회해야 한다. queued_cancelled=false도 server 작업 시작의 증거가 아니다.

이 동작은 원 EndpointClient를 controlled 호출한 SDK 경로다. 일반 CLI --endpoint 실행은 기존 one-shot execute를 유지하며 새 Ctrl-C/원격 cancel command나 server cancel ACK를 제공하지 않는다. 원격 실행 중 취소를 완성하려면 원 owner에 결합된 제출/조회/취소/ACK·결과 보존·유실 재조회와 권한/버전 계약이 필요하다. request_id만으로 중복 제거/취소 권한을 발명하지 않는다.
