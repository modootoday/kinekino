---
id: evidence:kinekino/20261029114346
title: R81 MKV 실제 재생·제어·close 및 재열기 첫 실패
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R81 MKV 실제 재생·제어·close 및 재열기 첫 실패

2026-10-28 중앙 PM. [분류 기반 demuxer 계약](../../contracts/production/review-playback-demuxer.md)을 검토 재생 어댑터에 연결했다. 보유 identity.classification을 arguments에 전달하고IsoBmffHeader는MOV+enable_drefs0,MatroskaHeader는matroska로분리했다. file프로토콜/자동외부로드거절/no-config/guard/소유권은유지했다. 원2파일preimage는mirror/runtime/verification/preimage-r80-mpv-demuxer.

SOURCE `D9AE9E4F34644C0C23E22E7DC304C3A2DAED573B272C71CB5E66B8290EC6F255`, inventory `6034DF452F05AEFCA5976B30CBD5F8A92EE5555AB796C87DCC96E24F0C6F80FE`/2324파일,mirror/runtime/verification/input-r81. fixed-r81에시험exe/PDB/정상CLI보존. CLI SHA `EEF4EAE51B58914D65F8297C40581D1E3A64A515614E6525F64FDCDB0FA8175B`,시험 `CA48D59C59F1DBB867152BB2F26A736F3BBDA1F6190CE25D4DDAD779C3C9F340`.

원로그 runtime/verification/source375-datalab-b0-r1-logs:1712 fmt적용0,1713 Native strict0,1714 lib/startup no-run0,1715실제새인자시험1pass/1569filtered(다른workspace lib의0tests는별도),1716snapshot0,1718defaultstrict0,1719fmtcheck0. 전체lib/190/전체CLI성공으로확대하지않는다.

최종1720 custody exit0·2324입력 content/identity 동일. 이후player-cli/mpv/ffmpeg/ffprobe 조회대상0개는재열기정상EOF의receipt로계상하지않는다.

1717 실제MKV시험은FAILED/exit101/66795ms이나첫세션의실제기능은다음범위까지관측했다. raw는mirror/runtime/verification/actual9-mkv-cli-controls-r81-r1/first다.

|범위|원근거·결과|
|---|---|
|association 생성/검증/replay|create/verify/verify-replay 성공·revision2|
|실제engine연결|status-7전후 connected true·engine_instance1|
|오소유자/범위거절|control-wrong-instance·close-foreign revision_conflict,play-before-seek/seek-outside-window needs_input|
|시크·재생·일시정지|window-start-0·playing-0·paused-again-0 관측. playing은108000000us·paused=false·duration888032000us·has_error=false|
|볼륨·음소거|volume-muted-0에서33.0/true관측|
|정상close/재사용/종료후거절|close·close-replay 성공,closed-0 owner_finished/physical_closed/family_empty/ipc_exited 모두true·has_error=false,control-after-close not_found|
|재열기|reopen 새session ordinal2응답성공,연결은시험5초기한내미관측|

실패는controls.rs wait의고정5초에서reopened-4의process_created=false/has_error=false를보고panic한것이다. 시작설정10초및큰실파일검증을포함한재열기시간과시험대기범위가불일치한다. 완료/엔진실패를추정하지않는다. 다음은기존제어관측5초와별도로재열기준비의명시bounded대기계약을정의하고재실행하는것이다. 원실패/raw를보존하며단순silent delay나상태생략으로우회하지않는다. 전체시험RETURN/host정상EOF·restart는미완료다.

vo=null/ao=null기능시험이며시각/청각품질승인이아니다. 원AI9개NeedsRepair·사용자승인0/9·historical권위false유지. 기존R78~R80실패·WIP/소유권/private overlay·수동GUI/WARP/추론/다운로드/push보류유지. 지정190/정상2bins는원R77근거다. C-only/heavy1/CPU2/BelowNormal/offline/locked/-j1유지. 적용modootoday/software-qa/test-fixture-and-mock-build A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100,code-change-review-verification 9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B:실제상태관측·선택성공·전체실패를분리했다.
