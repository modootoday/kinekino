---
id: evidence:kinekino/20261029114151
title: 정상 CLI → endpoint → 원 source/JobsCore — 2026-10-27
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 정상 CLI → endpoint → 원 source/JobsCore — 2026-10-27

## 실제 소비 경로

[source 종료 코드 R13](source-sdk-windows-exit-bits-20261027.md)을 실제 product `player-cli.exe`가 소비하도록 격리 시험을 연결했다. 기존 LocalHost/PlayerSdk.with_jobs/retaining JobsCore/download_registry/ProcessSourceOpener/StagedDownloadOutput을 재사용했다. 새 GUI 자동화·별도 application 정책·새 process driver를 만들지 않았다. CLI process는 기존 owned_process_evidence::run_process_until로 원 Deadline 안에서 원 handles/family/raw/receipt를 확인했다.

root lib의 self alias는 cfg(test)에도 열어 기존 helper를 단위 문맥에서 소비하게 했다. 정상 제품 빌드에서 기존 native alias 조건은 유지한다. Root 제품/canonical은 수정하지 않았고 private mirror에서만 진행했다. source permission/고정16byte/BytesProbe는 계속 fixture다. 실제 browser issuer·HTTP·media decoder 검증이 아니다.

## 첫 실패와 선택 입력

1088 첫 strict는 helper의 **예산 거절 시험**이 integration-only CARGO_BIN_EXE_player-cli를 env!로 요구해 lib test에서 컴파일되지 않았다. 해당 두 예산 시험은 실제 spawn 전에 거절되는 것이 목적이므로 존재하지 않는 명시 `must-never-launch.exe`를 사용하도록 고쳤다. 실제 CLI 소비자는 별도 고정 정상 BIN/SHA/size를 요구한다. Cargo 환경 변수를 위조하거나 기존 시험을 삭제/비활성화하지 않았다. 변경 전 helper/lib/cli leaf는 preimage-r13-cli/preimage-r14-cli-typing에 보존했다.

1089는 공통 helper의 이 소비자 밖 미사용 API와8개 RPC 인자 경고였다. 두 client 주소 인자를 묶고 원 observation deadline/완전한 stdout·stderr 검사를 추가했다. 공통 helper module의 미사용 API만 reason과 함께 허용했고 기능 검사/lint 전체를 끄지 않았다. 변경 전 cli leaf는 preimage-r14-cli-lint에 있다.

선택 input-r14 SOURCE.md SHA256 `1E82AD769BFFA88FF5B96D0EA87C8A248F15C0A84D4FF89B12ECC64C6A54D77D`,inventory `826E42181BEB6F71CFC983C66DAD7F6DD7D3500EA2CBD4985A9D3944B0FBFE4A`,2286files.

|로그|실제 결과|
|---|---|
|1090/1091|인자/사용 범위 수정 후 strict,최종 raw/예산 검사 추가 후 제품 all-target/시험 feature strict 통과|
|1092|정상 player-cli bin build 통과;Cargo profile.test=false|
|1093|같은 feature 제품 lib test no-run 통과/34.38s|
|1094|normal CLI+원 source Completed exact1pass/921filtered/1.07s|
|1095|high-bit source failure의 CLI 보존 exact1pass/921filtered/0.48s|
|1096|factory 부재 Unsupported CLI exact1pass/921filtered/0.15s|
|1097|예산 거절→NotSpawned/owner 조작 금지 시험 exact1pass/921filtered|
|1098/1099|전체 workspace fmt 통과/선택2286 content·identity 동일|

다른 helper inline 시험들은 등록/컴파일됐지만 이번에 전수 실행했다고 계상하지 않는다. 새 GUI BIN이나 Native-feature 전체 회귀/190을 실행한 것도 아니다.

## 고정 producer/guard

