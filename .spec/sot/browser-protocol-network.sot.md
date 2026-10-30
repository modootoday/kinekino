---
id: sot:kinekino/browser-protocol-network
title: 소유 browser의 protocol network 관측
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 소유 browser의 protocol network 관측

이력: 순수 projection/수명/ledger/body codec의 foundation13회귀를 먼저 통과했다. 이후 native observer·capture enable barrier·worker·body 원문 archive·CLI를 등록했다. 첫 [통합 검증](../../verification/application/player-asset-cli-2026-10-19.md)은 fmt/strictClippy/전체264pass17ignored/두bin 통과 뒤 actual CLI1failed/34.80초 timeout이었다. 부모·자식 DOM과 원문4개를 보존했으나 자식 protocol scope/body 확보에 실패했다. [직접 자식 문서 검증](../../verification/application/frame-document-cli-2026-10-19.md)과 [기존 response 귀속 감사](../infrastructure/browser-provider-media-observation.md)에 연결한다.

현재 상태: 후속 [세션 수리 인수](../../verification/application/player-asset-session-cli-2026-10-19.md)는 fmt/strictClippy/전체271pass17ignored/두bin 및 동일 actual CLI1pass17.55초다. 실제 iframe Target attach·named frameTree·default detach reason=swap을 관측하고 해당 session의 HTTP200 JavaScript26795bytes를 확보했다. raw5 전체 CLI 재읽기와 EOF worker 종료를 확인했다. 결과는 NeedsInput/player_runtime_observation_required이며 최종media/native 재생은 계속 미완료다.

이후 [runtime 표본 인수](../../verification/application/player-runtime-cli-2026-10-19.md)는 전체276pass17ignored·actual1pass20.11초다.8표본에서 영상 요소 생성까지 관측했으나 readyState0/networkState2/paused/time0이다. bootstrap POST200과 아직 응답 미도착인 자연 발생 GET XHR `.vtt`/`.txt`를 구분했다. 다음은 표본 상한과 provider 전체 완료를 분리해 해당 응답의 완료/실패·bounded body·manifest signature를 인수하는 것이다.

## 실제 근거와 관측 순서

보존 media-cdn DOM은 player wrapper와 hidden `.play-button-outer`를 포함한다. 정적 onclick은 loader 표시와 fireload 호출이지만 함수 소유·실제 ready·최종 source는 미확정이다. packed inline을 실행하거나 임의 media URL을 구성하지 않는다. 다음 관측은 같은 child 수명의 실제 script/resource 응답→typed ready/video/error/가시성→필요한 고정 컨트롤 활성화→manifest/body 순서다. hidden control의 CSS를 바꾸거나 programmatic click을 native user activation으로 표시하지 않는다.

WebResourceRequested의 현재 source-kind 정보나 response 시점 현재 page/Referer만으로 요청을 특정 frame 재탐색 세대에 귀속하지 않는다. 대안 CDP 관측은 protocol requestId/frameId/loaderId와 Page frame 계보를 한 소유 세션 안에서 결합한다. native Frame5 숫자 ID와 CDP 문자열 ID가 같다고 가정하지 않는다.

## 작성한 순수 계약

`application/browser_network`는 Page frameAttached/frameNavigated/frameDetached, Target attachedToTarget/detachedFromTarget, 고정 Page.getFrameTree 응답과 Network requestWillBeSent/responseReceived/loadingFinished/loadingFailed를 해석한다. JSON128KiB, URL8192bytes, opaque ID128bytes를 제한하고 malformed 값·음수/소수 byte 수·안전 정수 범위 초과를 거부한다. 오류에 원문을 복사하지 않는다. 요청 header는 실제 Referer만 투영하며 Cookie/Authorization과 기타 header를 보존하거나 inline 출력하지 않는다. 없는 frame ID/Referer와 worker의 빈 loader는 부재 그대로 남긴다.

DocumentScopes는 검증된 최초 /e root 관측에서 시작한다. 같은 owned protocol session의 root loader 변경은 이전 child scope를 모두 무효화한다. 직접 parent frame ID가 현재 root와 맞고 실제 child URL이 허용 player-document 형태일 때만 scope를 만든다. 같은 URL이어도 loader가 바뀌면 새 수명이다. scope 밖 재탐색·detach도 이전 수명을 먼저 제거하며 현재 page로 보충하지 않는다. root 관측32회/현재 child8개를 제한한다. 이 scope는 CDP namespace의 증거이며 native document identity의 대체가 아니다.

## 다음 adapter와 인수 조건

native receiver는 초기 탐색과 activation 전에 Page/Network enable 완료를 확인해야 한다. event callback은 bounded queue에 관측만 넘기고 JSON projection·정책·SHA·저장은 Rust worker/application에서 처리한다. WebView2 event의 target SessionId를 분리하고 다른 target/session의 동일 문자열 ID를 같은 request로 취급하지 않는다. OOPIF/session 연결이 입증되지 않으면 해당 source를 미귀속으로 남긴다.

