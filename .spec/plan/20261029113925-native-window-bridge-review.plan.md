---
id: plan:kinekino/20261029113925
title: NativeWindowHost 소비 경계 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# NativeWindowHost 소비 경계 검토

2026-10-19. Meari 담당의 읽기 검토와 PM 소스 대조에 따른 수리 후보다. 실행 재현·수리 검증 전이며 [bridge 실행 계획](native-window-bridge-gate.md)의 준비본 r1을 보존한다.

## 일시 포화와 영구 거부

| 경계 | 읽기로 확인한 상태 | 필요한 검증 |
| --- | --- | --- |
| registry pending 예산 | 한도 초과는 ticket 생성·request ID 증가·adapter dispatch 전에 Busy. 현재 bridge는 바깥 오류를 Rejected로 소비한다. ID 고갈도 Busy이므로 일괄 재시도하거나 메시지 문자열로 분기하지 않는다. | 확정된 일시 포화만 suffix로 보존. 실제 terminal 전 SDK 효과 0, terminal 후 명시 Completion에서 정확히 한 번 입장. |
| text 합산 예산 | 개별 문자열 자체 초과와 기존 큐 합산 초과를 TextBudget 하나로 반환한다. 둘 다 큐 변경 전 반환한다. | 서로 다른 필드 5000바이트씩/예산8192에서 누적 포화만 Capacity. 단건 자체 초과는 영구 거부. 상한 확대나 SDK 정책 변경 불필요. |

제품 adapter/queue/시험 수리는 UI 소유다. 공개 machine-readable 구분이 필요하면 Meari 소유 변경을 먼저 조율한다. 관측된 후보를 이유로 모든 Busy를 Capacity로 바꾸지 않는다.

PM은 UI가 제안한 최소안을 승인했다. 실제 registry 생성에 사용한 immutable registry_pending을 Mounted→Parts→Bridge로 함께 이전해 동일 registry의 pending_count를 preflight한다. setter 없음·동일 UI thread·중간 await/외부 callback 없음이 전제다. 다른 예산을 별도로 주입하거나 오류 문자열로 판별하지 않는다. shared API 수정은 하지 않으며 UI r2 준비본/선언 시험으로 보존한다. 이 선택은 구현·실행 통과를 뜻하지 않는다.

## 재개 신호

처음부터 registry가 가득 차 accepted0이면 driver는 자동 반복하지 않으며 입력도 차단된다. 외부 parent owner의 실제 terminal 처리 뒤 NativeWindowWake::wake를 통해 Completion을 보내는 연결이 필요하다. 이 연결 없이 Capacity 분기만 추가한 결과를 실제 창의 진행 보장으로 표시하지 않는다. byte 합산 포화에서 첫 요청이 수락된 경우에는 기존 drain 이후 driver wake 경로를 소비한다.

## 기존 검증의 범위

예산 단일 소유 수리 뒤 선택 재실행23050ms/exit0에서 factory/bridge17개와 panel2개가 통과한 원로그를 PM이 확인했다. 이후 재투영 예산 불변 검증을 보강한 입력으로 전체 회귀가 실행 중이다. 선택 성공과 최종 입력 전체 성공은 별도로 인수한다.

최초 선택 실행75719ms/exit101에서 factory17개 중15pass·2fail, panel은 fail-fast로 미실행이었다. 실제 factory의 초기 예산은8/8192인데 bridge 별도 인자의1/4096은 후속 재투영부터 적용돼 초기 큐와 달랐다. PM과 UI가 소스를 대조했고, factory가 사용한 PanelBudget을 단일 소유로 이전해 별도 bridge 인자를 없애는 최소 수리를 승인했다. 기대값 완화 없이 원소스/실패를 보존하고 선택 --no-fail-fast부터 다시 인수한다. 아직 수리 검증 전이다.

동기 stage/apply/session commit의 receipt, 단일 factory owner 이전, suffix가 남을 때 export 교체 보류, 단계별 close checkpoint 및 ForcedIncomplete는 공개 API 의미와 부합한다. 신규7개는 아직 미실행이다. Completion 뒤 저장 snapshot 불변만으로 SDK 호출 자체 0을 입증하지 않으므로 가능하면 기존 SDK 경계의 호출 계수/receipt를 확인한다. 정상 Immediate adapter와 방어용 unexpected Pending, parent Pending, runtime Drop은 서로 다른 경계다.

추가 cargo/GUI는 SDK 오디오 슬롯 반환 후 조율한다. 실제 OS 입력·시각 일치·제품 root 등록·fresh CLI/project/provenance의 잔여는 유지한다.
