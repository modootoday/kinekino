---
id: evidence:kinekino/20261029114814
title: "R46: 명시 도구를 사용하는 jobs host의 공통 프레임 API"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R46: 명시 도구를 사용하는 jobs host의 공통 프레임 API

후속 [R46 지정190/정상bins gate](source-sdk-r46-integrated-regression-20261028.md)를 실제 통과했다. 아래의 gate 미실행 기록은 이 후속으로 해소되며 활성 frame 종료/취소·전체 기능 인수 한계는 유지한다.

2026-10-28 중앙 PM. `--serve-jobs`에 네 frame 도구 옵션을 함께 지정하면 같은 `PlayerSdk`에 검증된 프레임 어댑터를 주입한다. 옵션이 없으면 기존 jobs host의 프레임 미지원 상태를 유지한다. 프레임 캡처는 동기 SDK 명령이며 새 durable job executor가 아니다. 기존 worker·transport·SDK 종료 계약을 그대로 사용한다.

## 입력과 첫 실패

변경 전 파일 5개는 mirror의 `runtime/verification/preimage-r44-job-frame-host`에 보존했다. 파일별 SHA는 fixture.rs `FEE262EDBF2B8B28558062EE6A64DA85C21CCC194E52BCF089AD30026EDCEDE3`, frame_tools.rs `01E300A5D08F0377B533AF2CC215E1A93C6FC6A042CDFDDE2D23BD727498F060`, job_host.rs `B58DE845C460CE773B101B224304D4515610B43C0951E2C4EF314FB8D575F662`, jobs.rs `088344E11F4587CC1980C19E0FCE579AA0C7433CF641ACB6546CCF4A672156C3`, runner.rs `037110CDB1F975D2FE5BD78FFD95EE4F4013E721F8AFE5344DB8246C8DF1574B`다.

1360 Native strict의 첫 실행은 SDK 제네릭 타입 분기 불일치 E0308로 exit101/963ms였다. 실패 입력 R45 SOURCE `7985E19F626DC9EAEE161F8E50A39B3FAAE9D8867FE67E044D80D99E867A1D93`, inventory `A5FE845F36876ED76FEF0D9E4DA14AFF354269D128920A7570E3BD1480C662D7`/2302와 원 stderr를 보존했다. 원 Meari dependency의 fetch_update deprecated 경고도 숨기지 않았다.

실패 job_host.rs는 `preimage-r45-frame-host-type/job_host.rs`, SHA `371ABDCEA755F9C1850F47B72716A7AC5C2E8871629C6133B441C6AB801C4E70`에 보존했다. 기존 `ApplicationClientHandle`로 두 SDK 타입을 조립하여 수정했다. 중간1361 strict는 통과했으며 최종 fmt 후1364를 다시 통과했다.

최종 R46 SOURCE `9CA271E1617112BCC3B0DE73359B3A7854B8FDA78362EC9925ECED1934732C80`, inventory `3B58F8235E9849B5561DA50A1D9CD714E92979E4C3F4F91A8D75E34E8B5C69C7`/2302. snapshot은 compile과 읽기 전용으로 중첩했고 시작 후 제품 소스를 수정하지 않았다. AGENTS `40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C` 확인. private mirror이며 canonical 채택이 아니다.

## 실제 검증

|로그|검증|실제 결과|
|---|---|---|
|1362|Native jobs no-run|terminal0/24370ms|
|1363|default all-target strict|terminal0/2069ms|
|1364|Native all-target strict|terminal0/1379ms|
|1365|전체 fmt|terminal0/3975ms|
|1366|실제 jobs host의 endpoint CLI·typed capture·덮어쓰기·EOF|정확1pass/66filtered,2.15s; terminal0/14270ms|
|1367|틀린 핀으로 host 시작|SourceExpired, exit8/129ms|
|1368|중복 frame 옵션으로 host 시작|InvalidArgument, exit2/11ms|
|1369|옵션 없는 기존 jobs lifecycle|정확1pass/66filtered,3.09s; terminal0/14687ms|
|1370|후속 입력 custody|terminal0/48388ms,2302 content/identity unchanged|
|1371|기존 host 입력 계약|3pass/32filtered; terminal0/16054ms|
|1372|기존 CLI 계약|4pass,0.93s; terminal0/1454ms|

