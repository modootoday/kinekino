---
id: evidence:kinekino/20261029114155
title: 새 정상 R40 CLI의 제한 종단 인수 — 2026-10-28
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 새 정상 R40 CLI의 제한 종단 인수 — 2026-10-28

R40 고정 입력의 [지정190/정상bins gate](source-sdk-r40-integrated-regression-20261027.md)에 이어,새 정상 player-cli.exe 자체를 기존 offline SDK endpoint 시험에 연결했다. 기존 R14 CLI 성공을 승계하지 않았다. source 변경0이며 같은 SOURCE DB7C9807F9891A80BDF105992E760796D0DA1F71179E63DCE8F296767218E97B/inventory D0428705C89204706EC4E63CE2A9223E83BCFFE254ACAD7E5D22D21C51A9CFC3/2297을 사용했다.

## 실제 실행파일·메타데이터 보존

- CLI:fixed-r40/player-cli.exe ED1D4899A4825626F25B058CF47BB74856ABB2D54C082021EB0ABAC11DEBBE94/36718592bytes.1285 Cargo JSON의 profile.test=false/features native-proposal-window normal artifact다.
- host test:fixed-r40/sdk-source-tests.exe A33907D0B14087827FDC41DCBF1C60288B04533FF4F786982CECD100ABF3A025/109552128bytes,producer1269/source-adapter-test-support.
- worker:원 R15/1101의477ED83B9286BA0F556DD61F97C230FFD6F2C4A8770FFD45B6D161EA63E051F7/1000960bytes 재사용.새 worker 컴파일 주장이 아니다.
- 새 operator index:runtime/verification/source-sdk-r40-newcli-index.json SHA B6A6AB3FCCD351A7B9C1FDFB63D0BF44C6F9D7B68FC9377A4CA9290F033981FE,proof=false.

1287/1288의 초기 정상/취소 실행은 이미 새 ED1D CLI 환경을 사용해 통과했지만 old R40 index의 cli_producer 설명은 R14로 남아 있었다. 그 원 index/로그/결과를 수정하지 않았다. 새 CLI producer·SHA·size를 명시한 별도 index를 만들고1289/1290을 반복해 최종 인수를 확정했다. 초기2를 추가 기능 완료 수로 더하지 않는다. 메타데이터 문제와 실제 runtime 실패를 구별하며 이번 runtime 실패는 없다.

## 최종 선택3 실제 결과

|실행|검증|실제 결과|
|---|---|---|
|1289|normal_cli_uses_shared_source_and_original_publication|exact1pass/958filtered;CLI5RPC·같은 SDK/원source→검증/발행→Completed·정리|
|1290|normal_cli_cancel_retains_partial_until_probe_close_and_records_terminal_cancelled|exact1pass/958filtered;CLI7RPC·probe 대기 취소→원 partial/lease 유지→원 source/probe 정리→Cancelled/미발행|
|1291|absent_factory_stays_unsupported_through_normal_cli|exact1pass/958filtered;CLI3RPC·factory 미주입의 jobs.submit은 Unsupported/No executor for this job operation;source worker 실행0|
|1292|최종 custody|actualreceipt terminal exit0/41516ms·2297 content/identity unchanged|

각 최종3의 resource-runner receipt는terminal/exit0다. 고정 C2 driver outer는 selected_one=true/ownership complete/physical_closed=true/first_error=null/outerprocess exit0·Empty·nonforced다. 실제 고유 case는3이며 RPC는 원 rpc-count.txt에서5/7/3(합15)을 확인했다. RPC15는97명령 인수15개가 아니다.

원 outer는 runtime/verification/source-sdk-r40-newcli-{normal,cancel,absent}-bound-os1/outer다. 원 inner는 같은 tag tmp의 sdk-source-cli-21540-1793113256775568100/4356-1793113340093635500/36992-1793113410483700400이다. 각 rpc-N/owned/receipt.json·request/response·raw와 local-close.txt/rpc-count.txt를 보존했다. 정상 rpc-1 receipt의 실제 executable=fixed-r40/player-cli.exe/PID22692/creation134375868567858023/exit0·Empty·nonforced를 직접 확인했다. factory 부재 rpc-3 원 response는 code unsupported/message No executor for this job operation이다. raw process cleanup와 fixture의 원 CLI SHA/size guard를 함께 사용한다.

정상/취소의 본문·browser admission·probe는 offline fixture다.새 CLI가 실제 provider를 지원하거나 HTTP/media download를 수행했다는 뜻이 아니다. factory 부재에서 worker는 guard-only다. 공유 SDK 동작/오류/취소/저장과 GUI 기능/시각 인수를 구분한다.

## 마감·미완료

동일 R40의 Native190/정상2bins·fmt/default/support/Native strict는 각 원 실행으로 유지한다. 이번 source 수정이 없어 strict/컴파일을 반복했다고 표시하지 않는다. C-only cache/tmp·conservative2/CPU2/BelowNormal·heavy1·동일handle terminal 후 직렬을 유지했고 새 설치/네트워크/추론/다운로드/수동GUI/WARP/push0이다.

source375 private overlay/canonical 미적용·기존 WIP/소유권/원 실패·원 DFB0 근거를 유지한다. production replay 정책/factory/source 등록·실제 HTTP/probe 통합·나머지 CLI 구성/명령별 인수·GUI/AI 원본 비교/재생/수정/명시판정/이력·실행 시안 비교는 남는다. AI 사용자 품질0/9와 원NeedsRepair·배포 제외·기존 보류를 유지하며 전체 goal을 닫지 않는다.

기독 modootoday/software-qa/test-fixture-and-mock-build SHA A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100의 새 실제 executable/원 fixture·정리·실제와합성 구분 및 code-change-review-verification SHA9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B의 producer/원 오류·metadata와실제 결과 분리를 적용했다.
