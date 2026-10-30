---
id: evidence:kinekino/20261029114229
title: R43 Native 정상 CLI의 실제 선택 기능 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R43 Native 정상 CLI의 실제 선택 기능 검증

2026-10-28 중앙 PM. [통합gate](source-sdk-r43-integrated-regression-20261028.md)에서 고정한 새Native CLI를 직접 소비했다. defaultCLI53F82621/과거R40 CLI의 기능 성공을 승계하지 않았다. 제품 소스 변경0, R43 SOURCE DD8F66FD7F920762B27B106879CA62D708765824A842AA4224DF4E4BAC800528/2300 유지.

## 고정 producer

- CLI: fixed-r43/native-bins/player-cli.exe,DED1BF0DE4E3FF5B741ADC05A4949CEF7DF3D81FD0B11318E18E92431ED00755/36756480.1343 normal Native compiler artifact.
- jobs 시험: fixed-r43/exact190-product/jobs-d75793112fc410a6.exe,C456EFD54BAC3474B379726A52D77EC1630E9FE8C3ED2435220A4685659E3EFC/28808704.1336 Native no-run artifact.
- operator index source-sdk-r43-nativecli-jobs-index.json,9EB703418EF282C06F99023297D3764192C0718600D90D0EA76E05E095BF9734/750,proof=false. 이파일 자체는실행증거가아니다.
- AGENTS40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C 확인.원소유권/WIP/보류 유지.

## 실제 결과

|로그|범위|결과|
|---|---|---|
|1345-source-sdk-r43-nativecli-jobs-lifecycle|actual_cli_explicit_host_and_sdk_share_durable_jobs_without_gui|정확1pass/65filtered,3.24s,runner terminal0/15682ms|
|1346-source-sdk-r43-nativecli-jobs-cancel|cancel::actual_cli_running_cancel_ack_and_publication_winner_keep_real_file_receipts|정확1pass/65filtered,5.69s,runner terminal0/6485ms|
|1347-source-sdk-r43-nativecli-pinned-capture|명시8.1.1도구로기존portrait500000µs 캡처|runner terminal0/525ms,720×1280·199763bytes|

원로그는runtime/verification/source375-datalab-b0-r1-logs에, jobs C2증거는source-sdk-r43-nativecli-jobs-{lifecycle,cancel}-os1/outer 및 inner에 있다. fixture는위CLI를복사하며원본/복사본SHA를동일하게확인한다. C2두outcome은 first_error=null,selected_one=true,ownership=complete,physical_closed=true,child_exit0,family=Empty,forced=false,stdout/stderr EOF=true였다.각 실행은total60000/cleanup5000,CPU2/BelowNormal로직렬종료했다.

1345는명시subtitle worker host의변환·출력보존·중복/충돌·watch/list·손실/재시도·fresh readonly·강제중단뒤Interrupted/재시작자동실행없음을검사한다.1346은convert/range의발행예약전후네취소경로다.예약후를파일저장후로해석하지않는다.기본CLI전체executor지원이아니다.

1347의요청·PNG는runtime/verification/source-sdk-r43-nativecli-capture-r1에보존했다.원본portraitSHA0FD9B2DE74774B4D2DCEA9F9B66466899B44AFF1300B121D548E10AF4C386483/932830,출력SHA0B73162584F488C517833254E029CCB4D26F6C966D126D7DE7E63F1DD7B9DE2A.응답의500000µs/6144ticks/timebase1/12288·출력hash가일치했고원본/CLI hash도유지했다.명시도구는R43계약의ffmpeg09948D4C/ffprobeA6618E99이며실제dependency.available=true다.과거1332의별도PTS관측은해당원실행으로보존하고이번재실행으로계상하지않는다.이Native CLI의capture거절/시간초과전수는미실행이며defaultCLI1331~1334결과를승계하지않는다.

## 판정과 다음 작업

최종1348 custody는동일R43 SOURCE 핀에서terminal0·2300 content/identity unchanged를확인했다.새source epoch나전체190/bins재실행을만들지않았다.

새Native CLI의선택jobs2시험과실제capture1경로를인수한다.등록190재실행·전체CLI·실제HTTP·GUI/색상시각·AI사용자품질승인이아니다.다음은명시frame설정의CLI help/기본host소비와안정discovery,미지원SDK/CLI경로구현이다.전체CLI/GUI율은미산정,AI0/9·원후보NeedsRepair·source375 private overlay/canonical미적용·기존보류유지.

modootoday/software-qa/test-fixture-and-mock-build SHAA393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100 및 review-verification SHA9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B의현재producer실제소비·통제취소/실제파일·전체인수구분을적용했다.
