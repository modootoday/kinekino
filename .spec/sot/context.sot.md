---
id: sot:kinekino/context
title: 공개 카탈로그 읽기 context
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 공개 카탈로그 읽기 context

상태: 2026-10-19 독립 `packages/media-catalog` 구현. 최신 원문 분리·진단본의 fmt/6회귀/strict Clippy는 통과했다. 실제 세 카테고리는 HTTP 응답 전 connect 실패이며 selector 실증과 제품 CLI 지원은 미완료다. [실제 관측과 한계](../../verification/application/catalog-http-2026-10-19.md), [CLI 이관 대조](../../../plans/player/presentation/cli-migration-gaps.md)를 따른다. root workspace/API/CLI에는 미등록이다.

domain은 Category/PageLocation·안정 ID·오류·목록·원문 provenance, application은 CatalogReadPort, infrastructure는 SourceHttp와 HTML parser를 소유한다. root 역의존·GUI/브라우저 자동화·미디어 실행이 없다. 기존 domain/navigation·domain/episodes 및 episode-observer.js와 중복되는 URL/DOM 규칙은 이관 대조 대상이며 기존 구현을 삭제하거나 전체 browser navigation 규칙을 더 좁은 catalog 규칙으로 대체하지 않는다.

첫 범위는 방영중/종영/극장판, /c/작품, /e/회차 HTML 읽기다. https·source.example.test·443·userinfo 없음·fragment 없음으로 제한하고 카테고리의 단일 page=1..10000만 허용한다. genres[]·sort·검색 query는 현재 미지원이다. 카테고리 성공을 catalog.search/필터 지원으로 확대하지 않는다. frame.example 이동·영상 resolve는 이 context의 현재 HTTP 범위 밖이다.

자동 redirect를 끄고 매 hop의 URL/리소스 종류를 검증한 뒤 최대5회까지 요청한다. 전체 read 예산15초·connect5초·본문2MiB와 UTF-8 HTML만 허용한다. 상태401/403·challenge/login marker는 AccessRequired, 그 외 비200은 HttpStatus, 해석 못한 화면은 UnrecognizedPage로 구분한다. 우회·auth·cookie 취득은 구현하지 않는다. requested/final URL·redirect chain·수신 원문 SHA256/bytes·수집 시각·parser revision을 반환한다.

작품/회차는 기존 `.wr-subject a[href]` 선택과 회차 역순 규칙을 보존했다. category도 같은 행 selector를 쓰는 첫 가설이며 실제 HTML 검증 전이다. pagination은 a.pg_next 또는 rel=next의 같은 카테고리 page 링크만 반환하며 자동 전체 순회를 하지 않는다. 목록 항목은 최대2000개, 제목512bytes, URL 기반 안정 ID로 중복을 제거한다. 명시 .empty_list만 빈결과로 인정하는 것도 초안 규칙이며 실제 사이트의 빈결과 구조를 확인해야 한다. DOM 불일치를 정상 빈목록으로 숨기지 않는다.

synthetic fixture 회귀를 먼저 작성했다. 실제 사이트 fixture로 주장하지 않는다. 파서/URL 오류 회귀와 필수 Rust 검사는 PM 슬롯 예약 후 실행한다. 이후 실제 read-only HTTP 수집 결과와 제한/실패를 관측 문서로 분리하고 SDK typed port에 인계한다. 제품 CLI 지원 판정은 SDK 등록 및 실제 CLI 수직 인수 후에만 가능하다.

## 검사 checkpoint와 원문 인계

최신 카테고리 구조는 [보존 DOM 기반 파서 검토](../../verification/application/catalog-dom-parser-2026-10-19.md)를 따른다. v2-board-grid는 실제 board-list/grid 카드의 desc/ttl을 추출하며, 과거 .wr-subject category 가설을 현재 구조로 간주하지 않는다. v2 소스는 포맷만 확인했고 offline/실제 CLI 인수는 중앙 검사 대기다. 실제 현대 empty marker와 나머지 두 category/series/episode는 아직 관측 범위를 확대하지 않는다.

### 전송 경로와 최신 사용자 관측

사용자는 기존 실제 브라우저에서 source 접근이 정상적이었다고 확인했다. 이는 현재 직접 reqwest 요청의 connect 실패와 다른 경로의 관측이다. 봇 탐지 가능성은 가설이며 HTTP 응답/원문이 없는 기존 실패로 확정하지 않는다. 직접 클라이언트와 WebView2/Chromium은 TLS/ALPN/HTTP·쿠키·JS 실행·관측 수명이 다르다. UA 변경만으로 해결하거나 인증서 검증을 끄지 않는다.

CLI 우선은 HTTP-only 제약이 아니다. 기존 infrastructure/browser의 WebView2/Wry 수집·회차·resolve를 presentation 없이 명시 provider host 수명·공통 application 포트·typed 이벤트·광고차단 정책으로 구동하는 후보를 SDK가 평가한다. 기존 Browser::new는 parent handle을 받고 Rc 기반 수명을 소유하므로 현재 CatalogReadPort의 Send+Sync를 위해 unsafe Send를 추가하지 않는다. 엔진 전용 스레드/메시지 포트와 host 시작·종료·timeout/cancel·세대 일치 계약이 먼저 필요하다. 사용자 GUI 클릭 자동화나 제품 GUI 선행 인수로 돌아가지 않는다. 새로운 브라우저·GUI 실행은 이번 조사에 없다.

### 후속 진단 소스 checkpoint

SDK와 변경 범위를 조율한 뒤 transport_diagnostic 모듈을 분리했다. client_build/request_send/body_read와 reqwest flags, 최대32노드의 source/io.get_ref 그래프, 노드별 IO kind/OS code, 첫 non-None OS code, 중복/한도 잘림을 보존한다. 128bytes 제한 내 정확히 일치하는 connector/tunnel label만 enum hint로 남기고 원문 Display/Debug/URL/자격증명을 직렬화하지 않는다. hint는 원인 확정이 아니다. private hyper 타입 downcast와 새 TLS 의존은 없다. 공통 fetch/raw/read/URL 포트는 유지하며 확장된 진단은 Box로 보관한다.

노드 payload·앞 OS code 보존·깊이/순환·원문 sentinel 미노출·phase 회귀를 추가했다. rustfmt 작성은 종료0이며 이 후속 코드의 컴파일/회귀/Clippy는 SDK 중앙 묶음 대기다. 실제 HTTP 요청은 추가하지 않았다.

최초 checkpoint는 `runtime/quality-review/catalog-r1/logs/{fmt,test,clippy}.log`이며 독립 manifest fmt·5개 URL/parser 회귀·all-target strict Clippy가 모두 종료0이었다. 실제 HTTP는 아직 실행하지 않았다. 이 결과를 다음 원문 분리 수정의 검증으로 확대하지 않는다.

SDK 요청으로 CatalogFetchPort::fetch→RawDocument와 parser::parse_document(&RawDocument)를 추가했다. RawDocument는 HTTP status/content-type/content-encoding·수신 완료한 bounded exact bytes·provenance를 가진다. parser는 raw를 소비하지 않으므로 SDK가 성공·HTTP 오류·challenge·파서 실패에서도 원문을 artifact로 보존할 수 있다. 기본 RPC에 본문을 inline하는 계약은 아니다. 기존 CatalogReadPort는 fetch와 parse를 조합한 편의 포트다. 초과본문·전송실패·deadline 중단은 완전 수신 원문이 없으며 완전원문 SHA를 발명하지 않는다. 이 후속 소스와 원문 보존 회귀는 다음 예약 슬롯 검증 대기다.
