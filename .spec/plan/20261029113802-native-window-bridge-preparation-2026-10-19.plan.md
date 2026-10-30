---
id: plan:kinekino/20261029113802
title: "과거 기록: 제품 NativeWindowHost bridge 소비 준비"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 과거 기록: 제품 NativeWindowHost bridge 소비 준비

이 파일은 r1/r2 소스 준비 당시 기록이다. 현재 결과는 [완료 gate](../native-window-bridge-gate.md)와 연결된 실행 정본을 따른다.

상태: 저자원 소스 준비·개별 rustfmt 확인, 컴파일/실행0. 이전 [factory 검증 완료](../product-document-factory-gate.md)와 별도 입력이다. SDK audio 단일 heavy 중 추가 cargo/GUI·SDK assembly/manifest/root 등록 변경0이다.

현재 준비 입력은 r2다. [admission 정본](../../../../sots/player/presentation/shared/proposal-native-admission.md)에 단건 영구 거부·누적 TextCapacity·동일 immutable registry 한도/검증·실제 terminal→명시 Completion/SDK counter 계약을 분리했다. r1은 그대로 보존했으며 r2 신규 bridge 선언10개는 모두 미컴파일/미실행이다.

## 작성한 소스와 책임

신규 proposals/window_bridge.rs는 실제 공개 NativeDocumentHandle/NativeHostRuntime/NativeWindowHost·HostStamp·IntentAdmission·동기 try_reproject receipt·CloseDecision을 소비한다. factory의 into_host_parts가 같은 document/session/parent/registry/context/선택/inventory를 한 번 이전한다. 두 번째 SDK/store/문서/registry나 미공개 가상 host API를 만들지 않는다.

admit_callback은 실제 session callback enqueue만 실행한다. QueueCallbackAdapter의 실제 IntentError::Capacity에만 Busy를 부여하고 bridge는 그 Immediate error만 Capacity로 반환한다. registry dispatch 자체 Busy·텍스트 예산/서명/foreign/stale 오류는 거부·보존하며 용량으로 숨기지 않는다. NativeRuntime이 미수락 suffix를 보존하고, after_dispatch는 수락된 queue를 한 번 drain한 뒤 suffix가 없을 때만 최신 명시 선택·props/bindings를 동기 재투영한다. receipt를 반환하므로 driver가 두 번 적용하지 않는다. SDK 오류는 기존 session에 유지하고 repaint/completion이 mutation을 재실행하지 않는다.

source scope뿐 아니라 host_id·definition/document revision·turn을 현재 handle stamp와 대조한다. 실제 항목 변경은 명시 index의 현재 generation을 확인한다. 마지막 stage cleanup 실패와 SDK 확정 결과·projection 오류를 구별하고, 새 준비 실패가 old export를 폐기하지 않도록 기존 reproject_with를 그대로 사용한다.

close는 injected 제품 정책으로 cleanup 시작 전 veto를 받고, driver가 준 실제 CallbackCloseReceipt와 consumer checkpoint를 유지한다. session 성공 뒤 parent만 실패하면 checkpoint로 session 단계를 건너뛴다. ForcedOsDestroy는 best effort 뒤에도 ForcedIncomplete다. parent/예외 pending ticket의 CancelRequested는 완료가 아니며 terminal 이후만 정리를 완료한다. 창/표면 제거는 driver가 담당한다.

## 미실행 시험과 명시 한계

기존 document_factory 테스트 파일에 bridge 실제 SDK edit/save/accept·completion 비재실행, 사용자 close veto/parent retry, forced incomplete, Capacity suffix 재입장·old field 거부/저장 없음, foreign host stamp·잘못된 항목, pending parent cancel/terminal 후 종료 소스를 추가했다. 실제 통과 수는 다음 실행에서 확정한다.

추가로 이미 닫힌 factory 패널의 handle/콜백/일반 mutation 권한을 다시 여는 것을 거부하는 소스를 작성했다. r1의 declared bridge 시험은7개인 미실행 이력이다. r2는 누적 bytes/단건 초과/공유 registry terminal 재입장·실제 SDK counter3개를 더한10개 선언이다. 실제 실행 수는 다음 gate에서 확정한다.

문자 입력은 공개 document API의 기술 입력이며 실제 OS 키보드/IME 증거가 아니다. 공격적인 동일 batch의 서로 다른 field 입력은 첫 편집 뒤 이전 generation suffix를 stale로 명시 거부하는 보수적 동작을 검사한다. 정상 driver 이벤트는 입력마다 turn/reprojection을 수행한다. 이를 서로 다른 field의 동일 batch 편집을 모두 성공시킨 것으로 표시하지 않는다.

현재 QueueCallbackAdapter는 항상 Immediate다. unexpected UI Pending 방어 분기는 ticket을 보존하지만 정상 adapter로 그 분기를 만들지 않는다. 준비된 pending 검증은 기술 parent adapter의 실제 registry Pending/cancel Requested/terminal acknowledgement 경로이며 SDK 추론·정상 UI Pending 성공으로 확대하지 않는다. failed renderer preflight/old export 보존은 기존 factory 회귀와 함께 인수하고 bridge의 stale stamp 거부·last-good 유지도 별도로 확인한다.

## 다음 gate

현행 r2는 runtime/coordination/ui-native-window-bridge-prepared-r2/source 및 owned-pins.json SHA256 `c609a23b987818a059218e64439aa64948ee12c4950e1680682f4b9d727d9b4f`다. owned5는 r1의4개와 intent_queue.rs이며 r1-r2 diff와 검증한 원 queue baseline을 보존했다. 원 queue archive SHA6235c2…bb76가 r1 regression pins와 일치함을 확인하고 사용했다. r2 안의 single-text preflight 이전 중간본도 별도 보존했다. 아래 r1 snapshot은 이전 준비 이력이다.

고정 준비본은 runtime/coordination/ui-native-window-bridge-prepared-r1이다. owned4는 window_bridge.rs/document_factory.rs/callback_adapter.rs와 tests/document_factory.rs이며 source 복사본·owned-pins.json SHA256 `00b458101797107046996762c75c208cd757d321ac41c87e3b8a420d67ff3299`를 보존했다. 기존 proposals/consumer/plugin Rust source와 원본 TSX·assembly factory·독립 manifest/lock도 별도 regression/module pins로 구별했다. declared-unexecuted-bridge-tests.txt의7개는 선언 목록이고 실행 결과가 아니다. 이 consumer 입력은 다음 슬롯까지 동결하며 SDK 오디오가 소유한 player_app.rs/transport.rs·공유 assembly/manifest는 수정하지 않는다. 실제 새 root SDK/Meari 동결 입력은 gate 시작 시 다시 대조한다.

현재 SDK 반환·등록 입력 동결 후 새 owned/shared/manifest pins로 bridge/factory/panel 선택→기존 R3/CardHL 포함 전체 --no-fail-fast→strict alltargets→fmt를 실행한다. 처음 실패를 보존하고 UI 최소 수리만 수행하며 shared API 변경은 owner와 먼저 조율한다. 기존 runner2CPU/BelowNormal/offline locked-j1/testthread1/cache·추가 GUI0을 유지한다.

그 다음 root 제품 parent lease 실제 등록과 NativeWindowDriver 창 연결·사용자 종료·fresh CLI/project/provenance 인수는 별도 실제 수직이다. 기존 eframe 공유 loop/GL import가 해결됐다고 주장하지 않는다. 전체 디자인/가상 모니터·13화면·커서/음소거·실제 청취와 seek stall은 기존 후속을 유지한다.
