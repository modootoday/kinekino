---
id: evidence:kinekino/20261029114708
title: 실제 매니페스트 응답과 Referer 경계 실패
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 매니페스트 응답과 Referer 경계 실패

## 고정 source와 검사

`runtime/quality-review/player-manifest-cli-r1`은 관측 POST/XHR body, 요청 terminal·수신 시각·종료 reason, hls-bounded-v1, typed 결과 저장/reader와 강화 actual case를 포함한다. fmt0, 첫 strictClippy7.342초101(collapsible_if 한 곳) 뒤 표현만 수리했다. 수정 전 source-product와 수정 parser.rs의 source-repair를 구분하며 원로그를 보존했다. 수리 strictClippy receipt36.124초0(stderr Cargo32.08초), 전체283pass17ignored(root190pass1ignored)/compile3분18초/runner337.004초0, bins907ms0이다.

| 파일 | SHA256 |
| --- | --- |
| player-cli.exe | 737a62390661c4203787f686aba8f758caf527b0323e7a12c787560a7d8e9c4f |
| player.exe | a29f2fedec15f455dfff8ed2f8db038df5b19e45a6ba41baf2ff30972def1e25 |
| jobs-harness-final.exe | 86ec84fbb42afd9b858a89b829eefe1b0bf95207dc4e794f311d5fb69b92c3fc |
| resource-runner.exe | 692c22751fac7f9999f2d833571dab6605afd58e66711370494e62dd8a62c463 |
| Cargo.lock | 50323b23fb1bbf496e3ad42be71b4ccbdc3e1f38de1de5a5154a1ba5d78da5f7 |

Cargo의 실제 jobs-d1505f40f71d4673.exe와 --list exact case를 대조해 고정했다. GUI 실행·모델 추론·다운로드는 없다. 모든 실행은 실제 affinity3·두CPU·BelowNormal(priority16384) 및 자식 상속을 기록했다.

## 실제 실패

같은 actual CLI는0pass1failed/21.14초, runner21.169초/launcher4040/child98980/terminaltrue/exit101이다. 원 job30초 deadline과 assertion은 유지했다. 처음 실패한 지점은 trace observation_end=response_rejected가 manifest_verified와 다르다는 assertion이다. 이후 Completed/typed 재열기/tamper-copy assertion에는 도달하지 않았으므로 해당 실제 인수를 통과했다고 주장하지 않는다.

canonical job은 NeedsInput/code=permission_denied, message=Observed manifest transport or Referer is outside policy, publication_started=false/outputnull이다. provider_state=running, 종료 reason=response_rejected이며 provider crash나 timeout이 아니다. EOF receipt browser_worker_exited=true를 확인했고 소유 runner/harness는 종료됐다.

이 실행에서 bootstrap POST XHR200/text/html628bytes와 GET XHR200/text/plain678bytes를 실제 같은 named session/frame/loader·native owner로 읽었다. 매니페스트 후보는 EXTM3U/VERSION/STREAM-INF의 master 구조이며 domain HLS 검증 이후 source Referer 정책에서 거절됐다. 이 사실은 실제 실행의 오류 분기와 보존 body에 근거한다. Typed 결과는 게시하지 않았다.

## 정확한 Referer 구분

매니페스트 capture.network.referer는 null이 아니며 길이0인 문자열이다. 현재 decoder가 기본 requestWillBeSent의 headers에서 투영한 **관측 빈 값**이다. 기존 verified source constructor가 navigation_allowed(빈 문자열)를 거부해 permission_denied가 됐다.

이 기본 event 값만으로 실제 wire에서 헤더가 부재했는지, 빈 헤더가 전송됐는지, 해당 API가 추가 header 관측을 제공하지 않았는지 확정하지 않는다. 임의 문서 URL로 보충하거나 빈 값/미관측을 하나로 합쳐 성공화하지 않는다. [공식 CDP Network 계약](https://github.com/ChromeDevTools/devtools-protocol/blob/master/pdl/domains/Network.pdl)의 requestWillBeSentExtraInfo는 network stack의 wire headers를 제공하지만 모든 요청에 발생하지 않으며 기본 event와 순서를 보장하지 않는다.

후속 수리는 실제 session+유일 request ID의 ExtraInfo를 연결하고 None(추가 관측 없음)과 관측된 absent/empty/URL을 typed 상태로 분리하는 것이다. redirect 여러 hop/중복 ExtraInfo의 모호함은 합치지 않는다. Cookie/Auth 및 관련 cookie 배열은 보존하지 않는다. 비어 있지 않은 URI·control/credential/scheme 정책과 legacy from_request는 유지한다. 직접 POST·URL 추정·Referer 전송/재작성·숨김 상태 변경은 없다.

## 원문과 trace

private catalog 기준 `runtime/quality-review/player-manifest-cli-r1/actual-cli/resolve-watch-cli/catalog`이다. 아래7개 body는 고정 CLI catalog.read_raw로 전체 재읽기 및 저장 bytes 일치를 확인한 뒤 trace assertion에서 실패했다.

| 종류 | bytes | SHA256 |
| --- | ---: | --- |
| 초기 DOM | 136372 | 3db69e189a63d2465f1c7ee2beedc3dee7ffcb1baa16dba9014d3c9fa1fd1cb4 |
| Frame-host DOM | 28014 | 04d80603ab09117849b7d65cc9fbef5fbb92acde652675410c5fe9bf9b993e65 |
| 자식 DOM | 15115 | d312a74c9e352f2f933303cb7a7a38cdb6f6c33433738bdd692ec67c70cca5bf |
| script | 26795 | 5d78dd5505f1e076ecd1d41186577623165c9f2b6279ca1c541d935b77840841 |
| bootstrap 응답 | 628 | e06f80a4809c339660fa610aae9216811303dd01d6328a826d1c775391dffaa0 |
| 매니페스트 후보 응답 | 678 | 0c1a11c226e2d5fcb655bb54c989210ce06de3ca0f6c95c96095352d5ca71852 |
| trace | 52688 | 62efa3ce32b827c2d019bfcb8b2c3117515b0a0514db7f1e166b6d4e0dccfe6d |

trace는8 runtime 표본, body3/unavailable0/unscoped request40, request terminal finished18/pending7을 기록했다. 수신 Unix 시각도 보존했으며 첫 event1792399577236ms다. pending을 실패로 바꾸거나 모든 자원이 완료됐다고 표시하지 않는다.

## 소비와 남은 작업

등록 source는 수리 중 UI23 소비를 위해 r1 검증본으로 고정했다. 동결 알림과 두 domain 파일 등록 수정의 짧은 경합은 변경분 보존 후 정확한 r1 SHA로 원복했다. 원본·변경·원복 SHA 및 수집 가능한 시각 범위는 `runtime/coordination/manifest-wire-repair/freeze-race.md`에 기록했으며 원복의 미관측 정밀 시각을 만들지 않았다. 신규 RequestReferer/RequestHeaders 파일은 아직 미등록·미검증이고 UI 반환 후 연결한다.

이 실패를 typed manifest Completed·새 reader 손상 거부·media/native 재생 성공으로 승격하지 않는다. [결과 계약](../../player/application/episode-resolution-result.md), [이전 실제 runtime 인수](player-runtime-cli-2026-10-19.md)와 구별하며 전체 SDK/CLI·미디어/자막/제작/AI goal을 유지한다.
