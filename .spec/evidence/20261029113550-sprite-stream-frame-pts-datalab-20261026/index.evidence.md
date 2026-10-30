---
id: evidence:kinekino/20261029113550
title: 기존 Sprite 원본 content·bounded frame PTS 관측
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 기존 Sprite 원본 content·bounded frame PTS 관측

2026-10-26 중앙PM의단일media기술관측배정. 기존 [최소계획](../../../../runtime/development/sprite-pts-sample-plan-r1/PLAN.md)의stage1~3 일부를 수행했다. 실제9의원영상이나AI하이라이트표본으로등록하지않았다. 신규registry·미디어복사·추론·모델/미디어다운로드·GUI·WARP·판정쓰기0이다. 원미디어/원증거를변경하지않는로컬읽기다.

## 고정 입력과 자원

-원본 runtime/ai-spike/source/sprite-fright-sprite-source-r1/source.mp4,102392846B. **새실측SHA7CFC8B8C6EEB19685048A289ACA208A01356F5082B470FB9F4D933FE4B2087EF**는기존선택pin과일치한다. 현재FileID AE916536-000400000013B555·creation/mtime는각INPUT.txt에보존했다. 옛D manifest/FileID는수정하지않았다.
-기존로컬FFprobe8.1.1-full_build,실행파일SHA **A6618E99BB58869DED3C6F37B53AA1A8D701C3591DBB7B5B317D47369C112BE2**. FFprobe227193344B/현재FileID·원버전응답을보존했다. 설치/전역설정변경없음.
-source/tool의ordinary local file및조상directory guard를잡고새content hash를계산했다. guard동안write/delete를거절하며관측후identity동일을확인했다. guard해제/재획득은독립fixture시험으로확인했다.
-기존resource-runner CPU2/BelowNormal,중앙heavy1·TMP/TEMP=C runtime/build-cache/tmp. FFprobe threads1·file protocol만·MOV external drefs비활성·probesize1048576/analyzeduration1000000,기한15s. accepted stdout/stderr 각각65536B,ownedChild만kill/wait한다. 출력상한은poll/final검사로성공응답을제한하며잠깐의file overshoot까지하드디스크quota로막는구현은아니다.

## 실제 관측

151 로그/receipt는 runtime/verification/source375-datalab-b0-r1-logs/151-sprite-stream-metadata.*,원결과는 runtime/verification/sprite-stream-metadata-datalab-r1 이다. exit0/447ms,직접FFprobe PID20752/48ms,guarded_identity_unchanged=true.

| 스트림 | 관측 |
|---|---|
|video0 H264|time_base1/12288,start_pts0,start_time0,duration_ts7739392,duration629.833333|
|audio1 AAC|sample_rate44100,time_base1/44100,start_pts0,start_time0,initial_padding0,duration_ts27778048,duration629.887710|
|format|start_time0,duration629.887710|

이표는metadata보고값이다. 특히initial_padding0/start_pts0만으로decoderpriming/PCM매핑을인증하지않는다. streams.stdout.json SHA C16511DDD2B4DE22AC4BF63DDD7EA85244468A86580156F8B6450D88C27373D9.

152 로그/receipt는같은logs의152-sprite-frame-pts.*,원결과는 runtime/verification/sprite-frame-pts-datalab-r1 이다. source/tool을다시guard/hash검증한뒤v:0,read_intervals16%26의한창만관측했다. exit0/564ms,직접FFprobe PID27312/304ms. stdout38211B,frame240개,first_pts196608,last_pts318976,strictly_increasing=true. 원stream timebase1/12288도함께보존했다. 프레임선택요청과실관측을분리하고seek/keyframe preroll가능성을FRAMES.txt에명시했다. frame PTS나best_effort값을missing일때대체/추정하지않는다. frames.stdout.json SHA **BA26BD11B6BEE0FEF8FCB5A461F4B9601BEEE05ABCF26E275BB935D2DF954A34**.

두RETURN은verified_timeline=false/pcm_mapping_verified=false다. 151은frame_pts_observed=false,152만true다. 메타데이터관측을프레임관측으로소급계상하지않았다.

## 관측기 구현·시험·실패 보존

std-only비공개Rust custody도구에정확한기존Sprite한파일만받는sprite-metadata/sprite-frames명령을추가했다. sourceSHA/길이불일치면실행하지않는다. 임의미디어등록/원request의path추론을추가하지않았다. 새outputdir만사용하며원raw결과/command/input/process/return을보존한다.

metadata준비시도구11시험/fmt/strict/build통과. frame준비의최초시험은11pass/1fail로,음수PTS fixture가원frozen JSON parser의`unsupported or invalid literal`에거절됐다. 아직frame FFprobe는실행하지않은단계였다. 당시소스는151디렉터리의frame-preparation-failed-source.rs와frozen-json-before-observation.rs에보존했다. 수정은parse_observation전용signed integer모드를분리한것이며원parse는음수를계속거절한다. `-01`/`--1`/`+1`/소수/null·누락/중복PTS등을거절한다. 최종 **도구13시험/fmt/strict/build통과** 후152를실행했다. timeout/response-limit 실패주입은아직별도미검증이다.

metadata당시sprite_metadata.rs SHA4EA81C0181749C8FA0D55314DA33EC3E8982E4DB6AEDCE5DABDD277A6A9FFB26(151의sprite_metadata-source-r1.rs사본),toolbinary67871AF948026D3A290911D171E77DB541AB75E2C28FDCAB6F309459432B642A. 최종frames후계sprite_metadata.rs SHA07C438F70A77B93D19A1779E58CBE40662D3C9B1B2794E073167955BC62719DB,json.rs SHAB44583E0E11C344EB2D87C9648B9B7B29B5E882FECF9EC3A28179095C0467A12,toolbinary355B7AFA1A89175D1E21ED089EE7C047C84D2A17A61ADBCBB52757DFA9F06FE8. 옛source375/B0도구기록과각후계결과는분리한다.

스킬은기존modootoday/software-qa/test-fixture-and-mock-build의실guard/해제/재획득원칙및code-change-review-verification의원주장/실행근거구분을이어적용했다. 본문/출처·참조SHA는 [case](../application/review-source-case-custody-20261026.md),[공통산술](../application/media-timeline-arithmetic-20261026.md)을따른다.

## 아직 입증하지 못한 범위

기존latin-17 PCM의start_sample272000/sample_count136000을위영상PTS에연결하는offset/priming/resampling/continuity증거는아직없다. decodedvideo최초frame원점·audio sample대응·같은content의공통SDK seek/clock/capture/error/cancel/close도후속이다. 실제9 source는Sintel텍스트cue계열이므로Sprite의PTS를붙이지않는다. Sprite실제ASR/music는하이라이트출력이아니며quality_approved=false/권리의기존Unknown·출처선언구분을보존한다. actual9 range9null·NeedsRepair·원decision은불변이다.

다음은이번raw관측을typed adapter/공통산술과연결할계약및기존PCM의정확한생성/offset근거대조다. 새추론/다운로드/GUI를자동재개하지않으며원SDK/CLI전체인수·Verified opaque proof·AI사용자품질은계속미완료다.
