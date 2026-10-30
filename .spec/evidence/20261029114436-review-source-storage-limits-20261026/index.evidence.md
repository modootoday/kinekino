---
id: evidence:kinekino/20261029114436
title: F1 저장 한도 공개·결합 CLI 저장 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# F1 저장 한도 공개·결합 CLI 저장 검증

2026-10-26. SOURCE9AB60253…의 전체190 통과 후계다. 저장 한도 자체를 바꾸지 않고 실제 adapter의 한도를 공통 UserReviewStore→ReviewSourceService→PlayerSdk capability로 전달했다. StoreLimits는 DecimalU64의 max_case_transactions/max_transaction_bytes다. 알려지지 않은 adapter는 None/null이며 무제한을 뜻하지 않는다. 실제 UserReviewFiles는4096/262144를 선언한다. committed_key_expiry=none, capacity_behavior=reject_new_publication_preserve_committed_replay도 표시한다. 전역/타프로젝트/원자료를 변경하지 않는다.

## 실제 검증

원로그와 receipt는 runtime/verification/source375-datalab-b0-r1-logs/141~144다.

-141 limits-tests: lib review_source **21pass/0fail/1internal-helper ignored**,1412filtered,exit0/90396ms,시험2.45s. 독립 합성 committed envelope4096개를 실제 파일로 구성해 loader가 검증·보유한 뒤 원key replay 및 동일request publish가 같은receipt를반환하고 새publication만ResourceExhausted로거절하는지확인했다. 기록4096개유지·readonly reopen/replay도통과했다. 이는4096회CLI mutation 실측이나처리량 benchmark가아니다. fixture runtime/build-cache/tmp/review-source-files-32492-8의json4096개를보존한다.
-142 limits-cli: 정상CLI **3pass/0fail**,exit0/97784ms,시험23.67s. 기존5명령/readonly 종단에combined save를추가했다. association+빈user draft를한요청으로출판→새process replay→각각조회→transactions envelope1개및반환body/실파일일치를확인했다. 이 빈draft는모델range나판정으로전환하지않는다.
- 같은CLI capabilities에서4096/262144의Decimal문자열과key expiry none/strong_cancel false를확인했다. 현재실행CLI SHAA10199CAC2FE40600A01557CD0B86A7072795A93690B6600B406A02F44E0D1DB. fixture runtime/build-cache/tmp/review-source-cli-27328-{0,1,2}의총25process요청/stdout/stderr/PID/exit/전후CLI SHA를보존했다. 원package/asset SHA·decisions/cases불변도각call마다확인했다.
-143 fmt --check exit0/2516ms.144 root all-targets/native-proposal-window strict exit0/31364ms. 기존paint deprecation은원로그보존.

locked/offline/jobs1/CPU2/BelowNormal/C native/tmp·기존toolchain. 두lock과원Media seal B47E9913…pin불변/rootindex0·프로젝트heavy/CLI잔여0. 기존WIP·소유권·추론/다운로드/수동GUI/WARP/push보류유지.

## 주요 source pin

| 파일 | SHA256 |
|---|---|
| src/application/review_source/ports.rs |45ADB6CBAC6FDFB1623FDB87FE84F4AC7FAB37E8E0FCF46590FCA979B6E100A8|
| src/application/review_source/api.rs |5521E9A432AE264A5EDF10A06B434985E32A155BFC806D07C43622C9B5135012|
| src/application/review_source/schema.rs |9542A0472948B425FF5E6B20B98D534FAD398DC888BEA4B65CF7CCC8DD5BDCBF|
| src/application/api/mod.rs |E2821959516FAEBE65BEE8AFDC81747087829FABD86D7E8BEFDCE07EFE8D5D1E|
| src/infrastructure/review_source/mod.rs |1500DF45A78968766B9DDED26878D1D21511FDAEF3C3A05F33D4527A34E28B56|
| src/infrastructure/review_source/tests.rs |75C895D907935B7B426C99630D0E69F7122517A94CAA1B32419557D397D89943|
| tests/review_source_cli.rs |81F777BEE72BA9F8C3570BF47AB7589FEBA71B1B2BBB762087B5937A3FCB7F6F|
| tests/review_source_cli/combined.rs |77E6D526CC0BD4ADFF8AC94C71D45D316043C9810EEB3A53BC41AD92B01018BB|

스킬: 기존modootoday/software-qa/test-fixture-and-mock-build 본문/해시·출처는 [case근거](review-source-case-custody-20261026.md)와같다. 실제capacity fixture·재획득/reopen·mock과실CLI의구분을적용했다. RD03의combined CLI를보강했으나schema독립전수검증/RD05full-usecase경쟁/다른custody·실제9/Verified·AI/GUI잔여는유지한다. 이새source후계의전체190/정상2bins재검증은아직이며이전9AB성공을승계하지않는다.
