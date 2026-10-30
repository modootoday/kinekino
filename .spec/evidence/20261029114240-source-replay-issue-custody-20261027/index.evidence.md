---
id: evidence:kinekino/20261029114240
title: 원 core 인수 후 일회 replay 발급 — SDK R39/R40
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 core 인수 후 일회 replay 발급 — SDK R39/R40

## 구현과 제품 경계

제품 source factory 조사에서 production MediaReplayPolicy가 없고 기존 설계가 명시 정책 없는 등록을 금지함을 확인했다. 시험의 무조건 성공 정책을 제품 정책으로 승격하지 않았다. 이번 변경은 실제 정책을 주입할 수 있을 때에도 원 document/metadata가 먼저 core에 보유되도록 발급 순서를 연결하는 범위다. production 정책·DownloadHostFactory 구현/등록·실제 HTTP는 미완료다.

BrowserReplayAdmission::pending은 원 provider·OwnedMediaStream·document owner·명시 Arc<MediaReplayPolicy>를 같은 기존 admission 안에 보유한다. private Grant enum은 Ready 또는 Pending(Some(original stream,policy))이고 Clone/Serialize·새 registry/public pending Box가 아니다. ProcessSourceOpener::adopt_in_session이 기존 SourceCustody에 원 opener를 먼저 넣은 뒤 ReplayAdmission::prepare를 호출한다. 그 다음 원 request binding/HttpBootstrap 준비가 이어진다.

prepare는 원 input/binding·deadline·releasing 상태와 provider 포인터/ticket의 document-stream 결합을 확인한다. 원 Pending은 provider/policy callback 전에 take해 오류·panic·재호출 시 같은 발급을 반복하지 않는다. 원 provider의 check_media_stream/issue_media_replay와 BrowserWork의 기존 reserve/current/credential 정책을 사용한다. stream URL/JSON을 권한으로 만들거나 credential Unknown을 허용하지 않는다. 기존 ready-grant 생성자와 기존 소비자는 호환 유지하며 모든 외부 trusted Rust 호출이 자동으로 core 인수 경로를 사용한다는 주장은 하지 않는다.

Native Handle::issue_media_replay는 같은 원 work에서 callback을 catch하되 shared mutex를 해제한 뒤 원 payload를 그대로 resume한다. source core의 정상 사용은 poison으로 거절되고 cleanup은 같은 원 state를 회수한다. 공유 browser mutex까지 오염돼 원 문서 정리가 막히는 일을 방지한다. Browser 담당의 원 issuance/current 책임, Media의 parent transport, SDK의 원 core/CAS/terminal 책임을 유지한다.

R40에서 준비 단계 original deadline 만료는 Timeout, releasing owner는 SourceExpired로 구별했다. 새 deadline 발급/연장이나 실패 뒤 재submit은 없다.

## 원본·입력 핀

preimage-r38-replay-issue-custody는 document_owner/provider mod/replay fixture/browser_replay/process_source/live jobs/cancel/cli/native provider mod·tests의 변경 전 파일을 보존한다. preimage-r39-replay-preparation-deadline에는 R39의 browser_replay/issuance 및1267 실패 당시 issuance-test-layout-failure.rs를 보존했다. root/donor/source375 원본/WIP를 덮어쓰지 않았다.

|입력|SOURCE.md·inventory·producer|
|---|---|
|R39|E362857F4080287D9C452579D05389F0003E2F9E5A81C5EE2CCD616DB1DB60C1;9E20E9A0C02D5A3024169F0BB25B6014FBC5F5EB540E261CF10D99CCEB03417C/2297;1262|
|최종 R40|DB7C9807F9891A80BDF105992E760796D0DA1F71179E63DCE8F296767218E97B;D0428705C89204706EC4E63CE2A9223E83BCFFE254ACAD7E5D22D21C51A9CFC3/2297;1269|

fixed-r39/sdk-source-tests.exe=055C417F4F86DCBCAD26C02B1ACFED3475BAD673A8953E92A033F8F2A795B154/109547008bytes. 최종 fixed-r40/sdk-source-tests.exe=A33907D0B14087827FDC41DCBF1C60288B04533FF4F786982CECD100ABF3A025/109552128bytes.1269 compiler JSON의 lib test/source-adapter-test-support/executable/freshfalse와 terminal exit0을 확인한 뒤 copy/핀했다. r40 operator index SHA DAA2ECE63EF6F14F733262D00744B3CCCC3BFDCB66B714515719CA51874CA4F6,proof=false.

worker는 원 R15/1101의477ED83B9286BA0F556DD61F97C230FFD6F2C4A8770FFD45B6D161EA63E051F7/1000960,CLI는 R14/1092의249DD5ECEFFD0A476D3EC7B89B0F7003F9DB5DC13D285AE1126A4B9938DED32B/36405248을 재사용했다. 새 정상 bins로 표시하지 않는다. root lockB2E3D540665FFC8E68D24D0A6C531D2B983658FB19B5ABBA525659BA1ACC371D/mirror955DEBA0338A3A32ED64D6B2A500C192CD55517336FE531D6EED374D0B2916A5/AGENTS40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C을 최종 대조했다.

