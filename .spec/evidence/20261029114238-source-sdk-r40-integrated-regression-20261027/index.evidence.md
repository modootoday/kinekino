---
id: evidence:kinekino/20261029114238
title: SDK private R40 지정190·정상 실행파일 통합 회귀
status: active
created: 2026-10-29
updated: 2026-10-29
---
# SDK private R40 지정190·정상 실행파일 통합 회귀

## 판정과 범위

후속 private SDK 후보 R40에서 Native strict/no-run·지정 Product145/Core18/Native26/자동OS1·정상 player/player-cli·최종 입력 동일성을 실제 확인했다. 원 DFB0의 성공을 승계한 것이 아니라 아래 새 실행의 결과다. Meari는 내용 동일을 확인한 기존 producer를 재해시해 시험을 다시 실행했으며 새 Meari 컴파일로 표시하지 않는다. canonical 적용·전체 Cargo 시험·전체 CLI·AI 사용자 품질·GUI 시각 인수는 별도 미완료다.

## 입력·도구·경로

SOURCE runtime/development/source-sdk-integration-r1/runtime/verification/input-r40/SOURCE.md SHA DB7C9807F9891A80BDF105992E760796D0DA1F71179E63DCE8F296767218E97B,inventory D0428705C89204706EC4E63CE2A9223E83BCFFE254ACAD7E5D22D21C51A9CFC3/2297. generic snapshot의 canonical 제목은 source375 canonical 적용을 뜻하지 않는다. 이 gate는 source 코드 변경0이며 기존 R39/R40 preimage/첫 실패는 [발급 custody 정본](source-replay-issue-custody-20261027.md)에 유지한다.

Root HEAD d13cbfca140e41863b4d0bd27930be3f57a1123f/Meari0844f6c24caf224617ca8abbb2dd75695a971b99 확인. mirror lock955DEBA0338A3A32ED64D6B2A500C192CD55517336FE531D6EED374D0B2916A5/Meari lock23792484F39356E19904DBBB34B1496979E87793CF035491EF71F7C4134CC1EE,AGENTS40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C 유지.

stable MSVC cargo AA5B8980BE3681899FDE765BA47E2ED361463AE9DD8EE7CD1963C18D23743A0F/rustc BA03E3BA03A73D9FEF19AB3659140F94B43615EC1D587FE9BEFC9AED3BD5AA72/custody78DD39F4DBB8C47CD0242B43A7E766CC92CD1DB17087F640E2BB64B6D94672D9를 재확인했다. Pretendard Regular6D0AF5258997AEC7354A6E340FC2325BA321C410CA48B3AF858C8C3D6E92A324/SemiBold5E1C548732AF70873103066C16E1369B9A8A871F0B38C321A1D5BC73E43CEA2D/JP B14698ED6D82737F45E918D079BCA6FC498A107BCB094B56C5229DBDDF3246D3도 원 핀과 일치했다.

제품 CARGO_TARGET_DIR는 실제 <원 호스트 저장소>/runtime/build-cache/native/source-sdk-integration-r1,TMP/TEMP는 runtime/build-cache/tmp다. Meari 원 producer는 meari-native 문맥을 유지한다. global Cargo/Rustup 설정·다른 프로젝트 변경0,새 설치/네트워크0. NVMe에서도 원 heavy1·conservative2/CPU2/BelowNormal·locked/offline/-j1을 유지하고 매 exec handle terminal 확인 뒤 다음 heavy를 배정했다.

## 실제 gate

|실행|결과|
|---|---|
|기존1278 fmt/1277 default strict/1268 support strict|동일 R40 입력의 실제receipt exit0를 유지;이번 gate에서 반복 실행했다고 표시하지 않음|
|1279 Native all-targets clippy -D warnings|terminal exit0/170746ms;원 blitz-paint dependency의 deprecated fetch_update 경고 보존|
|1280 Native lib/jobs no-run|terminal exit0/450745ms;features native-proposal-window의 새 원 producer2/freshfalse 확인|
|1281 Product|145/145;실제 등록/ignored 일치·execution_result와producer_integrity 모두Ok|
|1282 Meari Core|18/18;동일 검사Ok|
|1283 Meari Native|26/26;동일 검사Ok|
|1284 자동OS|1/1;실제 window_user_close_os의 원 veto/completion/timer-owner 시험,ignored 명시 실행·동일 검사Ok|
|1285 정상bins|player/player-cli profile.test=false/features native-proposal-window/freshtrue,terminal exit0|
|1286 최종 custody|terminal exit0/41116ms,2297 content/identity unchanged|

1279/1280 시간은 이 C 호스트의 해당 cold Native 검사/빌드 관측이며 성능 benchmark나 제품 납기 예측이 아니다. Meari 경고를 lint 예외로 새로 숨기지 않았다. 이번 gate의 첫 빌드/시험 실패는 없다. 과거 실패·source375 원 실패/미실행을 재분류하지 않는다.

