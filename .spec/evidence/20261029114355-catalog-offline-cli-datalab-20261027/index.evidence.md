---
id: evidence:kinekino/20261029114355
title: 보존 카탈로그·회차 오프라인 조회 — 2026-10-27
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 보존 카탈로그·회차 오프라인 조회 — 2026-10-27

과거 actual DOM·회차 결과를 새 호스트에서 오프라인 재해석/조회했다. 현재 사이트 수집·모든 selector·영상 해석·재생·광고차단·GUI 인수로 계상하지 않는다. 원본을 직접 시험하지 않고 `runtime/verification/catalog-offline-capture-copy-r1/{airing,library,episodes}`에 복사해 읽었다. 추론/다운로드/수동GUI/WARP/push 없음.

## 실행·입력 근거

로그는 `runtime/verification/source375-datalab-b0-r1-logs/784~789-*`, 실제 결과는 `runtime/verification/catalog-offline-replay-actual-r1/`에 보존했다. 원 CLI SHA256=1DF8ADD0F6210B4E7C38CEC46943D0EA6647530086902DCB752752D4C735378D를 fixture가 복사 전후 확인했다. 784는778의 jobs producer를 사용했고,785는 회차 archive 시험을 추가한 no-run exit0/10178ms다.

| 새 호스트 실제 실행 | 관측 결과 |
|---|---|
| 784 saved_actual_dom_is_reparsed_without_mutating_original_capture | 과거 방영중101495B/SHA56b3430f96481e8d6106ed8a39ec6f2693f1ad3f8c7bc848ef5523c9d670e109, 해석58개. CLI catalog.read_raw2chunk를 이어 전체 원문과 비교. capture parser revision 보존 |
| 784 saved_library_category_doms_are_reparsed_without_network | 과거 completed221453B·movies205480B, 각각24개. 원 observed_at(2026-10-19)와 capture parser revision을 보존하고 새 해석 revision은 source-html-v3-category-grid로 별도 기록 |
| 786 saved_episode_result_is_reopened_by_fresh_cli_without_network | 실제 회차 결과SHA b1a2af4afa4491ef5743f2a1095d7b9973b6bc4268f1edf70dd9482446bea3bc를 공통 CatalogFiles.read와 fresh CLI episodes.list(result_artifact_sha256)로 조회. entries4 전체 일치·각URL Episode 정책 검증·원 결과/DOM 재조회 동일 |

784는2/2 exit0/2314ms·실제CLI2회이며 library 파싱은 SDK 직접 실행이다.786은1/1 exit0/1625ms·실제CLI1회다. scope.json/interpretation.json의 new_network_requests=0은 해당 오프라인 경로에 한정한다. 프록시 전체 네트워크 감사나 현재 사이트 응답 관측이 아니다. 787 해당 Native jobs strict·788fmt는 exit0다. 첫 실패 없음. 기존 원 capture의 실패와 후속 성공 이력은 소급 변경하지 않았다.

## 원 증거·경계

- airing 원 root: `runtime/quality-review/browser-provider-cli-r2/actual-cli/provider-actual/catalog`, metadata fae1aed877f01dc026148158593a8974231a412e213b7319b3bccc1623f568ca, observed_at=2026-10-19T03:27:24.887Z.
- library 원 root: `runtime/quality-review/catalog-parser-raw-r1/actual-cli/provider-three-categories/catalog`, completed metadata646d64e22463c02d789ac4109b359321cba91c50a2014f312acef1bdf661088b, movies4c1900b11d5989033d93212da513dbcc90de8ee908e34f5a1322d0018b9fba97.
- episodes 원 root: `runtime/quality-review/series-cli-r1/actual-cli/provider-series/catalog`, DOM134681B/SHA f0c73cacbfc4526bffada0012912c6e01429c376a8ef496f89251ed34f01d8c3, metadata cf48a62cb39a8c34f6224370c973f33982a026432eafd080b8fc275be31c3550.

이전 live series 성공은 원 호스트/원 시점의 기록이며 이번 새 호스트 live 성공이 아니다. 58/24/24는 보존된 한 페이지 해석 결과이며 pagination 전체 인수나 명시 끝페이지 확인이 아니다.

## 시험 핀·잔여

790 전후 검증은 exit0/40638ms로 종료했다. 전체2244 입력 content/identity 동일이며 아래 진행 중 문구는 종료 전 기록이다.

789 고정 완료: SOURCE.md SHA256=DFB0EB9021A8ED965D2E9FCBF43DF08B63C0C0DC74BE04EC369458C9171F150D, inventory SHA256=ABD3A6013D739B0AB5109E446433785CAA8B2176BAD4146017B833EF4C436A6D, files2244. `runtime/verification/source375-episode-archive-input-r1/`에 있다.790 전후 동일 검증은 진행 중이며 아래 고정 진행 중은 이전 이력이다.

제품 정책 코드는 유지했다. `tests/jobs/catalog_replay.rs` SHA256=6DB8E175B89F2BBCC2A2134DD820B87378FEACD85EE4AFF411357E28008DF518, preimage `runtime/verification/catalog-replay-episode-preimage-r1/catalog_replay.rs` SHA256=9F382415FFC5343C12BEE5A1983882C50B6F1E8E89660DD92BBB68EDEDBCDC82. 784는 SOURCE94211783,786은 시험 전용 확장 후 실행했다.789 새 입력 고정은 진행 중이다. 전체190/정상bins/전체strict를 새 시험 입력에서 다시 실행한 것으로 계상하지 않는다. 원 WIP/소유권·private overlay·품질0/9·보류 유지.

기존 modoo.today test-fixture-and-mock-build 본문 A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100의 격리 입력·새 실행/과거 관측 구분을 적용했다. live 탐색·resolve·재생과 전체CLI/AI/GUI 인수는 남는다.
