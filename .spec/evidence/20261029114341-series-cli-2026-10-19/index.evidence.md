---
id: evidence:kinekino/20261029114341
title: 불변 카테고리 결과에서 실제 회차 CLI 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 불변 카테고리 결과에서 실제 회차 CLI 검증

상태: Rust fixture 준비·fmt 통과, 컴파일/실제 실행 전. [v3 카테고리 검증](catalog-v3-2026-10-19.md)의 제품 source/고정 CLI를 유지하며 UI consumer 변경과 분리한다. 현재 UI 검사 process가 실행 중이므로 예약된 다음 SDK 슬롯에서 필요한 검사만 수행한다.

## 입력과 경로

tests/jobs/series.rs의 actual_cli_series_episodes_preserves_dom_and_stops_owned_host는 CATALOG_SEED_ROOT와 CATALOG_SEED_RESULT_SHA256을 명시 입력으로 받는다. CatalogFiles의 공통 CatalogResults.read가 결과·source descriptor·참조 raw SHA를 검증한 뒤 원래 목록 순서의 첫 series를 선택한다. 별도 DOM/URL 문자열을 발명하거나 새 카테고리 요청을 하지 않는다. 선택 URL은 공통 PageLocation/ResourceKind로 다시 검증하고 seed.json에 원 결과 SHA와 함께 저장한다.

현재 재사용할 입력은 runtime/quality-review/catalog-parser-v3-r1/actual-cli/provider-three-categories/catalog과 airing result SHA `3f1c3ee3af5d08c67f77bde974d14700ed16d6e3d518d300636f118f9c64b8e6`이다. 제품 CLI SHA는 `0176b004b5765badb2a3196a471bee596f21d7ab3b7229d233752828be19c169`다. fixture가 복사 전후/복사본 SHA를 검사하며 새 harness 빌드와 제품 바이너리 검증을 구별한다.

## 증거 범위

별도 private jobs/catalog/profile roots의 소유 host를 띄우고 jobs.submit episodes.list를 실행한다. 관측된 terminal job·실제 CLI jobs.get·원문·오류를 보존하고 host의 정상 EOF 종료를 먼저 확인한 뒤 마지막 기능 assertion을 실행한다. 실패가 발생해도 NeedsInput/timeout을 성공으로 바꾸지 않는다. 실패에 raw reference가 있으면 종료 후 fresh CLI catalog.read_raw로 모든 chunk를 복원해 실제 저장 원문과 대조한다. 성공 시 fresh CLI episodes.list(result_artifact_sha256)로 읽고 모든 episode URL·series identity를 공통 정책으로 검증한다. seed 결과는 다시 읽어 검증하며 원본을 수정하지 않는다.

성공은 선택한 작품의 회차 목록 인수다. episode HTML/player 이동·frame.example/media·native 재생·전체 pagination은 별도 후속이다. DOM/status=null·기존 adblock 요청/설치/차단 관측과 정상 worker 종료를 구별한다.

## 실행 순서

제품 코드 변경0이므로 예약된 검사에서는 fmt, strict workspace/all-target Clippy, jobs target을 검사한다. 이후 고정 v3 CLI·새 jobs harness로 ignored 실제 회차 case를 한 번 실행하고 JOB_EVIDENCE_DIR 아래 새 provider-series 디렉토리에 로그를 보존한다. source 준비만으로 실행 성공을 선기록하지 않는다. 실제 실패가 parser 구조에서 발생하면 같은 DOM을 먼저 offline 재현해 수정하고 별도 revision/checkpoint로 검증한다. 반복 네트워크로 실패를 덮어쓰지 않는다.

## 실제 결과

runtime/quality-review/series-cli-r1에 source-fixture와 로그를 보존했다. fmt0·strictClippy1.65초0·jobs target5pass/5ignored(compile6.75초/run25.61초)를 통과했다. 제품 source/bin은 바꾸지 않았고 v3 고정 CLI를 재사용했다.

실제 ignored case는1pass/10.23초/exit0이다. 카테고리 원래 순서의 첫 작품을 선택해 episodes.list job이 Completed/publication_started=true로 끝났고 4회차 URL을 얻었다. result SHA는 `b1a2af4afa4491ef5743f2a1095d7b9973b6bc4268f1edf70dd9482446bea3bc`(4210bytes), DOM은134681bytes/SHA `f0c73cacbfc4526bffada0012912c6e01429c376a8ef496f89251ed34f01d8c3`, metadata ID는 `cf48a62cb39a8c34f6224370c973f33982a026432eafd080b8fc275be31c3550`이다. 실제 source/selected entry는 actual-cli/provider-series/seed.json, terminal/output은 observation.json에 보존했다.

DOM/status=null, parser v3, WebView2 155.0.4283.62, document/request generation1, requested32/blocked2, blocking_requested/installed=true다. 기존 공통 adblock의 실제 차단 건수 양수를 관측했으나 모든 광고 제거/사이트 전체 동작을 뜻하지 않는다. 정상 EOF는 worker_exited=true/completed_total=1이며 fresh CLI episodes.list 결과 SHA·전체 raw chunk와 저장 원문 일치를 확인했다. 원 seed는 다시 검증했고 변경하지 않았다. 선택 작품 회차 목록만 인수했으며 episode DOM/media/native 재생·모든 작품/전체 pagination은 후속이다.
