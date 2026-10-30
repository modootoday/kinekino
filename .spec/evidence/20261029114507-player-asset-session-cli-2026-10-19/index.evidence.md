---
id: evidence:kinekino/20261029114507
title: 실제 Target 세션의 player 자산 응답
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 Target 세션의 player 자산 응답

## 검사 범위와 고정본

첫 [자산 응답 CLI 실패](player-asset-cli-2026-10-19.md)의 timeout을 Target/session/frame-tree 수직 연결로 수리했다. 원실패의 deadline·assertion·원문은 변경하지 않았다. `runtime/quality-review/player-asset-cli-r2/source-product`와 final-bin, 각 stdout/stderr/receipt를 보존했다. 모델 추론·다운로드와 GUI 실행은 없다.

fmt --check0, strict workspace/all-target Clippy15.737초0, 전체271pass17ignored(root178pass1ignored), test compile2분37초/runner260.231초0, bins891ms0이다. Cargo가 실제 사용한 jobs-d1505f40f71d4673.exe를 고정하고 --list의 exact case를 확인했다. resource-runner는 launcher/child의 affinity3·두CPU·BelowNormal(priority16384)와 자식 상속을 기록했다.

| 고정 파일 | SHA256 |
| --- | --- |
| player-cli.exe | e825a8fedac0486f6847f71beb8bd59e887194269054dca7ca03c84b107ec764 |
| player.exe | 374d1a74162f40c691236f48f5a129f259c00048eefc12ffe50622bfd025d537 |
| jobs-harness-final.exe | 35800fb7d9a77f32f1c9a320a554965845b661eccc189b5bcc017f87a89f83b4 |
| resource-runner.exe | 692c22751fac7f9999f2d833571dab6605afd58e66711370494e62dd8a62c463 |
| Cargo.lock | 50323b23fb1bbf496e3ad42be71b4ccbdc3e1f38de1de5a5154a1ba5d78da5f7 |

## 실제 CLI 결과

동일 `resolve_cli::actual_cli_resolve_preserves_observed_player_script_response`가1pass/17.55초/exit0이다. runner17582ms/launcher13816/child72292/terminaltrue/exit0이다. 접수와 typed client의 동일 idempotency 재요청은 같은 job을 반환했다. canonical 결과는 NeedsInput, stage=player_runtime_observation_required, player_asset_observed=true/HTTP success=true/status200이다. media_verified=false이며 출력·publication·native 재생 성공을 만들지 않았다.

실제 기본 세션 frameAttached에서 parent/child CDP ID를 확보했다. 이어 related iframe Target attach(waitingForDebugger=true), 해당 실제 세션 Page/Network enable와 Page.getFrameTree에서 동일 child/parent ID를 확인했다. 기본 세션 frameDetached.reason=swap과 named frame navigation도 관측했다. 따라서 이번 실행의 프로세스 전환은 가설만 남은 r1과 구별한다. 실제 targetId가 child frameId와 같았지만 구현은 그 동일성에 의존하지 않고 tree/attachment 계약으로 연결한다. native 숫자 frame ID와 CDP ID를 비교하지 않는다.

script는 실제 named 세션의 request/loader/response/loadingFinished와 일회 capability로 읽었다. HTTP200, MIME application/javascript, decoded browser body26795bytes/SHA `5d78dd5505f1e076ecd1d41186577623165c9f2b6279ca1c541d935b77840841`이다. response 초기 encodedDataLength517과 loadingFinished9089는 각각 원관측대로 보존하며 decoded body 길이와 동일하다고 주장하지 않는다. source는 실제 요청한 media-cdn player asset이며 새 GET·Referer 재작성·packed script 실행으로 얻은 자료가 아니다.

본문 관측 시 native owner는 gen2/navigation5이며 이는 부모 reload/cancel guard다. protocol scope는 별도 실제 named session/root revision2/root frame·loader/child frame·loader를 갖는다. 원문 URL·query는 private artifact에만 보존한다.

## 원문과 정리

`runtime/quality-review/player-asset-cli-r2/actual-cli/resolve-watch-cli/catalog`에 아래 다섯 body와 capture metadata가 있다. 고정 CLI의 catalog.read_raw로 모든 body를 끝까지 읽고 저장 bytes와 일치함을 검사했다.

| 종류 | bytes | body SHA256 |
| --- | ---: | --- |
| 초기 DOM | 136372 | 802e8edd0d89bf8b11883aec703d5d109bbe9c4a46b068d9b7eb7e087a70f85d |
| Frame-host DOM | 28014 | 04d80603ab09117849b7d65cc9fbef5fbb92acde652675410c5fe9bf9b993e65 |
| 직접 자식 DOM | 15115 | 6678183f1a0d372fe10f69a2af8f9a782d3a6a4e1976059d6eb2419a49cc1aba |
| player script 응답 | 26795 | 5d78dd5505f1e076ecd1d41186577623165c9f2b6279ca1c541d935b77840841 |
| browser trace | 23543 | 5a716d090f6e496d7da5b4dc255b93fc4637195e71c2e46371c4c22ab26c8123 |

trace는 body1/귀속 request12/unscoped request38/unbound session0/unavailable body0을 기록한다. Page frameAttached1/frameDetached1/frameNavigated3/frameTree1/TargetAttached1, Network request12/response11/finished11을 보존했다. 전체 native 요청을 귀속된12개로 축소하지 않는다. EOF receipt의 browser_worker_exited=true와 정상 harness 종료 뒤 cargo/rustc/player-cli 및 소유 launcher/harness 부재를 확인했다.

## 다음 인수와 한계

이 결과는 airing 첫 회차 하나의 자산 응답 관측이다. script 함수 정의·실제 runtime ready/video/error/컨트롤 가시성을 확인하고 필요한 고정 활성화를 거쳐 manifest/media를 검증해야 한다. script 수신은 실행 완료·player ready·최종 media 또는 mpv 재생 성공이 아니다. 전체 카테고리·다른 작품·회차전환·모든 광고 제거도 입증하지 않는다. 공통 SDK/CLI 전체 기능과 자막·제작·AI 후속 범위를 유지한다.

계약과 공식 규격 출처는 [network 관측 정본](../../player/application/browser-network-observation.md), fixture 적용은 [시험 계약](test-fixture-contract-2026-10-19.md)에 연결한다.
