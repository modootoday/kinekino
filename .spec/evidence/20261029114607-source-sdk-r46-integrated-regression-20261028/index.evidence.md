---
id: evidence:kinekino/20261029114607
title: "R46: 지정 190시험과 Native 정상 빌드 회귀"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R46: 지정 190시험과 Native 정상 빌드 회귀

2026-10-28 중앙 PM. [jobs host frame 연결](job-host-pinned-frame-20261028.md)의 R46 입력으로 지정 계획 전체를 실제 재실행했다. 이번 gate에서 제품 소스 변경은 없다. private overlay이며 canonical 채택·전체 CLI/GUI 인수와 구별한다.

## 입력과 실행 조건

SOURCE `9CA271E1617112BCC3B0DE73359B3A7854B8FDA78362EC9925ECED1934732C80`, inventory `3B58F8235E9849B5561DA50A1D9CD714E92979E4C3F4F91A8D75E34E8B5C69C7`/2302. AGENTS `40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C`, mirror lock `955DEBA0338A3A32ED64D6B2A500C192CD55517336FE531D6EED374D0B2916A5`, Meari lock `23792484F39356E19904DBBB34B1496979E87793CF035491EF71F7C4134CC1EE`를 재확인했다.

stable MSVC cargo `AA5B8980BE3681899FDE765BA47E2ED361463AE9DD8EE7CD1963C18D23743A0F`·rustc `BA03E3BA03A73D9FEF19AB3659140F94B43615EC1D587FE9BEFC9AED3BD5AA72`와 고정190 계획 `082C68B479E656A290F11A6CBA3AB0AF9AAE0C9651B265AA337767D454ADD464`도 재확인했다. C-only cache 문맥·locked/offline/-j1·CPU2/BelowNormal·heavy1을 유지했다. 원 R45 타입 실패/원본 preimage는 위 연결 기록에 보존하며 이번 성공으로 재분류하지 않는다.

## 실제 결과

|로그|범위|원 receipt|
|---|---|---|
|기존1363/1364/1365|같은 R46의 default/Native strict·fmt|각 actual0, 이번 반복 실행으로 계상하지 않음|
|1373|Native lib/jobs no-run|terminal0/458ms, 두 producer freshtrue|
|1374|Product145|145/145,terminal0/18812ms|
|1375|Meari Core18|18/18,terminal0/1250ms|
|1376|Meari Native26|26/26,terminal0/5518ms|
|1377|자동 OS1|1/1,terminal0/1338ms|
|1378|Native player/player-cli 정상 빌드|terminal0/467ms,profile.test=false,freshtrue|
|1379|최종 입력 custody|terminal0/41650ms,2302 content/identity unchanged|

각 group-return은 execution_result=Ok·producer_integrity=Ok와 위 R46 SOURCE 핀을 반환했다. 등록/ignored 일치와 실제 각 시험의 raw·spawn·전후 exe/PDB 식별을 `runtime/verification/source-sdk-r46-exact190-r1/{product,core,native,os}`에 보존했다. 원 runner 로그는 `source375-datalab-b0-r1-logs/1373~1379-source-sdk-r46-*`다. 이번 gate의 새 실패는 없다.

독립 Meari의 전체 packages 및 workspace Cargo.toml과 mirror는 각각 native git diff exit0였다. 기존622/624의 원 producer를 사용하여 시험을 실제 재실행했으며, Core/Native/OS producers-before.tsv는 R43과 각각 diff0였다. 새 Meari 컴파일이라고 주장하지 않는다. 자동 OS는 고정 계획의 원 창 veto/completion/timer-owner 시험이며 제품 수동 GUI/시각 인수가 아니다.

1373/1378의 freshtrue는 유효한 현재 Cargo 산출물 재사용이다. no-run을 새 link로 설명하지 않으며 시험 실행 자체를 과거 성공으로 대체하지 않았다.

## 산출물 보존

- Native 정상 `fixed-r46/native-bins/player.exe`: `985FBF58B1981932385682EB3ED4AF4537FAC6400398D296ADC1795B88FE7F9B`/163364864.
- Native 정상 `fixed-r46/native-bins/player-cli.exe`: `8965FEB1C38F80FBF06B170837AB92E5C68F7B901380EBDE712D6C062664105F`/36935680. 앞서 실제 host/capture 검증한 fixed-r46/player-cli.exe와 동일하다.
- jobs 원 exe는 `EB048F99870572F4ADA57E0745AAF29A4741B6069537BB990D0825882E27666F`/28844032, 원 pdb는 `E3CF8E0A180B8CE3C8FCEFA029DF6AE6A7A5B0AAC3CBA5AFB79607E8B02B0C90`/618508288. fixed-r46/jobs.exe·jobs.pdb로 보존했다.
- lib exe `F1B10E1A526A5BF922D9E4B9BEBDE04FA0755960052010E5EB094E33B00F6C53`/191372288와 pdb `209109D21FA18269BDEBECBF757DDF0B9D463A457024EE1B4968F34801D150F3`/1174056960는 현재 원 producer와 R43 보존본의 hash가 동일함을 재확인했다. 동일 bytes를 보존하는 fixed-r43/exact190-product를 재사용하고 중복 복사하지 않았다. R46 실제 실행/원 FileID는 새 TSV에 있으며 copy의 FileID로 대체하지 않는다.

## 남은 범위

지정190/정상bins gate 통과이며 전체 Cargo/CLI 기능·GUI/AI 결과 품질 완료가 아니다. 활성 frame 처리 중 종료/취소와 오류 행렬, 미지원 SDK/CLI 경로·실제 provider/source 정책·HTTP 연결, GUI 비교/수정/판정/이력과 실행 시각 검토가 남는다. R44 discovery 정합도 해당 원 입력으로 보존한다.

source375 private overlay·원 실패/미실행·기존 소유권/WIP·AI 사용자 품질0/9·추론/다운로드/수동GUI/WARP/push 보류를 유지한다. 전역 설정·설치·네트워크·stage/commit/push 변경은 없다. 이 gate만으로 전체 goal을 완료하지 않는다.

modootoday/software-qa/test-fixture-and-mock-build SHA `A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100` 및 review-verification SHA `9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B`의 원 producer/실제 재실행·과거 증거·인수 범위 구분을 적용했다.