고정 계획 runtime/verification/product-meari-os-joint375-d-r5-gate/00-exact190.json SHA082C68B479E656A290F11A6CBA3AB0AF9AAE0C9651B265AA337767D454ADD464 그대로다. 현재 원 로그는 runtime/verification/source375-datalab-b0-r1-logs/1279~1286-source-sdk-r40-*,각 exact/list/ignored/raw/spawn/전후 파일 식별자/group-return은 runtime/verification/source-sdk-r40-exact190-r1/{product,core,native,os}다. 각 group-return의 source_receipt SHA는 위 R40 DB7C9807이다. operator input-index는 proof=false이며 실제 실행 증거를 대신하지 않는다.

## producer 보존과 Meari 재사용

1280 Cargo JSON의 package_id는 mirror kinekino-player0.1.0,features는 native-proposal-window다. jobs-d75793112fc410a6.exe 및 kinekino_player-37bec848ae83f5da.exe를 원 deps producer로 사용했다. exe/PDB의 SHA·FileID·bytes·creation/mtime를 원 TSV와 각 실행 전후에 기록하고 마지막 전체 내용 hash를 대조했다.

|고정 보존본|SHA256·bytes|
|---|---|
|fixed-r40/exact190-product/kinekino_player-37bec848ae83f5da.exe|457A0FCC115766C544302B63CF584F4AF2982F602020660AC9322BE17595F616/191361536|
|동일 stem pdb|2EECC7100528A4123759967A66FBAC24A38FD844E3EA492C69F49EFB9C4E9AC5/1050759168|
|fixed-r40/exact190-product/jobs-d75793112fc410a6.exe|F5979426A3468C0EA04DA77731D94C51F26BE56E7C67E36647D7290B071981C2/28808704|
|동일 stem pdb|C0988A6052328572B51A787E8FCAF5131A1CA250B3FA6EE1E28A422DE896BBE2/501272576|

별도 copy 보존본의 SHA가 원 producers-before.tsv와 일치함을 확인했다. 복사본의 FileID를 원 producer FileID로 표시하지 않는다. 전역 cache 전체 보호/동일성 주장은 하지 않는다.

Meari는 원622-speakers-core-no-run.stdout/624-speakers-native-no-run.stdout의 Cargo producer를 재사용했다. complete packages/meari/packages와 mirror 대응 디렉터리,workspace Cargo.toml을 git diff --no-index --stat로 비교해 각각exit0를 확인했고 independent lock도 원 핀과 같았다. core/native src만의 비교도exit0였다. 새 source 위치에서 Meari 시험을 새로 컴파일했다는 주장이 아니며 원 producer의 현재 exe/PDB SHA와 식별 전후를 새 exact 도구가 재검증했다. Core18/Native26/OS1은 이번에 실제 재실행했다.

## 정상 실행파일과 남은 인수

후속 한국시간2026-10-28 [새 정상 CLI 선택3 인수](source-sdk-r40-new-cli-20261028.md)에서 이 ED1D 실행파일 자체로 정상/취소/factory 부재3과CLI15RPC·2297동일을 확인했다.아래 새CLI 전체명령 미인수와 원R14개별근거는 유지한다.

1285의 normal compiler profile.test=false 두 bin을 fixed-r40에 별도 보존했다. build의freshtrue는 같은 입력1280에서 이미 생성된 normal artifact를 Cargo가 재사용한 결과다. 이번 명령의 새 link나 과거 DFB0 바이너리 재사용으로 설명하지 않는다.

- fixed-r40/player.exe:26D750B8E07D1F39D1CC18156B80C12CD772C4F7A80307A4AD3BBE775F06622E/163359232bytes.
- fixed-r40/player-cli.exe:ED1D4899A4825626F25B058CF47BB74856ABB2D54C082021EB0ABAC11DEBBE94/36718592bytes.

정상 GUI 실행·수동 시안 비교·이 새 CLI의 전체 명령 인수는 수행하지 않았다. 기존 source-adapter CLI 실제 시험은 해당 고정 R14 CLI를 사용한 각 R40 정본의 근거로 유지한다. 지정190은 계획 전체이며 모든Cargo시험·전체CLI·AI품질/GUI 완료율이 아니다. production replay 정책/factory/제품source등록·실제HTTP/probe 통합·CLI 잔여 행렬·GUI/AI 원본 비교/수정/판정/이력·실행 시각 인수는 계속 남는다.

원 DFB0 190/정상bins는 원 입력으로 보존한다. source375 private overlay·담당 소유권/WIP·원증거·AI 사용자 승인0/9·원NeedsRepair 유지. WARP/추론/다운로드/수동GUI/push0,배포 제외. 이 gate 통과만으로 전체 goal을 닫지 않는다.

기독 modootoday/software-qa/code-change-review-verification SHA9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B 및 verdict9D9619A829AE24D73645EF5F7C8126C8257CAEDDAC80800632866257008AD2A9의 원 producer/실제 실행·historical 경계 분리,test-fixture-and-mock-build SHA A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100의 고정 입력/독립 exact/정리·합성/실제 범위 구분을 적용했다.
