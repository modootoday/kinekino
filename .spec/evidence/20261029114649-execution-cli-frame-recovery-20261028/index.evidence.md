---
id: evidence:kinekino/20261029114649
title: R87 정상 CLI 실제 probe receipt 복구
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R87 정상 CLI 실제 probe receipt 복구

2026-10-28 중앙 PM. R86 정상CLI의명시execution host를통해실제프레임API의probe receipt충돌→복구대기→명시retry→원오류terminal→ACK/EOF를검증했다. 제품코드/CLI실행파일은변경하지않았다. 시험은기존execution_host_cli의실제host harness에추가했고preimage는mirror/runtime/verification/preimage-r86-cli-frame-fault이다. 새receipt_fault.rs는외부Rust감시자로원host의새frame-owned디렉터리에create_new 충돌파일을만든다. 제품환경변수/시험훅/대체실행파일을주입하지않는다.

R87 SOURCEAC088062B0ACD85086B4F089B3D6C6DB47C1E3BB3922137D3EE9FB1754374638,inventory37484AE71AFA42A6F323A55A5BA94C6452C0CBF989959693D20B3CD6848BF14D/2332. mirror/runtime/verification/input-r87. fixed-r87시험exe SHA9BE5364F03A9BBBB716A23CC6725B97FB18AC7008F10256248CD73955600F62A/PDB 및정상CLI C99A1E666E53A84E4541C09B1CFF207095A8DAAD43CEF38D1BA45C035B9D6734보존. CLI는원R86과동일byte지만이번실제실행은별도원로그다.

로그root runtime/verification/source375-datalab-b0-r1-logs:

|로그|범위|실제결과|
|---|---|---|
|1785/1787|fmt적용|exit0|
|1786|초기Native strict|exit0|
|1788|실제CLI시험no-run|exit0|
|1789|snapshot|exit0|
|1790|정상CLI실제probe receipt복구|1pass/1.21sec·2filtered|
|1791/1792/1793|최종Native/default strict·fmtcheck|각exit0|
|1794|최종custody|exit0·2332content/identity동일|

raw root는mirror/runtime/verification/execution-cli-r87-frame-fault-r1. 원existing portrait.mp4(932830bytes/SHA0FD9B2DE74774B4D2DCEA9F9B66466899B44AFF1300B121D548E10AF4C386483)와R86도구핀을사용했다. 최초요청은production.capture_frame/500000us이며probe receipt충돌때문에최종PNG는발행하지않는다. 성공캡처·영상품질인수가아니다.

원host pid38480의root <원 호스트 저장소>/runtime/build-cache/tmp/frame-owned-38480-1793139532784810400-4에receipt-0000.json sentinel을생성했다. 명시retry전running/generation1/retry_requestedfalse·receipt0001부재·원미디어exclusiveopen거절로guard보유를확인했다. retry RPC는requested/completionunobserved이며이후terminal의원EngineFailed·physical_closedtrue/receipt_recordedtrue와같은evidence_root를대조했다. 원sentinel불변·원미디어SHA동일·frame.png미발행을확인했다.

receipt-0001.json은원ffprobe pid33060/creation_filetime134376131327865740/executable고정핀,child_exit0/familyEmpty/forcedfalse다. 원정리receipt를RECOVERED-RECEIPT.json에복사보존했다. 프로세스전체재실행여부의별도독립enumeration은하지않았으며원operation/evidence_root·receiptsequence·최초오류와원SDK실행구조로복구귀속을확인했다. FAULT.json의operation_reexecutedfalse는이계약의설명이며독립프로세스감사대체가아니다.

원operation명시ACK후hostpid38480은stdinEOF/exit0/forcedfalse/originalwait/streamsjoined를기록했다. 정상host종료와원operationEngineFailed를구분한다. 반복실패/16세대wire상한·부분쓰기/권한fault·응답손실/중복·미해결복구EOF/강제crash·scope/serveridentity거절전수·capabilities/schema는아직남는다. 다음은명시host발견/schema와오류행렬이며같은기본100명령지원/전체CLI완료로표시하지않는다.

원R82지정190/정상GUIbins/MKV수명은각원입력근거로유지한다. 기존WIP/소유권/원실패/privateoverlay·WARP/추론/다운로드/수동GUI/push보류·사용자품질0/9·goalactive유지. C-only/heavy1/CPU2/BelowNormal/offline/locked/-j1준수. 적용modootoday/software-qa/test-fixture-and-mock-build A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100 및code-change-review-verification 9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B:실제도구/외부fault·원오류/host종료·receipt복구/PNG발행을구분했다.
