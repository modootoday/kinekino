---
id: evidence:kinekino/20261029114332
title: 실제 SDK/IPC 오프라인 body 종단 — 2026-10-27
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 SDK/IPC 오프라인 body 종단 — 2026-10-27

## 범위

[SDK adapter R4](source-sdk-process-adapter-20261027.md)의 실제 정상 byte 흐름을 추가 검증했다. 격리 mirror에만 `source-adapter-test-support` feature와 Rust `source-sdk-fixture-worker` bin을 추가했다. worker는 원 spawn_source가 상속한4개 handle/cutoff argv만 소비하며 원 SourceIpc를 보유한다. HTTP client·DNS·브라우저·모델을 호출하지 않는다. bootstrap/Open/credit를 원 framing/Exchange로 검사하고 고정16byte `offline-sdk-body`의 Metadata/Data/BodyEnd를 보낸 뒤 원 IPC cancellation completion/local close를 확인한다. URL/본문을 로그에 출력하지 않는다.

시험의 LiveAdmission은 명시적인 synthetic permission이며 원 browser issuer·sealed JobAttemptIdentity가 아니다. SDK trait를 직접 호출했고 JobsCore prepare/run/candidate publish를 이 시험에서 실행하지 않았다. 실제 HTTP/제품 다운로드 성공·사용자 AI 품질 인수로 계상하지 않는다. 제품 root/R36 및 기존 보류를 유지했다.

## 고정 입력/producer

`runtime/development/source-sdk-integration-r1/runtime/verification/input-r5/SOURCE.md`: SHA256 `75D7A4419317E357F7F1EB4F96531618DB5AF4B17B580F1F76881EA600A25A0E`,inventory `3F8F7A366EE434CA5EBF24044786E3C029EA7F1CE37B123A64A6C95EA2BA89BA`,2282files. lock955DEBA0 유지. 이후 수정 대비 Cargo/lock/worker/adapter/tests의 bounded 복사 preimage를 mirror runtime/verification/preimage-r5에 보존했다. 복사본 식별자를 원 식별자로 간주하지 않는다.

|로그|실제 결과|
|---|---|
|1043|fixture bin strict 통과; 이후 시험 추가 전의 범위|
|1044|선택 소스 제품 all-targets/시험 feature strict 통과|
|1045|정상 fixture worker bin build 통과,profile.test=false|
|1046|같은 feature의 제품 lib test no-run 통과/53.58s|
|1047|정확한 SDK/OS1,1passed/0failed/902filtered/0.09s|
|1048|전체 workspace fmt --check 통과|
|1049|선택2282 content/identity unchanged,exit0|

고정 `native/source-sdk-integration-r1/fixed-r5/source-sdk-fixture-worker.exe`: SHA256 `2895FD22769FFA7AA113193D385F957E119BDF86F375BE2267CD6BDD26E22518`,999424B.

고정 `fixed-r5/sdk-source-tests.exe`: SHA256 `94D70F7853D3DA53B172DCCDF305E510AA1AE19C2D5EC2433C219D5742870E28`,108838912B. 실행 후 두 artifact hash 동일을 확인했다. operator index `runtime/verification/source-sdk-integration-r5-index.json`은5157AF72A7237FCC2ADEF9CC015C93D5927D6A53F14BDC3D4A0A1086874AB7FF/516B,proof=false다. 원 C2 fixed driver7E7AD258이 두 artifact를 guard하며45s/cleanup5s로 정확한 test를 실행했다. 실제 worker는 같은 고정 child artifact다.

## 실제 관측

정확한 case `infrastructure::download::process_source::tests::live::sdk_open_metadata_credit_body_end_and_physical_close_use_same_original_owner`.

원 SDK open은 Metadata 뒤 반환하고 length16/선택 MediaIdentity/독립 trusted SHA=None을 유지했다. 원 admission.consume는1회,3byte read buffer로 전체16byte를 동일하게 읽었다. 명시 BodyEnd 뒤에만 read0을 받았다. physical close 전 origin은 거절되고 같은 session.close_owned로 원 host/IPC/raw/final receipt 정리 후 origin 검사를 통과했다. 그 뒤 fixture source를 revoke하면 SourceExpired다. transport record는1회이며 session drop 뒤에도 guard가 유지되고 마지막 opener drop에서1회 해제됐다.

원 worker PID27744/creation134375649889893991,exit0/Empty/nonforced. 같은 원 IPC는 io_reaped/local_endpoints_closed=true,had_error=false,control/data EOF=false를 그대로 보존했다. raw stdout/stderr EOF 및 process final receipt Recorded를 확인했다. local endpoint close를 peer EOF로 바꾸지 않았다.

외부 test PID35944,exit0/Empty/nonforced,raw EOF/selected_one=true/ownership complete/first_error=null. 원 `runtime/verification/source-sdk-live-body-os1-r5/outer` outcome/receipt/raw 및 inner-observed copy,원 tmp sdk-source-live-*를 모두 보존했다. source-receipts.txt에는 원 process receipt와 transport facts를 실제 flush/sync했다. 첫 실행 실패는 없었다. 직렬 heavy1·각 live handle terminal 확인·C-only/offline/locked/j1/conservative2를 유지했다. 원 제품 lock은 B2E3D540으로 동일하다.

## 다음 실제 공백

원 coordinator가 발급하는 sealed attempt identity/cleanup deadline을 받는 DownloadPreparation→JobsCore run→검증→candidate publish를 연결해야 한다. 현재 synthetic admission을 실제 권한으로 승격하지 않는다. BodyEnd 부재/부분 frame/pipe EOF·실행 중 취소/current 무효화·오류 뒤 정리 등의 정상 종단 인접 실패 행렬도 남는다. 실제 browser replay issuer/receipt persistence/제품 registry·CLI/HTTP는 미완료다. 기존 R4 OS3은 해당 원 입력의 실행이며 이번에 재실행했다고 계상하지 않는다. 원 DFB0 190/bins와 전체CLI/AI/GUI는 각각 별개 인수다.

modoo fixture/mock·postcondition 지침을 재사용해 실제 native IPC/byte/owner 관측과 synthetic permission/고정 body/실제 browser 권한 부재를 구별했다.
