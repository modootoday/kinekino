---
id: evidence:kinekino/20261029114524
title: 원 명령과 명시 실행 계약 대조
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 명령과 명시 실행 계약 대조

2026-10-26 중앙 PM. 실제 bundle 발행 전에 normalized audio/video 실행기록과 원 command/INPUT/process를 연결했다. 원자료에서 읽지 않은 값을 parser 기대값으로 채워 입장시키지 않는다. 실제 bundle과 production admission은 아직 발행하지 않았다.

## 구현

OriginalTrace의command/inputs/process참조와ExecutionContract는모두필수다.계약enum은Default를제공하지않고audio의AudioCopyTsMono16kS16FirstStream과video의VideoFirstStreamFrameWindow{interval}를명시한다.누락contract/window나역할다른contract는거절한다.신뢰입장/실행계약의production경로에는편의Default를추가하지않았다.기존Default검색결과는테스트store/control fixture이며실행의선택상태로쓰지않는다.

원command첫행의Rust Debug quoted argv를JSON string stream으로읽어전체argv를계약과대조한다.원INPUT의source/tool/원WAV SHA와원경로문자열도연결한다.경로는대조만하고프로그램/셸/파일을열지않는다.오디오의0:a:0·copyts·async=0·timebase1/16000 및video v:0·명시window가일치해야한다.

원process는중복없는PID/result/elapsed/terminal을요구하고result=Ok(())/terminal=true를확인한다.원observer소스(sprite_audio_replay/process.rs와sprite_metadata.rs)는status.success일때만Ok를반환했다.이는wrapper accepted관측이며원문에없는numeric exit field를새직접관측으로발명하지않는다.기존normalized success와도일치해야한다.

원source identity는INPUT의source SHA와현재package의원media SHA를별도로대조한다.PCM/PNG의파생byte일치로source identity를대신하지않는다.모든참조는선택case item의원세션guard로보유한다.

## 실행 결과

202 include-ignored timeline시험17pass/0fail,시험2.11초.기존15에필수contract상태검사와보존원명령검사2개를더했다.명시실제파일시험은원command/INPUT/process6파일을SHA고정해읽기만했다.새decode/추론/다운로드/GUI0.

새negative는normalized async_correction=false인데원command가async=1인경우,다른audio stream,종료미확인,잘못된/없는contract,video window불일치를resolver에서거절한다.원자료시험에서도원command문자열사본만바꿔async/stream/window/terminal불일치를검사했고원파일은변경하지않았다.

fmt --all -- --check 종료0,203 all-targets strict 종료0/24.23초.기존blitz-paint 의존deprecation원로그유지.시험/strict동안코드변경없음.

| source | SHA256 |
|---|---|
|original_execution.rs|101015EFD9DAB6BD89972010AABDC8297C45C63C92882B1DA97C806F31CB9C72|
|original_execution/tests.rs|8CA10D9536FC6A17ABD1846E7C67ADB9E2910326C77DFED980AAFF155C4330EA|
|package/manifest.rs|03E8218A112D165F60F9A4F4AA232625FE5BA5851CE8216D9B3C5AAF799E7C86|
|package/mod.rs|E63C8B575609563B28BAD6C23515216F3C87F9E899C3DB9509C62459F8772019|
|package/tests.rs|B5605C530C3B1435BB248F3F335AF42DBA958BD4AD62CE7A71AAB046D7C3B538|

## 다음 인수

capture의원request/response/process/INPUT을직접대조한뒤별도실제기술표본package를발행·고정·입장시킨다.실제관측과현재내부JSON계약사이의누락은미검증으로남기며과거실행을새실행으로표시하지않는다.현재wire Verified/SDK재생clock·actual9/AI품질/GUI는미완료다.원WIP/소유권/원증거·heavy1/C캐시·보류유지.

기존modoo.today test-fixture-and-mock-build/code-change-review-verification의원관측/정규화계약·주장범위분리를적용했다.[bundle계약](../../player/application/timeline-producer-bundle.md),[resolver기준](review-timeline-package-resolver-20261026.md)을따른다.
