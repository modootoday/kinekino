---
id: sot:kinekino/receipt
title: 프레임 receipt 실패의 명시 복구 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 프레임 receipt 실패의 명시 복구 계약

후속 R59는 원 SDK의16세대소진을 attempt_budget_exhausted로 관측하고 마지막token의 추가요청을 ResourceExhausted로 거절한다. 실행계층16세대/거절/worker pending, 실제probe 2연속충돌의 새세대 명시재시도를 선택 인수했다. [근거](../../verification/application/receipt-recovery-budget-20261028.md). 소진후 관리복구·CLI/wire·부분쓰기/권한오류는 미완료다.

후속 R58은 원 SDK→file frame adapter→Session 명시 복구를 연결하고 실제probe 충돌/원guard 유지/재기록/첫EngineFailed 보존을 검증했다. [실제 근거](../../verification/application/frame-sdk-receipt-recovery-20261028.md)를 따르며 CLI/wire·16상한·연속 저장실패 등 남은 범위는 계속 미완료다. 아래 초기 미구현 설명은 원 설계 시점의 상태다.

후속 R57: 원 SDK 실행 계층의 typed pending 관측·세대 권한·명시 요청/단일 소비를 구현하고 synthetic client 선택 시험을 진행했다. 실제 frame adapter로의 port 전달·원 Session 재기록 연결은 아직 없으므로 제품 frame 복구 지원으로 표시하지 않는다. 구현/근거는 [SDK 복구 권한](../../verification/application/sdk-receipt-recovery-authority-20261028.md)을 따른다.

중앙 PM, 2026-10-28. 이 문서는 다음 구현의 요구 계약이며 공개 SDK의 현재 지원을 뜻하지 않는다. 현재 Session은 첫 receipt 실패 후 원 owner와 guard를 보유한 채 pending이며 복구 요청 API가 없다. 기존 [family 계약](frame-process-family.md)과 원 SDK ExecutionOwner/OperationId를 확장하되 별도 프로세스 registry·background detach·PID 재개방을 도입하지 않는다.

## 요구와 책임

|ID|계약|완료 근거|
|---|---|---|
|R0|최초 record 실패·원 오류·기존 목적지/부분 bytes 보존|원 목적지 충돌 후 무변경, 원 오류의 최종 envelope 유지|
|R1|physical_closed와 final_receipt_recorded를 독립 표시|실제 원 child exit/Empty라도 receipt 실패 중 terminal/ACK 없음|
|R2|반복 poll/취소/시간 경과는 재시도 권한 아님|명시 요청 전 FileSink sequence 증가·새 receipt 없음|
|R3|동일 원 operation만 명시 복구 요청 가능|opaque OperationId의 owner Arc/sequence를 검증, 다른 owner·ACK후·terminal·Queued 거절|
|R4|대기 상태의 generation에 바인딩된 한 번의 재시도|동일 generation 중복 요청은 같은 disposition, 실패시 새 generation 및 새 명시 요청 필요; 고정 상한과 초과 오류 정의|
|R5|원 SDK worker 안에서 동일 Supervisor·FileSink로 재기록|원 PID/creation/executable 불변, 기존 파일 덮어쓰기 없이 새 receipt 시퀀스에 기록|
|R6|복구 성공은 정리 완료이며 원 operation 성공 아님|최초 EngineFailed/Cancelled/Timeout 우선순위 유지, physical_closed/receipt_recorded만 갱신|
|R7|직렬화·리소스 상한·종료 의미|원 entry가 ACK까지 capacity 보유, 고정 크기 pending 관측/오류/요청 세대, close는 worker 미종료를 숨기지 않음|
|R8|저장 실패·부분 생성·종료 실패는 별개 복구|receipt 재기록으로 termination 재시도나 입력 작업 재실행을 시작하지 않음|

## 연결할 경계

SDK 실행 계층은 original entry의 bounded cleanup-pending observation과 요청 세대를 소유한다. 프레임 어댑터는 원 Session/Supervisor/FileSink와 첫 오류를 소유한다. 상호 연결은 cancellation과 분리된 application 계층 typed recovery port로 설계한다. review_source::MutationControl에 파일 경로나 프로세스 정책을 추가하지 않는다. FrameCapturePort legacy capture/단순 execute에는 복구 지원을 광고하지 않는다.

실행 핸들의 명시 요청은 보유 중인 해당 operation에만 전달한다. 외부 JSON의 PID/경로·환경 변수·파일 생성 신호를 권한으로 사용하지 않는다. wire 노출·CLI 명령은 typed SDK 계약 및 해당 오류/중복/외부 owner 시험을 먼저 인수한 뒤 별도 구현한다. 미연결 상태에서 capabilities에 지원으로 표시하지 않는다.

Session이 최초 receipt record 오류를 얻으면 물리 종료 관측과 원 오류의 bounded 요약을 SDK entry에 알리고 동일 호출 stack에서 기다린다. 명시 요청의 원 generation을 한 번 소비한 경우에만 기존 owner.record_cleanup을 같은 sink로 호출한다. 성공하면 실제 정리 사실을 확인한 뒤 원 첫 오류를 반환한다. 실패하면 기존 목적지/원 첫 오류를 보존하고 다음 대기 generation을 관측한다. 재시도는 작업 cutoff를 갱신하지 않는다.

## 검증 순서와 현 상태

첫 단계는 실제 Windows child에서 receipt 충돌→physical_closed/미기록→자동 재시도 없음→동일 owner의 명시 원 primitive 재기록→최초 오류 보존을 확인하는 것이다. 이 선택 시험은 공개 SDK의 R3/R4/R7을 증명하지 않는다. 다음 구현은 원 operation의 typed pending/명시 재시도·foreign/stale/duplicate/close 행렬이며, 그 뒤 실제 SDK frame receipt fault와 CLI 노출을 인수한다. 전체 제품 목표를 이 복구 하위 항목으로 축소하지 않는다.

root/source375 canonical·WIP/소유권·원 증거/실패·기존 보류를 유지한다. 새 GUI·추론·다운로드·WARP·push는 실행하지 않는다. modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100), code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)의 원 owner·실제 결과·지원 경계 구분을 적용한다.
