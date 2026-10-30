---
id: sot:kinekino/native-intent-admission-backpressure
title: Native intent admission의 영구 거부와 backpressure
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Native intent admission의 영구 거부와 backpressure

상태: [bridge r2 실제 gate](../../../verification/presentation/native-window-bridge-2026-10-19.md)에서 선택19/전체46pass·failed0·ignored0·strict/fmt0을 확인했다. 초기/후속 queue의 budget 불일치 실패를 수리해 factory의 단일 private PanelBudget을 이전한다. r1/수리 전 r2 snapshot을 보존한다. 아래 '시험 준비'는 계약 작성 당시 표현이며 현재 headless 실행 근거와 실제 OS wake 미인수는 구별한다.

| 조건 | 결과·소유권 |
|---|---|
| 단건 UTF-8 bytes가 빈 큐의 max_text_bytes도 초과 | TextBudget→InvalidPayload→Rejected. drain으로 해결되지 않으므로 Capacity로 보존하지 않음 |
| 단건은 허용되나 기존 큐 합계와 합치면 초과 | TextCapacity→Immediate Busy→Capacity. 기존 큐를 변경하지 않고 runtime이 suffix를 보존 |
| 항목 수 포화 | 기존 Capacity→Immediate Busy→Capacity |
| foreign/stale node·handle, 서명/정책 오류 | 기존 검증 오류로 거부·보존. 포화로 바꾸거나 SDK를 실행하지 않음 |

validate_single_text를 request validation과 enqueue에 함께 사용해 실제 Core registry가 포화여도 단건 자체 초과를 먼저 거부한다. 누적 bytes는 enqueue 전 계산하고 실패 시 coalescing/entries를 변경하지 않는다. UI 전송 메모리 한도이며 저장 검증은 SDK가 담당한다. Rejected input은 runtime에 typed 원인을 보존하고 controlled props를 재투영한다.

## 같은 registry의 pending 예산

factory가 실제 PluginRegistry::new에 넣은 registry_pending 값 하나를 Mounted→Parts→Bridge로 이전한다. 생성 뒤 별도 limit를 주입하는 public 필드는 없다. Core limit setter가 없고 단일 UI thread·중간 await/외부 callback이 없는 고정 profile에서 동일 registry의 pending_count를 대조한다. 새 숫자 상수·더 큰 큐·SDK 정책은 만들지 않는다.

포화를 관측해도 dispatch 검증을 생략하지 않는다. HostCallbacks의 opaque handle/scope/signature·payload와 고정 QueueCallbackAdapter의 순수 validation을 거친다. Core registry가 adapter.dispatch/ticket/requestID 이전에 반환한 Err Busy를 **동일 pending budget 포화가 관측된 경우에만** Capacity로 바꾼다. 고정 producer는 PluginPayload::encode만 하고 validation은 큐를 읽을 뿐 SDK/intent enqueue를 실행하지 않는다. foreign/stale/잘못된 payload는 먼저 Rejected된다. 포화 아닌 Busy·requestID 고갈·다른 registry 오류는 Rejected로 유지하고 문자열을 해석하지 않는다.

future producer가 외부 코드를 호출하거나 registry limit 변경 API가 생기면 이 전제를 재검증한다. 범용 machine-readable Core admission API가 필요하면 Meari 담당과 협의한다. 이번 r2에서 공유 Meari source는 수정하지 않았다.

## liveness와 미실행 시험

registry 포화로 accepted0이면 consumer drain으로 예산이 풀리지 않는다. owner가 실제 terminal event를 처리한 뒤 NativeWindowWake::wake/Completion을 연결해야 한다. 자동 spin·SDK 재실행은 하지 않는다. r2 headless 시험은 CancelRequested 뒤에도 accepted0/SDK counter0, terminal Cancelled 뒤 명시 Completion에서 Save1회 admission→edit1/inspect1, 다음 Completion의 counter 불변을 검사하도록 작성했다. 실제 OS wake 연결 성공은 아니다.

누적 bytes는 서로 다른 field5000bytes 두 개/8192bytes 큐로 suffix 보존과 재입장을 확인하도록 준비했다. 같은 synthetic batch의 오래된 두 번째 field generation은 별도 Stale 거부이며 byte admission 실패·자동 저장 성공과 혼동하지 않는다. 정상 driver는 event마다 controller/reprojection 경계를 수행한다.

SDK 호출은 실제 제품 adapter의 ticket/context factory에서 capability를 관측한다. snapshot 불변만으로 호출0을 추론하지 않는다. 신규 시험은 모두 미실행이며 기술 Pending adapter·headless Completion과 실제 worker/OS/제품창·품질 인수는 구별한다.
