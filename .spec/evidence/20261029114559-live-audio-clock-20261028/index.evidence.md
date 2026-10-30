---
id: evidence:kinekino/20261029114559
title: R77 live 오디오·새 MKV 시간축·SDK association 인수
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R77 live 오디오·새 MKV 시간축·SDK association 인수

2026-10-28 중앙 PM. [live audio 계약](../../contracts/production/live-audio-clock.md)으로원R76상한실패를수리했다. 새실제실행에서video/PCM/replay/capture/publication/reader와SDK association revision2·동일요청재사용까지통과했다. 이는**명시선택미디어의기술적시간축**이며원AI실행의historical clock·재생·GUI·사용자품질승인이아니다.

R77 SOURCE470BAD6B32CA95470F8C33B74BFB5D87358AB032056D12A44B1C4917C8DC4036, inventoryBDBB50B27253ED9DE895A2207353109453E7A9BF3FB6998DCB61BD9C1DB67FC9/2322파일. mirror runtime/verification/input-r77와preimage-r76-live-audio에원4파일을보존했다. 전역RetainedRaw::open·archive의12MiB는유지하고live입장/production/reader만원configured budget(최대64MiB)을명시한다. 알려진matrix/downmix진단과엄격한frame행을구분하며전체raw를보존한다.

원로그 root는 `runtime/verification/source375-datalab-b0-r1-logs`다.

|로그|범위|실제결과|
|---|---|---|
|1669|Native all-targets strict|exit0/23249ms|
|1670|Native lib/actual integration no-run|exit0/158302ms|
|1671|R77 snapshot|exit0/67983ms|
|1672|live/legacy행·PTS·형식·중복/unknown진단·크기|4pass/1보존실관측opt-inignored|
|1673|archive 입장/변조/원case보유|3pass|
|1674|새실제MKV producer→publication→reader→SDK association|1pass/exit0/36731ms|
|1675|default all-targets strict|exit0/18467ms|
|1676|fmt --check|exit0/2832ms|

고정exe/PDB는 `runtime/build-cache/native/source-sdk-integration-r1/fixed-r77`, 목록SHA/bytes는 `runtime/verification/source-sdk-r77-fixed-artifacts.json`이다. unit선택7과actual1을전체lib실행으로표현하지않는다.

## 실제 근거

root `runtime/verification/actual9-matroska-timeline-r77-r1`에서원R74/R75/R76와별도새producer를실행했다. 입력은R72준비caseB1E2BE1056E044CD3A8F9E41D16D768B6292DE81C2467890D79EEE844268E390/gemma270-highlights·실제681285280bytes/SintelF12C070E295B38CFC94EBD61AC3357C3BAC82D1015985F1E8DA93A4C6496C46D다. 원도구pin·1CPU/Job1GiB/원120초deadline/64MiB output을유지했고한도를우회하거나오디오를줄이지않았다.

- video841프레임의원PTS/duration·단일위상cadence를검증했다.
- audio27752프레임/16000Hz,첫PTS0·총14208512samples. 원PCM SHA6F6998DF98485823528BE8F2BB0096194AE33A174FE3719316056C5DC1CFD4CC와replay의전체bytes/SHA가같다.
- clip시작1716000sample/424800samples, SHA9A5611A0E87A6CC75718BFCF0151CAE0C6DFA1BB56A1D2D643AD9C1F75333E34를원슬라이스와대조했다.
- origin/start/end 실제캡처와source/tool/clock교차검증·원bundle발행·published reader재검증을통과했다.
- video/audio/replay/capture3개의원PROCESS6개모두exit0/JobEmpty/nonforced를기록했다. PID재열기나과거성공으로대체하지않았다.

published case는543C3C99D067A24664270E690D4FB747C8D7B23AA0E92EE34F37D1A00D5DD738, evidence는live-d3ffb8e1b1cd-manifest/B8DDCA6A355AED511260BF784FBA86342A183610536D2A51F1D1759C321EF7CD다. `published-identity.json`에원base_case·새case·media·historical_media_verified=false/quality_approved=false/playback_executed=false를보존한다.

SDK create후opaque PublishedTimelineResolver의근거로association actual-mkv를revision2로검증했다. `association-verify.json`의source_start_us107250000/source_end_us133800000,source timebase1/1000,origin0,start frame107250/end frame133833을확인했다. 같은key replay는같은receipt이며playback_ready=false·automatic_application_enabled=false다. 원9의proposal/판정/range=None를이technical association으로자동이동하거나덮지않는다.

## 한계와 다음 단계

실측은실제9중gemma270-highlights의명시미디어선택한case다. 원9전부의새producer/Verified association이나원모델의같은source clock을인수한것이아니다. 정상CLI의MKV --serve-review→association→mpv 제어/EOF·재시작,GUI원본/후보/수정/판정/이력·시각인수는남는다. engine재생은이번미실행이고사용자품질0/9다. 취소/저장/진단종류/더큰source/VFR전체행렬도미완료다.

원R74 MOV오지정·R75 PTS거절·R76기본12MiB실패는원snapshot/raw/실행파일에유지한다. 새실행을원실패의성공으로재계상하지않는다. 이번first compile/actual실패없음. source375 private overlay/canonical미적용·원WIP/소유권/보류유지,새추론/다운로드/수동GUI/WARP/push0,goal active.

같은R77의지정190/정상bins·최종custody는[통합회귀](source-sdk-r77-integrated-regression-20261028.md)로분리한다. 적용스킬은기존 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100),code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)이며관측/후보·물리종료/검증/사용자품질을구분했다. C-only heavy1/CPU2/BelowNormal/locked/offline/-j1유지,snapshot만read-only로compile과겹쳤다.
