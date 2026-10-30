---
id: sot:kinekino/live-clock
title: Live 오디오 clock의 명시 예산·진단 행
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Live 오디오 clock의 명시 예산·진단 행

2026-10-28 중앙 PM. R76의 source PCM stderr12593930bytes는기본12MiB를11018bytes넘었지만원64MiB combined output예산안이었다. 전역 RetainedRaw::open 및archive의기본12MiB를변경하지않는다.

live producer는 `RetainedAudioClock::open_live(path,sha,max_bytes)`로원실행의명시출력예산을전달한다. 기존 RetainedRaw::open_bounded의0초과/64MiB이하·전체hash·원파일/디렉터리guard·최대읽기한도를그대로사용한다. live publication reader는constructor-private token의같은budget으로원original-pcm stderr를읽고같은live parser로다시검증한다. raw PCM replay와archive는기존정확행/기본입장을유지한다. 호출자가요청JSON으로별도audio한도를높이는API는없다.

원raw는자르거나새행만추출해대체하지않고전체내용/hash를보존한다. live parser는기존프레임행의n/PTS/정수samples·s16/mono/16k·정확연속성검사를유지한다. float pts_time은시간권위로사용하지않는다. 최대50000프레임·관측행4096bytes와원byte budget을적용한다.

프레임기록뒤의known side-data진단은matrix encoding/downmix두종류만허용한다. 각종류는프레임당한번,첫프레임전에는거절하며다음프레임에서허용상태를초기화한다. 진단에n:/pts:/nb_samples:/fmt:/channels:/rate: 같은프레임필드가섞이면거절한다. 알수없는ashowinfo행이나필드누락frame을광범위하게skip하지않는다. 진단은clock프레임/sample을추가하거나변경하지않는다.

default parse/open은이live구분을자동선택하지않는다. 기존archive에side-data행을사후주입하거나한도인상으로원실패를성공으로바꾸지않는다. 실제source/tool/process·fullPCM/replay/clip대조·origin/video/capture·opaque 입장·association이모두통과해야하며부분parser성공은전체timeline/재생/품질인수가아니다.

검증은정상side-data의동일프레임결과와archive거절,첫행전진단·중복·프레임필드혼합·unknown진단·누락frame·PTS불연속을확인한다. 이후새root에서같은고정source/tools의실제producer전체를실행한다. 원R74/R75/R76실패·raw·WIP/소유권/보류와사용자품질0/9를유지한다.
