---
id: evidence:kinekino/20261029114344
title: R52 SDK 취소 변경 후 통합 회귀
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R52 SDK 취소 변경 후 통합 회귀

중앙 PM, 2026-10-28. 원 SDK 취소 연결을 포함한 private R52 후보의 지정 회귀를 실행했다. source375 canonical 채택·전체 CLI/GUI/AI 품질 완료가 아니다. [취소 구현/선택 시험](frame-sdk-cooperative-cancel-20261028.md)과 연결한다.

입력2306, SOURCE SHA256 AF68B1282EB5E00AD550AA5C615073762B0687AD7D8B3191D233008A66F2F7B8, inventory E0650DEC222409464EFB9AC4EF0CFAD5A09D238659BE6A4EC15404E7BB41B990. root/mirror AGENTS 40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C, mirror lock 955DEBA0338A3A32ED64D6B2A500C192CD55517336FE531D6EED374D0B2916A5, Meari lock 23792484F39356E19904DBBB34B1496979E87793CF035491EF71F7C4134CC1EE, 지정190 plan 082C68B479E656A290F11A6CBA3AB0AF9AAE0C9651B265AA337767D454ADD464를 재확인했다. C-only cache·heavy1·CPU2/BelowNormal·locked/offline/-j1, 선행 handle terminal 뒤 후행 배정을 유지했다.

|로그|검증|결과|
|---|---|---|
|1419/1420|Native strict/fmt|원 R52 실제 exit0 유지; 이번 재실행 아님|
|1422|기본 구성 all-target strict|terminal0/22239ms|
|1423|Native lib/jobs no-run|terminal0/15494ms|
|1424|실제 로컬 영상·핀 도구, CLI/SDK/typed host frame 및 EOF 정리|exact1 pass/66 filtered,2.21초; runner0/14604ms|
|1425|Product 지정145|145/145,terminal0/19279ms|
|1426|Meari Core 지정18|18/18,terminal0/1139ms|
|1427|Meari Native 지정26|26/26,terminal0/5911ms|
|1428|자동 OS 지정1|1/1,terminal0/1461ms|
|1429|Native 정상 player/player-cli|terminal0/1410ms,profile.test=false,fresh=true|

각 group-return에서 execution_result=Ok 및 producer_integrity=Ok와 R52 SOURCE 핀을 확인했다. raw/등록/ignored/생성자 전후 식별은 runtime/verification/source-sdk-r52-exact190-r1/{product,core,native,os}에 있다. Meari packages 전체/workspace Cargo.toml과 독립 입력 diff0, 각 producers-before.tsv는 R50과 diff0다. 기존622/624 생성자를 재사용해 시험을 실제 재실행했으며 새 Meari 컴파일로 주장하지 않는다. fresh=true도 현재 산출물 재사용이며 새 링크로 계상하지 않는다. 자동 OS1은 제품 수동 시각 인수가 아니다.

fixed-r52 보존 핀:

|파일|SHA256 / bytes|
|---|---|
|frame-process-tests.exe|4EE5068C6AAB56DE48E0A8531C6A8957F69BEB8E68E2AFA289F7164EFD2186DF /191731712|
|frame-process-tests.pdb|23BE5634CAB17F1BA59A0722C8051BE4B0EE7B697A89C7C1C1127022508D46F8 /1178636288|
|jobs.exe|5900E8A9BE682D124178112C90425EC8BAE8EA2EBBAB1000CE6560E611C0E883 /28844544|
|jobs.pdb|C5735D3D8C05C1C5B10E237557A1B013478D7A687D77385B194C55AD15826AA2 /618901504|
|native-bins/player-cli.exe|92D7A58DF9631295C5EAD7FE714FA45600D09BF1DED2237713AFC01E5E88385C /37019648|
|native-bins/player.exe|9DF7B98EE97665214F1A3528F5E7C359CC33D7DE230828355088A40B9633B002 /163616256|

정상 CLI는1424 실제 frame 회귀의 fixed-r52/player-cli.exe와 같은 bytes다. PDB/복사본의 식별을 원 생성자 FileID로 혼동하지 않는다. 최종 custody1430은 terminal/exit0으로 입력2306 content/identity unchanged를 확인했다.

남은 범위: receipt 실패 명시 복구·실행 중 취소의 발행 경합/활성EOF·부분 launch/termination fault·비Windows, 실제 provider HTTP source/제품 등록·전체 CLI 전수 인수, GUI 비교/수정/판정/이력·실행 시각 인수 및 AI 사용자 품질0/9. root/WIP/소유권·원 실패/미실행·추론/다운로드/수동GUI/WARP/push 보류 유지. 배포 제외. 목표는 active로 유지한다.

modootoday/software-qa/test-fixture-and-mock-build SHA256 A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100, code-change-review-verification 9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B 적용: 실제 재실행/생성자 재사용/fixture/역사 실패/제품 인수 범위를 분리했다.
