---
id: sot:kinekino/player
title: 소유 player의 직접 자식 프레임 문서 관측
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 소유 player의 직접 자식 프레임 문서 관측

상태: application 프레임 상태·합성 회귀4개, 직접 자식 FrameCreated/Frame2 탐색·Frame5 ID·DOM 어댑터, frame_dom_snapshot 원문·별도 provenance, 공통 job/CLI를 구현했다. [검증 정본](../../verification/application/frame-document-cli-2026-10-19.md): fmt/strictClippy/전체245pass17ignored/두bin이 통과했고, 두 CPU·BelowNormal 제한의 실제 CLI1case20.17초0에서 media-cdn 직접 자식 HTTP200·탐색성공·DOM15115bytes와 원문4개 재읽기/EOF worker 종료를 확인했다. 부모/자식 세대와 ID를 분리했다. HTTP응답 body/manifest prefix 관측기는 여전히 미등록이며 최종media/native는 미완료다. [media 응답 귀속](browser-provider-media-observation.md)에 연결한다.

## 관측과 필요한 수명

보존 Frame-host top-level DOM은 video/source/audio0, iframe1이다. 실제 iframe src는 이전 회차의 선언된 media-cdn 문서 URL과 동일하다. wrapper의 referrerpolicy 선언은 실제 수신 header나 과거 fresh GET 실패 원인을 입증하지 않는다. 새 top-level GET·수동 Referer 합성·iframe URL의 media 승격으로 대체하지 않는다.

다음 adapter는 기존 소유 WebView의 Core FrameCreated 이벤트로 직접 자식 frame 객체를 얻는다. 부모 document generation/native navigation ID를 **그 생성 관측에서** 고정한다. 그 frame의 Frame2 NavigationStarting/NavigationCompleted와 Frame5 FrameId를 사용해 자식 문서의 별도 세대/native navigation ID를 고정한다. ID0을 부재로 해석하지 않으며 필요한 native API가 없으면 임의 ID를 생성하지 않는다. 직접 자식 생성 경로만 입증한 상태에서 손자·worker 계보까지 주장하지 않는다.

프레임 관측에는 부모 세대/부모 native ID, native frame ID, 프레임 내부 문서 세대, 프레임 navigation ID, 실제 URL, 관측 시각을 둔다. 프레임 완료 진단은 부모 완료 진단과 분리한다. HTTP status 미제공/null과 실제0, IsSuccess/WebErrorStatus를 그대로 유지한다. URL만 같은 이전 프레임이나 재탐색을 현재 문서에 붙이지 않는다.

## 문서 확보와 경계

Frame2 NavigationCompleted 뒤 해당 frame의 ExecuteScript로 고정된 최소 DOM 직렬화만 요청한다. 공통 Rust에서 JSON decoding·URL/세대 검증·HTML 정책·SHA·저장을 수행한다. script에는 site 정책·route 변환·library 추정·외부 입력 selector·복호화·추적 header를 넣지 않는다. top-level DOM을 다시 읽어 cross-origin 자식 DOM을 확보했다고 표시하지 않는다.

ExecuteScript 요청·완료에서 같은 frame 내부 세대/native ID와 부모 세대/native ID를 확인한다. 새로운 부모 탐색, 프레임 재탐색·파괴, 취소·종료·EOF 이후 callback은 현재 결과를 바꾸지 않는다. DOM은 최대2MiB이며 초과·잘림은 부분 성공으로 게시하지 않는다. callback은 bounded channel로 관측을 넘기고 HTML 해석·파일 저장을 수행하지 않는다. COM 객체·handler·문서 수명은 소유 STA에 남기고 unsafe Send를 추가하지 않는다.

재탐색 시작은 이전 문서를 superseded로 만든다. 이전 DOM은 역사적 원문으로 보존하지만 현재 준비 조건에서는 제외한다. 새 URL이 허용 player scope 밖이라 새 문서를 수집하지 않는 경우에도 이전 문서는 먼저 superseded 처리한다. superseded와 실제 native Destroyed 관측을 혼동하지 않는다. 이 정적 검토에서 발견한 준비 조건 결함을 수정했고 별도 순수 회귀가 통과했다. 실제 native 반복 재탐색 전체를 인수한 것은 아니다.