새 보존 CLI `fixed-r46/player-cli.exe` SHA `8965FEB1C38F80FBF06B170837AB92E5C68F7B901380EBDE712D6C062664105F`/36935680, 시험 `fixed-r46/jobs.exe` SHA `EB048F99870572F4ADA57E0745AAF29A4741B6069537BB990D0825882E27666F`/28844032. 원1362 Cargo producer를 복사했다. operator index `source-sdk-r46-frame-host-index.json`은 proof=false이며 실제 실행을 대체하지 않는다.

원 로그는 `runtime/verification/source375-datalab-b0-r1-logs/1360~1372-source-sdk-r{45,46}-*`다. 실제 C2 증거는 `source-sdk-r46-frame-host-os1/outer`와 `source-sdk-r46-legacy-jobs-os1/outer`에 있다. 1366은 명시 ignored 시험으로 실행했다. 신규 host 시험의 C2는 selected_one=true, first_error=null, ownership=complete, physical_closed=true, child_exit0, family=Empty, forced=false, raw EOF=true였다. heavy1/CPU2/BelowNormal, total60000/cleanup5000을 유지했다.

1366의 원 종료 기록은 dispatch=completed·dispatch_poisoned=false·pipe=joined·sdk=joined였다. worker의 cancelled/completed/interrupted 카운터는 모두0이다. 직접 frame API를 job 성공으로 계상하지 않았다. EOF는 캡처 요청 완료 뒤 정상 종료 사례이며 실행 중 frame 취소/EOF fault 전수는 아직 미검증이다.

실제 기존 portrait SHA `0FD9B2DE74774B4D2DCEA9F9B66466899B44AFF1300B121D548E10AF4C386483`를 사용했다. 명시 도구는 ffmpeg `09948D4C…`/ffprobe `A6618E99…`의 원 R43 전체 핀이다. endpoint CLI와 typed client 모두 500000µs·720×1280, PNG SHA `0B73162584F488C517833254E029CCB4D26F6C966D126D7DE7E63F1DD7B9DE2A`를 반환했다. 덮어쓰기 거절 뒤 출력·원본 hash를 보존했고, PNG 두 파일도 실제 hash가 일치했다.

1367/1368은 ready를 반환하지 않고 실패했으며 `source-sdk-r46-host-admission-r1` 안에 요청한 두 job store가 생성되지 않았다. 틀린 핀은 jobs 시작 전 검증하고 다른 도구로 fallback하지 않는다. 원 nonzero 종료를 성공 코드로 바꾸지 않았다.

## 인수 범위와 다음 작업

명시 옵션을 사용하는 jobs host의 공통 frame API와 정상 EOF, 기존 jobs 경로를 인수한다. ready의 frame_capture_configured는 설정 여부이며 실제 지원/품질 승인 표시가 아니다. 실제 지원은 capabilities와 캡처 결과로 확인한다. GUI·다른 serve host는 연결하지 않았다.

R46 전체190·정상 GUI/CLI bins gate, frame 실행 중 종료/취소·오류 행렬, 다른 container/색상/시각 품질·전체 CLI 인수는 남는다. R43의 지정190/bins와 R44의97 discovery는 각 원 입력으로 보존한다. source375 private overlay·원 실패/미실행·WIP/소유권·AI 사용자 품질0/9·추론/다운로드/수동GUI/WARP/push 보류를 유지한다.

modootoday/software-qa/test-fixture-and-mock-build SHA `A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100`과 review-verification SHA `9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B`의 실제 표본/통제 거절·원 실패·현재 producer 범위 구분을 적용했다.
