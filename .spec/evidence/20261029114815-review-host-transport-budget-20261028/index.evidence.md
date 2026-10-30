---
id: evidence:kinekino/20261029114815
title: R80 host 기한 연결·실제 엔진 첫 실패
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R80 host 기한 연결·실제 엔진 첫 실패

2026-10-28 중앙 PM. 고정SHA review-host 설정의 선택필드 transport_timeout_ms를 기존 TransportOptions에 연결했다. 생략5000ms·명시10..30000·producer 전 범위거절이다. client/host/SDK작업/producer/재생 기한은 독립이다. preimage는 mirror/runtime/verification/preimage-r79-host-transport. 제품 변경2파일(review_host.rs/settings.rs), 실제 표본 설정1파일(matroska.rs)이다.

SOURCE `DFBDF49684132548072ED826E2567AD34DF96C78674D51FD8BF61E4841744E5F`, inventory `4FFF015D8F6B0A0C7CA192CDC518A666727430B5F43EDB7718FF9FBCB85AAABD`/2324. mirror/runtime/verification/input-r80에 보존했다. fixed-r80에CLI/시험exe/PDB보존. CLI SHA `5FEEF4E88D795F3AB75C20D19F4C279C38E8363F084CF66A75AC81CD80907F8F`, 시험 `37FC35447CC805FFECED329F3230B9B26750B3BF9D1F5BF4F960977BA29CA743`.

원로그 runtime/verification/source375-datalab-b0-r1-logs:1702/1703 fmt적용0,1704 Native strict0,1705 CLI/startup no-run0,1706 host설정 시험3pass/34filtered,1707 snapshot0,1709 default strict0,1710 fmtcheck0. 전체CLI/190회귀 통과로확대하지않는다.

최종1711 custody exit0·2324입력 content/identity 동일. 이후 player-cli/mpv/ffmpeg/ffprobe 조회대상0개. host정상EOF/PROCESS는없으며engine PROCESS의familyEmpty/nonforced와구분한다.

1708 실제 MKV 시험은 association.create/revise/replay 및 playback.open 응답까지 성공했다. 새case A0A0C741에기술시간축을연결했으며원역사권위/품질을바꾸지않았다. 첫엔진상태관측에서 process_created=true,connected=false, child_exit2, family_empty=true,physical_closed=true,forced=false,ipc_exited=true를확인해시험FAILED. 제어/정상EOF/restart는미실행이다. raw는 mirror/runtime/verification/actual9-mkv-cli-controls-r80-r1.

원 review-session-32888-1793136714010429300/INPUT.json은classification MatroskaHeader·MIME video/x-matroska인데argv --demuxer-lavf-format=mov와enable_drefs=0을기록했다. PROCESS.json은pid35336/engine_instance1의exit2·familyEmpty/nonforced를보존한다. stderr는비어있어구체decoder오류를추측하지않는다. 현재arguments.rs가MOV를고정하는것을확인했으므로다음은보유한typed source classification으로demuxer를선택하고MOV전용옵션을분리하는수정이다. 분류/hash/protocol/file/자동외부로드거절·원안전인자를유지하며실제engine재실행으로검증한다.

이전R78/R79 timeout/connection실패·원raw·WIP·소유권·private overlay를유지한다. 지정190/정상2bins는원R77근거이며새입력완료로승계하지않는다. 수동GUI·WARP·추론·다운로드·push0·품질0/9·goal active. C-only/heavy1/CPU2/BelowNormal/locked/offline/-j1유지. 적용스킬modootoday/software-qa/test-fixture-and-mock-build A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100 및 code-change-review-verification 9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B: 전송·SDK연결·engine생성·실제재생판정을분리했다.
