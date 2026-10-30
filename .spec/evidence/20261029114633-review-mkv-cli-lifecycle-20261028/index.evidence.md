---
id: evidence:kinekino/20261029114633
title: R82 실제 MKV 정상 CLI 수명 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R82 실제 MKV 정상 CLI 수명 검증

2026-10-28 중앙 PM. R81의 재열기5초 첫실패는 보존하고, 제어후관측5초와 준비·재열기관측20초를 분리했다. 성공시 budget/elapsed/마지막상태를 WAIT.json으로보존하고기한후도착을거절한다. 변경은시험controls.rs1파일이며preimage는mirror/runtime/verification/preimage-r81-reopen-budget이다. 제품기한/구현은이번변경없다.

SOURCE `511D3F5946A6155C51527E54543EC8E1209F8C95383DFE41C654D6905A824272`, inventory `F74F0AF43577D1CB07578982AE4FFE4A54D6A90C6511B961E02A59CBD89EBF7E`/2324. mirror/runtime/verification/input-r82. fixed-r82에startup시험exe/PDB/정상CLI보존. 시험SHA `099119BD5DBF2E774AA7162E6450B7DCDB5311DB7B514982169361387266CF9F`,CLI는원R81 EEF4EAE51B58914D65F8297C40581D1E3A64A515614E6525F64FDCDB0FA8175B/37662208bytes를보유guard로검증했다.

로그root runtime/verification/source375-datalab-b0-r1-logs:1721fmt적용0,1722Native strict0,1723startup no-run0,1724snapshot0.

1725 실제 controls_and_eof는1pass/69.34sec. 원root는mirror/runtime/verification/actual9-mkv-cli-controls-r82-r1. create/verify/replay·실제연결·오instance/외부범위/foreignclose거절·시크/play/pause/볼륨/음소거·close/replay·종료후control거절·새session재열기연결·원stdinEOF까지통과했다. reopened-WAIT.json은budget20000ms/elapsed8946ms·connected=true다. first/PROCESS.json은pid27456/exit0/68898ms/forcedfalse/stdin_eoftrue/original_child_waited/streams_joinedtrue. RETURN은controls_checkedtrue,restart_checkedfalse,원source/producerguard동일,historicalmediafalse/qualityfalse다.

1726 host2회재시작은1pass/92.23sec다. mirror/runtime/verification/actual9-mkv-cli-restart-r82-r1의RETURN은restart_checked/old_session_rejected/old_stored_association_preservedtrue,normal_hosts2,source/producer동일이다. 첫host pid26568/46473ms·둘째pid5280/45299ms모두exit0/forcedfalse/stdinEOF/originalwait/streamsjoinedtrue. 새producer의서로다른case/scope와원association revision2보존을확인했다. 실제표본은gemma270-highlights의명시선택MKV이며전체9개·역사모델시간축/품질승인이아니다. vo/ao=null로화면·소리출력품질을입증하지않는다. 기존WIP/소유권/원실패/privateoverlay·수동GUI/WARP/추론/다운로드/push보류·품질0/9·goalactive유지.

1727default strict0·1728fmtcheck0을확인했다. 후속동일입력지정190/정상bins는[통합회귀기록](source-sdk-r82-integrated-regression-20261028.md)을따른다.

적용modootoday/software-qa/test-fixture-and-mock-build A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100 및 code-change-review-verification 9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B. bounded시간·실제상태·원process정리와품질승인을분리했다. C-only/heavy1/CPU2/BelowNormal/offline/locked/-j1유지.
