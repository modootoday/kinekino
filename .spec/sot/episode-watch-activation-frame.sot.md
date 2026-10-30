---
id: sot:kinekino/episode-watch-activation-frame
title: episode watch activation과 frame 관측 경계
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# episode watch activation과 frame 관측 경계

상태: 공통 Rust activation 계약·native 진단·experimental CLI job을 구현했다. [최초 CLI 검증](../../verification/application/episode-activation-cli-2026-10-19.md)에서 watch 뒤 frame.example.test main navigation 시작을 관측했고, [후속 검증](../../verification/application/player-document-cli-2026-10-19.md)에서는 같은 세션의 새 문서 completion/DOM을 확보했다. 최종 media는 미검증이다. [resolver](episode-resolve.md)의 선언 URL fresh GET에서 오류 페이지를 얻었으므로, 이를 실제 watch 동작으로 대체해 해석하지 않는다. 정적 client asset 취득도 실패했으며 URL 변환/POST/세션 규칙은 미확정이다.

## 공통 application 요청과 admission

후속 BrowserProviderPort 요청은 ticket과 expected_document_generation만 받는 제한된 episode activation이다. caller가 JavaScript·selector·실행 경로·header·다음 URL을 지정하지 않는다. 최초 ResolveEpisode의 검증된 /e 문서, Running/current generation·문서 관측·취소/종료 여부를 Rust에서 확인하고 같은 문서에 한 번만 접수한다. 다른 purpose·stale generation·pending navigation·반복 접수는 명시 오류다. native queue 공간이 없으면 상태를 바꾸지 않으며 접수와 호출 결과를 구별한다.

Rust adapter의 actuator는 고정 button.watch-now에 대한 최소 동작만 수행한다. route/URL 변환·POST body·쿠키 생성·정책 판단을 새 JavaScript에 구현하지 않는다. 기존 site code가 실제 browser 안에서 처리하는 동작의 결과를 관측하며, arbitrary handler 문자열 실행·URL 합성·GET으로 요청 재작성·인증서 우회·GUI player.exe 자동조작은 하지 않는다. 실제 user activation이 필요하면 그 제약을 NeedsInput으로 남기며 성공을 발명하지 않는다.

admission receipt는 accepted/unobserved다. bridge가 호출됐다는 관측도 player/frame/media 준비의 acknowledgment가 아니다. callback은 ticket·document generation 및 가능하면 native navigation identity를 고정하고 현재 작업과 다시 대조한다. 동일 URL 재탐색도 새 문서이며 URL만으로 이전 callback을 새 문서에 귀속하지 않는다. enqueue 뒤 navigation이 경쟁하면 늦은 outcome/frame을 성공으로 수락하지 않는다.

## 실제 frame/document 후보

실제 native frame navigation 또는 document request에서 parent document identity·frame/navigation identity·관측 시각·URL·가능한 method/Referer/source kind를 bounded record로 수집한다. 없는 method/Referer/frame identity를 현재 page에서 추정해 채우지 않는다. candidate는 관측된 요청이며 유효 player/media나 새 top-level 이동 승인과 다르다. 기본 진단에는 origin·hash·artifact ref만 포함하고 opaque path/query/token은 private evidence에서만 정책 검증에 사용한다.

같은 세션의 실제 iframe 동작과 fresh top-level GET은 별도다. profile/cookie·Referer·POST·iframe/worker 차이를 실제 관측해 필요한 adapter 계약을 정한다. 허용 host/path·request budget·문서 세대와 연결되지 않은 후보, popup·scope 이탈·source kind 미확정은 명시 NeedsInput/오류로 남긴다. source 원문과 실패·취소·종료를 보존하고 job recovery에서 activation을 자동 재실행하지 않는다.

## native navigation 진단

[Microsoft NavigationCompletedEventArgs2](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2navigationcompletedeventargs2?view=webview2-1.0.4022.49)는 HTTP status가0일 수 있고, HTTP 성공값과 IsSuccess/WebErrorStatus가 다를 수 있음을 명시한다. 따라서 각각 별도 값으로 보존하며0을200으로 바꾸지 않는다. pinned webview2-com-sys0.39.1에는 이 API와 FrameNavigationStarting binding이 존재한다. 구현/실제 호출 성공은 아직 미검증이다.

