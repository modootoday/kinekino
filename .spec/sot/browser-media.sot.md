---
id: sot:kinekino/browser-media
title: 소유 browser의 media 응답 귀속 감사
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 소유 browser의 media 응답 귀속 감사

2026-10-19 SDK 후속 읽기 감사다. 현재 media_observer.rs는 미등록 소스이며 컴파일·실제 media 관측을 입증하지 않는다. [application 계약](../application/browser-provider.md)의 ticket/request generation과 document generation, 취소/EOF 및 bounded 원문 계약을 바꾸지 않는다.

## 읽은 코드와 공식 근거

준비된 MediaObserver는 request→ObservationIdentity(ticket/document URL/generation) resolver와 is_current를 외부에서 주입받는다. 응답 GetContent 완료 뒤에도 is_current를 확인하고 IStream 읽기는 callback 밖 STA poll_one에서 최대4096bytes, 대기/ready 합계4개로 제한한다. Drop은 closed를 세우고 ready/failure를 비우며 COM handler를 제거한다. 이 구조가 있어도 resolver가 현재 page만 반환하면 이전 요청의 늦은 응답을 새 문서에 귀속할 수 있다.

[Microsoft 응답 API](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2webresourceresponsereceivedeventargs?view=webview2-1.0.4129.50)는 응답 이벤트에서 committed request를 제공하고 network stack의 추가 header도 포함할 수 있다고 설명한다. [공식 응답 API 설계](https://github.com/MicrosoftEdge/WebView2Feedback/blob/main/specs/WebResourceResponseReceived.md)는 WebView 응답 처리와 host callback 간 순서 보장이 없다고 설명한다. 이 자료는 request 단계와 response 단계 COM pointer가 같은 identity라는 증거가 아니다. 조회한 pinned webview2-com-sys0.39.1의 RequestedEventArgs2는 RequestedSourceKind를 제공하며 network request ID/document ID의 대체가 아니다.

## 등록 전 불변 조건

- 최초 request 관측에서 작업 ticket과 문서 세대를 고정하고 response 및 비동기 GetContent 완료·prefix read 뒤에도 동일 identity/current를 확인한다. URL만 같은 요청 또는 response 시점 현재 page로 대신하지 않는다.
- COM IUnknown identity를 연결 키로 사용할 경우 두 이벤트의 실제 동일성 근거와 별개 wrapper/누락 매칭의 fail-closed 인수를 먼저 마련한다. pointer 주소 재사용은 소유 COM 참조와 lifetime 내에서만 다루고 unsafe Send로 STA 밖에 내보내지 않는다.
- 검증 가능한 연결 ID가 없다면 한 문서 수명의 관측기를 새 staged navigation 전에 닫고 새 소유 수명으로 분리하는 대안을 검토한다. 자동 redirect·iframe·같은 URL 재탐색에서 이전 응답을 새 문서로 승격하지 않는 실제 인수가 필요하다.
- 연결이 없거나 모호하면 성공으로 채택하지 않는다. bounded ledger 초과/async queue 초과는 명시 오류이며 임의 오래된 항목 제거 후 최근 세대에 재매핑하지 않는다. 외부 request에 추적용 header를 주입해 동작을 바꾸지 않는다.
- filter는 같은 기존 adblock 엔진·kind/method/referer 정책을 사용한다. 원문 저장·SHA·manifest 해석·result publication은 callback 밖에서 수행한다. prefix4096bytes는 전체 HTTP body가 아니며 Complete capture로 표시하지 않는다.

## 다음 실행 근거

직접 자식 프레임은 [소유 frame 문서 관측](browser-frame-document.md)에 구현했다. [실제 CLI 검증](../../verification/application/frame-document-cli-2026-10-19.md)에서 같은 소유 세션의 직접 자식 media-cdn HTTP200·탐색성공·DOM15115bytes를 확보했다. native frame ID와 부모·자식 세대를 분리했으며 media 요청은0이다. 이 문서의 response/prefix observer는 여전히 미등록이고 이 결과로 검증됐다고 표시하지 않는다.

카테고리→series→episodes.list의 실제 CLI 관측과 불변 결과는 확보했다. [watch 실제 CLI](../../verification/application/episode-activation-cli-2026-10-19.md)에서는 고정 actuator 뒤 frame.example.test main navigation 시작까지 관측했다. 선언 media-cdn URL의 fresh GET은 정상 player를 입증하지 못했으므로 이를 동등 요청으로 다시 합성하지 않는다. 다음 player-document-cli-r1에서 같은 세션의 새 문서 completion/DOM을 확보한 뒤 필요한 media 응답 수명을 연결한다.

준비된 MediaRequested는 현재 소유 WebView의 문서 URL/세대/native ID를 request 시점에 고정한 **후보**다. 실제 resource를 요청한 iframe/worker의 parent lineage를 입증하지 않는다. 실제 Referer가 없으면None이며 현재 page를 실제 header로 위조하지 않는다. 이 후보만으로 MediaReady나 jobs Completed를 만들지 않고, DOM 수집을 종료하지 않는다. 실제 response/prefix는 위 귀속 불변 조건을 충족해야 한다.

untrusted HTML·웹 메시지로 셸/임의 실행파일을 호출하지 않는다. stale ticket/document, cancel 선행, terminal 선행, EOF 후 callback, queue 한도와 실패 회귀 및 실제 manifest provenance/전체 재생은 각각 별도 증거다. 이 감사는 observer 지원 선언이 아니다.
