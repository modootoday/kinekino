---
id: evidence:kinekino/20261029114703
title: 실제 자식 player runtime 표본 인수
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 자식 player runtime 표본 인수

## 고정 구현과 검사

[자산 응답 인수](player-asset-session-cli-2026-10-19.md) 뒤 bounded read-only runtime 관측을 공통 application·실제 Frame2·worker·job·private trace에 등록했다. [관측 계약](../../player/application/browser-frame-runtime.md)의 identity/pending/간격/상한과 취소·교체 거부를 사용한다. `runtime/quality-review/player-runtime-cli-r1`에 source-product, 로그/receipt, final-bin을 보존했다.

fmt0/strict workspace all-targets Clippy18.409초0/전체276pass17ignored(root183pass1ignored)/test compile2분40초·runner279.977초0/bins1.624초0이다. 기존 실제 자산 case의 assertion과30초 job deadline을 유지하고 runtime8표본·typed 필드를 추가했다. 실제 exact case는1pass20.11초/exit0, runner20.138초/launcher64960/child106768/terminaltrue/exit0이다. 모든 실행은 resource-runner가 실제 affinity3·두CPU·BelowNormal(priority16384)와 자식 상속을 확인했다.

| 파일 | SHA256 |
| --- | --- |
| player-cli.exe | 342cf37169acd1e948554ec781da7c1188d3262dcbbe7253d8719403a285afc1 |
| player.exe | 92cdbcce87e98a1f86c127f6a399b2e2555571c004f77c067943e09e4c64ed1d |
| jobs-harness-final.exe | 5acfe972c26a12d914d243ffd6a81af26c99432a44578542abe716b007b22800 |
| resource-runner.exe | 692c22751fac7f9999f2d833571dab6605afd58e66711370494e62dd8a62c463 |
| Cargo.lock | 50323b23fb1bbf496e3ad42be71b4ccbdc3e1f38de1de5a5154a1ba5d78da5f7 |

Cargo의 실제 jobs-d1505f40f71d4673.exe 경로와 --list exact case를 대조했다. 검사 중 root 등록 소스·SDK/domain/infrastructure/Cargo 입력은 고정했다. 미등록·비소비 `presentation/web_ui_adapter/proposals`의 별도 UI 소스 작업은 병행했으므로 현재 전체src가 과거 source-product와 동일하다고 주장하지 않는다. GUI를 실행하거나 모델을 추론·다운로드하지 않았다.

## 실제 표본이 입증하는 것

동일 native child의8개 표본이250ms 이상 간격으로 들어왔다. 표본1 시각1792397501945, 표본8 시각1792397503761로1816ms 범위다. 각 callback은 현재 native identity를 다시 검사했고 trace의 runtime_pending=false/runtime_sampling_finished=true다.

| 관측 | 실제 값/변화 |
| --- | --- |
| document ready/visibility | 모든 표본 complete/hidden |
| FirePlayer/fireload 존재 | 모든 표본 true/true |
| jwplayer 존재 | 표본1 false, 2–8 true |
| 사이트 player_loaded | 표본1–4 false, 5–8 true |
| video element | 표본1–5 없음, 6–8 한 개 |
| 마지막3 video 값 | readyState0/networkState2/paused=true/ended=false/muted=false/error=null/time0/duration=null/크기0×0 |
| 마지막 play-button-outer | present=true, displayed=false/layout=false/center-hit=false |
| 마지막 playerbase | present/displayed/visible/layout/DOM center-hit=true |
| 마지막 popup overlay | absent, 모든 표시/layout/hit flags=false |

문서 complete·함수 존재→내부 플래그→video 생성이 서로 다른 시점이라는 실제 근거다. JW ready 이벤트를 수신했거나 영상 decode·시간 진행·오디오 출력을 확인한 것은 아니다. document hidden과 DOM center-hit는 실제 사용자 화면/physical gesture의 증거가 아니다. hidden control을 표시하거나 popup을 허용하지 않았다.

script는 같은 actual named protocol session/frame/loader 및 별도 native owner context에 귀속된 HTTP200/application/javascript26795bytes다. body SHA는 이전과 동일하다. native 숫자 frame ID와 CDP 문자열 frame ID를 매핑하지 않는다. 원문 URL·query·currentSrc는 private catalog에만 둔다.

## 원문과 결과

고정 CLI의 catalog.read_raw가 아래 다섯 body를 끝까지 재읽고 저장 bytes와 일치함을 검사했다. 각 capture metadata도 같은 catalog에 남겼다.

| 종류 | bytes | SHA256 |
| --- | ---: | --- |
| 초기 DOM | 136372 | 47fa7175a0fd6ece69a2e0a2a738a4275883c88df13c1776a8595a189400f147 |
| Frame-host DOM | 28014 | 04d80603ab09117849b7d65cc9fbef5fbb92acde652675410c5fe9bf9b993e65 |
| 직접 자식 DOM | 15115 | d312a74c9e352f2f933303cb7a7a38cdb6f6c33433738bdd692ec67c70cca5bf |
| script 응답 | 26795 | 5d78dd5505f1e076ecd1d41186577623165c9f2b6279ca1c541d935b77840841 |
| trace/runtime | 49896 | e6ab766c61a92bf406c9befd2441c8bb30fbe0744acf547f63adb94c48eef924 |

canonical job은 NeedsInput/player_runtime_observed, runtime_observed=true/sample_count8, player_asset_status200, media_verified=false/native_playback=false/media_request_count0/publication_startedfalse/outputnull이다. EOF receipt browser_worker_exited=true 및 소유 runner/harness와 cargo/rustc/player-cli 부재를 확인했다.

trace는 body1/unscoped request40/unbound session0/unavailable body0이다. 실제 bootstrap POST XHR은HTTP200/text/html/finished encoded1010이지만 본문은 아직 확보하지 않았다. 이 source 내부 상대 endpoint 종류와 실제 요청 path가 맞음을 읽기로 대조했으며 endpoint를 새로 호출하지 않았다. 별도의POST204와 혼동하지 않는다. GET XHR의 `.vtt`와`.txt` 두 건은 관측 종료 시 response/finished가 아직 없었다. 이를 네트워크 실패·빈 응답·media 없음으로 단정하지 않는다.

다음은 실제 진행 중인 응답의 완료/실패와 bounded body를 같은 session/request/loader/owner capability로 보존하는 것이다. 자연 발생 manifest의 signature·transport·수명과 필요한 activation·track·소유 mpv 재생/제어를 별도 인수한다. 전체 카테고리·다른 작품·회차전환 및 자막/제작/AI 전체 CLI goal은 유지한다.
