---
id: sot:kinekino/sot-003-adblocking
title: 광고 차단 규격
status: active
created: 2026-10-29
updated: 2026-10-29
tags: [player]
verify: manual
---
# 광고 차단 규격

## 세 층의 처리

1. 네트워크: WebView2 WebResourceRequested에서 요청 URL·출처·종류를 adblock-rust에 전달한다. 일치하면 빈 403 응답으로 종료하여 로드를 막는다. document-start 스크립트만으로 네트워크 차단을 대체하지 않는다.
   요청 필터 등록은 ICoreWebView2_22의 AddWebResourceRequestedFilterWithRequestSourceKinds(ALL)을 사용한다. 종전 AddWebResourceRequestedFilter는 외부 iframe 요청을 놓치므로 사용하지 않는다.
2. 화면: 신뢰된 고정 스크립트에서 광고 슬롯·광고 iframe·banner 이미지를 가진 링크를 CSS로 숨긴다. MutationObserver는 스타일 노드가 제거된 경우 재설치하며 매 프레임 전체 DOM을 스캔하지 않는다. 플레이어 iframe과 video는 포괄적으로 숨기지 않는다.
3. 탐색: 최상위 사이트는 HTTPS의 source.example.test, frame.example.test, media.example.test으로 제한하고 새 창은 기본 거절한다. 관측된 재생 이동만 같은 앱 뷰로 라우팅한다. `file:`, `javascript:`, 외부 프로토콜 및 광고 팝업을 OS로 전달하지 않는다.

탐색 허용 목록과 서브리소스 차단 목록은 다르다. CDN 호스트가 바뀔 수 있으므로 모든 외부 서브리소스를 거절하지 않는다. HTTP 메서드·리소스 종류·Referer를 실제 요청에서 읽으며 Referer가 없으면 현재 문서 URL로 대체한다. iframe별 정확한 initiator 및 최상위/하위 document 구분은 추가 검증 대상이다.

## 필터 소스 및 현재 범위

현재 포함한 `assets/filters.txt`는 조사에서 본 호스트 기반의 프로젝트 자체 시드 규칙이다. 범용 AdBlock과 동등한 커버리지를 주장하지 않는다. 앱 작업 디렉토리의 `filters.local.txt`에 ABP 문법의 추가 규칙/예외를 둘 수 있으며 시작 시 읽는다. 현재 라이브 업데이트는 없다. 출처는 [adblock-rust 문서](https://docs.rs/adblock/0.13.3/adblock/), [WebView2 요청 이벤트](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2webresourcerequestedeventargs).

후속 목록 배포는 EasyList·EasyPrivacy 및 한국어 목록의 출처·라이선스·체크섬·버전·갱신시간을 기록하고 최대 크기·문법 검증 후 원자 교체한다. 다운로드 실패 시 마지막 유효 목록을 유지한다. 임의 원격 JS/scriptlet을 내려받아 실행하지 않는다. 현재 구현은 일반 cosmetic 문법 전체를 해석하지 않고 사이트 전용 고정 셀렉터만 적용한다.

## 사용자 제어

광고 요청 차단은 기본 켬이다. 현재 토글은 네트워크 필터만 제어하며 cosmetic 필터는 유지한다. UI 문구도 광고 요청 차단으로 표시한다. 목표 UI에서는 사이트별 예외와 cosmetic 별도 토글을 제공한다. 재생 실패의 자동 복구라는 이유로 전체 보호를 끄지 않는다. 차단 카운터는 요청 수이며 제거된 광고 개수나 절감 바이트가 아니다.

## 미디어 내 광고의 경계

DOM 슬롯·서드파티 요청과 달리 영상에 직접 합성된 로고·홍보 문구, 같은 콘텐츠 스트림에 삽입된 광고는 현재 차단으로 제거되지 않는다. 조사 중 영상 자체의 워터마크도 보였다. 원본 영상을 가리거나 잘라내는 방식은 기본 제공하지 않는다. 광고 없는 재생이라는 표기는 웹 광고 제거 범위와 이 한계를 구분해야 한다.

## 검증 기준

- 관측된 광고 호스트 차단, media-cdn iframe 및 샘플 미디어 URL 통과, 예외 규칙 우선, 호스트 위장·권한 스킴 거절을 자동 검증한다.
- 실제 세 분류에서 차단을 켜고 재생시간 30초 이상 증가, 탐색 후 진행, 소리 및 전체화면을 수동 검증한다. 모두 통과하기 전 사이트 호환성을 완료로 표기하지 않는다.
- 광고 슬롯 숨김 후 플레이어가 남아 있고 빈 공간이 줄었는지 확인한다. 동일 DOM fixture에 동적 슬롯 추가/삭제 회귀 검사를 추가한다.
- 로그에는 서명된 미디어 URL·쿠키·광고 추적 쿼리를 기록하지 않는다. 카운트·호스트·단계·오류 분류만 저장한다.

실제 앱의 차단 ON 검증은 [재생 검증 정본](../../../verification/playback/sot-005-playback-validation.md)을 참조한다. 단순 초기 기동 시험의 차단 수 0과 외부 플레이어 재생 시험을 혼동하지 않는다.