후속 adapter는 native NavigationStarting의 navigation ID와 Rust document generation을 동일 관측에서 고정하고 같은 ID의 completion만 받는다. Wry URL callback과 나중 completion을 URL-only로 붙이거나 handler 등록 순서를 추정하지 않는다. DOM capture.status=null과 별도의 navigation_http_status/IsSuccess/WebErrorStatus를 구별하고 기존 DOM/SHA/metadata를 소급 변경하지 않는다. frame과 main-document status도 섞지 않는다.

## 인수와 다음 연결

### 새 문서 완료 대기 후속 계약

구현 상태: `player-document-cli-r1/source-product`에 변경 소스를 고정했다. fmt0/strict workspace-alltargets Clippy17.00초0, 전체240pass/17ignored, 두bin build0, [실제 CLI1case24.66초0](../../verification/application/player-document-cli-2026-10-19.md)을 확인했다. activation_history 최대2개와 작업당 단일 접수, 같은 세대/native ID completion+DOM readiness, native MediaRequested 후보 최대16개 및 실제 Referer 부재의None 보존을 구현했다. 실제 Frame-host generation2/navID5의HTTP200·탐색성공과DOM28014bytes를 함께 확보했다. frame/media request0이며 최종media/native는 미검증이다. 앞선 실제 관측 정본은 변경하지 않는다.

현재 실제 case에서 watch 후 generation2 시작이 until을 끝냈다. 다음 구현에서는 시작과 완료를 분리한다. 초기 /e DOM과 native completion을 먼저 보존하고, 활성화 접수 기록은 현재 문서 필드와 별도의 bounded 불변 이력에 남긴다. 문서 이동으로 현재 activation을 지워도 이력의 이전 세대/navID는 유지한다. 늦게 도착한 이전 문서 콜백은 현재 세대 상태를 바꾸지 않고, invocation 미관측을 임의 성공으로 채우지 않는다. 같은 소유 작업에서는 이미 접수한 watch를 다시 실행하지 않는다.

새 top-level 세대의 URL이 실제 허용 player scope일 때, 동일 세대/native navigation ID의 completion과 동일 세대/최종 URL의 DOM이 모두 확보될 때까지 원래 job deadline 안에서 기다린다. redirect로 세대가 다시 바뀌면 중간 DOM이나 이전 completion을 새 문서 완료로 사용하지 않는다. HTTP0·실패 IsSuccess·WebErrorStatus·HTTP오류도 원래 값으로 보존하고, DOM이 존재한다는 사실만으로 정상 player를 선언하지 않는다. 실패 navigation에서도 가능한 DOM을 보존하되 deadline 이후 새 탐색이나 자동 재시도를 하지 않는다.

현재 SDK의 request-only MediaReady는 요청 후보 관측이며 response/prefix 검증 근거가 아니다. 새 문서 대기를 이 신호가 조기 종료해서 DOM을 잃지 않도록 후보와 검증된 결과의 수명을 구별해야 한다. prefix 관측기를 연결할 때는 실제 request identity와 고정 문서 세대의 증거가 필요하며 current URL만으로 늦은 response를 귀속하지 않는다. 불완전한 media 관측기는 등록하거나 성공 결과로 게시하지 않는다.

합성 회귀 순서는 초기완료/DOM→activation접수→새탐색시작→이전 completion/DOM/activation callback→새 completion만→새 DOM이다. 시작이나 completion만으로 준비가 되지 않고 마지막 동일 세대 DOM에서만 문서 준비가 된다. 추가 회귀는 같은 URL 재탐색, 중간 redirect, 오류 status0/404, 취소와 timeout, 늦은 후보, EOF 소유 종료다. 합성 fixture는 native browser 동작이나 실제 제공자 성공의 증거로 세지 않는다.

순수 회귀는 wrong purpose/세대/중복·queue-full 무효 상태·cancel-before/terminal-before·동일 URL reload의 late completion/frame 거부를 검증한다. 실제 case는 소유 /e 세션에서 제한 activation 한 번→실제 frame/main request 및 가능한 status→raw 보존→cancel/owner 종료→고정 CLI 재읽기를 수행한다. 미관측을 정상 player나 HTTP200으로 세지 않는다. 이후 공통 CLI episodes.resolve job/result·정직한 NeedsInput/raw 실패와 bounded media 관측/native host로 연결하며 이 계약 자체를 지원 완료로 표시하지 않는다.
