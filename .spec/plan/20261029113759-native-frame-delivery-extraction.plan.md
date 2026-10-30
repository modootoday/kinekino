---
id: plan:kinekino/20261029113759
title: Frame delivery 책임 분리 — 별도 checkpoint 제안
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Frame delivery 책임 분리 — 별도 checkpoint 제안

2026-10-21. 읽기 계획이며 코드·실행 배정0. V1 source10에 합성하지 않는다. 현재 frame_delivery.rs 약727행은 계약·mailbox·Flow·Native backend·typed consumer를 함께 담고 있다.

제안 경계는 contract.rs(request/result/error 데이터), mailbox.rs(move-only reader와 bounded publication), flow.rs(clock/budget/replay/phase 및 private backend port), native_backend.rs(원 registrar/checkpoint/proof adapter)다. 기존 frame_delivery.rs는 module 조립과 typed consumer bind 및 좁은 재공개를 맡긴다. tests.rs의 원 Flow14와 Viewer4는 같은 이름·단언을 유지한다. 정확 경로는 root1+신규4+기존 tests1 최대6 후보이며 source 배정 전 최종 가시성/필수 import를 읽어 좁힌다.

분리 자체는 의미 불변이어야 한다. deadline 전후 clock·publication replay·Busy owner 보존·Weak observer·original failure carrier·same-turn suffix·고정request와 budget을 바꾸지 않는다. private 가시성은 필요한 내부 sibling으로만 한정하고 Native proof/backend를 외부 API로 승격하지 않는다. 기존 dead_code 유예26의 개수를 유지하기 위한 억제나 새 module-wide allow를 두지 않는다. 실제 V2 설치 후 미사용 API가 사용될 때 제거할 대상은 별도 추적한다.

인수는 exact before/source/diff 및 독립 읽기→양strict/fmt→원 Flow14/Viewer4와 실제 영향 회귀로 한다. 필요 분모는 당시 최종 root에서 재산정한다. 생산 설치·새 request 정책·V2 cleanup carrier·main 전환·새 OS 시험을 이 추출과 섞지 않는다. V1 및 SDK CLI 최종 인수 뒤 별도 PM checkpoint가 선행한다.

## 읽기에서 확인한 이동 경계

| 대상 | 옮길 기존 책임 | 의존·보존 경계 |
|---|---|---|
| contract.rs | FrameRequest/Rejection/ResultState/OwnedResult/ReceiveError/Failure 및 원 오류 source | Native 오류 타입 참조를 유지한다. domain 독립 계약으로 과장하지 않으며 lease/proof 필드는 추가하지 않는다 |
| mailbox.rs | Mailbox/FrameReceiver/FrameHostUpdate/same_source·Completion read | move-only reader, 원 deadline, foreign/old-turn 거절 시 데이터 보존, late Pending redraw 취소/historical 완료 허용 유지 |
| flow.rs | FrameContext/Progress/FrameBackend/Phase/Flow와 publish/flush/notify/fail/observe, test observe_at | 원 backend port와 clock 인자, publication-before-context·processed replay·once redraw·distinct budget 순서를 그대로 유지 |
| native_backend.rs | NativeBackend/within_deadline/consume_checked 및 FrameBackend 실제 구현 | 원 checkpoint/원 registrar·registration failure를 보유. advance/verify 전후 clock 위치와 실제 checked proof callback 수명 유지 |
| frame_delivery.rs | 기존 동일 공개 경로의 좁은 re-export, NoMediaFrameConsumer 생성/조회/bind/observe, 모듈 조립 | caller의 FrameRequest/Receiver import와 typed QuarantineChild hook을 바꾸지 않음. 원 origin 검사 후 actual binding 이동 순서 유지 |
| tests.rs | 원14 test의 import만 필요한 경우 조정 | private 접근을 위해 실제 검증을 줄이거나 assertion/fixture/시간을 바꾸지 않음. Viewer4 파일은 편집 범위 밖이며 기존 경로로 재사용 |

현재 root consumer는 Flow 및 NativeBackend의 field를 직접 조립/조회한다. 최소 추출에서는 필요한 field/type만 frame_delivery 내부 parent/sibling 가시성으로 제한하고 외부 pub(crate) API를 늘리지 않는 안을 먼저 대조한다. setter·새 초기화 순서·새 observer/executor로 바꾸는 리팩터링은 이번 의미 불변 추출과 분리한다. tests의 `use super::*`가 보던 항목은 내부 import로 유지하며 Native proof를 테스트 생성 가능 타입으로 노출하지 않는다.

mailbox 발행 코드의 일부가 현재 Flow에 있다는 이유로 새 publication 정책을 도입하지 않는다. 최초 checkpoint에서는 데이터/reader를 mailbox로 옮기고 기존 Flow flush 알고리즘은 원문 순서를 유지한다. 추출 뒤 책임 경계가 충분한지 읽기 인수에서 판단하고, 추가 API 분리는 별도 설계·증거가 있을 때만 배정한다. 원 cfg(test) reflection_channel_for_test의 owned capture `+ use<>`도 유지한다.

검증 후보는 Flow14와 기존 Viewer4의 정확 이름18, 양 all-target strict/fmt다. 이때 root actor/compiler의 generic binding 영향을 입증할 기존 shell 영향 필터가 필요하면 당시 실제 등록 이름을 확인해 최소 범위를 추가한다. 기존 ownwindow OS 성공을 재실행하거나 추출의 OS 성공으로 합산하지 않는다. 현재 SDK B가 soleheavy/shared이며 이 계획의 source 추출·Cargo·OS는 모두0이다.
