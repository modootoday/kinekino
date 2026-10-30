---
id: sot:kinekino/transport-diagnostics
title: 로컬 전송 오류 원인과 선택적 진단
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 로컬 전송 오류 원인과 선택적 진단

2026-10-19 구현 계약. [상위 런타임 목표](local-runtime.md)와 [실제 검증](../../../../evidence/player/local-host-parallel-diagnostics-2026-10-19.md)을 연결한다. registry·다중 host·durable jobs의 구현을 뜻하지 않는다.

## 클라이언트 오류

기존 error.code/exit/delivery/retryable/automatic_retry 계약을 유지하며 error.details.diagnostic에 정적 phase/operation과 nullable win32_error를 추가한다. 요청·endpoint·SID·토큰·peer profile 값은 반환하지 않는다. 성공 SDK 응답은 수정하지 않는다.

`GetNamedPipeServerProcessId`가 `ERROR_PIPE_NOT_CONNECTED(233)`를 반환한 **전송 전** 조합만 engine_failed(exit8)/delivery=not_sent/retryable=false다. 다른 operation의233을 일반화하지 않는다. PID/시작 시각/peer 정책 불일치와 실제 접근 거부는 계속 permission_denied(exit6)다. query가 실패해도 peer를 신뢰하거나 보안 검사를 생략하지 않는다.

프레임 쓰기 시작 이후 read/write 실패는 실행 여부를 확정할 수 없어 execution_unknown이며 자동 재시도/fallback하지 않는다. client deadline 만료는 timeout(exit10)이며 시작 신호를 기다리는 테스트 watchdog과 구별한다. server session deadline과 client deadline의 시작 범위는 다르며 값은 이번 수정에서 변경하지 않았다.

## 호스트 관측

기본 LocalHost::start는 observer를 만들지 않는다. 명시적 `LocalHost::start_observed(client, options, TransportDiagnostics)`만 최대1024개의 in-memory 이벤트를 보존한다. monotonic elapsed_us, 정적 phase/operation, nullable error_kind/win32_error/check_operation을 기록한다. 파일/네트워크 sink나 전역 환경 설정은 없다. snapshot/wait_for는 소유 fixture/진단 caller가 사용한다.

연결·길이/본문 읽기·message verification·dispatch·응답·ACK·세션 종료·다음 pipe 준비·stop을 구별한다. session_timeout 관측과 연결된 조회 오류가 없는 옛 exit6만으로 세션 만료를 확정하지 않는다. 이 진단은 작업 완료나 정상 GUI/모델 품질 승인으로 사용하지 않는다.
