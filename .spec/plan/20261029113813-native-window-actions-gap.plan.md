---
id: plan:kinekino/20261029113813
title: UI-HOST-WINDOW-ACTIONS-01 · Native 창 동작 갭
status: active
created: 2026-10-29
updated: 2026-10-29
---
# UI-HOST-WINDOW-ACTIONS-01 · Native 창 동작 갭

2026-10-21 source 읽기 및 Meari 담당 교차 확인. Meari22fe0d/source412, 제품 root e93a0b7 기준이다. 실행 재현·타입 진단·새 API 구현은 하지 않았다. [기본 caller 조사](native-main-caller-boundaries.md)와 연결한다.

## 최소 source 사례와 진단

제품 `web_ui/components/window-chrome.tsx`는 공통 최소화/최대화/닫기를 선언하지만 모두 disabled이며 callback이 없다. TransportBar의 전체화면도 disabled다. 원 Viewer attached의 소유 wake로는 request_close를 보낼 수 있으나 minimize/maximize/fullscreen 요청을 보낼 공개 port가 없다. 따라서 버튼 활성화만으로 실제 동작을 구현할 수 없다.

Meari 담당은 Native source 전체와 공개 driver/wake/parent 경계를 읽어 해당 명령/처리/공개 port가 없음을 확인했다. NativeWindowDriver는 attach_prepared/attach/wake_handle/last_submission/presentation_errors, NativeWindowWake는 wake/retry_failed_completion/request_close를 제공한다. NativeScopedParent는 HasWindowHandle만 제공한다. 내부 WindowBinding.window()의 Arc window는 pub(super) 경계이며 제품에 제공된 창 action API가 아니다. raw HWND 재구성이나 내부 getter 공개 확대를 제품 workaround로 삼지 않는다.

이는 원 surface/checked child proof/NoMedia API가 부족하다는 진단과 다르다. 해당 API는 기존 source8 종단에 충분하며 창 상태/host intent라는 별도 책임이 빠져 있다. 확인되지 않은 컴파일 에러나 실제 클릭 실패 로그를 발명하지 않는다.

## 라이브러리 담당과 합의할 기대 계약

아래는 요구 조건이며 API 이름/서명을 확정한 것이 아니다.

| 요구 | 기대 동작 |
| --- | --- |
| typed 동작 | minimize, SetMaximized(bool), fullscreen 상태 요청을 원 Native window에 전달. 관측 상태 없이 toggle을 반복해 OS 상태를 추정하지 않음 |
| 범위와 수명 | 원 loop·UI-thread·host/WindowStamp·Open lifecycle에 묶인 port. 외부 HWND/임의 창 선택·다른 parent/source epoch·Closing 이후를 거절 |
| 입력과 실행 | 기존 host local action/실 callback scope에서 bounded request를 수용하고 원 loop로 전달. 입장 및 실행 시 stale/closed를 확인하고 새로운 executor/worker를 만들지 않음 |
| 상태와 결과 | 접수/대기/실제 OS 관측을 분리. maximized/fullscreen/minimized 관측 snapshot과 실제 capability를 제공하며 접수 성공만으로 상태가 바뀌었다고 표시하지 않음 |
| 실패 보존 | Busy/미지원/관측 불가/OS 실패를 구분하고 원 document/runtime/owner·기존 오류를 보존. 실패 시 임의 재시도/성공 ACK 합성0 |
| 닫기 | 기존 제품 NavigationGuards→owned wake→원 ChildWindowHost barrier→Viewer shutdown/ACK 경로 유지. 새로운 창 action port에서 직접 Destroy/다른 close executor를 만들지 않음 |

제품은 같은 React/TSX WindowChrome·TransportBar의 typed props/host local actions와 Rust presentation 상태를 통해 소비한다. SDK playback wire에 창 동작을 추가하지 않고 브라우저 메시지로 임의 OS 명령을 받지 않는다. window-pointer cursor 포트도 이 갭과 별도 책임이며 전역 cursor 변경은 포함하지 않는다.

## 인수·다음 작업

