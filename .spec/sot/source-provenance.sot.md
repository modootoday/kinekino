---
id: sot:kinekino/source-provenance
title: 카탈로그·회차 source provenance 경계
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 카탈로그·회차 source provenance 경계

2026-10-19 [공통 API 이관 조사](catalog-episodes-playback-api-handoff.md)의 출처·URL·browser 책임 자료다. 새 사이트 접속이나 응답 수집 없이 현재 source와 기존 정본만 읽었다.

## 현재 출처 흐름

1. [GUI workspace](../../../../../../src/presentation/workspace.rs)는 카테고리 주소 탐색을 연다. Rust typed 검색 결과 수집기는 없다. [최초 사이트 관측](../../browser/sot-001-browser-research.md)은 목록/회차/팝업/iframe을 구별했으며 JSON API·검색/연도/요일 파라미터·SSR 범위는 미확인으로 남겼다. 확인된 장르/정렬/다음 쪽 표본을 모든 query 매핑으로 일반화하지 않는다.
2. [기존 episode observer](../../../../../../assets/episode-observer.js)는 top-level source.example.test의 c/e 경로에서 h1·series href·wr-subject 회차 href를 읽고 DOM 순서를 뒤집어 전달한다. 숫자 회차나 제목으로 URL을 합성하지 않는다. 이 JS는 기존 이관/최소화 대상이며 새 CLI 정책을 여기에 추가하지 않는다.
3. [browser IPC](../../../../../../src/infrastructure/browser/mod.rs)는 message512,000 bytes 이내, 허용 origin URI와 현재 document URI 일치, EpisodePage.page와 IPC URI 일치를 검사한다. [EpisodePage](../../../../../../src/domain/episodes.rs)는 HTTPS/정확 host/443/credential·query·fragment 없음, c/e 경로·제목512 bytes·목록1..2000을 검사한다. [EpisodeQueue](../../../../../../src/application/episode_queue.rs)는 현재 page와 일치한 목록만 받고 URL 중복을 제거한다.
4. [EventSink](../../../../../../src/infrastructure/browser/events.rs)는 generation을 전달하고 player page 외 media를 버린다. [location](../../../../../../src/infrastructure/browser/location.rs)은 source 변경 때 generation을 갱신한다. 비동기 content callback은 세대를 고정하고 현재 GUI가 오래된 세대의 결과를 버린다. 이 판정은 새 CLI에도 필요한 application 정책이다.
5. [요청 filter](../../../../../../src/infrastructure/browser/requests.rs)는 광고 판정 후 비차단 media 후보를 전달한다. [응답 observer](../../../../../../src/infrastructure/browser/responses.rs)는 MPEGURL MIME 또는 media-cdn의 txt 응답을 대상으로 앞4,096 bytes를 읽는다. [MediaSource](../../../../../../src/domain/media.rs)는 EXT 헤더 내용으로 HLS를 판정하므로 misleading 확장자도 취급한다.

이 흐름은 관측 출처이지 publisher authenticity·완전한 요청 시점 association을 보증하지 않는다. [기존 전환 검증](../../../../verification/playback/sot-008-transition-reliability.md)도 같은 URL reload/iframe과 원래 요청의 완전한 결합이 남았다고 명시한다. 응답 callback의 generation만으로 새 resolve ticket의 원시 요청과 완전히 연결됐다고 표시하지 않는다.

## validation의 차이와 포트에서 필요한 보강

| 현재 코드 사실 | 공통 포트의 인수 요구 |
|---|---|
| navigation_allowed는 HTTPS와3개 정확 host를 확인. EpisodePage보다 port/credential/query 규칙이 느슨함 | page·episode·media 용도별 URL 값 객체와 정책을 명시. 더 느슨한 검사를 공통 API에 그대로 전파하지 않음 |
| media_candidate는 임의 HTTPS host의 media 확장자, from_manifest는 HTTPS·허용 Referer·EXT 내용으로 판정 | CDN 후보의 관측 route/session/원본회차·response 조건을 결합. MIME·확장자·hls bool만으로 성공/안전/전체영상 승인 금지 |
| 원래 MediaSource는 URL/Referer/hls만 보유 | source handle/lease, provider/episode, session/generation, 관측 시점·policy revision·만료/인증 미상 상태를 분리 |
| request 후보는 성공 응답을 아직 확인하지 않았을 수 있음 | candidate/observed manifest/selected source/actual opened를 구별. 실패·미관측을 빈 결과나 resolved 성공으로 대체하지 않음 |
| Loaded 이벤트만으로 episode 목록·media의 출처를 확정할 수 없음 | fresh ticket·document/route·snapshot 관측을 확인하고 superseded/cancel/timeout 결과를 차단 |

signed URL/query/header는 메모리 전용 lease로 취급하고 CLI·로그·프로젝트에 무분별하게 노출하지 않는다. 출력에는 안전한 provider/host·source handle·지원 범위와 오류를 사용한다. 예전 v 값→iframe ID의 변환 규칙은 미확인이므로 추측해 주소를 만들지 않는다. blob/HTML 주소를 mpv media URL로 간주하지 않는다.

## 광고 차단과 브라우저 의존

[adblock adapter](../../../../../../src/infrastructure/adblock.rs)는 seed filter와 filters.local.txt를 구성하고 WebView2 요청 context/method/source로 판정한다. GUI checkbox는 browser의 enabled 상태를 바꾼다. cosmetic/media/observer 주입 JS도 별도 기존 부채다. application은 차단 정책/사용자 예외/활성 여부의 revision과 결과를 공유하고 COM 이벤트 처리·네트워크 I/O는 infrastructure에 둔다. 실패 시 차단을 조용히 해제하지 않는다.

native mpv의 후속 playlist/segment/key 네트워크 요청이 WebView2 filter를 통과하는 것은 아니다. 현재 protocol whitelist도 동일한 광고 필터·host/DNS 접근 정책을 증명하지 않는다. Rust HTTP/HTML adapter가 동등한 기능·검증 수준이면 우선하되, watch 버튼에 정적 href가 없었던 기존 관측 때문에 HTML parser만으로 resolver를 대체할 수 있다고 단정하지 않는다. browser가 필요한 기능은 독립 actor/port 능력으로 공개하고 GUI 조작 자동화로 숨기지 않는다.

## 남은 지원 범위

카탈로그 typed 결과/검색 parameter/빈 목록의 완료 신호, provider별 URI 정규화, 요청 시점·프레임 association, cookie/인증·토큰 만료·재resolve, 후보 선택과 nested media fetch 정책, 독립 browser runtime, JS 최소화의 동등성 fixture가 남는다. 기존 3개 웹/네이티브 표본은 [과거 재생 검증](../../../../verification/playback/sot-005-playback-validation.md)의 제한된 근거이며 이번 조사에서 재실행하지 않았다.
