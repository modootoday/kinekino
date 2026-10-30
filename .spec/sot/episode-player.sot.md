---
id: sot:kinekino/episode-player
title: episode 원문에서 player 문서 대상 선택
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# episode 원문에서 player 문서 대상 선택

상태: 공통 Rust 정책/decoder와 필수 검사 및 실제 target 문서 보존을 확인했으나, 정상 player 의미 검증은 오류 페이지로 실패했다. CLI episodes.resolve·최종 media·native 재생은 아직 지원 완료가 아니다. [관측 원문](../../verification/application/episode-dom-2026-10-19.md), [browser 계약](browser-provider.md), [소유 수명 ADR](../../../adrs/player/browser-media-document-lifetime.md)을 따른다.

후속 전체 검사는231pass/16ignored·fmt/strictClippy/build를 통과했다. 실제 SDK target 문서 관측은1pass/8.90초로 raw/종료/readback을 확인했으나 본문이 404 Not Found를 표시한 오류 페이지였다. 이는 선언된 URL의 정상 player 의미 검증 실패이며 실제 수신 HTTP status는 아직 null/미관측이다. 선언 URL을 최종 media/정상 player로 게시하지 않는다. 상세 근거는 [resolver 검증](../../verification/application/episode-resolver-2026-10-19.md)을 따른다.

## 실제 입력 근거

보존 episode DOM의 script[data-page='app'][type='application/json']에서 /props/playing/body는 type=video, iframe=true, url은 HTTPS media.example.test의 /video/{32자리 ASCII hex}·query 없음으로 관측됐다. 원문 SHA는 `f98c71a4e95bb47fe9f26455c138a0e9595c51d45351cdf8afd53d35a35f10f9`다. source와 opaque URL의 실제 값은 private artifact 안에서 읽고 정책 검증에 사용하며 공개 진단으로 복사하지 않는다. frame.example는 이 관측에서 확인되지 않았으므로 URL을 합성하거나 검증된 분기로 표시하지 않는다.

## 책임과 계약

- domain PlayerDocumentLocation은 HTTPS/443·정확 host·관측 path 형태·userinfo/query/fragment 부재를 검증한다. Debug는 opaque 부분을 숨긴다. 이는 player 문서 대상이며 media URL admission이 아니다.
- media-catalog infrastructure decoder는 bounded UTF8 DOM의 정확히 하나인 app JSON script와 필수 url/props.playing/slug/content_id/body(type,iframe,url)를 읽는다. 동일 필수 키 중복·중복 script·깨진 JSON·타입/상한 위반은 거부한다. 관련 없는 다른 props는 제품 정책으로 승격하지 않는다.
- application EpisodeMarkupPort/player_target은 요청 episode, 관측 DOM final URL, JSON page URL, playing slug의 identity가 같고 content_id>0인지 확인한다. 관측된 video/iframe 타입만 받으며 target domain 정책으로 통과한 값에 원문 SHA·document generation·source pointer를 결합한다. DOM/schema 해석이나 URL 정책을 GUI/CLI 이벤트 안에 복제하지 않는다.
- infrastructure HtmlEpisodeMarkup은 위 decoder를 공통 application port에 연결한다. 외부 script/웹 메시지를 실행하거나 임의 프로그램/셸을 호출하지 않는다.

## player 문서 관측 목적

공통 browser의 ReadPlayerDocument purpose는 검증한 정확 target에만 접근하며 다른 main-document target·query·fragment·IP 우회를 거부한다. terminal Document는 DOM 확보일 뿐 media 성공이 아니다. 이 목적에서는 request 기반 media 관측을 켜지 않는다. 미등록 manifest observer를 지원으로 세지 않는다.

후속 실제 SDK fixture는 불변 episode 결과와 원문을 다시 검증하고 위 실제 application/decoder로 target을 선택한 뒤 fresh profile에서 player 문서를 읽는다. 이는 같은 session cookie·iframe Referer·POST 호환성을 입증하는 검사가 아니며 CLI resolver도 아니다. 성공/실패 raw와 취소 disposition·terminal·owner 종료·고정 CLI 원문 재읽기를 보존한다. 동작에 추가 요청 문맥이 필요하면 실제 근거를 바탕으로 명시 adapter 계약을 추가한다. POST를 URL-only GET으로 재구성하지 않는다.

## 남은 연결

필수 fmt/strictClippy/workspace test/build와 실제 player 문서 관측 후 CLI resolver job/result·raw 보존 계약으로 연결한다. 최종 media의 bounded prefix·실제 source/referer/response 귀속·private 주소/redirect 정책·취소/publication·native host open/control/회차 전환은 각각 후속 인수다. 이 source 단계로 전체 resolve나 media SSRF 경계를 완료로 표시하지 않는다.
