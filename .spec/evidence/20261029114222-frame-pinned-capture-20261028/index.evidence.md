---
id: evidence:kinekino/20261029114222
title: R43 명시 도구의 실제 로컬 프레임 캡처
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R43 명시 도구의 실제 로컬 프레임 캡처

2026-10-28 중앙 PM. [도구 핀 구현](frame-pinned-tools-20261028.md)을 사용했고 제품 소스 변경 없음. R43 SOURCE DD8F66FD7F920762B27B106879CA62D708765824A842AA4224DF4E4BAC800528/2300에 한정한다. 고정 CLI53F826215AFD5E4CB4DBA241624ADDCD52A97F8701DDD3B8F4E7D4B2BDF086DF/36489728, 명시 ffmpeg09948D4C·ffprobeA6618E99 핀을 전 실행에 지정했다.

## 원본과 산출물

기존 로컬 `runtime/ai-spike/results/production-export-76968/portrait.mp4`,932830bytes,SHA0FD9B2DE74774B4D2DCEA9F9B66466899B44AFF1300B121D548E10AF4C386483. 새 미디어 생성·다운로드·추론 없음. 요청 자료와 결과는 `runtime/verification/source-sdk-r43-pinned-capture-r1`에 보존했다.

1330 실제 CLI production.capture_frame은 요청500000µs→실제500000µs,6144ticks,timebase1/12288,720×1280 PNG를 반환했다. frame.png는199763bytes,SHA0B73162584F488C517833254E029CCB4D26F6C966D126D7DE7E63F1DD7B9DE2A로 응답 output_sha256와 파일 hash가 일치했다. 원 제품은 공개 전 bounded PNG decode/CRC 검사를 수행한다. 이번 별도 검증에서 새 독립 PNG decoder를 실행한 것으로 주장하지 않는다.

## 실제 실행 결과

후속1335-source-sdk-r43-capture-custody의 verify-current는 동일 R43 SOURCE 핀에서 terminal0·2300 content/identity unchanged를 확인했다. 이 turn의 source 변경은 없으며 별도 새 snapshot epoch를 만들지 않았다.

원 로그는 `runtime/verification/source375-datalab-b0-r1-logs`의 아래 번호다. 모두 CPU2/BelowNormal/conservative2로 직렬 실행 후 terminal을 확인했다.

|로그|실행|원 terminal 결과|
|---|---|---|
|1330-source-sdk-r43-pinned-capture|정상 로컬 캡처|exit0/585ms,PNG 발행|
|1331-source-sdk-r43-capture-no-overwrite|같은 요청 재실행|exit5/400ms,RevisionConflict,기존 출력 유지|
|1332-source-sdk-r43-frame-independent-pts|같은 고정 ffprobe의 별도 bounded frame 목록 관측|exit0/40ms,5632ticks=.458333s 뒤6144ticks=.500000s|
|1333-source-sdk-r43-frame-beyond-duration|999999999999µs 요청|exit2/420ms,InvalidArgument,미발행|
|1334-source-sdk-r43-frame-timeout|timeout_ms=1|exit10/374ms,Timeout,미발행|

1331/1333/1334의 nonzero는 의도한 거절이며 성공 코드로 바꾸지 않았다. 폴더 목록에는 요청JSON3개와 정상frame.png만 있고 beyond.png/timeout.png·임시 출력은 없었다. 마지막 원본/PNG/CLI/두 도구의 hash는 모두 위 원 핀과 동일했다. 별도 FFprobe 관측은 제품 GUI 현재 프레임·시각/색상 동등성의 증거가 아니다. resource-runner의 child terminal과 원 raw/receipt이며 C2 process-family Empty 증거로 확대하지 않는다.

## 인수 범위와 남은 작업

명시 로컬CLI 구성의 실제 정상·덮어쓰기·PTS범위·시간초과 근거다. 현재 color conversion/HDR/player parity·rotation·burn-in 제한은 원 응답대로 미검증이며 GUI 시각 인수와 AI 사용자 품질 승인0/9는 유지한다. 모든 container/VFR/rotation/IO fault·취소/전체SDK/CLI 인수를 뜻하지 않는다. 기존 R40 지정190/Native정상bins는 각 원입력의 결과로 보존한다.

다음은 누적 R41~R43 입력 변경의 영향190 및 Native 정상bins 검증이다. CLI help·기본/serve/GUI host의 명시 도구 구성 소비와 안정97 discovery는 여전히 후속이다. 기존 source375 private overlay·canonical 미적용·WIP/소유권·추론/다운로드/수동GUI/WARP/push 보류 유지.

modootoday/software-qa/test-fixture-and-mock-build SHAA393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100 및 review-verification SHA9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B의 실제 표본/통제 거절·scope 구분을 적용했다.
