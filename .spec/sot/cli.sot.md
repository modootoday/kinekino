---
id: sot:kinekino/cli
title: CLI 전송 기한
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# CLI 전송 기한

`player-cli --endpoint ENDPOINT --transport-timeout-ms MS`는 명시적 Windows 로컬 host 호출 한 건의 전송 기한을 설정한다. `--input FILE|-`와 함께 사용할 수 있다. 생략시 기존5000ms, 명시값은 숫자로만 된10..30000ms다. endpoint 없는 사용·범위 밖·부호/지수/overflow·중복·누락은 요청 전송 전에 InvalidArgument다. 기존 EndpointClient의 TransportOptions 허용 범위를 확장하지 않는다.

이 값은 SDK Request.context.timeout_ms와 독립이며 JSON payload/operation 정책을 변경하지 않는다. 전송 후 기한/연결 실패의 execution_unknown·자동 재시도 없음·fallback 없음은 유지한다. 클라이언트 timeout은 host mutation 취소나 rollback의 증거가 아니다. 재시도는 명령별 기존 idempotency/CAS 계약에 따라 명시한다.

host의 세션 기한도 별도로 존재한다. R79 review host는 기본5000ms여서 client30000ms만으로 장시간 응답이 보장되지 않았다. R80 구현은 고정SHA review-host JSON의 선택 필드 `transport_timeout_ms`를 기존 TransportOptions에 연결한다. 생략5000ms, 명시10..30000ms이며 도구/producer 실행 전에 범위를 거절한다. 필드값은 설정SHA 보호 대상이며 작업·시작·재생·정리 기한을 변경하지 않는다. 장시간 응답에는 client와host 양쪽을 명시해야 한다. 실제 실행 인수는 별도 증거를 따른다.

실제 표본 시험은 별도 opt-in 경로로30000ms를 사용하고 그 CLI 프로세스의 supervisor를35000ms, 결과 수신을36000ms로 설정한다. 기존 시험의 기본15000/16000ms는 유지한다. 전체 host/producer/playback 기한과 종료 정리 증거는 각각 별도로 검증한다.

구현 책임은 CLI runner/transport_timeout.rs의 입력 해석과 기존 infrastructure/local_host/EndpointClient다. application/domain의 기능 정책을 CLI에 복제하지 않는다. R78 첫 실패와 기존 raw는 [검증 기록](../../verification/application/review-matroska-cli-first-failure-20261028.md)에 유지한다.
