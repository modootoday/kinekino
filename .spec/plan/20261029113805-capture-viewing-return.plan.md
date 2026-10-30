---
id: plan:kinekino/20261029113805
title: Capture 편집에서 Viewing 복귀
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Capture 편집에서 Viewing 복귀

2026-10-23 source-only 후보다. [기존 actual116](../../../sots/verification/presentation/capture-product-caller-surface-2026-10-21.md)과 [lifecycle 후보](capture-choice-slot-lifecycle.md)를 보존하며 새 컴파일·실행·GUI 근거는 없다.

## 제품 흐름과 소유권

기존 공통 Button과 aw-button 토큰, CaptureLabels의 번역 가능한 “시청으로 돌아가기” 용어를 사용한다. 사용자 callback은 원 CapturedSurface의 bounded input queue에 ReturnViewing을 기록한다. batch barrier 뒤 기존 request_viewing_return/advance_viewing_return 트랜잭션으로 전달한다. 독립 router나 SDK 정책은 추가하지 않는다.

기존 live editor의 cards.active/현재 document 조건은 유지한다. capture 경로에는 별도의 ReturnOwner::Capture가 필요하며 attached entry의 원 scope, 저장된 새 origin, 현재 host stamp, 실제 capture/surface의 존재를 확인한다. observed studio.cards를 확인한 기존 ShellState만 view.start ticket을 발급한다. route 요청·props commit·frame 관측을 서로 대체하지 않는다. React Router 전체 호환을 주장하지 않는다.

CaptureCardModel의 local dirty와 기존 CaptureExecutionProgress/SDK adapter를 재사용한다. 현재 adapter를 요청 직전과 release 전에 다시 poll한다. dirty, queued/running/unsubmitted/미소비 terminal/ACK 미완료, 처리 기록 미저장·미인수, rejected intent는 복귀를 막는다. Return 클릭으로 저장·채택·버림·ACK를 자동 결정하지 않는다.

명시 결과 처리와 기록 저장 후 기존 poll_capture_release로 input export를 퇴역시키고 원 capture attachment를 release한다. 기존 read owner를 같은 scope에 재수출하고 원 ProductFrameBlueprint로 Viewing을 atomic commit한다. 성공 뒤에만 capture surface/adapter/entry를 제거한다. 처리 기록 owner와 원 artifact는 유지한다. 실패 시 원 ViewingReturn phase·오류·ticket과 editor 소유자를 보존하고 기존 명시 retry/cancel 계약을 따른다. 이 후보는 별도 GUI recovery 화면을 추가하지 않는다.

재진입은 동일한 저장 프로젝트의 원 immutable capture를 선택한다. live 문서를 자동 inspect/refresh하거나 origin을 새로 발급하지 않는다. 이후 stale CAS는 기존 SDK가 판단한다.

## 작성한 시험과 근거 경계

요소 추적은 기존 Cards 편집기의 기능적 navigation affordance 하나로 한정한다. 전체13화면/시각 인수 범위는 축소하지 않는다.

| 요소 | 선언/데이터/공통 책임 | 기준과 상태 |
|---|---|---|
| studio.cards.capture.return-viewing | CapturedCardPanel의 공통 Button, aw-button variant, CaptureLabels.return_viewing, captureCanReturn/onCaptureReturnViewing; UI host guard만 표시 | 현재 source 후보는 편집기 heading 앞에 배치. 기준 시안 crop/논리 치수/좁은 창/hover·pressed·focus·1:1·2x·4x 비교 미측정·미검증. UI/UX 위치 승인이나 시각 일치를 주장하지 않음 |

기존 창 캡션·Pretendard·공통 font/assets·border/radius/focus token을 소비하며 새 색·치수·SVG·전용 스타일을 추가하지 않는다. 실제 레이아웃·키보드·접근성 인수는 CLI/SDK 선행과 중앙 GUI 보류 조건을 충족한 뒤 별도 검토한다.