- `native/source-sdk-integration-r1/fixed-r14/player-cli.exe`: SHA256 `249DD5ECEFFD0A476D3EC7B89B0F7003F9DB5DC13D285AE1126A4B9938DED32B`,36405248B,1092 정상 producer. 시험은 원 read handle을 write/delete 공유 없이 보유하고 같은 handle의 byte수/SHA를 검사한다.
- `fixed-r14/sdk-source-tests.exe`: SHA256 `B3431C3185EE7F2110708075C798F5C334C186647AC506E40E147A84CF9414E0`,109185024B,1093 producer.
- source worker는 원 R12 fixed822E9B6E/1000960B다. R14에서 재빌드했다고 계상하지 않는다.
- operator index A3C6196AECC7CFFFD1B4CA991149E06E942DBABFB7456FEDDF719F71CE30BBA7/385B는 proof=false다. 원 C2 driver7E7AD258이 test/worker를 guard했고 CLI는 위 별도 원 Rust guard로 보유했다.

새 snapshot/실행 때 C-only target/tmp·locked/offline/j1/conservative2/heavy1을 유지하고 각 live terminal 뒤 다음 heavy 작업을 시작했다. CLI 요청들은 동일 inner 원30초/cleanup5초 범위에 각 cap10초를 단축 적용했고 원 job10초를 갱신하지 않았다.

## 실제 CLI13회와 종료

|원 runtime/verification root|실제 요청/결과|
|---|---|
|source-sdk-cli-normal-os1-r14|5회: capabilities/schema/jobs.submit/jobs.get/jobs.watch. 원 source consume1/transport1/guard1,Completed/실제16byte 발행|
|source-sdk-cli-failure-os1-r14|같은5회. RPC jobs.get은 정상 조회지만 job은 Failed/EngineFailed·미발행/출력 없음·partial 정리. 정상 CLI exit0을 작업 성공으로 바꾸지 않음|
|source-sdk-cli-absent-os1-r14|3회: capabilities/schema/jobs.submit. operation 미광고·submit Unsupported/CLI exit3,source consume0/transport0/guard0·출력 없음|

각 CLI 원 process는 예상 exit0 또는 Unsupported의3,Empty/nonforced·원 raw complete/예산 확인·원 receipt를 통과했다. 원 request/response/raw/receipt/elapsed 기록은 각 inner/rpc-N에 있다. Root caller의 direct JobsApi polling은 terminal 대기 관측만 수행했고 submit/get/watch 소비를 대체하지 않았다. 원 local endpoint는 세 경우 모두 pipe Joined/sdk Joined/dispatch Some(Completed)였고 같은 원 retaining job host도 실제 종료 receipt를 반환했다.

외부 test PID27844/33648/35056은 모두 exit0/Empty/nonforced/raw EOF/selected_one=true/ownership complete/first_error=null이다. outcome/outer receipt/raw·inner-observed copy와 원 C tmp sdk-source-cli-*를 보존했다. 실행 후 고정 normal CLI/test hash 및 root lock B2E3D540 동일도 확인했다.

## 인수 경계

이것은 새 호스트의 **구성된 offline fixture endpoint**에 대한 실제 정상 CLI13회다. 기본/미주입 CLI나 제품의 browser download 지원을 선언하지 않는다. 13은 요청 수이며97개 명령 전체 인수 분모가 아니다. CLI 취소/재시작/중간 EOF·실제 OS probe·unassigned rollback·browser replay/current issuer·실제 HTTP와 전체 기능 인수는 남는다. 원 DFB0 190/정상2bins·source375 private overlay·WIP/소유권·AI 사용자 품질0/9·GUI 시각 미인수를 유지했다. 추론/다운로드/수동GUI/WARP/push를 재개하지 않았다.

modoo fixture/mock·postcondition 지침을 재사용해 실제 정상 CLI/원 RPC와 fixture admission/body/probe,명령 성공과 작업 성공,process/pipe/sdk/job host 각각의 종료 증거를 분리했다.
