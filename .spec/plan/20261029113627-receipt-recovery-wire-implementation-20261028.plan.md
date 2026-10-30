---
id: plan:kinekino/20261029113627
title: 원 SDK 실행의 CLI receipt 복구 연결
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 SDK 실행의 CLI receipt 복구 연결

2026-10-28 중앙 PM. R82 이후 다음 구현 배정이다. 기존 기능 소유권·WIP·원 실패/증거·private overlay를 유지한다. 이 문서는 구현 계획이며 CLI 지원 또는 검증 완료 선언이 아니다. 배포·추론·다운로드·WARP·수동GUI·push를 재개하지 않는다.

## 현재 소스에서 확인한 공백

|근거|현재 사실|구현에 미치는 영향|
|---|---|---|
|src/infrastructure/local_host/executor.rs SHA AF32F7265433E25E02C873EF3D739C16F127AB2877B2FD709213952064EFEA43|직렬 worker는 client.execute를 직접 호출하며 Accepted/Running 중 새 SDK admission을 거절한다|이 경로에는 ExecutionOwner의 recoverable port가 없다. 같은 worker에 retry 명령을 넣으면 원 호출이 끝날 때까지 소비하지 못한다|
|src/infrastructure/local_host/server.rs SHA F2C51FA6700B85991821D8E968E9BF29A30954708F25E813F111780307827C3D|연결별 serve 완료/기한 뒤 다음 pipe 인스턴스를 만든다|기존 endpoint에 제어 명령 이름만 추가해서는 running 요청과 동시에 복구를 전달할 수 없다|
|src/application/sdk_execution/recovery.rs SHA C906F65AB55D7FB87F8BE0456CEEB175723153EDC94F19A6A0D76047543BAC2A|원 owner/operation·세대별 Arc 토큰·한 번 소비·16세대 상한이 구현돼 있다|JSON의 PID/경로/숫자 generation으로 토큰을 재구성하면 안 된다|
|sdk_execution/attachment.rs 및 types.rs|exclusive attachment가 submit/peek/cancel/recovery/retry/ACK 권한을 소유한다. OperationId는 Serialize가 없다|host가 실제 attachment와 opaque ID/token을 보유하고 외부 식별자를 해당 보유 권한으로만 해석해야 한다|

모두 runtime/development/source-sdk-integration-r1의 현재 R82 소스다. 원 SDK 복구 성공과 CLI/wire 부재를 구분한다. 원 [복구 계약](../../../sots/contracts/production/frame-receipt-recovery.md)의R0~R8을 변경하지 않는다.

## 구현 경계와 순서

1. 공통 application 계층에 bounded host 실행 소비자를 둔다. 한 ExecutionOwner와 exclusive attachment를 보유하고 submit/peek/cancel/receipt 상태/명시 retry/terminal ACK/close를 위임한다. 새 작업 정책·프로세스 레지스트리·PID 재개방을 만들지 않는다. 첫 구현은 기존 SDK 실행 budget보다 큰 저장소를 만들지 않으며 entry는 ACK까지 보유한다.
2. 외부 operation 참조는 host scope와 host가 발급한 식별자다. host는 이를 자신의 보유 OperationId로만 해석한다. scope 불일치·ACK 후·unknown·terminal·queued retry는 기존 typed SDK 오류에 맞춰 거절한다. 원 Arc 토큰을 직렬화하지 않는다. pending 세대를 관측할 때 host가 보유한 정확한 snapshot/token과 외부 기대 세대를 대조해 retry한다. 원 세대가 바뀌면 stale 요청을 새 토큰으로 자동 갱신하지 않는다.
3. infrastructure는 별도 명시 실행 host를 조립한다. 짧은 submit/관측/제어 RPC만 처리하고 실제 작업은 원 ExecutionOwner worker에서 실행한다. 각 RPC가 즉시 반환하는 방식이면 기존 직렬 pipe에서 running SDK 호출을 기다리는 교착을 피할 수 있다. 기존 동기 endpoint/host를 일괄 교체하거나 보유 작업을 다른 host로 옮기지 않는다. 별도 endpoint는 현재 사용자 인증·server identity·framing/크기 제한을 그대로 재사용한다.
4. CLI는 명시 host endpoint로 위 RPC를 호출한다. 일반 capture 재호출을 복구로 바꾸지 않는다. submit 응답은 admission, retry 응답은 요청 수락, peek terminal은 원 결과이며 서로 구분한다. stdin EOF는 새 admission을 닫고 원 pending owner를 보유한 정리 관측으로 들어간다. receipt 미해결 상태에서 성공 종료/ACK를 만들지 않는다.
5. 발견/schema/typed client를 연결하는 시점은 위 권한·수명 시험과 실제 SDK receipt 충돌 시험 후다. legacy execute·기본 CLI·미구성 host에는 지원을 광고하지 않는다.

최초 검토의 '제어용 두 번째 pipe'는 필수 조건이 아니다. 실제 작업을 RPC 내부에서 기다리지 않는 submit/peek 모델을 선택한다. 이 선택은 동일 프로세스의 원 ExecutionOwner를 유지하며 기존 host 보안·직렬 admission을 재사용한다. 구현 전에 기다리는 동기 요청을 섞지 않는지 검토한다.

## 요구사항 추적표

|ID|요구|현재 증거|다음 완료 근거|
|---|---|---|---|
|W0|원 작업과 복구 RPC의 진행 분리|위 server/executor 소스에서 미연결 확인|작업 running 중 상태 조회/retry를 실제 처리, 무관한 새 worker로 작업을 재실행하지 않음|
|W1|원 owner/attachment/operation 보존|R57~R59 typed SDK 근거|host 간/ACK 후/queued/terminal/unknown 참조 거절 및 원 owner 보유 시험|
|W2|정확한 pending generation·중복 의미|R59 16세대 SDK 근거|동일 retry 중복·이전 세대·새 세대 자동 retry 없음·상한 관측 wire 시험|
|W3|복구 후 원 첫 오류 보존|R58 실제 frame SDK 근거|실제 정상 CLI→원 SDK frame 충돌→명시 retry→원 EngineFailed terminal·물리 정리/receipt 기록 분리|
|W4|transmission 불확정·retry 구분|R79~R82 client/host 기한 근거|전송 전 거절/전송 후 execution_unknown·중복 요청·조회 복구 시험|
|W5|종료·ACK·capacity 유지|기존 SDK close 계약|미해결 EOF·미ACK terminal·close pending·재시작 foreign scope 및 최종 owner join 근거|
|W6|지원 표시·구성 분리|현재 CLI 복구 미지원|명시 host schema/capabilities와실제 기능시험 일치; 기본/legacy 미지원 유지|

우선 구현은W0/W1의공통 소비자와합성 recovering client 시험이다. 실제 frame 재실행·CLI 기능완료로확대하지않는다. 그뒤 W2~W5·CLI 실제 충돌·회귀를 중앙 heavy1로 직렬 배정한다. 소진후관리복구·부분쓰기/권한fault는 별도 미완료이며 이 기능으로 자동 해결되지 않는다.

관련 현황: [CLI 제품 여정](../cli-product-journeys-20261028.md), [목표표](../product-goal-status-20261028.md), [R82 CLI 수명](../../../sots/verification/application/review-mkv-cli-lifecycle-20261028.md). 적용 modootoday/software-qa/test-fixture-and-mock-build A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100 및 code-change-review-verification 9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B: 소스 관측/요구/실제 인수 경계를 분리했다.
