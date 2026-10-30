---
id: evidence:kinekino/20261029114521
title: R43 공통 프레임 도구 핀과 실제 CLI
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R43 공통 프레임 도구 핀과 실제 CLI

2026-10-28 중앙 PM. [계약](../../contracts/production/frame-tools.md). 원R41 FFmpeg 불안정 원인으로 Timeout을 소급 단정하지 않는다. private mirror 변경이며 source375/canonical 적용·사용자 품질 인수와 구별한다.

## 입력·preimage

mirror/runtime/verification/preimage-r42-pinned-frame-tools에 원 mod.rs SHA6D85E758A1B1894C0AD02A173DCF0FB86FAB79153EFCBBE1EC828F5601B0933E, runner.rs SHA5BBEA112A714ADFED1137DF1A75C9B3208CF58F2ABD3F9D0A8630D80160A787A, player-cli.rs SHAD347532B00676BFBE6A304CC963E68EDB6D7EA84B251C98E919A7979FD7937F9 보존.

input-r43 SOURCE DD8F66FD7F920762B27B106879CA62D708765824A842AA4224DF4E4BAC800528, inventory176A246A5679F6D048AFFB1FBEBD2ED21FBF683C302CA46657148E872D298A23/2300. snapshot은1323 compile과 읽기 전용으로 중첩했으며 시작 후 소스 변경 없음. 현재 AGENTS SHA40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C 확인.

## 실제 검증

후속 custody verify-current는 동일 R43 SOURCE 핀에서 terminal0·2300 content/identity unchanged를 확인했다(직접 handle48693, 별도 runner receipt 없음).

|로그|검증|원 receipt|
|---|---|---|
|1322|default all-target strict|terminal0/18099ms|
|1323|고정 해시 불일치 거절·guard 동안 쓰기/삭제 차단·해제 후 정리|선택1pass/936filtered,terminal0/36442ms|
|1324|정상 default CLI build|terminal0/44796ms|
|1325|실제 FFmpeg/FFprobe8.1.1 명시 핀 capabilities|terminal0/383ms,available=true|
|1326|ffmpeg 해시를0×64로 바꾼 명시 구성|terminal8/127ms,source_expired|
|1327|ffmpeg 경로만 준 불완전 설정|terminal2/8ms,invalid_argument|
|1328|Native all-target strict|terminal0/23107ms|
|1329|전체fmt|terminal0/3729ms|

1326/1327은 의도한 거절이므로 원 nonzero exit를 그대로 유지했다. 실제 원시파일은 runtime/verification/source375-datalab-b0-r1-logs/각번호-source-sdk-r43-* 이다. 보존 CLI `runtime/build-cache/native/source-sdk-integration-r1/fixed-r43/player-cli.exe` SHA53F826215AFD5E4CB4DBA241624ADDCD52A97F8701DDD3B8F4E7D4B2BDF086DF/36489728. Native feature 정상bins를 빌드했다고 주장하지 않는다.

실제 선택 도구는 WinGet의 `Gyan.FFmpeg_Microsoft.Winget.Source_8wekyb3d8bbwe/ffmpeg-8.1.1-full_build/bin`, ffmpeg SHA09948D4CDD0650DA6FF5A87577469F2A218DC2615AE379F8F734D24C49DE0F73·ffprobe SHAA6618E99BB58869DED3C6F37B53AA1A8D701C3591DBB7B5B317D47369C112BE2. CLI 입력은 소문자 SHA다.1325 dependency에 두 canonical 경로와8.1.1 version,available=true를 확인했다. 잘못된 핀은 실제 available 대체 도구가 있어도 성공으로 fallback하지 않았다.

## 범위·다음 작업

실제 미디어 캡처·안정97명령 구성·원 전체190·GUI/AI 승인으로 확대하지 않는다. R40 지정190/bins는 원R40 입력의 근거다. 다음은 CLI help/기본 host의 명시 구성 소비, 실제 frame 정상/거절·영향 회귀와 전체 CLI 잔여 인수다. 실제 추론·다운로드·수동GUI·WARP·push 보류와 기존 소유권/WIP/원증거를 유지한다. 설치/전역PATH 변경 없음. heavy compile/test/build/CLI는 terminal 확인 후 직렬, CPU2/BelowNormal이다.

modootoday/software-qa/test-fixture-and-mock-build SHAA393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100의 synthetic file guard/실제 도구 구분, review-verification SHA9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B의 scope별 원 결과 보존을 적용했다.
