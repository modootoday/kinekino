---
id: evidence:kinekino/20261029114435
title: 카테고리 wrapper v3와 browser 실패 진단
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 카테고리 wrapper v3와 browser 실패 진단

## 재현과 분리

[실제3category 실패](browser-provider-cli-2026-10-19.md)의 종영·극장판은 원문 저장에 성공했으나 section.board-list 조상이 없어 v2 modern 경로가 선택되지 않았다. [보존 DOM 분석](catalog-three-category-parser-analysis-2026-10-19.md)을 근거로 원문을 수정하지 않고 v3-category-grid와 최소 가짜 wrapper 회귀를 추가한다. 방영중 timeout은 별도 미확정 원인이며 동일 원인이라고 표시하지 않는다.

## 수정 범위

목록 경계는 .grid-wrap>.grid>.row와 .text a.desc/.ttl다. 기대 리소스 유형·URL 정책·항목 수 상한·row/link 수 일치·제목·미관측 empty 거부를 유지한다. 메뉴 전체 anchor나 페이지 전체 grid를 긁는 fallback은 추가하지 않는다. 후속 검토는 컨테이너 모호성, row별 정확히 하나의 제목/link, pagination unknown과 끝페이지 구분이다.

공통 BrowserCatalogJobs의 terminal 실패는 bounded browser_snapshot을 남긴다. state·engine version·현재 URL(1024bytes 이하만)·탐색 세대/대기/수·차단 설정·요청/차단 수·문서/결과 관측 여부·worker failure code를 포함하며 DOM·manifest bytes와 전체 history를 복사하지 않는다. 긴 URL을 생략했는지 명시한다. 접수/취소 acknowledgment·deadline 정책은 바꾸지 않는다. snapshot은 timeout 원인 판별 자료이며 엔진 초기화/네트워크 지연이라는 원인을 미리 확정하지 않는다.

## 검사 상태

runtime/quality-review/catalog-parser-v3-r1에 로그를 보존한다. 최초 strictClippy는 items_after_test_module 한 건으로 자연 종료101했다. 테스트 모듈을 파일 끝으로 이동한 뒤 fmt0·strictClippy0(15.79초)를 확인했고 workspace test는 진행 중이다. 정상 검사를 중단하거나 네트워크 요청을 추가하지 않았다. 새 보존 종영·극장판 offline ignored test는 실제 metadata observed_at을 정확히 확인·변환하고 원 SHA/내용을 다시 읽어 불변을 검증하며 별도 해석 출력만 저장한다. 아직 해당 실행 성공을 선기록하지 않는다.

Rust 구현에는 rust-pro와 lint-and-validate를 적용한다. fixture 정의/오류·자원 회수·mock 한계는 [modoo.today 스킬 적용 기록](test-fixture-contract-2026-10-19.md)을 따른다. 다른 언어 검증 로직이나 AI 추론·모델 다운로드는 추가하지 않는다.

이미 컴파일된 새 jobs harness의 saved_library_category_doms_are_reparsed_without_network를 직접 실행해 1pass/1.15초/exit0을 확인했다. 보존 종영·극장판 각각24개 항목을 v3로 해석했고 원본 v2 capture의 SHA·내용·관측 timestamp는 유지했다. runtime/quality-review/catalog-parser-v3-r1/offline-cli/library-category-replay의 두 interpretation.json과 logs/offline-library.log가 근거다. 이 fixture가 복사한 CLI는 기존 raw-r1 고정본이며 이번 검증은 새 Rust parser의 offline 해석이다. v3 CLI 실제 사이트 성공이나 pagination 완료로 해석하지 않는다. 새 네트워크 요청0이다.

후속 workspace 검사는自然 종료0, 로그의 전체 합계222pass/13ignored(root133pass/1ignored, catalog14pass, jobs5pass/4ignored), 컴파일1분56초다. fmt0·strictClippy15.79초0·명시 build --bins0.85초0을 확인했다. source-product에는 root/src/tests와 5개 package source/manifests·Cargo.lock·.gitignore를 보존했다. 미등록 media_observer 준비 소스는 포함되지만 실행 지원 근거가 아니다. Cargo.lock SHA는 `0cf44760431b804cdf18afd50aa82f57b50579cfe1465f9f6d6e9ee7b1aa96cc`로 유지됐다.

final-bin의 v3 CLI SHA는 `0176b004b5765badb2a3196a471bee596f21d7ab3b7229d233752828be19c169`, GUI SHA는 `545214ee973693da48f6f9c7133be76b763a7344e60385572efef1a5c2bac079`이며 GUI는 실행하지 않았다. 컴파일 슬롯을 반환하고 이 v3 고정 CLI로 별도 actual-cli/provider-three-categories에서 한 차례 실제 관측을 진행한다. 이전 실패 원문/receipt는 덮어쓰지 않는다. 현재 진행 중인 실제 관측을 미리 성공으로 표시하지 않는다.

## v3 고정 CLI 실제 결과

기존 새 harness 직접 실행은 1pass/30.73초/exit0이다. 모든 job이 Completed/publication_started=true/output receipt를 남겼고 실제 CLI로 결과를 읽었다. 정상 EOF 뒤 이전 endpoint 폐쇄와 같은 roots의 새 host 재획득/정상 EOF를 확인했다. 두 stop receipt의 worker_exited=true/completed_total=3이며 fresh CLI에서 결과 SHA 검증 및 모든 raw chunk를 이어 실제 저장 원문과 대조했다. 추가 컴파일 없이 실행했고 GUI 제품은 실행하지 않았다.

| category | entries | DOM bytes | body SHA256 | metadata ID |
| --- | --- | --- | --- | --- |
| airing | 58 | 101564 | a8d00750669de7542aece57ce3058bbdebdc906e3d18d3781fbafa41a00eeae6 | 95bb39d92bab567350f7e497d962e3154afa2b1f77740f1d12d5c69bcb74becf |
| completed | 24 | 221453 | 97c7cd91f7086058ac680426c23acb932a712496d3fef8cbb83ebf8e8db62808 | 0d519d203cddc1293f20b2915a9ef4b4becb8fc2728fb1626d2d8ee8ca12c604 |
| movies | 24 | 205480 | 711c0c7b3a3a792b9d23cf406be3732157c830f95eeeadf30ce7478f93c9c80f | becae080273ff235c5606841046475a164a7b7c015db0ab928f92e29ebd58929 |

모두 DOM/status=null이고 filter requested/installed=true, requested=71/36/36·blocked=0이다. 차단 건수 양수나 HTTP 원문을 입증하지 않는다. next_page=null은 끝페이지 증거가 아니다. 이전 airing timeout은 이번 명시 실행에서 재현되지 않았으나 원인이 해결됐다고 주장하지 않는다. 증거 경로는 같은 checkpoint의 actual-cli/provider-three-categories와 logs/actual-three.log다. episodes.list·resolve·native 재생과 전체 pagination은 후속이며 experimental/site_verified=false를 이 결과만으로 supported로 바꾸지 않는다.
