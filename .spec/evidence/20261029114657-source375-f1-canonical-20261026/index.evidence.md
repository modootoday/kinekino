---
id: evidence:kinekino/20261029114657
title: F1 최신 canonical 입력 전체190·정상bins 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# F1 최신 canonical 입력 전체190·정상bins 검증

2026-10-26. 원375/private r9/이전 canonical8A와 분리한 F1 후계다. 최신src/tests/assets/packages와root Cargo/lock/build/README/AGENTS/toolchain/config의 bounded2129파일을읽기전용으로고정했다. runtime/모델/미디어/원증거를복사하거나새지원으로추정하지않는다. 전체제품완료가아니며 [RD01~12 잔여표](../../../plans/player/review-source-rd-audit-20261026.md)를유지한다.

## 입력·도구 pin

- runtime/verification/source375-f1-canonical-input-r1/SOURCE.md SHA256 **9AB60253E89C1B717184D88DD8BC9A29A4B9FAE3CAF78D42CCC5C514FA609A53**.
- source-current.tsv SHA256 **24BEE706C30657BFFA77CFE62E633507F728308F42B977D0105D2DFFA0AA19C4**. 2129개relative path/content SHA/FileID/bytes/creation/mtime를기록했다.
- 원exact190 SHA **082C68B479E656A290F11A6CBA3AB0AF9AAE0C9651B265AA337767D454ADD464**를바꾸지않았다. 원source375 A09B51AC…와B0 preimage/역사실패는그대로보존한다.
- Root HEAD d13cbfca140e41863b4d0bd27930be3f57a1123f /Meari0844f6c24caf224617ca8abbb2dd75695a971b99,기존WIP유지/index0.
- Rust/cargo1.99.1,기존MSVC/WindowsSDK·두lock·Media seal B47E9913…불변. locked/offline/jobs1/CPU2/BelowNormal·C native/meari-native/tmp·heavy1.
- custody도구후계는snapshot-current/verify-current를추가했다. source375-custody.exe SHA6DA6D7EA686E093C1EE95657E837ED00800426B22BC29CD9015B36DFED9BA7E6, snapshot.rs SHAA2CFDEED16F2FEBC79B463A57284A9802114BA7B68A4DD75A6A51082A11CA804. 기존stage walker의visibility만확장했으며overlay채택/기존검증목록을변경하지않았다. std-only/tool cache분리·tool10시험/fmt/strict/build통과. 새path검출·receipt/inventory변조거절시험을포함한다.

## 실제 실행

원로그/receipt: runtime/verification/source375-datalab-b0-r1-logs.

| 단계 | 실제결과 |
|---|---|
|129 source snapshot|2129개고정,exit0/38427ms|
|130 product no-run|native-proposal-window/lib+jobs,exit0/33500ms;원Cargo산출물JSON보존|
|131 product exact|145/145 실제등록·ignored대조·exact실행·producer전후무결성Ok,exit0/19605ms|
|132 core no-run|meari lib+range_commit,exit0/6100ms|
|133 core exact|18/18·등록/ignored/producer무결성Ok,exit0/1717ms|
|134 native no-run|window/lib+range_input/source_callbacks/window_user_close/window_user_close_os,exit0/30778ms|
|135 native exact|26/26·등록/ignored/producer무결성Ok,exit0/9875ms|
|136 OS exact|1/1·등록/ignored/producer무결성Ok,exit0/2494ms. 수동GUI인수가아닌기존exact OSfixture|
|137 normalbins|player/player-cli2개canonical C native/debug,profile.test=false/fresh=true,exit0/442ms|
|138 source verify|2129개path/content/identity모두동일,exit0/39292ms|
|139 latestnormalCLI|재빌드된E828…정상exe로5명령/저장재열기·오류·readonly21프로세스 재시험,2pass/0fail,exit0/28350ms|
|140 finalsource verify|139뒤2129개전체path/content/identity다시동일,exit0/39052ms|

각 exact그룹의 runtime/verification/source375-f1-{product,core,native,os}-exact-r1/group-return.txt는 execution_result/producer_integrity 모두Ok이며 같은SOURCE SHA9AB60253…를기록한다. 등록/ignored대조없이기존성공을복사하지않았다. 최신추가F1영향20시험/1internalhelper ignored·fmt127/strict128은 [process근거](review-source-process-recovery-20261026.md)를따른다.130의rootlibtestproducer는fresh=true로동일기존산출물을사용했다. Meari의이전type/fmt/strict근거와이번새no-run/실실행을구분한다.

## 정상 산출물·실CLI

| 파일 | SHA256 |
|---|---|
| runtime/build-cache/native/debug/player-cli.exe |E828AF30BA06FD33C76E5EA770E824938B8C1B9E76F8517C69A5DFED50DC2D65|
| runtime/build-cache/native/debug/player.exe |1DC15674E6ACB5B460DF3B7C81A2DB612A445B9857B32868A2446D12B264AE3F|

139의fixture runtime/build-cache/tmp/review-source-cli-1848-{0,1}는요청/stdout/stderr/PID/exit/매호출전후CLI SHA를보존한다.실호출E828…와137정상bin이일치한다. 이전050E…실행을새E828…성공으로계상하지않고재실행했다. 실제GUI player는시작하지않았다. manifest/고정소스/실제표본의보장범위는각각구분한다.

## 남은 개발

전체190은선택된통합회귀범위다. 다른media custody/실제9원미디어연결·Verified timeline/PTS·모델별executor/다운로드/제작·AI원본/후보수정판정이력·typedGUI통합/시안1:1시각인수와실제사용자품질은여전히남는다. RD별diskfull/permission/root교체·schema전수·실CLI ACK유실등도완료로표시하지않는다. goal은active이며배포범위아니다.

스킬은기존 modoo.today build-cache-correctness-audit/shared-checkout 및test-fixture-and-mock-build의입력identity·producer독점·실시험과mock구분을이어적용했다. 정확출처/본문SHA는 [B0](../../../evidence/environment/source375-datalab-b0-20261026.md),[case](review-source-case-custody-20261026.md),[process](review-source-process-recovery-20261026.md)를따른다. 원WIP/소유권/증거·추론/다운로드/수동GUI/WARP/push보류유지.