| 후조건 | 읽을 상태/시험 입력 | 현재 근거 |
|---|---|---|
| dirty 복귀 veto | 실제 disabled Return 버튼의 무이벤트, 원 request guard, 무 SDK 변경 | ignored 시험 작성, 실행0 |
| 명시 수정 취소 뒤 복귀 | 실제 Return callback→원 queue→retained transaction→실제 Viewing graph/HostCommitReceipt | ignored 시험 작성, 실행0 |
| 미소비 결과 veto와 결정 후 복귀 | 실제 SDK save/accept, 명시 consume, 기존 immutable evidence 저장, Return callback | ignored 시험 작성, 실행0 |
| 재진입과 이전 콜백 퇴역 | 기존 선택 항목 재활성화, old callback 거절, 원 captured revision 유지 | ignored 시험 작성, 실행0 |
| 결과 버림이 local edit/백엔드 undo가 아님 | stale edit 결과의 명시 discard 후 dirty 유지, 명시 local revert 뒤 복귀 | ignored 시험 작성, 실행0 |
| frame 관측 분리 | 실제 props receipt 뒤 desired=view.start/observed=studio.cards; 이후 synthetic notice로 정책 전이 | 시험 작성, 실제 render/OS 관측0 |

시험은 기존 실제 SDK/IPC/ViewerHost/ExecutionOwner/원 cleanup fixture를 재사용한다. observation.rs의 cfg(test) 전용 wrapper는 기존 observer에 명시적으로 synthetic NativeWindowNotice를 전달한다. 새 window/renderer를 실행하거나 제출된 픽셀을 만들지 않는다. synthetic notice는 실제 frame·OS·시각 인수 근거가 아니다. lifecycle helper의 강제 workspace 투영은 제품 복귀에 사용하지 않는다.

최초 미입증 단계는 컴파일/실제 등록이다. 작성 이름·정적 검토·manifest 해시는 통과 수가 아니다. 다음 실행 배정에서 전체 fmt/default+native strict, 고정 BIN·실제 등록, 새2/lifecycle4 및 직접 영향 callback/route/cleanup 회귀를 대조한다. 새 실행 권한을 이 문서로 부여하지 않는다.

## 핀과 명시 합성

stage: runtime/development/ui-capture-viewing-return-r1

- baseline root e9aee3ec4359e33b2c46f3a3bf219f5281e4dc91 / library0b9ea03 / lockE469
- 원 source12: 47B6CA7C6B9F2CAE4CAA48A3DE72A86895623BC35322F046BDC4F5F294CB9413
- before12: 27CF0C41CDB7E57D501988967F4B758B5B5C5BF8F7BD87E5A37E11F0F17ED79A
- final-source12-r1: EFD820C29DE8B3ABB77E5BFA34ED8A2E660545588822630496AAF40012DD696A
- 최종 final-source12-r2: 70664B88A4D980576BE2EBB44E2F84B0FD2E14865679A4BDA07D5EBD324442E4
- explicit-union13: C3989284E14CF9CADAC7E5C9CE874949546B6DAFA51959A4D4DD034808B47C10
- 최종 explicit-union13-r2: 39985EC1A5F0E32425C0A27EAB1E79361543C53E2487BAD1D748673EFEA5AE50
- before-union13: 3DD03314D061C0BCBCC2002563B8CE3F871D2AA15D0B5C5F087D4778FA5F1448

Return 후보는 existing10/new2다. r1은 return_viewing.rs와 capture_surface/navigation.rs의 원 adapter 재확인만 보완한다. 원 source12를 덮어쓰지 않았다. lifecycle final-source2-r1 17956614...F342F와 겹치는 파일은 product_surface.rs 하나이며, union/ 사본은 두 하위 시험 mod 선언을 명시 결합한다. union13은 existing10/new3, 작성 ignored6이며 실행0이다. 원 lifecycle source2/수리2·반환 후보12를 자동 덮어쓰지 않는다. Media/SDK 다른 후보를 합성하지 않았다. root/index/commit/gitlink/S/T 변경0이다.

r2는 source 읽기에서 확인한 Rust 모듈 가시성에 맞춰 return_viewing.rs 1leaf만 수리한다. cards/capture.rs의 poll_capture_release는 cards 내부 전용이다. 이를 공개하지 않고 기존 CardRoute::poll_release를 사용해 원 capture release와 immutable evidence 인수를 함께 수행한다. 중복 persist 호출은 제거한다. 원 r1을 보존하며 컴파일 성공을 주장하지 않는다.

