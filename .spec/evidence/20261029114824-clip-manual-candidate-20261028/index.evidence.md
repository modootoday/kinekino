---
id: evidence:kinekino/20261029114824
title: 숏폼 실제 수동 렌더 후보 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 숏폼 실제 수동 렌더 후보 검증

R154 체크포인트 완료: 2422 사후custody 종료0, 2419파일 content/identity unchanged를 원stdout에서 확인했다. 실제후보/강화시간검사·strict/fmt/no-run·지정190/정상2bins/사후custody 완료다. 아래 'custody 실행 중'은 당시 기록이다. 제품 전체 goal·render_short의 CLI/Jobs 인수 완료가 아니며 원 실패/소유권/WIP/private overlay/보류와 AI 품질0/9를 유지한다.

## R154 실제 후보 시간축 재검증

2412 strict, 2413 no-run, 2414 명령 시험, 2415 실제 후보 시험(3.79초), 2416 fmt check, 2417 Product145/2418 Core18/2419 Native26/2420 OS1, 2421 정상2bins가 모두 종료0이다. 네 group-return에서 execution_result/producer_integrity=Ok(())와 같은 R154 SOURCE hash를 확인했다. Meari는 기존622/624 고정 producer의 실제 재실행이며 새 컴파일이 아니다. 2422 사후 custody 실행 중이다.

실제 증거는 <원 호스트 저장소>/runtime/verification/source-sdk-r154-render-r1/actual 이다. 후보는 202401bytes/SHA4b198a94795d9e85814757994dfb7859df1d24e4757061edce6588e0039bdc2d. 영상은1080×1920 H264/30fps/30frames, start_pts0/duration_ts15360/timebase1/15360으로1초다. AAC는 start_pts0/duration_ts48000/timebase1/48000으로1초이며47개 decoded codec frame을 관측했다. container start_time0/duration1.000000과 전체 decode 성공을 확인했다. 원 source/저장 발행 보존과 다른 문서/색 선언 거절도 통과했다. 원R152/R153 후보·길이 불일치·강화시험 첫 실패를 보존했다.

후보 facts에 recipe=manual-mp4-seekable-stdout/1, project 발행SHA b46ef840bd1afd08dc0c3ea0296836f155bc48752d17f93c435691b8edeb819a, encoder tool/inventory SHA, 경계 rawSHA, 정확 documentSHA c2b5399ebf7e79e6463db7005b1f12e33bab138d827cee2c4346b2f31d9c280d를 함께 보존한다. 색은 명시 BT709Limited 해석이며 검증/사용자 품질 승인이 아니다. volume 값의 청감·전체 source 구간/EOF·음수 실제표본·자막/구간crop·preset catalog·최종 발행/Jobs/CLI·GUI 인수는 남는다. 이 단일 후보를 전체 render_short 지원으로 계상하지 않는다.

고정 lib exe/PDB는 fixed-r154/native-core, exeSHA4238D957CE08B6CF4F64577244A2DB02AE67A44A409E72AC47A4159B5F668AAC. 정상 exe/PDB는 fixed-r154/native-bins, CLI C8476F80A27B34F22B1281FD99263A9454A09B1045A0F3DB3A2B6B258FC44322/GUI A93A989A48B61C303324DDFF0726F3455AF39072602AA482CB5AF36856776803. 아래 WIP는 당시 기록으로 보존한다.

R154 입력고정:2411snapshot 종료0,2419파일/inventory6472E5BB2FFB7BB2AFB19B391744B07852D32E5C67DAEE9D2B138258F2C3CF50,SOURCE.md209ACDA8710BD58119C0C1F4A9F5829AF7139785C0A2DD09F4A517265E7DABC3.2412기본strict종료0/2413기본no-run진행중. 아직fd출력의실제성공이나시간길이인수로표시하지않는다.

