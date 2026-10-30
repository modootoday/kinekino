---
id: evidence:kinekino/20261029114609
title: 격리 process-backed SDK adapter — 2026-10-27
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 격리 process-backed SDK adapter — 2026-10-27

## 선택 입력과 구현

[격리 준비/첫 실패](source-sdk-integration-preparation-20261027.md)를 잇는다. 제품 root 및 private R36을 수정하지 않고 `runtime/development/source-sdk-integration-r1`에서 실제 제품 SourceOpener/DownloadSession을 구현했다. 선택 input-r4 SOURCE.md SHA256 `434759F179C15747E93B2B62AB13776908E01C8BE7356E0411C6CC12C6D3999A`, inventory `2CC931C8E6CD6F962440682DE3865607C0D39D85310871924AC298FEEDB136A9`,2280files. lock955DEBA0 유지다.

opener와 session은 원 ParentTransfer/guard·admission·receipt sink·deadline을 같은 Arc/Mutex로 보유한다. launch/queue/read/IO callback도 원 control/current 검사에 연결했다. open 시도는 검사 전 spent가 되고 Metadata 뒤에만 session을 반환한다. read_owned는 원 opaque checkpoint cancellation을 사용하며 Pending을 EOF로 바꾸지 않는다. Metadata length는 관측값이고 trusted digest는 None이다. 원 core의 저장/CAS/verification/publication 정책을 adapter에 복제하지 않았다.

core는 정상 run에서도 session.close_owned를 먼저 호출하므로 session이 같은 원 deadline 내에서 실제 close를 구동한다. opener.poll_close는 원 deadline을 단축하고 같은 소유자를 polling한다. receipt 시도 전 attempted를 고정하며 실패한 원 ApiError를 보존/재반환하고 재시도·ACK하지 않는다. poison 시 정상 사용은 거절하지만 정리는 원 owner를 회수해 계속할 수 있다. 기록 sink에는 원 facts뿐 아니라 원 ParentTransfer 참조를 공급해 실제 raw IO/host 오류를 보존할 책임을 둔다. 실제 sink 구현·모든 cleanup fault 인수는 남는다.

전송 current와 발행 직전 origin을 분리했다. check_origin은 확인된 BodyEnd/physical close 및 SDK 첫 오류·cleanup 오류 부재를 요구하고 원 admission의 별도 check_origin을 호출한다. publication control의 execution_identity 부재를 새 권한으로 바꾸지 않는다. 실제 issuer는 원 attempt/source guards를 보유·검증해야 한다. **구체 browser issuer/registry는 아직 없고 제품 Unsupported를 변경하지 않았다.**

## 실제 게이트

|로그|범위/결과|
|---|---|
|1033|R3 default workspace library test no-run,exit0/273923ms. 이전 진행 중 기록은 종료됐다|
|1034/1035|R3 fixed producer의 정확한 cleanup OS2,각1pass. 원 R3 범위로 보존|
|1036|R4 `-p kinekino-player --all-targets` strict 통과|
|1037|R4 workspace `cargo fmt --all --check` 통과|
|1038|R4 `-p kinekino-player --lib --no-run` exit0/33954ms|
|1039/1040/1041|R4 정확한 ignored SDK/OS3,각1pass|
|1042|변경 owned-process/media-source-host 각각 all-target/all-feature strict 통과/19190ms|
|후확인|선택2280 content/identity 동일,원 R36 76 content/identity 동일,원 root lock B2E3D540 유지|

실행기 conservative2/CPU2/BelowNormal·C-only 전용 target/tmp·stable MSVC·locked/offline/j1/heavy1을 유지했다. live exec handle의 terminal을 확인한 뒤 다음 heavy 작업을 시작했다. 추가 설치·실제 inference/HTTP·수동GUI·push는 없었다. R4는 Native feature 전체 회귀/190/bins 재실행이 아니다.

## 고정 producer와 실제 OS3

R3 `native/source-sdk-integration-r1/fixed-r3/sdk-source-tests.exe`: SHA256 `426D81CEDBA32FBA129DE68F30291152DEF3BFAA51B62CB9B2DC29FD625B707D`,108750336B.

선택 R4 `native/source-sdk-integration-r1/fixed-r4/sdk-source-tests.exe`: SHA256 `99A9C1F6B8BF948EDDD6603197FB6FC0154A14EAC736451235BF81DB1EAA7584`,108767744B. 검사 후 동일 hash도 확인했다.

operator index `runtime/verification/source-sdk-integration-r4-index.json`:5C774D05D788FC4156477D5719BFFE8173B0C177DE78C0AFAA7627C48549A6F3/386B,proof=false. 원 independent C2 fixed driver7E7AD258/45s+cleanup5s가 실제 test를 보유했다. child 인자의 R22 worker2BDCA67F는 guard-only이며 시험에서 실행하지 않았다.

|로그/root runtime/verification|실제 검사|
|---|---|
|1039 source-sdk-rejected-open-os1-r4|원 SDK open PermissionDenied→재호출 spent; 원 IPC/evidence no-child close; consume0; guard는 session drop 뒤에도 유지되고 마지막 opener drop에서1회 해제|
|1040 source-sdk-sink-failure-os1-r4|원 transport receipt의 실패 request_id 보존,호출1회/재시도0/ACK0; 원 guard 유지|
|1041 source-sdk-closed-origin-os1-r4|BodyEnd fixture 전 origin 거절,실제 no-child physical close 뒤에도 body 미확인 거절; fixture BodyEnd 뒤 transport current는 거절하되 별도 origin 검사 성공; source revocation 뒤 SourceExpired|

각 test stdout은1passed/0failed/901filtered이다. 외부 원 test family PID37836/13132/37588은 각각 exit0/Empty/nonforced,raw stdout/stderr EOF,selected_one=true,ownership complete,first_error=null이다. 내부 fixture sink의 의도된 오류를 외부 실행기 실패로 바꾸지 않았고, 반대로 내부 ACK 성공으로 계상하지도 않았다. 각 root에 원 outcome/outer receipt/raw와 inner-observed 복사본,원 C tmp sdk-source-*도 보존했다. inner copy의 파일 식별자를 원 handle 증거로 대체하지 않는다.

세 시험은 실제 Windows prepared IPC/evidence handle 정리와 SDK trait 호출을 검증한다. session 및 BodyEnd 일부는 명시 fixture이며 실제 browser admission/Metadata/Data/HTTP 종단 성공이 아니다. 원 guard는 시험의 수명 관측 fixture이고 제품 executable admission을 증명하지 않는다.

## 남은 작업

정상 SDK open→Metadata→credit/Data→BodyEnd→probe→close→candidate publish와 오류/취소·원 binding/current 무효화·panic/cleanup 실패의 실제 연결 검증이 남는다. browser 일회 replay/current/attempt issuer,구체 receipt persistence/오류 정책,제품 DownloadPreparation/registry/CLI 연결도 미완료다. fixture transport 종단과 실제 HTTP 인수를 구별해 이어간다. 실제 다운로드/추론·수동GUI 재개는 기존 보류 범위다. root DFB0 지정190/정상bins는 원 입력의 성공으로 유지하며 이 후보의 성공으로 승계하지 않는다. AI 사용자 품질0/9·GUI 시각 미인수·source375 private overlay와 원 WIP/소유권/첫 실패를 유지한다.

modoo fixture/mock 및 postcondition 지침을 재사용해 실제 원 handle/외부 family 관측과 fixture authority/BodyEnd를 분리했다. 이번 적용은 SDK 계약·first failure·재시도 금지·물리 종료 후조건에 한정한다.
