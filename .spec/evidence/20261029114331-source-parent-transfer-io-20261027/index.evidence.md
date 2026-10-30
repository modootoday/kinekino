---
id: evidence:kinekino/20261029114331
title: Private 부모 wire/host I/O 조립 — 2026-10-27
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Private 부모 wire/host I/O 조립 — 2026-10-27

## 구현 범위

ParentTransfer는 원 ParentLaunch/guard·bootstrap request·ParentStream·control pending bytes·data FrameReceiver를 같이 보유한다. start는 launch를1회 수행하고 bootstrap만 queue하며 Open은 별도 명시 동작이다. Open/credit는 원 current callback 검사 후 queue하고 실제 poll_io도 전후 검사한다. control write completion count만큼 offset을 진행하고 data read는 별도로 poll한다. 최대16chunk/256KiB parent queue·outstanding credit·명시 BodyEnd는 기존 ParentStream이 담당한다. read는 Pending/Data/End를 그대로 반환한다. IO/current/codec 오류 뒤에도 원 host와 bytes/guard를 보존하고 close는 원 launch/host에 위임한다. raw io::Error는 별도 최대8개/overflow로 보유하고 URL/body를 로그로 출력하지 않는다.

Open의 clock 값은 worker에 실제 상속한 값이어야 한다. private 복사본 `packages/owned-process/src/infrastructure/windows/source_pipe.rs`에 읽기 전용 SourceIpc::source_cutoffs getter만 추가하고 ParentHost→ParentStream으로 같은 값을 전달했다. 새 예산·clock 재계산·권한·새 launcher는 만들지 않았다. **원 B 문맥은 편집하지 않았으며1027에서 원32files content/identity 동일을 확인했다.** 복사본은 원 B gate와 구별하는 후계 source이고 기존 B 성공을 이 수정의 성공으로 계상하지 않는다.

preimage lib/host/parent_stream 및 복사본 source_pipe는 `runtime/verification/source-parent-transfer-preimage-r1`에 있다. 원 SDK SourceOpener/DownloadSession·D replay issuer·registry는 아직 연결하지 않았다. callback과 request bytes는 원 replay 권한을 발급하지 않는다.

## 선택 입력과 게이트

R34 input75에서1012 strict 통과. R35 input76에서1013 strict/1014 no-run은 통과했으나 dependency 자체의 fmt 검사1015에서 getter 형식 차이를 발견해 원 diff를 보존했다. 해당 leaf만 rustfmt skip_children으로 수정했다. 선택 input-r36 SOURCE.md SHA256 `A19E621F27F99A93BEB5D06F02CF74D21D41D4911892FA3BC7AE5BAA7473CBA8`,inventory `467CB1DD5E028481A344B362AA089521D595A9A30D759CA85F721BA5A316ED4A`,76files,media lock BC3E5C43 유지다.

|로그|실제 결과|
|---|---|
|1016|변경 dependency 자체 fmt 통과|
|1017/1018|복사 owned-process 및 media-source-host 각각 all-target/all-feature strict 통과|
|1019|actual OS test producer no-run 통과|
|1020~1022|exact ignored OS3 각각1pass|
|1023|media 메모리/순수47 통과,OS16ignored|
|1024|복사 owned-process lib7 통과|
|1025|media fmt 통과|
|1026/1027|선택76 및 원 B32 content/identity 동일|

dependency lib7은 실제 빈 Job 예산·invalid original handle·파일 reader/쓰기 거절·원 count와 overflow 검사를 포함하며 전부 순수 계산 시험이라고 표시하지 않는다. 별도 B OS10을 재실행한 것은 아니다. C-only target/tmp·locked/offline/j1/heavy1·stable MSVC1.99.1을 유지했다.

## 실제 OS3 — Open 미전송

고정 `fixed-r36/parent_transfer_os.exe` SHA256 `04B4C59DC2197F3240C89F75E2077F60AFE5D228E85118BBC92444847B244FF9`/2254848B. worker는 R22 pin2BDCA67F/6648832B다. `source-parent-transfer-r36-index.json` SHA `601676F1AF1B6799E232A44AD69119B266171C3894704EF2D621C0DBDB522BAD`/464B는 proof=false operator index다. same 독립 baseline driver7E7AD258/45s+cleanup5s의 고정 guard를 사용했다. 각 live exec handle의 terminal 확인 후 다음 시험을 시작했다.

|로그/root|실제 인수|
|---|---|
|1020 source-parent-bootstrap-io-os1-r36|bootstrap actual control IO 완료,Open=false/metadata=false/첫 read Pending,원 host/receipt/raw close|
|1021 source-parent-open-rejected-os1-r36|bootstrap 이후 current 거절로 Open 미전송,첫 Current 유지/read도 Current,원 host/guard close|
|1022 source-parent-open-queued-os1-r36|Open을 메모리 queue에만 넣고 IO poll 없이 close,worker는 실제 Open을 받지 않고 PeerEof|

각 worker raw는 고정 readiness+`first=Some(PeerEof);secondary=0;overflow=false`와 stderr EOF였다. 원 worker exit23/Empty/nonforced 및 원 IPC reap/local close·raw EOF·최종 receipt를 확인했다. parent control/data EOF=false는 그대로 보존했다. guard는 close 완료 전 drop0,owner drop 후1을 in-process 확인했다. 외부 test는 각각 exit0/Empty/nonforced/raw EOF/selected_one=true/ownership complete다. 각 root에 원 outer outcome/receipt/raw·inner-observed copy를 보존하고 원 tmp도 유지했다. 선택 fixed artifact hash는 실행 후 동일하다.

request URL은 fixture이며 실제 완성 bootstrap은 native IPC로 전달했다. Open queue는 bytes 생성만 하고 전달하지 않으며 close가 이를 자동 송신하지 않음을 worker 진단과 코드 경로로 확인했다. 네트워크 packet 계측·정상 HTTP 전송·download 성공을 주장하지 않는다. actual inherited cutoff와 ParentStream 설정이 일치함을 확인했으나 Open payload의 실제 worker 수신/clock 경계 실측은 아직 하지 않았다.

## 미완료와 보존

정상 Open/Metadata/Data/credit/BodyEnd의 실제 OS 종단, 데이터 읽기/쓰기 실패·대기 중 현재성 무효화·강제 종료·시간축 검증, 원 provider replay/current/attempt와 SDK/CLI 연결이 남는다. Root 제품 DFB0 190/정상bins·source375 private overlay·기존 소유권/WIP/원 실패/AI품질0/9·GUI 미인수와 구별한다. 추론/다운로드/수동GUI/WARP/push 보류는 유지한다. modoo fixture/postcondition 지침을 재사용해 actual IPC와 callback fixture/packet 미계측 범위를 구별했다.
