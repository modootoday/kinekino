---
id: sot:kinekino/sdk-cli-2
title: 카탈로그 공통 SDK·CLI 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 카탈로그 공통 SDK·CLI 계약

현재 구현 상태는 experimental이다. 공개 사이트 HTTP 응답 전 연결 실패를 관측했으며 실제 selector·목록·회차 성공을 검증하지 못했다. 기존 조건부 검증 명령 27/60 집계에 추가하지 않는다. [수집 관측](../../../verification/application/catalog-http-2026-10-19.md)과 synthetic parser 회귀는 구별한다.

사용자는 기존 실제 브라우저의 정상 접근을 관측했다. 이는 직접 HTTP 클라이언트의 connect 실패 원인 확정 자료가 아니다. TLS/ALPN/HTTP·쿠키·JS 실행 환경과 봇 탐지 가능성은 미확정이며 인증서/시스템 설정 변경이나 UA 위장만으로 해결했다고 주장하지 않는다. CLI 우선은 HTTP-only 제약이 아니다. 후속 WebView2 adapter도 GUI presentation 없이 명시 수명·공통 Rust 포트로 조립할 수 있다. complete network response와 DOM snapshot은 캡처 종류와 provenance를 구별해야 하며 서로 같은 exact network bytes로 취급하지 않는다. 이 경로의 실제 실행은 아직 없다.

## 입력과 조립

기본 SDK는 catalog.search/episodes.list를 unsupported로 반환한다. 명시 CatalogApi 주입 또는 Windows CLI --catalog-root ABSOLUTE_DIRECTORY가 실험 backend를 제공한다. CLI root와 --endpoint는 함께 쓰지 않는다. GUI 조립·브라우저 자동화는 없다.

- catalog.search: category=airing/completed/movies, page=1..10000(기본1). 현재 의미는 카테고리 목록이다. query/genres/sort 등 미정의 필드는 거부한다.
- episodes.list: series_url=https://source.example.test/c/<id>. query/fragment/userinfo/다른 origin은 거부한다.
- 두 명령은 대안 입력 result_artifact_sha256만 받아 저장 결과를 다시 읽을 수 있다. live 입력과 섞지 않는다. archive-only SDK/명시 provider host는 live_read_available=false/result_read_available=true로 구분하고 동기 네트워크를 실행하지 않는다. CLI의 stored-result 조립은 HTTP client 초기화와도 독립이다.
- context timeout_ms/idempotency_key/expected_revision은 지원하지 않는다. 전송의 전체 HTTP 예산은 15초, connect 5초, redirect 최대5회, body 최대2MiB이다. 자동 재시도는 없다.
- text search, genres, sort, frame.example 및 episodes.resolve·미디어 재생은 이 계약에서 구현하지 않는다.

## 원문과 결과

catalog.read_raw는 실패 receipt에도 남는 raw_artifact.artifact_id로 원문을 읽는다. body SHA가 아니라 metadata ID를 입력하며, offset은 decimal byte offset, limit은 기본16384/최대65536, encoding은 utf8 또는 명시 hex다. 매 chunk 읽기 전에 전체 metadata/body SHA·크기를 재검증한다. UTF8 codepoint 중간 offset·진행 불가능한 작은 limit을 거부하고 next_offset을 반환한다. hex는 원본 byte당 소문자 두 자리이며 chunk SHA/bytes는 인코딩 전 byte 기준이다. 네트워크와 HTTP client 초기화 없이 archive backend로 동작한다. 이 명령을 포함한 catalogue는61개이며 기존 검증27개 집계에 experimental 원문 조회를 자동 합산하지 않는다.

CatalogFetchPort → 원문 identity/URL 경계 검증 → CatalogArtifacts.save → CatalogParser 순서다. 수신 완료한 HTTP 오류/challenge/미인식 DOM도 해석 전에 보존한다. 응답 전 전송 실패 및 잘린/초과 body에는 완전 원문 artifact를 만들지 않는다. 실패를 빈 목록으로 바꾸지 않는다.

고정 root의 objects/<body_sha256>.body에는 exact bytes, captures/<artifact_id>.json에는 CaptureMetadata의 exact JSON을 게시한다. artifact_id는 metadata JSON SHA256이다. 같은 body를 재사용할 수 있으며 각 관측 metadata는 별개다. 기존 파일은 덮어쓰지 않고 동일 bytes인지 재확인한다. 일반 로컬 파일/디렉터리와 reparse guard는 기존 immutable_storage_paths, 게시 구현은 media-file-publication을 공유한다. 게시 실패로 생긴 참조되지 않은 body의 자동 삭제·retention은 없다.

RPC는 원문 inline 대신 RawArtifact(body SHA/decimal bytes/두 상대 경로/artifact ID)를 반환한다. CaptureMetadata는 requested/final URLs, redirect chain, UTC RFC3339 observed_at, parser revision, HTTP status/content headers와 body identity를 포함한다. 성공은 title/series/entries/next_page와 두 metadata를 반환한다. site_verified는 현재 false다. 이후 parser/HTTP 해석 오류는 error.details.raw_artifact/capture에 같은 참조를 포함한다. 수신 자료 없는 전송 오류는 이 필드가 null이며 구조화 transport 진단만 남긴다.

browser jobs는 capture_kind=dom_snapshot/status=null/browser provenance를 보존한다. DOM의 redirects는 빈 배열이며 HTTP redirect chain을 만들지 않고 실제 navigation 관측을 별도 기록한다. 기존 HTTP metadata는 capture_kind=http_entity/status=수신값이다. BrowserResponseBody/ManifestPrefix enum은 후속 표현을 위한 구분이며 현재 DOM executor 지원을 뜻하지 않는다.

명시 --serve-provider --job-root/--catalog-root/--browser-profile-root의 jobs.submit이 catalog.search/episodes.list를 실행한다. 입력 source SHA는 정규화된 request descriptor이며 웹 본문 고정이 아니다. 최종 results/<sha256>.json은 종류·operation·source descriptor SHA와 typed 결과를 저장한다. receipt는 큰 목록 대신 참조를 반환하며 같은 명령의 result_artifact_sha256 입력은 결과와 raw 참조/metadata/source identity를 다시 읽고 검증한다. 결과 JSON 상한은2MiB다. [접합 검증](../../../verification/application/browser-provider-cli-2026-10-19.md)은 실제 site 성공과 분리한다.

## 책임과 검증 상태

packages/media-catalog는 URL/HTTP/HTML 경계다. root application/catalog는 입력·원문 보존 순서·SDK 결과/오류 계약, infrastructure/catalog는 Windows 저장소·parser/HTTP 조립, application/api/catalog는 Request/wire adapter, CLI는 명시 옵션 조립만 담당한다.

root 통합은 [검증 근거](../../../verification/application/catalog-sdk-2026-10-19.md)의 201pass/9ignored·fmt/strictClippy/bin build와 고정 실제 CLI metadata/단일 전송 실패 경계를 통과했다. 수집 패키지 synthetic 회귀와 실제 사이트 성공은 구별한다. 실제 목록/회차·브라우저 엔진 경로는 아직 미검증이다.