native guard는 소유 STA에서 receiver 등록→세션별 Page/Network enable 완료 카운터→준비 상태를 제공한다. 기본 세션 준비를 다른 target의 준비로 재사용하지 않는다. SessionId API 미관측/실제 빈 기본 ID/실제 target ID를 별도 값으로 둔다. event256개 queue, 기본 세션과 관련 iframe 세션 최대8개, 비동기 body4개, event JSON128KiB와 body envelope 제한을 갖는다. body 명령은 capture capability의 관측 request ID를 사용한 고정 Network.getResponseBody뿐이며 eval·새 GET·header 변경·탐색을 하지 않는다. source queue overflow/초기화 실패는 준비 상태로 복귀하지 않고, Drop은 closed를 세운 뒤 handler를 해제한다. 기본 세션만 연결한264pass 고정본의 actual timeout 이후, Target/session 연결 소스는271pass 및 실제 자산 인수를 통과했다. root reload/late callback 회귀는 synthetic이며 실제 연속 reload stress를 입증하지 않는다.

후속 소스는 해당 WebView의 직접 관련 iframe만 Target.setAutoAttach(flatten/waitForDebuggerOnStart)로 관측한다. 초기 root 탐색은 그 ACK까지 기다린다. 실제 attach 세션에서 Page/Network enable→Page.getFrameTree→application 계보 인수 뒤 고정 Runtime.runIfWaitingForDebugger를 호출한다. 이는 관측기 초기화 barrier이며 사용자 입력·player ready 증거가 아니다. frame-tree의 실제 parentId가 현재 root와 맞거나, 실제 기본 세션 frameAttached의 child ID와 tree root frame ID가 같아야 세션을 연결한다. targetId를 frameId로 간주하거나 URL로 부모를 추정하지 않는다. root 교체는 attachment/target proof를 모두 제거한다. default의 swap 분리는 기본 child scope만 폐기하고, remove/target detach/named frame 분리는 해당 proof를 폐기한다. protocol scope와 보존 event에 실제 session을 넣고 요청 ledger도 세션별로 분리한다. 귀속되지 않은 request 수를 별도 기록해 전체 native 요청 부재와 구분한다.

request ledger는 최초 request의 session/frame/loader/request ID와 자체 attempt를 고정한다. redirect의 동일 request ID 재사용은 별도 attempt로 다루고 이전 비동기 body callback을 거부한다. response의 ID/loader·실제 URL/status/type, loadingFinished와 body 확보를 결합한다. URL 동일성만으로 join하지 않는다. 실제 응답 body는 byte cap·status·표현 종류·SHA를 갖고 보존하며 prefix와 complete body를 구별한다. body 조회 실패·제한 초과·frame 교체·cancel/terminal/EOF 뒤에는 성공을 만들지 않는다.

작성한 ledger는 한 소유 protocol session당 인스턴스 하나이며 request64개/응답 body2MiB를 제한한다. request ID·현재 scope·loader·실제 response URL·finished 관측이 결합돼야 일회 capture를 발급한다. 동일 request ID redirect는 실제3xx/이전 URL을 확인해 별도 attempt로 구분한다. root/child 수명 교체와 close 뒤 body를 받지 않으며 같은 URL의 다른 request ID로 완료하지 않는다. Script/type·실제 media-cdn player asset 경로가 맞으면 실제 MIME와 오류 HTTP 응답도 보존하며 HTTP 성공 및 runtime 성공과 구분한다. manifest 후보는2xx와 mpegurl MIME를 요구한다. 요청 type 누락은None으로 유지한다. 이 발급 자체는 실제 HTTP body나 manifest 검증이 아니다.

실제 player asset은 이미 문서에서 관측한 정확 요청만 대상으로 삼는다. scripts.php URL을 media로 넘기지 않는다. HLS signature·허용 transport·실제 response·현재 protocol 수명이 모두 확인된 뒤에만 verified media 결과를 준비한다. native mpv handoff는 그 이후 별도 gate다.

## 읽은 공식 근거

[Chrome Network 규격](https://github.com/ChromeDevTools/devtools-protocol/blob/master/pdl/domains/Network.pdl)은 request/loader 식별자와 network event 계약의 정본이다. [WebView2 event 인수](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/winrt/microsoft_web_webview2_core/corewebview2devtoolsprotocoleventreceivedeventargs)는 target의 SessionId를 제공한다. API 존재와 순수 decoder 작성은 실제 WebView2 CDP event 전달·OOPIF 동작·manifest body 확보를 검증한 것이 아니다.

후속 수리는 [Chrome Target 규격](https://github.com/ChromeDevTools/devtools-protocol/blob/master/pdl/domains/Target.pdl)의 관련 target·filter·flatten·pause/resume 계약과 [Chrome Page 규격](https://github.com/ChromeDevTools/devtools-protocol/blob/master/pdl/domains/Page.pdl)의 frame-tree/parent 계약을 직접 읽고 작성했다.
