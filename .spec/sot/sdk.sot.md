---
id: sot:kinekino/sdk
title: 모델 파일 검증의 SDK 실행 취소
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 모델 파일 검증의 SDK 실행 취소

공통 `ApplicationClient::execute_controlled(request, control)`은 참여하는 유스케이스에 취소 checkpoint를 전달하는 Rust port다. 기본 구현은 기존 execute를 호출하므로 다른 client/endpoint나 모든 명령의 실행 중 취소를 암묵 지원하지 않는다. `PlayerSdk`의 models.verify 경로가 외부 control을 전달한다. 다른 참여 유스케이스는 공통 SDK execution 계약과 각 기능 계약에서 별도로 명시한다. 기존 원 Request·응답 envelope·저장/발행 정책은 유지한다.

SDK execution worker는 원 Shared와 원 operation sequence에 결합된 control을 호출 동안 보유한다. ExecutionHandle.cancel의 기존 원 owner 검증·entry.cancelling을 그대로 사용한다. 별도 timer/observer thread나 전역 취소 플래그를 만들지 않는다. 참여하지 않는 동작의 실제 성공/오류를 취소 응답으로 다시 쓰지 않는다.

models.verify는 원 timeout Instant deadline과 외부 control을 합성한다. 각 checkpoint에서 원 operation의 취소를 먼저 보고 기존 deadline을 확인한다. LocalModelCatalog의 원 파일 guard·64KiB 읽기 checkpoint가 그대로 사용되며 취소는 정상 scope의 handle/slot 해제와 Cancelled 결과로 끝난다. 원 manifest와 모델 bytes를 수정하지 않고 runtime/품질 admission을 만들지 않는다. 읽기 자체의 hard preemption이나 즉시 취소 지연 상한을 보장하지 않는다.

실행 중 cancel 반환은 여전히 Requested다. 실제 leaf가 Cancelled를 반환한 terminal을 읽어야 취소 완료를 확인할 수 있다. terminal 뒤 취소는 원 response를 바꾸지 않고 요청 사실만 보존한다. queued cancellation·terminal ACK·원 worker join/close 계약은 [SDK execution](../api/sdk-execution.md)을 따른다. close는 실행 중 작업의 실제 종료를 기다리며 자동으로 running cancel을 요청하지 않는다.

이 기능은 같은 프로세스의 원 SDK execution에 한정한다. CLI/local-host transport의 원격 취소 ACK, 다른 명령의 중단, 추론/모델 설치·삭제, 강제 선점은 지원 범위가 아니다. capability는 in_process_sdk_execution_cooperative_cancel과 wire_cancel_ack=false/hard_preemption=false를 분리한다. 모델 검증의 기존 직접 control/deadline 시험과 실행기 전달 시험을 함께 보며 합성 gated port를 실제 모델 해시 읽기로 계상하지 않는다.
