---
id: evidence:kinekino/20261029114551
title: Capture 원 기록 교차검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Capture 원 기록 교차검증

2026-10-26 중앙 PM. capture의 원 INPUT/request/response/process와 PNG를 정규화된 capture 관측에 연결했다. 필요한 값은 원 기록에서 대조하며 Default로 채우지 않는다. 실제 bundle/production admission은 아직 발행하지 않았다.

필수 CaptureContract::SdkFirstSourceFrameAtOrAfter와 CaptureExpectation을 사용한다. Expected는 source/engine/probe/producer CLI SHA, requested us, actual signed PTS/timebase, PNG SHA/bytes/dimensions를 명시한다. 원 request는 production.capture_frame와 명시 timeout/시각/source/output을 가져야 한다. 원 response의 ID·버전·ok/error·시각/tick·PNG를 대조하고, 원 process의 동일 CLI SHA/FileID·PID·exit0과 INPUT의 source/tool/probe/CLI를 확인한다. 경로는 문자열 비교에만 사용한다.

기존 FrameCaptureCapability의 unavailable Default나 CapturedFrame.output_sha256의 legacy None은 권한 상태로 쓰지 않았다. 필요한 필드가 없으면 거절한다. elapsed는 관측 metadata이며 종료 성공을 대신하지 않는다. source-content SHA와 파생 PNG byte 일치를 구별하고, INPUT의 역사 FileID를 복사된 object의 현재 FileID로 오인하지 않는다.

## 검증

204 timeline include-ignored19pass/0fail,시험3.04초.기존17에명시capture계약검사와원3capture기록검사2개를추가했다.원INPUT1/request3/response3/process3/PNG3=13파일을기존Sprite11384폴더에서SHA고정해읽었다.0/17/25.5초와0/208896/313344tick,timebase1/12288,실PNG decoded dimensions/bytes가일치했다.새capture/decode/추론/GUI실행은없다.

합성resolver negative에는원request 시각/response tick/producer CLI/PNG SHA/응답상관ID/원exit/contract누락7조건을추가해거절을확인했다.명시상태에Default를쓰지않으며누락계약도거절했다.원실제파일시험은요청JSON의메모리사본만바꿔불일치거절을검사했다.

원producer CLI SHA는8D06F1919D05ABEDCDAA799C9A7B43FA42CF8F14B5C092FD6EB4D6D058CF99F9다.이는보존관측을생산한역사binary의기록이며현재CLI의새실행으로계상하지않는다.원source7CFC8B8C…/FFmpeg09948D4C…/probeA6618E99…과분리해확인했다.

205 all-targets strict는진행중이다.종료확인전통과계상0.원WIP·소유권·원자료·heavy1/C캐시및추론/다운로드/수동GUI/WARP/push보류유지.이후계는396기준190/정상bins이후변경이다.

205 최종은종료0/34517ms,fmt --all -- --check도종료0이다.204/205동안코드변경없음.기존blitz-paint 의존deprecation원로그보존.index0/push0.

| source | SHA256 |
|---|---|
|capture_execution.rs|385532C3C0CC419C5D2FD00399400C21D81628B982CBD6B2AFE5A72D6EADE3E5|
|capture_execution/tests.rs|F96ADECD637B2341DA2A4F19FA941814C48840EB5AF6A7E4E11411ADB33EDA87|
|package/manifest.rs|A20579AEFB7FF63AE2FA4D32C79B4191CB2AC2A04C4CBEDCEB4350D26CD0647A|
|package/mod.rs|35C664ADC2060C235136DC75E6281256B9C0B9A1689271EDFF7BBCB5B71A3340|
|package/tests.rs|7D1A05317FE15063733B34AD882DE267EDCFDCA7858F1D051D8D83EEF8483FF4|

다음은원source/tool/process와관측파일을보존한별도실제기술표본묶음발행·고정·신뢰입장및실resolver인수다.이Sprite자료를actual9에붙이지않으며Verified/SDK재생clock·AI/GUI품질은미완료다.기존modoo.today test-fixture-and-mock-build/code-change-review-verification의역사관측/현재실행·원source/파생identity분리를적용했다.[원명령검증](timeline-original-execution-contract-20261026.md),[bundle계약](../../player/application/timeline-producer-bundle.md)을따른다.
