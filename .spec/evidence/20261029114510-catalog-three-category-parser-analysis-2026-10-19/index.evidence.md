---
id: evidence:kinekino/20261029114510
title: 카테고리 DOM과 v2 목록 파서 대조
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 카테고리 DOM과 v2 목록 파서 대조

상태: **원문·source 읽기 전용 분석**, 2026-10-19 12:55 KST. SDK가 확보한 completed/movies 두 DOM과 observation JSON만 확인했다. movies-observation.json이 이미 존재해 함께 분석했다. airing timeout/worker 수명은 범위에서 제외한다. 제품 코드·기존 문서·원문 변경, cargo·새 parser 시험·네트워크·브라우저 실행은 없었다. source는 SDK가 수정 중이므로 아래 v2 경로/줄은 이 관측 시점 기준이다.

기준: [카탈로그 계약](../../player/application/api/catalog.md), [기존 DOM 파서 검증](catalog-dom-parser-2026-10-19.md), [provider CLI 검증](browser-provider-cli-2026-10-19.md). SDK가 후속 v3 wrapper 회귀와 수정·실제 검증을 소유한다. 이 분석은 v3 통과 또는 사이트 전체 검증이 아니다.

## 확보 원문

공통 로컬 기준 경로는 `runtime/quality-review/catalog-parser-raw-r1/actual-cli/provider-three-categories/`다. 실제 제목·콘텐츠·서명 URL·쿠키는 이 문서에 복사하지 않는다.

| 대상 | observation | 원문 위치(catalog 기준 상대) | bytes·직접 조회 SHA256 |
| --- | --- | --- | --- |
| completed | completed-observation.json | objects/af85cbaaeae7cc28baa2d985481fe9ce46e5e117439464a6a892194a348059a2.body | 221453 / af85cbaaeae7cc28baa2d985481fe9ce46e5e117439464a6a892194a348059a2 |
| movies | movies-observation.json | objects/2badebea2baeca62aaf8e03412eb50eb51e07b4a8431b1c9ba2a80f7c5801f23.body | 205480 / 2badebea2baeca62aaf8e03412eb50eb51e07b4a8431b1c9ba2a80f7c5801f23 |

양쪽 observation은 needs_input, provider_error=unrecognized_page, parser_revision=source-html-v2-board-grid이며 오류 문구는 `Catalog layout not recognized; empty result unproven`이다. capture_kind는 dom_snapshot, HTTP status는 null이다. HTTP 원본 bytes나 HTTP200 관측으로 바꾸지 않는다. blocking_requested/installed=true, blocked_count=0은 당시 저장 관측이며 모든 광고가 제거됐다는 증거는 아니다.

## 구조 관측과 실패 경로

아래 수량은 원문 태그·class·href 패턴의 정적 텍스트 관측이다. Rust scraper를 새로 실행한 CSS query 결과라고 표시하지 않는다. 두 원문의 관련 태그 순서에서도 `.grid-wrap → .grid → .row → .text → a.desc → .ttl` 카드 구조를 확인했다.

| 구조 표식 | completed | movies |
| --- | --- | --- |
| section 태그 / board-list class | 0 / 0 | 0 / 0 |
| grid-wrap / 정확 grid class | 1 / 1 | 1 / 1 |
| 정확 row / a.desc / ttl class | 24 / 24 / 24 | 24 / 24 / 24 |
| `/c/` 상대 series href | 48 | 48 |
| wr-subject / empty_list class | 0 / 0 | 0 / 0 |
| h1 | 1 | 1 |
| rel=next / pg_next class | 0 / 0 | 0 / 0 |

48개 series href는 이미지·설명 anchor가 함께 있는 구조와 부합하며 48개 목록 항목이라는 뜻은 아니다. 실제 parser의 description-anchor별 식별자·중복·제목 검증은 수정 후 SDK가 실행해야 한다.

`packages/media-catalog/src/infrastructure/parser.rs:159–167`은 Category이면서 **section.board-list** 아래 grid가 있어야 modern_category를 선택한다. 두 DOM에는 이 조상이 없으므로 현대 카드 경로가 선택되지 않고 `.wr-subject a[href]`로 내려간다. 해당 표식도 없어서 entries는 비고, `:212–218`의 empty_list 미입증 조건이 UnrecognizedPage를 반환한다. 저장 오류와 일치하는 source 경로다. row/title 수 불일치 검사(`:170–178`)는 현대 경로가 선택되지 않아 이번 실패의 직접 원인이 아니다.

`domain/location.rs`는 completed/movies를 Category로 분류하고 category?page=1을 허용한다. 카드 링크는 Category의 예상 Series(`/c/`) 유형과 구조상 부합한다. 당시 실패는 location 오류가 아니라 목록 wrapper 선택 경계다. 전체 URL·모든 링크의 검증 성공까지 이 정적 관측으로 주장하지 않는다.

## 최소 수정 권고와 남은 가설

권고는 **section.board-list를 필수로 고정하지 않고, 검증된 단일 `.grid-wrap` 안의 목록 grid/card 경계를 명시 인식**하는 것이다. 페이지 전체 `.grid` 또는 `a[href]`를 폭넓게 긁는 fallback은 메뉴·배너·추천 항목을 목록으로 섞을 위험이 있다. 직접 child 구조·단일 컨테이너·row당 description anchor/ttl 1개·예상 ResourceKind·행 수 대조·중복 ID·상한을 유지한다. 컨테이너 모호성·깨진 카드·empty 상태는 계속 fail-closed한다.

pagination은 두 DOM에 현재 selector 표식이 없어 **다음 페이지 없음이 증명된 것은 아니다**. v3 수정의 목록 인식과 별도로 실제 paging 구조/명시 disabled·last-page를 확인할 항목이다. 이번 분석에서 pagination을 임의 추정하거나 전체 문서의 query 링크를 next로 채택하지 않는다.

SDK 인수 회귀는 (1) 기존 section wrapper fixture 유지, (2) wrapper 없는 신규 container fixture, (3) 여러 grid/메뉴·배너 외부 링크 혼재/깨진 row/중복·유형오류/명시 empty·challenge 거부, (4) 두 고정 원문의 SHA를 유지한 Rust parser actual 대조와 별도 새 출력 보존이다. 원문 수정·제목 문자열 하드코딩·빈 성공 fallback으로 맞추지 않는다. 수정된 revision과 실제 CLI 결과가 확보되기 전 site_verified나 세 카테고리 완료 상태를 올리지 않는다.
