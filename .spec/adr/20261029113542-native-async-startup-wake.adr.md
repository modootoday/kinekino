---
id: adr:kinekino/20261029113542
title: "제안: 문서 생성 전 비동기 startup wake의 소유"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제안: 문서 생성 전 비동기 startup wake의 소유

상태: **Proposed·PM 우선 보완 방향 배정·공개 API/구현 미완료**, 2026-10-20. 최상위 창 backend 채택과 기존 런처 교체 결정은 [별도 ADR](native-top-level-window.md)을 유지한다. [제품 후속 gate](../../../plans/player/presentation/native-sdk-wake-gate.md), [범용 Native 경계 정본](../../../../packages/meari/.spec/evidence/reviews/plugin-pending-sdk-boundary-2026-10-20.md).

## 문제와 제약

제품 InitialPanelStartup은 실제 proposal scope가 생기기 전에 같은 SDK worker로 plain inspect를 수행한다. canonical snapshot·명시 선택 검증과 immutable raw/typed 준비 인수·초기 ACK를 마친 뒤 target document scope를 만든다. NativeWindowPreparation과 NativeWindowDriver는 document를 선요구하므로 현재 NativeWindowWake만으로 pre-document 초기 완료를 startup owner에 전달할 수 없다. 이 공백은 Meari 담당이 공개 source와 대조했다. dirty bit 보존은 알림 유실을 줄이는 상태일 뿐 초기 event loop를 깨우는 수단은 아니다.

UI 비차단, SDK 정책 단일 소유, 동일 worker/명시 선택/ACK 순서, 실제 owner 인계와 실패 회수, 같은 제품 event loop·공용 창으로의 연결을 유지해야 한다. SDK/UI/Native 입력은 현재 별도 담당 gate에서 동결·수리 중이다. 새 winit 버전·별도 JS runtime·가짜 proposal scope로 우회하지 않는다.

## 대안

| 방향 | 이점 | 비용·제약 | 판단 |
|---|---|---|---|
| Meari 범용 pre-document startup wake port | 기존 Native loop 소유와 실제 attach 인계를 한 라이브러리에서 관리. 제품 SDK 타입을 모르면서 제품의 비차단 준비를 지원 | 범용 startup owner·취소/폐기·attach 실패 회수·늦은 신호 거부·재관측 수명 검증이 추가로 필요 | PM이 우선 제안·보완 방향으로 배정 |
| 제품 infrastructure의 별도 event-loop adapter | 기존 공개 Native API를 변경하지 않을 여지 | 현재 root와 Native의 pinned event-loop 타입 차이, 접근 가능한 proxy·startup 위임·attach 이후 이벤트 라우팅을 별도 증명해야 함 | 우선 경로가 실측상 불가능할 때 재검토. 독립 loop를 임의 추가하지 않음 |
| 실제 loading-root를 먼저 mount하고 target으로 전환 | 사용자에게 로딩 화면을 먼저 표시할 수 있음 | target proposal scope와 별도 root 수명·문서 교체/라우팅/실패 복구가 필요하며 현재 해당 계약의 인수 근거가 없음 | 현재 공백을 감추는 임시 조립으로 채택하지 않음. 전체 제품 loading UX 요구는 유지 |
| UI에서 inspect/join을 동기로 기다리거나 가짜 scope 생성 | 겉으로 조립 순서를 맞추기 쉬움 | 비차단·실제 scope·초기 ACK 인수 요구를 위반 | 제외 |

## 우선 방향과 책임

Meari가 SDK를 모르는 범용 pre-document wake/owner 인계 포트를 제안하고 공개 수명·실패 의미를 검증한다. 정확한 타입/API는 Meari 문서가 소유한다. 제품은 확정된 포트를 infrastructure의 CompletionNotifier adapter와 assembly의 startup owner에 연결한다. presentation은 canonical 준비 상태와 사용자 입력을 표시하고 플랫폼 event loop를 직접 제어하지 않는다.

제품 startup owner가 같은 ExecutionOwner와 immutable 준비 상태를 소유한다. initial terminal 수용·ACK·proposal mount/registration·actual Native attach가 각각 다른 checkpoint다. attach에 실패하면 아직 인계하지 않은 owner 또는 회수 가능한 동일 partial owner를 유지하고, 성공한 단계나 SDK inspect를 다시 실행하지 않는다. Native attach가 SDK terminal/품질 ACK를 대신하지 않는다.

startup wake는 notification만 전달한다. SDK terminal 저장·상관 검증·ACK·저장 정책은 기존 SDK/adapter/UI-thread drain에 남긴다. 동시 notify/bind/dirty clear에서 알림이 사라지지 않아야 하고, 인계 완료 후 오래된 startup signal이 다른 document/owner를 깨우거나 권한을 재생성하지 않아야 한다. startup 실패·취소 시에도 실제 terminal 수용과 최종 worker join을 확인할 재관측 경로가 필요하다. 이를 무조건 자동 ACK/즉시 wake 반복으로 구현하지 않는다.

## 비용과 재검토 조건

범용 라이브러리 보완과 제품 consumer 재검증 비용을 수용한다. 기존83은 이전 입력의 증거로 보존하고 새 SDK/Native pin에서 회귀·실제 startup/OS gate를 다시 인수한다. UI thread block·가짜 scope·정책 복제보다 이 경로가 원래 전체 제품 목표와 일치한다는 설계 판단이며 성능 우위 측정은 아니다.

범용 port가 동일 loop·정확 owner 인계·실패 회수·늦은 신호 거부를 충족하지 못하거나 새 필수 dependency/loop 중복이 요구되면 PM·Meari·UI가 제품 infrastructure 대안을 다시 대조한다. 기존 winit/manifest/기본 런처는 검증 전 변경하지 않는다. NW01~08과 기존 전체13화면 범위를 축소하지 않는다.

## 2026-10-20 소비 gate 결정의 구체화

Native 최종 strict startup/attach_observed를 제품 UI가 소비하고 SDK가 얇은 notifier와 owner bundle을 제공했다. 제품의 두 번째 mailbox·loop·SDK worker는 추가하지 않았다. doc0 및 mounted bridge 모두 typed close를 관측하며 worker-exit만 제한된 After, terminal ACK 대기는 OnWake, 원 fault 진단은 명시 복구로 유지한다. 실제 Ready와 render-time HostStamp가 있는 checked submission을 attached hook과 구별한다.

PM이 승인한 trusted notifier wrapper는 생성 시 한 번 적용하고 시작 실패 retry에서도 같은 Arc를 보존한다. UI trusted commit port는 construction-time apply(export) 경계이며 기본 Native commit에 그대로 위임한다. 시험에서 성공 이전 거절을 주입해 projection/terminal 소유와 명시 retry를 검증한다. 둘 다 직렬화·외부 URL·전역 late setter로 노출하지 않는다. 범용 Native/Core나 SDK 정책의 테스트 대체를 만들지 않는다.

최종 근거와 실패 이력은 [실제 SDK→Native 검증](../../../sots/verification/presentation/native-pending-window-sdk-2026-10-20.md)에 둔다. 소비 gate 성공은 root 제품 등록/기본 런처 채택이나 시각13화면 승인으로 확대하지 않는다.
