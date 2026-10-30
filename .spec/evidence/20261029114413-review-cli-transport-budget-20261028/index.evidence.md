---
id: evidence:kinekino/20261029114413
title: R79 명시 CLI 전송 기한·host 세션 기한 공백
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R79 명시 CLI 전송 기한·host 세션 기한 공백

2026-10-28 중앙 PM. [전송 기한 계약](../../contracts/production/cli-transport-timeout.md)을 CLI runner에 구현했다. 기본5000ms 유지, --endpoint 전용 --transport-timeout-ms10..30000·--input 동시 사용·숫자/범위/중복/누락 거절·Request timeout 독립이다. 기존 EndpointClient 한도와 execution_unknown·재시도 없음은 유지한다. 시험 supervisor는 명시경로만35000ms로 늘렸다. 원본 preimage는 mirror/runtime/verification/preimage-r78-transport다.

R79 SOURCE `4AAA0D8514FE51E9C1AA58978078CF6CA3F8D78F9B51965C6D39D1B7AF2C5C3F`, inventory `05749C7EB6C4C1A799C304803B391DBFA0A6B62DAB5237951CF660D9ED0CED34`,2324입력. fixed-r79에 정상 CLI와 startup 시험exe/PDB 보존. CLI SHA `C396F4E4B6E29D08527A7D62F3B9D409706E81EBA22302A79BE595C5CD1EB890`, 시험 SHA `D85F71B6427C51B4CF8DE766D838EEF4D3CD302932B6AEF66217A8936B9038C7`.

로그 root runtime/verification/source375-datalab-b0-r1-logs:1693 fmt적용0,1694 Native alltargets strict0,1695 정상CLI/startup 시험no-run0,1696 transport parser시험1pass(35filtered),1697 snapshot0. 전수CLI시험 성공으로 계상하지 않는다.

최종1699 custody exit0·2324입력 content/identity 동일,1700 default alltargets strict exit0·1701 fmt--check exit0. 이후 player-cli/mpv/ffmpeg/ffprobe 조회 대상0개는 정상 EOF 정리receipt를 대신하지 않는다.

1698 actual_matroska_product_cli_controls_and_eof는 FAILED/exit101/36990ms 전후다. raw는 mirror/runtime/verification/actual9-mkv-cli-controls-r79-r1. READY와create 성공 뒤 verify의CLI 응답이5042ms에 engine_failed/Win32233/read_response/client, delivery=execution_unknown이었다. 30초 client기한 만료가 아니다. 원server.rs의 tokio timeout(options.timeout,serve(...))와review_host.rs의 TransportOptions::default()를 확인해host세션기한도5초임을 확인했다. 저장 결과와 원 SDK 실행 완료를 단정하지 않는다. 원R78첫실패는 그대로 보존한다. 재생/제어/EOF/restart는 아직미실행.

다음은 고정SHA review-host 설정에 명시 host transport budget을 추가하고 기존 설정의5초기본·허용10..30000·범위거절을 유지하는 것이다. 서버기한/클라이언트기한/SDK작업timeout은 별도다. client기한만으로서버작업시간을보장하지않는다. 검증·hash생략이나전역한도변경으로우회하지않는다.

기존WIP/소유권/실패/raw/private overlay·C-only/heavy1/CPU2/BelowNormal/offline/locked/-j1유지. 수동GUI·추론·다운로드·WARP·push0,사용자품질0/9. 지정190/정상2bins완료는원R77근거이고이번입력으로승계하지않는다. 적용스킬modootoday/software-qa/test-fixture-and-mock-build A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100,code-change-review-verification 9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B. 실제전송불확정과기능완료를구분했다.