후속 Meari는 별도 overlay source11 후보 `561B7CFF989EF0F06674D649F2300A216318CC402DCAFCBF17DB928A3293D994`를 전달했다. 아직 컴파일/OS 인수가 아니며 root22fe0d에 적용하지 않았다. 실제 후보 서명은 NativeWindowPreparation.window_actions()→NativeWindowActions, NativeWindowDriver.window_actions()→Option<NativeWindowActions>다. handle은 원 binding Weak/UI-thread만 소유한다. snapshot/request/record/take_result와 ActionScope(host,WindowStamp), SetMinimized/SetMaximized/SetFullscreen(bool), Queued/Issued/Observed/Failed를 확인했다. fullscreen 관측 필드는 borderless_monitor_coverage이며 Observed는 상태값 일치이지 인과 ACK가 아니다.

후보 구현을 읽으면 원 ACTIONS wake bit가 같은 Application loop의 drive_window_actions로 전달된다. drive의 상태 변경은 활성화된 binding에서 원 COMPLETE wake를 보낸다. 원 about_to_wait도 같은 loop의 poll_window_actions를 사용한다. 제품은 이 기존 이벤트/Completion 경계에서 원 SDK 성공·callback barrier를 지킨 뒤 snapshot/결과를 읽고 typed props를 재투영하는 방향으로 연결할 수 있다. 새 timer/loop/executor를 만들 이유는 현재 확인되지 않았다. 첫 Ready snapshot의 알림 시점과 prepared handle을 원 Viewer state로 넘기는 정확 시점은 담당에게 확인을 요청했다. 최종 API/핀 인수 전 추측 소비 코드는 작성하지 않는다.

제품 측 연결 후보는 preparation에서 얻은 원 weak handle을 scoped Viewer window-state에 한 번 전달하고, 동일 callback registry의 host local actions를 SDK read actions와 책임별로 분리하는 것이다. UI는 maximize/restore·enter/exit fullscreen의 명시 bool 의도를 각 로컬 binding으로 전달하고 관측 전 toggle을 추정하지 않는다. 상태를 알 수 없으면 요청 지원 capability와 관측 불가를 구분한다. 접수만으로 restore 아이콘을 바꾸거나 동일 요청을 새 ID로 자동 재전송하지 않는다. 닫기는 원 NavigationGuards/owned wake 경로만 소비한다. 이는 consumer 설계 후보이며 새 caption/host module 편집 배정이 아니다.

담당의 원 소스 확인 회신: controller.start에서 **원 preparation 반환 직후** window_actions handle을 H의 공유 slot에 전달할 수 있다. Ready 전 snapshot은 NotReady이며 실제 Ready/원 loop 관측 이후 읽어야 한다. 변경 통지는 원 observe_dispatch→port.after_dispatch→H.after_dispatch(Completion)→update로 이어진다. Completion은 원 pending turn을 이어갈 수 있어 새 turn/FrameSubmitted와1:1이 아니다. 최초 request 전에는 외부 OS 변화의 별도 Completion 통지가 활성화되지 않으므로 초기 props는 기존 Ready/dispatch에서 snapshot을 읽는 경계를 사용해야 한다. 이 제한을 자동 재시도나 새 polling executor로 숨기지 않는다. 후보 source11의 실행·최종 인수 전이라는 상태는 유지한다.

라이브러리 담당이 실제 소유권/API를 설계·구현한 뒤 동일 Viewer consumer에서 최소화→복원, 최대화→복원, 전체화면→복귀의 request/실제 snapshot·scope/epoch/stale/Closing 오류를 검증한다. 유휴 pointer·키보드/a11y, 원 close/child-first 오류·정상 종료는 각각 회귀한다. CPU fake port만으로 OS 창 동작을 완료로 표시하지 않는다.

현재 PM에는 정확 부족 경계와 위 요구를 인계했다. source12는 동결이며 새 라이브러리/제품 생산 코드 및 실행은 별도 배정 전0이다. full native-default 전환은 이 창 동작과 실제 route/미디어·전체13화면/시각 인수를 생략하지 않는다.
