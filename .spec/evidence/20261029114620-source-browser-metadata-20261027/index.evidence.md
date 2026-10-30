---
id: evidence:kinekino/20261029114620
title: Browser metadata lease 격리 통합 — R17
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Browser metadata lease 격리 통합 — R17

## 범위·소유권

중앙 PM이 기존 Browser 담당 metadata7의 before→overlay delta만 최신 `runtime/development/source-sdk-integration-r1`에 rebase했다. 원 `runtime/development/media-download-provider-r1`, 제품 root, 원 WIP·원 실패·source375 private overlay 경계는 유지한다. Browser는 발급·원 ledger/current/spent를, Media/SDK는 소비 adapter를 소유한다. 제품 등록·canonical 적용·실제 브라우저/HTTP·추론·다운로드·수동GUI·WARP·push는 수행하지 않았다.

기존7 경로는 application/browser_network의 capture_state.rs·mod.rs·request_ledger.rs, application/browser_provider의 mod.rs·network_contract.rs·state.rs, infrastructure/browser_provider/mod.rs다. 후보 내부 `runtime/verification/metadata7-rebase.patch`와 `preimage-r16-metadata7/`에 원 delta/preimage를 보존했다. 신규 시험은 application/browser_provider/media_stream_tests.rs이며 기존 mod의 test mount를 포함한다.

## 검증 입력

- input-r17 SOURCE.md SHA256 `AC682BAF0A01A5B42979E77098AF82AF2F120C69D01DBD6BE56AC4E39706E2BF`.
- inventory SHA256 `DBBC5AE5991F8E25E39E6D6D6DD8E50B3422A87EA0FA08E24E6F7E7125F64A3E`,2287파일.
- root lock `B2E3D540665FFC8E68D24D0A6C531D2B983658FB19B5ABBA525659BA1ACC371D`, mirror lock `955DEBA0338A3A32ED64D6B2A500C192CD55517336FE531D6EED374D0B2916A5` 유지.
- AGENTS `40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C` 유지. C-only native/source-sdk-integration-r1, tmp, heavy1·locked/offline/-j1·conservative2 실행.

## 실제 결과와 첫 실패

로그는 후보 내부 runtime/verification에 보존한다. 1113 strict 첫 실패는 미소비 getter4 dead_code다. 네 메서드에만 staged metadata/replay consumer 미인수 사유의 allow를 두고 1114·1115 all-targets strict 통과했다. 권한 검사·admission을 우회하지 않았다.

- 1116 provider 소유권 신규2/2,928 filtered,terminal exit0: 원 generation/current 유지, reload 후 재발급 거절, 취소 후 원 metadata lease 무효화.
- 1117 browser_network 회귀35/35,895 filtered,terminal exit0. metadata before-finish/2MiB body capture 상한 보존, 관측 header 필요·재발급 금지, reload/close 무효화·모호 header 후 issued lease 무효화를 포함한다.
- 1118 cargo fmt --all --check 통과,terminal exit0.
- custody verify-current 원 실행 session95495 terminal exit0: 전체2287 content/identity unchanged. SOURCE.md hash를 인자로 사용했다.

## 미완료와 다음 범위

Metadata lease는 새 GET replay 허가가 아니다. GET/200/Referer/Cookie 부재로 public replay를 추정하지 않는다. non-Clone/non-Serialize 원 발급·일회 소비·실패 후 재발급 금지·current 검사의 replay typed port와 provider별 명시 정책은 미구현이다. 저장된 resolution JSON은 원 live owner를 재구성하지 않는다. episode resolve 종료 시 owner release와 실제 source 사용 수명도 연결해야 한다.

실제 native/browser 실행·네트워크/HTTP·media probe·제품 채택·전체CLI·GUI/AI 사용자 품질은 이번 결과가 입증하지 않는다. 지정190/정상2bins는 DFB0 원 입력에 유지하며 R17 재실행으로 계상하지 않는다. 다음은 원 replay issuer 정책·수명 계약을 실제 구현과 대조하고 typed 발급/소비 연결을 격리 작성하는 범위다. auth/unknown-cookie/response-only는 명시 미지원으로 유지한다.

## 적용한 스킬

기존에 읽은 modootoday/software-qa/test-fixture-and-mock-build 본문 SHA256 A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100을 재사용했다. 실제 포트/원 수명·setup/teardown·fixture와 실제 서비스의 경계 및 원 실패 보존에 적용했다. 별도 언어 검증 도구나 승인 절차를 추가하지 않았다.
