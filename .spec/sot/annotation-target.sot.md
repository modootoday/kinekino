---
id: sot:kinekino/annotation-target
title: 원 annotation과 실제 제출 target
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 원 annotation과 실제 제출 target

상태·인수 한계는 [C0 index](index.md)를 따른다. 기존 Rust export record에 wire metadata를 소급 재구성하지 않는다. v1 metadata를 admission 전에 보존한 row만 공식 wire 목록/참조에 노출하며, metadata 없는 legacy Rust row는 wire 관점에서 NotFound다. 같은 원64 예산은 공유한다.

| 원 값 | 발급·보존 책임 |
|---|---|
| native issuer/engine/resolution identity | SDK 원 scope·관측 grant·원 ticket. caller 문자열 복사로 대체0 |
| skip registry generation/session/pass/segment | 기존 host registry가 같은 호출의 원 상태에서 검증. native issuer generation과 별도 타입 |
| request_revision | submit 당시 session expected_revision. annotation revision으로 복사0 |
| annotation_revision/verified_revision | 선택된 원 ManualSegment.revision/verified_revision. admitted v1은 nonzero/current annotation 확인값을 보존 |
| timebase | 기존 SegmentTimebase::EnginePlayback, wire `engine_playback`. source PTS/timebase를 만들지 않음 |
| actual_target_us | 원 SDK가 검증해 실제 SubmitInput에 넣는 Seek.position_us에서 stamp. client JSON/나중 segment 조회로 생성0 |

선택 metadata getter는 원 skip session 상태 잠금·revision 확인과 같은 호출에서 선택1개만 복사한다. 전체 snapshot/segments Vec를 export용으로 clone하지 않는다. 기존 evaluate/attempted 예약·cancel/override 정책을 재구현하지 않는다. not_submitted/cancelled에 존재하지 않는 target·verified annotation을 만들지 않는다. 준비 metadata의 형상 검증은 실제 제출 경로에서 수행해 기존 거절/취소 순서를 바꾸지 않는다.

SDK의 새 additive `PreparedNativeResultExportV1`은 기존 준비값과 원 annotation/timebase를 결속한다. 기존 공개 ExportGuardedSeek literal/기존 Rust API 필드를 끼워 바꾸지 않고 별도 v1 입력·메서드를 추가한다. 실제 target/engine/resolution은 SDK 원 grant에서 생성한다. issuer가 packet.position_us를 나중 closure에서 덮어쓰는 현재 순서를 v1에서는 쓰지 않는다. Registry가 받은 같은 typed target을 row metadata와 원 SubmitInput 양쪽에 넣고 enqueue 전에 일치를 확인한다.

원 submission/read cell1회, 동일 id의 grant→command 전환, context 사전결속·send 실패 정리를 유지한다. nonce/metadata 준비·모든 fallible 검증·bounded 할당은 try_send 전이다. 성공 뒤 ref를 얻기 위해 두 번째 submission을 만들거나 별도 fallible export insert를 하지 않는다. wire ref는 원 row 번호와 준비된 issuer nonce에서 계산한다.

보존은 fixed annotation/verified/target scalars와 기존 bounded provenance를 사용한다. 원 Native identity/nonce를 매 row 문자열로 중복 저장하지 않는다. 원8KiB retained budget/오류prefix4KiB/read-owner256bytes charge를 유지하고 새 필드 실제 layout을 Rust const/실행 회귀로 계산한다. grant/tentative row·준비/응답 Clone·수명 lease의 transient peak는 별도 회계다. 예산이 맞지 않으면 무상 확장/오류 해시 삭제 대신 계약을 함께 수정한다.

후속 segment edit·새 pass·session close/현재 revision·route 교체가 과거 metadata를 바꾸지 않는다. core 후처리 오류 응답을 잃었을 때 list가 복구하는 것은 원 native ref/metadata/result다. 저장하지 않은 core 후처리 오류 전체나 원 응답 전달 여부까지 복원했다고 표시하지 않는다.
