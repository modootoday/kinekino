---
id: plan:kinekino/20261029113833
title: Captured Card 제품 입력 연결
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Captured Card 제품 입력 연결

채택 root47d6650의 명시 entry·guarded adapter·원 ExecutionOwner를 그대로 소비하는 source-only 후속이다. 구현 후보는 runtime/development/ui-capture-product-surface-r1에 분리하며 Cargo/GUI 실행은 다음 PM 배정 이후다.

기존 CardProposalPanel의 앞면/뒷면·출처·명시적 채택 구성과 공용 Button/aw-inspector-footer 토큰을 재사용한다. 캡션·폰트·색상·여백의 새 규격은 만들지 않는다. 새 응답 행동은 공용 CaptureActions 선언으로 묶는다. TSX는 기존 규칙의 선언/변환 입력만 담당하고 상태·승인·SDK 호출은 Rust가 담당한다.

| 요소 | 동작·상태 | Rust 책임 | 검증 계획 |
|---|---|---|---|
| Card 앞면/뒷면 | 저장된 capture의 선택 index만 로컬 편집, 원 source_refs 유지 | CaptureCardModel | 다른 Card·원 capture 불변, 입력 바이트 한도 |
| 저장/명시 채택 | dirty 저장, clean 명시 채택; origin 발급·current inspect 금지 | 원 CaptureMutationAdapter | 실제 callback→원 SDK 요청, stale 거절 |
| 응답 적용/버리기 | 원 terminal에 대한 별도 명시 선택, 자동 반영 없음 | 원 adapter consume/discard | 결과 미소비 close 차단·decode fault 보존 |
| 결과 확인 후 처리 | 명시 선택 뒤 ACK/release는 원 owner가 수행; 실패 시 원 항목 유지와 도메인 재시도 표시 | 원 adapter ACK | 실제 기존 seam 가능 여부 추적, 없으면 미검증 |
| 재시도 | 미제출 원 요청 또는 보존된 accept payload만 재시도 | 원 adapter | payload 동일·중복 적용 없음 |
| Card mount | 기존 ProductFrameBlueprint·StudioFrame·실제 NativeDocumentHandle 재투영 | ViewerHost surface | 원 commit receipt·오래된 callback 거절 |
| route | 원 observer에 실제 commit 기대만 등록, 실제 frame 전 관측 완료 금지 | 기존 StartRouteObservation | adapter/host의 commit와 frame 완료 분리 |

같은 callback batch에서 텍스트는 FIFO로 처리하고 SDK/응답 행동 하나를 대기시키면 후속 입력을 막는다. pending 중 텍스트·새 mutation을 막고 결과 확인·버리기 같은 허용된 명시 선택만 받는다. ACK·operation·attachment는 제품 기본 버튼으로 노출하지 않는다. 큐/텍스트/보존 ACK 증거는 제한하며 초과 시 원 상태를 보존해 Capacity를 반환한다. ACK 증거는 부모 소유자가 명시 인수할 때까지 유지한다.

originNone과 기존 selection binding은 계속 read-only다. 기존 entry 설정을 사용한 trusted 호출과 제품 기본 활성화는 구분한다. 디코드·SDK·mount/route 오류를 성공으로 바꾸거나 자동 refresh·자동 discard·새 worker로 복구하지 않는다. 원 소유 close barrier는 유지한다.

시각 확대·1:1 비교와 실제 GUI 재검토는 보류 중이다. 본 후보의 타입/host 시험 통과를 전체13 화면 시각 완료로 표시하지 않는다. 다음 gate 전 소스·preimage·원 부모·선언 자산·시험 목록을 동결한다.
