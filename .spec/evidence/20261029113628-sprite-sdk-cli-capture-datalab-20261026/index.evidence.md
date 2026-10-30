---
id: evidence:kinekino/20261029113628
title: Sprite 관측 PTS와 공통 SDK/정상CLI 실제 capture 대조
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Sprite 관측 PTS와 공통 SDK/정상CLI 실제 capture 대조

2026-10-26 중앙PM의단일offline media배정. [240frame 관측](sprite-stream-frame-pts-datalab-20261026.md)의같은원본SHA7CFC8B8C…에서production.capture_frame을현재정상CLI로실행했다. 원실제9/모델출력/품질판정·GUI를변경하지않는다. 이시험은현재공통Rust SDK의capture 경로이며별도decoder만실행한결과를SDK성공으로대체하지않았다.

## 실제 실행과 결과

runtime/verification/source375-datalab-b0-r1-logs/154-sprite-sdk-cli-capture.*: ignored기술시험을중앙에서명시선택해 **1pass/0fail/0ignored**,exit0/89174ms,시험9.09s. 일반cargo test에는이실미디어시험이ignored다. Cargo가제공한정상CARGO_BIN_EXE_player-cli를사용했다. fixture runtime/build-cache/tmp/sprite-capture-cli-33988의INPUT/request/stdout/stderr/process.json·두PNG를보존한다.

| 요청 | 실제 SDK 반환 | PNG |
|---|---|---|
|17000000us|actual17000000us,source_pts_ticks208896,timebase1/12288|1718×720/886992B,SHAAC5D6D071C716A943852261BAD441A695AA7CB53583340A8A8D6349FF108EED0|
|25500000us|actual25500000us,source_pts_ticks313344,timebase1/12288|1718×720/842365B,SHABF0E3D8CCFBEC31FE03745FC45521BBAEFF0D69253F594E324643E431B2E360F|

두tick은앞선152 raw frames에도실제로존재하며best_effort_timestamp가동일하고각duration512다. SDK응답의PNGsha를실제파일byte로대조했다. 각동일출력경로재시도는RevisionConflict이며PNGsha가그대로였다.630000000us의범위밖요청은InvalidArgument/출력없음이었다. 총6개의새CLI process(capabilities1/capture2/overwrite2/outside1)를사용했다.

원본/정상CLI/FFmpeg/FFprobe의ordinaryfile·조상directory guard를시험전체동안유지하고새hash/FileID/bytes/creation/mtime를INPUT.json에고정했다. 각관련호출후identity동일을확인했다. ownedCLI deadline40s/원capture context30000ms,stdin null·rawoutput파일·RAII종료/wait. CPU2/BelowNormal·heavy1·locked/offline/jobs1/C native/tmp이며프로세스/원본을임의교체하지않았다.

| 실행 입력 | SHA256 |
|---|---|
|정상player-cli.exe|8D06F1919D05ABEDCDAA799C9A7B43FA42CF8F14B5C092FD6EB4D6D058CF99F9|
|FFmpeg8.1.1 executable|09948D4CDD0650DA6FF5A87577469F2A218DC2615AE379F8F734D24C49DE0F73|
|FFprobe8.1.1 executable|A6618E99BB58869DED3C6F37B53AA1A8D701C3591DBB7B5B317D47369C112BE2|
|Sprite source.mp4|7CFC8B8C6EEB19685048A289ACA208A01356F5082B470FB9F4D933FE4B2087EF|

155 fmt --check exit0/2851ms.156 root all-targets/native-proposal-window strict exit0/1045ms. 기존paint deprecation은원로그보존. 다른프로젝트의celir-sql-ffi Cargo활동을발견했으나변경/중단하지않았다. 이프로젝트runner/CLI/FFmpeg는terminal반환했다. 두lock/원Media seal B47E9913…불변,rootindex0·WIP/소유권/추론·다운로드·수동GUI/WARP/push보류유지.

## 범위와 다음 연결

capture의원`-copyts`/다음source tick선택·double정밀도한도·원publication을사용했다. 이것은GUI현재frame capture나SDK playback.seek/playback.status의실player owner/clock관측이아니다. unknown source_color_space/transfer,rotation미적용·HDR/ICC/player색상동등성한계를원응답그대로보존한다. PNG를수동시각인수하거나모델하이라이트로등록하지않았다.

[기존PCM sample slice](sprite-pcm-lineage-datalab-20261026.md)의클립과video시각이같다는것은아직미검증이다. 영상→resampledPCM origin/priming/edit-list/continuity와새decoder재현,typedmapping proof·association Verified전이·실제9/AI사용자품질·GUI인수가남는다. 현재snapshot9AB190의성공을이후계전체회귀로승계하지않는다.

## 시험 source

-tests/sprite_capture_cli.rs SHA9A1EB524D37914E0C06C0B5BDD62A10F966C505A337FAD07F7DA96FB3FF80DC6.
-tests/sprite_capture_cli/custody.rs SHAA7F1C33B29E262944E80A1222B9FAF14CCAC116E81D9FC79965A6D68AB1F2498.
-tests/sprite_capture_cli/process.rs SHA7E305DDBA842B309CD501C29172CA98D1E3879FCE365F380906932C065C0FFC1.

modoo.today test-fixture-and-mock-build의실resource·원guard/자식소유권·mock과실CLI구분과code-change-review-verification의주장대조를이어적용했다. 본문/출처는 [case](../application/review-source-case-custody-20261026.md),[공통산술](../application/media-timeline-arithmetic-20261026.md)을따른다.