## 실제 실패와 실행

1260 첫 support strict는 독립 media_download mount에서 새 pub(crate) check_stream에 접근하지 못해 trait 재귀로 선택된 점과 fixture policy의 browser_network 참조 누락으로 실패했다. 검사만 수행하는 원 owner association 메서드를 public으로 노출하고 기존 kinekino_player 참조 경로를 사용해1261 통과했다. 새 raw ticket/stream/grant 생성 권한은 추가하지 않았다.1267은 추가 deadline 시험 모듈 뒤에 impl을 배치한 lint 실패이며 원 파일/로그 보존 후 module을 마지막으로 옮겨1268 통과했다. lint 완화/시험 제거는 없다.

R39에서는1263 policy error/1264 policy panic·1266 정상 CLI의 actual3,1265 원 Handle payload/mutex1과 no-run/strict를 통과했다. R39 custody verify session31862 terminal exit0/2297 동일도 보존한다. 최종 R40의 별도 실행으로 아래를 다시 확인했으며 두 입력의 actual을 중복 합산하지 않는다.

|최종 gate|실제 결과와 한계|
|---|---|
|1268 support strict/1269 no-run|각 실제receipt terminal exit0|
|1270 policy error|exact1pass/958filtered;원 NeedsInput/consume0/child0/publication0/output없음·준비IPC receipt1/guard drop1·원 retaining shutdown|
|1271 policy panic|exact1pass/958filtered;Failed/Internal·같은 core owner/poisoned source state의 정리·child0/consume0/미발행|
|1272 정상CLI/1273 취소CLI|각 exact1pass/958filtered;정상 원 grant→SDK/source/발행→Completed,취소 원 partial/lease 정리→Cancelled;5RPC/7RPC|
|1274 deadline|2pass/957filtered;원 expired cutoff 반복 확인은Timeout·releasing은later cutoff에도SourceExpired|
|1275 Native Handle 단위|8pass/951filtered;새1은 정책 panic의 원 Arc payload ptr 동일·shared mutex unpoisoned·원 work1 보유·cancel/poll 가능;COM/worker 실행 아님|
|1276 browser application|86pass/873filtered;원 ledger/발급/current/document 소유권 회귀|
|1277 default strict/1278 fmt|각 실제receipt terminal exit0|

actual4는 원 고정 C2 driver로 직렬 실행했고 각 outer selected_one=true/ownership complete/physical_closed=true/first_error=null/process exit0·Empty·nonforced다. outer는 runtime/verification/source-sdk-replay-issue-{error,panic,cli-normal,cli-cancel}-os1-r40/outer다. policy2의 worker는 guard-only로 실행하지 않았다. browser/본문/probe는 synthetic/offline이며 실제 서비스 replay 권한·HTTP 전송·COM 정상 해제 proof가 아니다.

policy error inner sdk-source-jobs-16740-1793111082324509900 및 panic inner sdk-source-jobs-19564-1793111140568027500은 동일 tag tmp에 보존했다. 원 source-receipts의 host=None/prepared_ipc Some(io_reaped/local_endpoints_closed=true,had_error=false)/raw_eof=[true,true]/launch_unreturned=false를 확인했다. peer EOF=false와 no-child를 실제 process exit0으로 바꾸지 않는다. R39 원 오류 diagnostic은 needs_input/Original fixture replay policy rejected/reported_error_request_id=original-replay-policy/미발행/verified_output null을 보존했다.

## 마감과 남은 목표

최종 verify-current session31972 terminal exit0/2297 content·identity 동일 확인. C-only cache/tmp·conservative2/CPU2/BelowNormal·heavy1·같은handle terminal 확인 후 직렬·locked/offline/-j1 유지. 설치/추론/다운로드/실제browser/HTTP/수동GUI/WARP/push0. 새 사용자 승인 절차나 별도 권리 credential 체계를 만들지 않았다.

production provider별 replay 정책,최초 browser/document 획득의 원 factory 보유,선택 ID→원 source 재해석·도구/output/probe 고정과 실제 제품 등록·HTTP/probe 인수는 남는다. 명시 정책이 없으면 등록하지 않는 기존 설계를 유지한다. CLI 전체·AI 결과 GUI 비교/수정/판정/이력·시각 인수도 미완료다. 원 DFB0 지정190/정상bins·R38 actual7·R37 파일/probe9는 각각 원 입력의 근거다. source375 private overlay·담당 소유권/WIP·원 실패·AI 사용자0/9/NeedsRepair·기존 보류 유지.

기독 modootoday/software-qa/test-fixture-and-mock-build SHA A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100의 원 callback error/panic·fixture/실제 구분과 원 teardown,code-change-review-verification SHA9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B의 원 producer·실패·증거 강도 분리를 적용했다.
