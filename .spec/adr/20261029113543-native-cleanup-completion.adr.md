---
id: adr:kinekino/20261029113543
title: Native child 정리 결과와 알림 수명 분리
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native child 정리 결과와 알림 수명 분리

2026-10-20. 결정: host 소유의 bounded Rust dispatcher를 사용한다. 구현 `3a754b4`의 집중 검증은 인수했으며 실제 NativeChild·OS parent·main 조립은 별도 미완료다.

## 이유

정리 함수의 결과가 준비됐어도 thread-local destructor가 끝나기 전에는 worker가 종료되지 않을 수 있다. worker가 return 직전에 알리면 UI가 아직 종료되지 않은 worker를 관측한 뒤 마지막 알림을 잃을 수 있다. UI에서 join하거나 주기 polling으로 이 경계를 숨기지 않는다.

## 결정과 책임

명시적 host가 dispatcher 하나를 소유한다. bounded admission에 성공한 작업은 dispatcher가 child worker를 시작하고 실제 join한 뒤 typed 결과를 보존하고 scoped wake를 호출한다. 거부된 요청은 아직 시작되지 않은 작업과 owner를 반환한다. child별 supervisor와 Windows pool callback 수명을 추가하는 방식은 채택하지 않았다.

child 결과, notification 호출 결과, 실제 GUI의 event 소비, dispatcher 자체 종료는 서로 다른 상태다. notification 실패·panic이 join된 child 결과를 지우지 않는다. dispatcher admission close·drain·join도 child Complete를 대신하지 않는다. Drop은 UI에서 join하지 않으며 종료 성공 기록이 아니다.

## 근거와 남은 연결

[검증 정본](../../../sots/verification/application/native-join-dispatcher-2026-10-20.md)의 dispatcher4·parent6 회귀와 lib/bins strict·fmt를 근거로 한다. TLS destructor가 종료를 지연시키는 경우 결과와 wake가 먼저 노출되지 않는 시험을 포함한다. 실제 OS나 GUI 전달 성공의 증거는 아니다.

[NativeChild 조립 계획](../../../plans/player/native-playback-child-assembly.md)은 같은 attempt·전체 CleanupStamp·원 !Send lease와 기존 dispatcher를 연결한다. 실제 parent 준비·영상 surface·완료 후 host dispatcher의 별도 종료 관측과 main 소유권은 후속 조립에서 인수한다. 기술 정리 성공을 미디어 출력·제품 시각 품질 승인으로 확대하지 않는다.
