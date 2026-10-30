---
id: evidence:kinekino/20261029114324
title: 원 관측 보존 cadence·실제841프레임 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 관측 보존 cadence·실제841프레임 검증

최종1664 exit0/42533ms·2322파일 content/identity 동일확인. 작성시custody진행중표기는이종료로보완한다. 추가1665기존evidence정책7pass,1666archive입장3pass,1667live argv계약1pass를같은고정lib에서통과했다. domain2/parser2와합쳐선택15pass이며전체lib/190완료가아니다. 원R76actual오디오실패는유지한다.

2026-10-28 중앙 PM. [양자화 계약](../../contracts/production/quantized-video-cadence.md)을 구현하고 R75 영상 실패를 새 R76 입력에서 검증했다. **영상 관측841프레임은 통과했지만 전체 timeline producer는 오디오 reader 상한으로 실패했다.** 새 association/Verified/재생 성공으로 계상하지 않는다.

## 고정 입력·구현

R76 SOURCE CDF0BAEA388B61096129A7402BECCF930DD0732273E0C454D2D7FCA746018F63, inventoryE3E4E413C6D2AFD1837E232EFD07CD8FBC2C99F900F6674630933BE06932022E/2322파일. mirror runtime/verification/input-r76와 preimage-r75-cadence에 원소스·후계를구분했다. 신규 domain/video_cadence와선택 live parser/producer/reader를연결했다. 기존 legacy/archive 관측에는cadence=None과원정확연속성규칙을유지한다.

rate 후보는 원 source/tool/process에서 관측한 두필드가같아야하며, domain은전체PTS가하나의공통위상구간에맞는지checked정수로검사한다. duration은floor/ceil 범위만검사하고원값을변경하지않는다. rate자체나매프레임별새허용오차로승인하지않는다. raw frames·선택 cadence를 retained object/bundle/snapshot에함께보존하고 reader가같은raw와domain조건을재검증한다.

원R75 source의진단1653은 timebase1/1000·avg_frame_rate/r_frame_rate24/1을보고했다. 이는후계설계의관측이며원실패JSON에추가하거나권위로재사용하지않았다. R76 fresh probe가새고정argv로rate두필드를직접생성했다. 공식후보의한계는[FFmpeg AVStream 문서](https://ffmpeg.org/doxygen/7.0/structAVStream.html)를따른다.

## 원 실행

로그 root `runtime/verification/source375-datalab-b0-r1-logs`:

|로그|범위|관측|
|---|---|---|
|1653|고정probe의읽기전용rate진단|exit0·24/1후보두개·1/1000|
|1656|R76 Native all-targets strict|exit0/39185ms|
|1657|Native lib/actual timeline no-run|exit0/200573ms|
|1658|snapshot|exit0/70433ms|
|1659|domain phase/원값보존/legacy·누락/중복/위치drift/부적절duration/rate/overflow|2pass|
|1660|parser후보일치·누락/불일치/legacy주입거절·기존signedPTS|2pass/1보존실관측opt-inignored|
|1661|새실제producer|exit101/7314ms·오디오byte bound실패|
|1662|default all-targets strict|exit0/28690ms|
|1663|fmt --check|exit0|

원 lib/actual integration exe/PDB는 fixed-r76와 runtime/verification/source-sdk-r76-fixed-artifacts.json에보존한다. lib전체1568을실행한것이아니라선택domain2/parser2를통과했다. 양strict는all-targets compile의근거이지전체시험실행이아니다.1664최종custody는작성시진행중이다.

## 실제841프레임과 다음 실패

실제root는 `runtime/verification/actual9-matroska-timeline-r76-r1`이다. source는원R72 준비case B1E2BE1056E044CD3A8F9E41D16D768B6292DE81C2467890D79EEE844268E390/gemma270-highlights, 실제681285280bytes의Sintel F12C070E295B38CFC94EBD61AC3357C3BAC82D1015985F1E8DA93A4C6496C46D다. 도구/자원/구간은원R75와같으며새root와새producer실행을사용했다.

video RETURN.json은frames841·video_frames_validated=true를기록했다. rawPTS/duration과같은고정source/tool의rate후보를보존한후공통phase검증을통과했다. 원video PROCESS는exit0/JobEmpty/nonforced다. 원R74 MOV오지정과원R75 PTS거절은과거실패로보존하며새성공으로덮지않았다. 이영상창의일관성을전영상CFR·원모델결과의historical clock/품질로확대하지않는다.

이후 audio-pcm PROCESS도exit0/JobEmpty/nonforced였으나RetainedAudioClock의기본RetainedRaw12MiB입장에서 ResourceExhausted(Content exceeds verification byte bound)로거절됐다. 원stderr12593930bytes는12582912bytes보다11018bytes크다. stdout28417024bytes와합계41010954bytes로원configured output64MiB보다작다. 단순producer timeout·출력64MiB초과·PCM불일치로표현하지않는다.

원audio로그에는정상frame행외에 Parsed_ashowinfo의matrix encoding/downmix side-data행이있다. 현재 strict parser는ashowinfo prefix의모든행에 n:/pts:/nb_samples:를요구하므로, byte한도만올리면다음형식오류가날수있는공백도확인했다. 이번에는그행을버리거나전역상한을늘리지않았다. frame0은PTS0/496samples, 마지막관측n27751/PTS14208496/16samples를보고하지만이번stage에서원clock전체검증을통과한것으로계상하지않는다.

## 다음 작업과 한계

live audio clock의원configured budget을명시한별도bounded입장/reader계약을만들고생산자·발행reader가같은한도를사용해야한다. archive/default12MiB와raw PCM replay의원계약은유지한다. live프레임행과알려진side-data진단행을구별하되잘못된frame행·알수없는ashowinfo행을무조건skip하지않는다. 원raw·hash·행/프레임한도와정확sample/PTS검사는유지하고,새fresh실행으로PCM/capture/publication/association까지이어져야한다. 로그를조용히하거나전체오디오를잘라증거를줄이는것을대체목표로삼지않는다.

전영상VFR·모든컨테이너/후보rate·누적drift전수·전체cadencefault는별도남는다. 실행한위치drift음성은원PTS한지점을2tick변경한사례이며모든누적drift행렬완료가아니다. 실제9 원range=None·사용자품질0/9·GUI미인수,원WIP/소유권/첫실패·보류유지. 지정190/정상bins는원R67,CLI100/103/24는원R72근거이며R76로승계하지않는다. 새추론/다운로드/수동GUI/엔진재생/WARP/push0,goal active.

적용스킬은 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100), code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)이며원관측·후보·일관성·물리종료·전체인수를구분했다. C-only heavy1/CPU2/BelowNormal/locked/offline/-j1유지, snapshot만read-only로compile과겹쳤다.
