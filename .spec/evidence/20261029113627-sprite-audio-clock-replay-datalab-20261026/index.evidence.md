---
id: evidence:kinekino/20261029113627
title: "Sprite PCM·audio clock 재현: 실패2건과 역사적 pin 성공"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Sprite PCM·audio clock 재현: 실패2건과 역사적 pin 성공

2026-10-26. [기존PCM slice](sprite-pcm-lineage-datalab-20261026.md)와[현재SDK capture](sprite-sdk-cli-capture-datalab-20261026.md)를잇는기술검증이다. 실제9·모델출력/판정/GUI는변경하지않았다. 원source7CFC…/fullPCM872987…/각decoderexe를guard/hash로고정했다. 한작업씩CPU2/BelowNormal·filter_threads1/decoderthreads2,stdout24MiB/stderr12MiB accepted bound·기한30s,원자료readonly·새PCM은C build-cache/tmp에만생성했다.

## 실제 실행과 판정

로그/receipt는runtime/verification/source375-datalab-b0-r1-logs/157~162다. 각rawreceipt의command.txt/INPUT/RETURN/process/CLOCK과rawPCM·stderr를보존한다.

| gate | decoder/조건 | 결과 |
|---|---|---|
|157 sprite-audio-replay-datalab-r1|현재8.1.1 SHA09948D4C…,-copyts/aresample16000:s16:mono:async0/asettb1/16000/ashowinfo|decode exit0·27128frame 정수clock연속. **기존PCM과byte378598부터불일치**,outerexit1/1816ms |
|158 sprite-audio-default-replay-datalab-r1|같은8.1.1,역사적기본-ac1/-ar16000/pcm_s16le. clocktrace없음|decode exit0,같은byte불일치,outerexit1. 두새rawPCM의SHA가동일하므로명시trace필터만의차이로설명되지않음 |
|159 sprite-audio-legacy-replay-datalab-r1|역사적pin과정확히같은6.1 SHA714147…로157과같은trace|**기존PCMdata전체일치**,27128frame 정수clock연속·inputidentity불변. outerexit0/2230ms,decoderPID26984/1362ms |
|160 정상CLI video origin|기본decoder는그대로8.1.1;0/17/25.5초capture·오류재시험|명시실미디어시험1pass/0fail,outerexit0/39134ms/시험8.76s. 8CLIprocess. 161fmt/162strict exit0 |

157/158의실패와결과를지우거나허용오차·offset을조정하지않았다. 이표는선택된고정build의재현차이이며모든8.1.1/6.1에대한일반품질판정이나특정라이브러리변경의원인확정은아니다. 6.1성공은과거성공의승계가아니라**새호스트에서같은exe SHA로직접재실행한결과**다.

## clock·byte 근거

157과159는first_pts0,end_pts10078204,samples10078204,frames27128,명시filtertimebase1/16000이다. 파서는모든n순서/PTS/nb_samples·s16/mono/16000형식과`next_pts=previous_pts+nb_samples`를검사한다. 누락/중복/불연속을보정하지않는다. float pts_time은판정에사용하지않는다. source timestamp는-copyts로보존하고async0이며asettb는단위표현을명시했다.

-원WAV의dataoffset78을파싱해기존20156408B PCM영역전부를대조했다. 159의rawPCM SHA **74F30535A9EF581F3A048917941DE6F41DAD1C40101149C1EBE84352A1300D8B**,기존data와exact match.
-157/158의각rawPCM도20156408B지만SHA **35D01424C116C7C4A031187A5A7DAB87282E9501BB0533AE7225DC69D3E95C85**,원data와불일치다.157stderr5070656B와실패raw모두보존한다.
-기존full→clip의[272000,408000)sample exact match는153에서입증했다. 지금확인한decoder/timebase/원점은이한source·한historicalPCM의기술근거다. 비디오duration과audio끝은서로다르므로전체video구간으로임의외삽하지않는다.

## 영상 원점의 공통SDK/CLI 대조

160의fixture runtime/build-cache/tmp/sprite-capture-cli-11384에서firstvideo requested/actual0us·tick0·timebase1/12288을실decode한PNG로확인했다. PNG5338B/1718×720/SHA6C2AC82030E7A1A584E85EF3053863287C376A00133F2E9462F480E693BAA38F.17/25.5초의208896/313344tick도다시통과했다. 앞선152 rawframe에도두tick이실제로존재한다.

동일CLI SHA8D06F1919D05ABEDCDAA799C9A7B43FA42CF8F14B5C092FD6EB4D6D058CF99F9와기본FFmpeg8.1.1 SHA09948…를guard했다. overwrite/범위밖no-output도통과. 이결과는SDK production.capture_frame이며SDK playback.seek/player clock이나GUI시각인수는아니다.

## 필수 의존성과 도구 검증

사용자의필수의존성최소설치승인에따라[공식6.1 archive](https://github.com/GyanD/codexffmpeg/releases/tag/6.1)에서project-local검증용exe/라이선스/README만확보했다. [의존성영수증](../../../evidence/environment/ffmpeg61-replay-dependency-20261026.md)에공식release/asset·26.6MB archivehash·714147…일치·전역/PATH/기본decoder불변을기록했다. 새추론/모델·미디어다운로드/수동GUI/WARP/push0이다.

비공개std-onlyRust관측기16시험/fmt/strict/build통과. 새clock시험은음수origin포함정수연속성과n누락/PTS불연속/잘못된rate/0sample거절을검증했다. 원PCMparser와compare를재사용하고firstmismatch byte를보고하도록했다. 157원관측기소스r1사본도실패receipt에보존했다.161rootfmt exit0/9802ms,162rootall-targets/native-proposal-window strict exit0/4833ms. 기존paint deprecation은원로그보존.

| source | SHA256 |
|---|---|
|private sprite_audio_replay/mod.rs|9FCBBBE90FF18A372C14A39F7A3110FD51B56B1787773EEE9E9AEB929315CB49|
|private sprite_audio_replay/process.rs|75377FD63CEE426397C93C0419DD1287866DA571255CF16FE11D41F73A0A7931|
|private sprite_audio_replay/clock.rs|C7FB37465E03E7B0045EEBD804A85E01383FC9F7D45FAA3F2119AD0F3662CBA4|
|private sprite_pcm.rs|A3024185317D829C54B7EEEF7A2F889157FB6BFCB2FD7C29A9A1A0CA97B6D58C|
|custody tool exe|78DD39F4DBB8C47CD0242B43A7E766CC92CD1DB17087F640E2BB64B6D94672D9|
|tests/sprite_capture_cli.rs(0시점추가)|359414F10D053525D41BC53C0EB0F50F6E9F069AACF87F00847D6ABC6C7E3AFF|

## 제품 연결 경계

현재RETURN은product_verified_timeline=false/actual9_binding=false/quality_approved=false다. 기술원자료가생겼다고JSON의caller값을Verified로역직렬화하지않는다. 다음은source/tool/관측hash를결합한typed evidence adapter·공통SDK media custody와실playback owner/clock 인수다. actual9는Sintelcue계열이므로이Sprite근거를붙이지않으며원range9null/NeedsRepair·판정불변이다. 다른RD·전체CLI/GUI·AI사용자품질과최신전체190회귀도범위에유지한다.

스킬은기존test-fixture-and-mock-build·code-change-review-verification의정확source/실패보존·독립fixture/관측과주장의구분을이어적용했다. 출처/본문은 [case](../application/review-source-case-custody-20261026.md),[산술](../application/media-timeline-arithmetic-20261026.md)을따른다. 두lock/Media seal·기존WIP/소유권/index0·보류유지.