R153 강화시간시험첫실패:2407no-run/2408명령시험종료0,2409실제후보시험은format.duration 기대1.000000/실제1.021333으로실패했다. 영상start0/duration1.000000/30frames는맞았으나AAC duration_ts49024/48000=1.021333이었다. 원R153 candidate/facts/output-probe/실패로그·입력/preimage보존. R154에서기존owned-process.stdout이read/write regular file이고reader를독립open해childcursor를공유하지않는것을원evidence_files/launch소스에서확인후MP4출력을pipe:1분할방식에서fd: -fd1의seek가능한원stdout으로전환했다. output경로/worker를새로만들지않는다. 후보facts에도project발행pin·encoder tool/inventory·경계rawSHA·recipe를추가했다.2410fmt종료0/2411snapshot진행중이며재실제검증전이다. [공식fd프로토콜](https://www.ffmpeg.org/ffmpeg-protocols.html)의regular-file seek계약을참고했고실제고정host에서확인전성공으로계상하지않는다.

R153 입력고정:2405snapshot 종료0,2419파일/inventoryCDBB33DD3CF212334518D4BA16677D7EB92911003954C97B008C8A832FA6D45E,SOURCE.mdB5AC3A9FC06CD76B2572F092040C56F57409DA6CC1EA504C1FC4210BDB45F49F.2406기본strict종료0,2407기본no-run진행중이며새강화시간시험전이다.

R152 사후custody2403종료0,2419파일내용/FileID동일확인. 전체지정190/정상bins기준선은R152로진전했지만1.066667초길이문제는미완료다. R153에서원command/tests를preimage-r152-mux-clock에보존후empty_moov→delay_moov·use_editlist=1·avoid_negative_ts=disabled로수정했고실제시험에video start_time=0/format.duration=1.000000검사를추가했다. 원후보는삭제/덮어쓰지않는다.2404fmt종료0/2405snapshot진행중이다. [FFmpeg 공식muxer문서](https://ffmpeg.org/ffmpeg-formats.html)와[upstream movenc](https://github.com/ffmpeg/FFmpeg/blob/master/libavformat/movenc.c)의delayed moov/edit list계약을참고하되실제고정8.1.1관측으로확인하기전수정성공으로계상하지않는다.

R152 실제후보/통합관측:2394기본no-run/2395명령1/2396실제후보1(3.66초)/2397fmt check/2398Product145/2399Core18/2400Native26/2401OS1/2402정상2bins 모두종료0. 네group-return execution_result/producer_integrity=Ok(())와동일R152 SOURCE hash를확인했다. Meari는기존622/624고정producer의실제재실행이며새컴파일이아니다.2403사후custody진행중. 고정lib SHA E0B6DBC1021E67DDAA68381F5D10EA5C3C4CEB2F57233D85F1640775D089734F,정상CLI CBB038E50A22CC0D2670F99CEB2921E4CEF824683EA3E6508DB98922196F9D41/GUI 2A41232484E8094C4EF831BA4DA46C7C3D123865CC464C8D40FD8704EEF6A492,exe/PDB는fixed-r152에보존.

실제후보<원 호스트 저장소>/runtime/verification/source-sdk-r152-render-r1/actual/candidate.mp4 는202069bytes/SHA020eff65f2de7ba3d0fb48b7094630eeb2c627c60713703f11e2aa69ae7e5800. native저장프로젝트→실제source/encoder/경계조립→1080×1920/H264/30frames/선택AAC48frames및전체decode를관측했다. 다른document/color선언거절과원source/원발행보존도확인했다. 다만format.duration=1.066667로요청1초와차이가있으므로길이/시간축인수는미완료다. 범위가좁은현재시험통과를전체렌더성공으로승격하지않는다. 원후보/metadata/receipt를보존하고mux timestamp·encoder지연/edit list를다음수정항목으로고정한다.

2026-10-28 R152 WIP. [계약](../../player/application/production/clip-manual-candidate.md)에따라실제source/encoder/decoded경계를결합한다. 기존mod/encoder요구기능은preimage-r151-clip-render에보존했다. 원AGENTS SHA40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C를재확인했고다른프로젝트celir-index-runtime의실행중cargo/rustc는변경하지않았다.

2390/2391fmt/2392snapshot/2393기본all-targets strict종료0,2394기본lib/Jobs/CLI no-run진행중. input-r152는2419파일/inventory1915F649BEBDBFD65A28B5F74CE71BFB02254CD200F79AB9D665AB12A7DB4863,SOURCE.mdDC726AFFD364CAF7CD2CA250FCE66B9CD65E56FD644F4021F4177DA4696FFBC1. 명령조립1/실제후보1시험은아직미실행이며최신전체190/정상bins완료기준선R150을유지한다. R151경계선택검증을전체gate로바꾸지않는다.

실제시험은원Sintel에서native프로젝트저장/source재admission/실제encoder inventory/15~16초decoded경계를준비해고정crop/fit·audio track1 volume75의1080×1920/30fps/H264/AAC후보를만든다. 새evidence의candidate.mp4·facts/경계/원probe/count_frames/전체decode·원source/저장발행보존·문서/색선언불일치거절을확인하도록작성했다. source색은시험의명시BT709Limited해석이며color_verified=false/quality_approved=false다. 실제encoded성공전이를통과로계상하지않는다.

최종발행·preset catalog자산검증·선택자막/구간별crop/subject·EOF/다른video clock·회전/HDR·정확색/AAC priming/AV길이·Jobs/SDKCLI·GUI·사용자품질은미완료다. fixture profile catalog SHA를제품공인preset으로표시하지않는다. 기존modoo QA스킬출처·적용은공통계약에기록했다. 새다운로드/추론/수동GUI/push없음·소유권/WIP/private overlay/보류유지.
