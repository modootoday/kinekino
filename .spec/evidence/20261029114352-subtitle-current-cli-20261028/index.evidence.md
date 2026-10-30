---
id: evidence:kinekino/20261029114352
title: R95 첫 실패·R96 현재 자막 CLI 인수
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R95 첫 실패·R96 현재 자막 CLI 인수

2026-10-28 중앙 PM. 직접subtitles.convert/export_range와proposals.export_subtitles의현재입력근거를검증했다. 제품코드·정상CLI변경0,시험의증거보존/기존구성fixture만수정했다. 최초conversion은임시파일을삭제하므로선택환경에서입력/출력·요청/stdout/stderr/process/CLI SHA를Rust로보존하게했고기존로컬코퍼스경로/SHA를명시선택하도록했다. 원시험은mirror/runtime/verification/preimage-r93-subtitle-evidence에보존했다.

## 원 R95 실패와 자원 규칙 위반

R95 SOURCE9DF9AB744BF254ADF76B36B1C9E8C7E8BD3C2C9A29F35635A7D551E31933AE8C,inventory5BC7BF774E03788138C3D82B894B74886898553EF8D1ABAE748D794818C59D0B/2336. 1877fmt/1878Native strict통과후1879no-run은workspace의동명plugin target도컴파일했지만실행하지않았다. 1880은-p kinekino-player로실제품target3개를확정했다. 1881snapshot후1882conversion은7pass/2fail/1ignored·exit101/12344ms다. 실패는configured_sdk_and_cli_discovery_match_conversion_support와object_safe_client_on_worker_matches_cli_typed_conversion의capabilities전체동등비교다. 원SDKfixture는현재CLI의LocalSpeakerFiles주입이빠져있었다. 원전체assert/비교의미를유지하고fixture에동일port를추가했으며제품지원표시를약화하거나비교필드를삭제하지않았다.

PM이1882의livehandle종료를확인하지않고1883코퍼스/1884range를시작해잠시중첩됐다. 이는heavy1/선행종료확인규칙위반이다. 원raw/receipt/실패를보존하고이중첩실행의통과를최종인수근거로사용하지않는다. 이후원handle의terminal을확인하고새R96입력·새evidence디렉터리에서각handle종료후다음실행으로직렬재검증했다. R95최종custody는수정전에실행하지않았으며새R96custody로대체성공표시하지않는다. fixture수정전R95소스는preimage-r95-subtitle-parity에보존했다.

## R96 현재 직렬 실행

SOURCED9111DC774A224299B111179B63CC0135B0A0F8330F8521DB6AE155C57138F88,inventory478DDB64401D2B60A1B4A760B1459A198FA2566B661A0A7D0C031B113DE6164E/2336. mirror/runtime/verification/input-r96. fixed-r95와fixed-r96에각원시험exe/PDB를보존했다. R96아티팩트목록은runtime/verification/source-sdk-r96-subtitle-artifacts.json이다.

|파일|SHA256|
|---|---|
|subtitle_conversion-ffa2fe11b445079b.exe|0F5881EF456D78D8209DBB2378D73040071ACC86E04AA3C3AF5D6775BBF75C36|
|subtitle_range_cli-8eb689467d6119e3.exe|FE208A7A444A5316EF9DB9C29EEDD4C7149D75D0CBF203401404758651B7A65E|
|accepted_subtitle_export_cli-e669e6974933555f.exe|68708707B7AB4DD353F6E649E4E669B3C72671F6B390283D7E7EA1566015C4D4|
|정상CLI(원R92동일byte)|5ECE053CD866500B21F3808A76503A43C421724F09B961D44B7565ED51621BAE|

원로그root runtime/verification/source375-datalab-b0-r1-logs:

|로그|범위|실제결과|
|---|---|---|
|1885/1886|fmt적용/Native strict|exit0|
|1887/1888|실제품no-run/snapshot|각exit0|
|1889|직접변환·CLI/typed/동시발행/거절|9pass/1기존opt-inignored|
|1890|기존Sintel자막실제코퍼스왕복|1pass·원ignored명시실행|
|1891|직접구간내보내기|4pass|
|1892|채택자막CLI저장경계batch|1pass/8filtered|
|1893/1894|default strict/fmtcheck|각exit0|
|1895|최종custody|exit0·2336content/identity동일|

총선택15pass는명령15개나CLI호출15회가아니다. 1889의ignored는1890에서동일입력의별도실제시험으로실행했다. 원input/output/request/process/raw는runtime/verification/source-sdk-r96-subtitle-r1/{conversion,corpus,range,accepted}에보존됐다. accepted는명시fixed-r92 CLI경로/SHA를다시검증했다. legacyconvert/range의wait_with_output관측을원family cleanup전수검증으로확대하지않는다.

기존runtime/ai-spike/source/sintel_ko.srt는1634bytes/SHA241F05979875655EAADF7ED98F5AE98CBB712845D33C80BC2FC540B349A61391이며실행전후동일했다. 첫빈줄을전처리하지않고SRT→VTT→SRT의cue텍스트/시간을대조했다. 새다운로드·추론·사용자미디어수정없다. 다른syntheticfixture와이기존코퍼스를구분한다.

변환/구간은Unicode·원본/시간·clip/rebase·손실승인·덮어쓰기/동일경로/hardlink·크기/인코딩/시간오류·한발행자승리·미지원context거절·typed/fileCLI의동등을확인했다. 채택내보내기는합성원SourceSnapshot의기술적accept를명시소비했고SRT/VTT·기존출력거절·u64큰시간정밀도손실의미확인NeedsInput/명시확인후출력·collapsed구간InvalidArgument·원proposal상태불변을확인했다. 기술적accept/산출물을AI사용자품질승인으로계상하지않는다.

GUI시각·실방송·ASR/화자분리·나머지모델executor·모든cancel/IO/crashfault는별도미완료다. 제품소스미변경으로원R92지정190/정상bins를새R96실행으로승계하지않는다. 기존WIP/소유권/원실패/privateoverlay·WARP/추론/다운로드/수동GUI/push보류·품질0/9·goalactive유지. C-only/cache경계·CPU2/BelowNormal/offline/locked/-j1유지,원heavy중첩위반/직렬수정재실행을위와같이구분했다. 적용modootoday/software-qa/test-fixture-and-mock-build A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100 및code-change-review-verification 9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B:동일구성전체비교·원실패/증거·현재실제/역사/사용자품질을분리했다.
