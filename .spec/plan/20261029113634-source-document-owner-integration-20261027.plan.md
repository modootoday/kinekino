---
id: plan:kinekino/20261029113634
title: 원 document owner의 발행·정리 수명 연결
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 document owner의 발행·정리 수명 연결

R39/R40에서 [core 보유 이후 replay 발급](../../../sots/verification/application/source-replay-issue-custody-20261027.md)을 추가했다. pending metadata/document는 같은 원 admission에 보유하고 policy error/panic·원 shared mutex/payload 보존 및 실제 offline CLI를 확인했다. production 정책·최초 document 획득/factory·등록과 실제 HTTP/COM은 미완료다. 아래 원 요구사항과 인수 범위를 유지한다.

R34에서 [core 인수 뒤 준비 callback/source custody](../../../sots/verification/application/preparation-source-custody-20261027.md)를 연결했다. adopt이후오류/panic2와정상/취소CLI12회가통과했다. adopt이전의실제source획득/부분IPC builder실패와owner반환은별도미완료이며새계약만으로완료처리하지 않는다.

R31에서 [원 session→명시 source owner](../../../sots/verification/application/document-source-owner-20261027.md)와 ReplayOriginOwner 구현을 연결했다. 합성 provider의 browser86·실제 offline CLI12회가 통과했다. 실제 native 해제/host경쟁·hard오류행렬·core custody 전 preparation/unwind 보존·production source 등록은 아직 미완료다. 아래 전체인수기준을 축소하지 않는다.

후속 R28/R29에서 [candidate release retaining 단계](../../../sots/verification/application/candidate-release-retention-20261027.md)를 구현·검증했다. 기존 slot이 정리 pending/error/panic 동안 candidate와 verified output을 보유하고 terminal을 보류한다. ReplayOriginOwner typed port는 추가했으나 실제 document owner의 이전/구현·native 실행은 아직 미완료다. 아래 원 요구사항과 인수 행렬을 유지한다.

R26의 [native release gate](../../../sots/verification/application/browser-native-capture-release-20261027.md)는 native Capture 보유 해제 관측만 분리했다. 실제 WebView/COM/공유 프로세스 전체 종료 proof가 아니며 제품 source 등록은 계속 미완료다.

## 유지할 요구사항

- 원 provider/document/ticket/generation/session/grant를 유지하고 publication 직전 current 검사를 통과해야 한다. resolve가 끝났다는 이유로 원 BrowserDocumentSession을 먼저 drop/cancel하지 않는다.
- source child는 기존 original ParentTransfer/Job/IPC/deadline으로 publication 전 정리한다. 원 browser lease는 publication current 검사까지 보유한다. 공유 browser host를 해당 HTTP child의 Job family로 표시하지 않는다.
- document cleanup pending/error/unwind는 원 owner와 첫 실패를 보존한다. generic G에 blocking Drop만 넣고 terminal success를 반환하지 않는다.
- publication CAS는1회이며 cleanup 재조회로 publish/verify를 재실행하지 않는다. 이미 발행한 파일·receipt를 후속 cleanup 실패에서 삭제하거나 미출판으로 바꾸지 않는다.
- 원 작업 deadline을 연장하지 않는다. native release 요청/관측과 실제 capture/COM/프로세스 해제의 증거 강도를 구분한다.

## 다음 구현 범위

기존 media-jobs retaining worker의 candidate publication/verification 뒤 terminal 기록 경로를 먼저 읽고, 같은 원 slot이 candidate+출력 결과+document owner를 cleanup 완료까지 보유할 수 있는 단계 계약을 확정한다. 별도 registry·detached cleanup thread·fake NotStarted 관측으로 우회하지 않는다. publication 실패/취소/원 panic에서도 같은 원 보유 객체가 cleanup을 poll해야 한다.

그 뒤 BrowserDocumentSession의 source 이전은 explicit owner 객체로 수행하고, 기존 catalog/episode 일반 Drop 동작은 필요한 호환 범위에서 유지한다. source adapter가 소유권을 받은 시점·한 번의 cancel admission·원 capture release 관측·ticket release를 기록하고, 아직 pending이면 같은 retaining slot을 유지한다. 원 BrowserProviderOwner의 host shutdown과 ticket release를 혼동하지 않는다.

Browser 담당은 원 native release/current·grant를, SDK 담당은 기존 retaining worker/candidate terminal gate를, Media 담당은 source adapter를 소유한다. PM이 private closure/preimage와 root index/heavy1을 직렬 배정한다. 실제 provider policy·HTTP·추론·GUI·push 보류를 유지한다.

## 필요한 인수 행렬

원 grant current 유지→publish1회→document close pending→해제 관측→terminal, cancel before publication→publish0/동일 owner 정리, publish 성공 뒤 cleanup 지연/실패→출력 불변/재publish0, publication 오류와 cleanup 오류·panic의 첫/후속 보존, owner mismatch/reload·timeout·queue full·worker disconnected·native drop 미관측, host shutdown과 ticket release 경쟁을 Rust fixture/실제OS 범위별로 분리해 검증한다. 실제 WebView 실행이 보류된 동안 해당 행은 미검증으로 남긴다.