문서 캡처는 별도 frame DOM 표현과 provenance를 갖는다. 원문 body는 실제 자식 DOM 바이트이며 top-level generation으로 자식 세대를 덮지 않는다. 부모 navigation history는 소유 문맥이고 자식 redirect chain으로 재사용하지 않는다. 기본 진단은 origin/종류/SHA/참조 위주이며 정확 URL과 opaque 값은 private artifact에 남긴다. 기존 원문·metadata를 소급 수정하지 않는다.

등록 frame 수, 프레임 탐색 관측 수, 보존 문서 수와 callback queue를 각각 제한한다. 한도 초과는 명시 실패이며 오래된 항목을 제거한 뒤 새 세대에 재귀속하지 않는다. Drop은 먼저 closed 상태를 세우고 native handler를 해제한 뒤 frame/WebView를 놓는다. 대기는 기존 job deadline 안에서 수행하며 실패·timeout 뒤 새 탐색이나 watch 재실행을 하지 않는다.

## 인수와 다음 연결

순수 회귀는 부모 교체, 같은 URL 프레임 재탐색, 이전 완료·DOM callback, 프레임 파괴, 취소/terminal 선행, status0과 오류 문서, 각 한도 초과를 포함한다. native 실제 인수는 불변 episodes seed→소유 watch→Frame-host 완료→직접 자식 FrameCreated→자식 완료/DOM→원문·provenance 보존→EOF worker 종료→고정 CLI 전체 raw 재읽기다. fixture 통과는 실제 browser frame 인수가 아니다.

이 단계의 성공은 FrameDocumentObserved다. MediaRequested와 검증된 MediaObserved는 별도다. 이후 실제 request/response identity와 bounded manifest prefix를 결합해야 하며, 응답 시점 현재 URL이나 iframe src 문자열만으로 매칭하지 않는다. 최종 transferable media 검증 이후에만 공통 SDK/CLI native host/mpv 경로로 넘긴다. 현재 근거는 첫 회차 한 개이므로 전체 제공자·다른 작품·세 카테고리의 회차 지원 검증도 남아 있다.

## 읽은 공식 API와 고정 바인딩

[Microsoft Frame2 문서](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2frame2)와 [프레임 개념 문서](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/frames)를 읽었다. Frame2는 프레임 탐색 이벤트와 현재 프레임의 script 실행을 제공한다. 실행 문맥이 준비되기 전에는 script 결과가null일 수 있으므로 null을 빈 정상 DOM으로 처리하지 않는다.

로컬 webview2-com-sys0.39.1 bindings.rs에서 Core4.add_FrameCreated, Frame2.add_NavigationStarting/add_NavigationCompleted/ExecuteScript, Frame5.FrameId를 확인했다. webview2-com0.39.1 callback.rs에는 FrameCreatedEventHandler, FrameNavigationStartingEventHandler, FrameNavigationCompletedEventHandler, FrameDestroyedEventHandler, ExecuteScriptCompletedHandler가 있다. 바인딩 존재는 현재 adapter 구현이나 실제 호출 성공의 증거가 아니다.

추가 소스 관측: pinned FrameNavigationCompletedEventHandler는 completed_callback/FnOnce wrapper다. 반복 frame 탐색에 그대로 등록하지 않고, 별도 Rust COM adapter의 FnMut callback을 작성했다. 같은 등록에 두 차례 Invoke하는 회귀가 통과했다. ExecuteScriptCompletedHandler의 callback 결과는 이미 String으로 변환되므로 이를 PCWSTR로 중복 해석하지 않는다. 실제 CLI에서는 한 직접 자식 navigation completion/DOM을 인수했다. native 반복 재탐색은 별도 실제 근거가 필요하다.