PM 독립 읽기에서 r2의 실제 source/preimage 핀과 guard→기존 release→read export→atomic commit 뒤 owner 제거를 확인했으며 확정 P1/P2 미발견이다. actual 통과 근거는 아니다. 현재 선택된 실행 대기는 r2 작성2와 lifecycle4의 union13이다.

이후 lifecycle child의 미수출 이름 검증 P2가 별도로 확인됐다. 위 “미발견”은 그 시점의 검토 이력이며 알려진 반례가 없다는 현재 판정으로 쓰지 않는다. production return12-r2는 불변이고, lifecycle final-source2-r2의 문자열1 수리만 결합한 제안 explicit-union13-r4 `75263EC3B88348F01597FA1008D33E5F4DA65C282B9526426A796320E9642465`를 별도 저장했다. 작성6 유지, r3 보충 제외, before13 불변이다. PM 재감사·선택 전 자동 실행 입력 대체0이며 actual0이다.

10/23 PM 독립 재감사에서 이름 validate→export lookup 순서의 반례와 exact1string 수리를 확인하고, raw2/raw13 일치·return12-r2 불변·r3 제외를 확인했다. 다음 UI 선택 입력은 위 union13-r4로 확정했다. SDK 단독 실행 배정은 유지되며 UI 작성6/컴파일0/actual0이다. 예상132는 최종 BIN 등록 후 확정한다. 선택 입력 확정은 실행 권한이나 기능 완료 판정이 아니다.

후속 실행 범위 읽기 감사에서 기존116에 포함되지 않은 직접 영향 시험10을 확인했다. 고정 BIN0979의 실제 등록 목록에 observer 정책9와 ignored generic Viewing-return owner/retry1이 있으며 이전 actual-tests-r9에는 없다. runtime/coordination/ui-capture-return-gate-plan-r1에 정확 이름과 20 normal filter/26 ignored exact 제안을 저장했다. 새 BIN 등록이 그대로라면 기존 normal97+observer9+기존 ignored19+generic1+작성6=고유132가 최소 제안이며 actual0이다. r3 보충은 제외했다. UI13과 Media effective28/SDK source6의 경로 교집합은 각각0이지만 행동 독립성이나 자동 합성 권한을 의미하지 않는다. Media/SDK 책임 채택 뒤 preimage와 최종 등록을 다시 대조해야 한다.

PM 동결 회신 직전 작성한 export 실패/retry 시험1은 revisions/r3/UNSELECTED.md의 별도 보충 후보로 보존한다. 제품 코드 변경0, 새 시험 leaf만 변경하며 컴파일·실행0이다. 원 r2 실행 입력을 자동 대체하지 않는다. 해당 보충은 read borrow failure와 원 SDK execution의 competing attachment를 사용한다. ACK/evidence/commit 실패 retry의 전체 조합과 실제 frame/GUI는 여전히 미검증이다.

## 적용 스킬과 판단

modoo.today `modootoday-free/ai-quality-plus/agent-run-postcondition-and-replay-check` 본문과 `references/worked-example.md`를 skill_call/skill_read로 실제 읽었다. 참조 SHA256은 80c9fe8a8a79dd8e08d84b36483d7742f273e55fb2769d2e6156a1111fdd5c11이다. 적용 범위는 에이전트의 완료 주장 대신 소스/원 결과의 독립 후조건을 확인하는 검증 근거 설계다. Rust UI 구현 지침으로 확대하지 않는다.

후속 이름만으로 실제 입력을 증명하지 않는다. source/preimage/BIN/첫 실패의 정확 핀을 남기고, 실제 SDK mutation을 재실행하는 replay는 격리된 기존 시험 store 안에서만 수행한다. 순수 source/hash와 비결정적 process/time/IPC 결과를 구분한다. 실패 시 첫 분기 지점과 원 phase를 보존하며 한 번에 최소 수리하고 실제 결과로 원인을 확인한다. 현재 실행0이므로 원인 해결·기능 완료라고 표현하지 않는다.
