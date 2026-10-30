---
id: evidence:kinekino/20261029114723
title: 실제 카테고리 DOM의 파서 불일치
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 카테고리 DOM의 파서 불일치

고정 CLI가 수집한 방영중 DOM은101495bytes이며 SHA256은 `56b3430f96481e8d6106ed8a39ec6f2693f1ad3f8c7bc848ef5523c9d670e109`이다. 원문은 runtime/quality-review/browser-provider-cli-r2/actual-cli/provider-actual/catalog/objects/ 아래 같은 SHA의 .body로 보존했다. 조회로 hash 일치를 재확인했고 원문을 수정하지 않았다. capture metadata는 같은 catalog/captures/fae1aed877f01dc026148158593a8974231a412e213b7319b3bccc1623f568ca.json이다.

metadata는 dom_snapshot/status=null·차단 요청true/설치true·요청71/차단0을 기록한다. 이는 network raw·HTTP200 또는 광고 차단 건수 양수의 증거가 아니다. 실제 DOM에는 방영중 목록의 h1.page-title과 section.board-list 내부 grid-wrap/grid/row/box/text/a.desc/.ttl 카드가 있다. challenge가 아닌 목록 구조를 확인했다. 이미지 링크와 설명 링크가 같은 작품을 가리키고 설명 링크에는 ttl 외 날짜/회차 meta가 포함된다.

기존 v1 파서는 .wr-subject만 찾았으므로 UnrecognizedPage를 반환했다. v2-board-grid는 해당 category 구조에서 desc/ttl만 추출해 이미지 중복·meta·목록 밖 navigation을 제외한다. row와 설명 링크 개수가 다르면 손상으로 거부하며 빈 grid를 정상 빈목록으로 바꾸지 않는다. 현대 목록의 실제 빈결과 marker는 아직 관측하지 않았다.

원문 작품 URL에는 제목 내 slash가 path에 남는 사례가 있다. 기존 PageLocation의 단일 slug segment 가정도 해당 정상 링크를 거부하므로 /c/·/e/ 뒤 비어있지 않은 복수 segment를 허용하도록 보완했다. https/host/443/userinfo/fragment/query 제한은 유지하고 URL 길이8192bytes를 제한한다. 검색·genres[]·sort 지원으로 확대하지 않는다.

실제 구조를 최소화한 Rust category_fixture.rs는 가짜 제목/URL/metadata만 포함한다. 원문 작품명·쿠키·token·서명 URL·script·thumbnail은 복사하지 않았다. 제목과 안정ID/순서, slash 제목, image/meta/navigation 제외, 누락 제목/행 불일치 거부, 미관측 empty 거부 회귀를 추가했다.

이 checkpoint는 source/rustfmt 종료0만 확인했다. cargo·실제 네트워크·GUI 실행은 하지 않았다. 최신 v2의 offline 파싱·필수 검사·실제3category/series/episode 인수는 SDK 중앙 후속 범위다. 기존 실패 capture의 parser_revision과 새 파서 revision을 소급하여 덮어쓰지 않는다.

## SDK 후속 오프라인 검증

catalog-parser-raw-r1 제품 source는 fmt/strictClippy와 workspace 218pass/12ignored를 통과했다. 이후 fixture 보완만 적용해 fmt/strictClippy 및 jobs 5pass/3ignored를 확인했다. 고정 CLI SHA는 `00cf42bd85154d3d3e4081f9d3ffcc311f2d619326cf03457c542b5a3a5e4dea`다.

이미 빌드된 jobs harness를 직접 실행한 `catalog_replay::saved_actual_dom_is_reparsed_without_mutating_original_capture`는 1pass/7.06초/exit0이다. 추가 컴파일·네트워크 요청 없이 보존 DOM을 v2로 재해석해 58개 항목을 얻었으며 next_page=null이다. CLI catalog.read_raw의 모든 UTF-8 chunk를 이어 원문 101495bytes와 대조했고 원 capture의 v1 revision·SHA·내용 불변을 확인했다. 해석 결과는 runtime/quality-review/catalog-parser-raw-r1/offline-cli/catalog-replay/interpretation.json, 실행 로그는 같은 checkpoint의 logs/offline.log에 보존했다. 이는 현재 사이트 3category·series·episode 성공을 대신하지 않는다.

## 실제 3category 실패 뒤 v3 보완

실제3category에서는 방영중 timeout, 종영·극장판 DOM 저장 후 v2 UnrecognizedPage를 얻었다. [읽기 분석](catalog-three-category-parser-analysis-2026-10-19.md)은 두 원문의 section.board-list 조상 부재와 각각 grid-wrap/grid 1개·row/desc/ttl 24개를 확인했다. SDK v3-category-grid source는 section 조상 요구를 제거하고 .grid-wrap>.grid>.row와 .text a.desc/.ttl 및 기존 row/link 수·리소스 유형·상한/empty 거부를 유지한다. 기존 section fixture와 wrapper 없는 가짜 종영·극장판 fixture, 원본을 보존한 별도 offline ignored 재해석을 추가했다. 현재 이 후속 검사는 진행 중이며 성공으로 선기록하지 않는다.

현재 next_page=null은 selector가 다음 링크를 찾지 못했다는 결과다. 실제 disabled/last-page 관측이나 전체 pagination 지원 증거가 아니며, unknown과 명시 끝페이지를 구별하는 후속 계약/인수를 남긴다. 방영중 timeout 원인은 아직 미확정이며 진단 snapshot 보존을 추가해 원인 판별을 돕는다. timeout을 wrapper 수정으로 해결했다고 주장하지 않는다.
