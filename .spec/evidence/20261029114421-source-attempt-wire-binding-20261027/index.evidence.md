---
id: evidence:kinekino/20261029114421
title: 원 AttemptScope와 wire 식별·byte budget — R24
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 AttemptScope와 wire 식별·byte budget — R24

## 구현

private SDK의 BrowserReplayAdmission에 원 PreparedJobInput과 AttemptScope의 결합을 추가했다. 생성은 provider/grant/owner/identity/deadline을 먼저 보유하고, 순수 입력 검증의 성공 또는 원 오류를 내부에 남긴 뒤 반환한다. 입력 검증 실패로 원 소유권이 constructor 밖에 남지 않는다. check/wire_binding/check_request는 저장된 원 오류를 반환한다.

별도 browser_replay/binding.rs는 input.operation=DOWNLOAD_OPERATION과 input.sha256=scope.identity.input_sha256를 확인하고 기존 DownloadRequest::decode로 validated byte limit을 얻는다. 실제 source descriptor/blob 검증은 기존 RequestBoundPreparation/DownloadRequest::from_prepared가 계속 소유하며 중복 구현하지 않는다.

wire digest는 domain `source-attempt-wire/1\0`, length-prefixed job_id/input SHA/source blob SHA/optional submission SHA, big-endian attempt·byte limit에서 SHA256으로 도출한다. optional fingerprint의 None/Some을 명시 구분한다. deadline을 새로 만들거나 digest를 권한으로 사용하지 않는다. 원 grant/current·실행 식별·deadline 검사는 별도로 유지한다. 요청 bootstrap의 binding이 다르면 RevisionConflict, maximum_bytes가 원 검증 예산을 넘으면 ResourceExhausted다. 더 작은 한도는 허용한다. 기존 URL/Referer 대조와 launch 전 request 검사도 유지한다. 예전 fixture의 [6;32] 임의 토큰을 이 경로에서 제거했다.

## 입력·변경·보존

기존 private 변경은 infrastructure/download/{browser_replay.rs,process_source.rs,process_source/tests/live/jobs.rs}, 신규 browser_replay/binding.rs다. process_source.rs는 기존 ApiError 복사 helper의 sibling 가시성만 확장했다. 원 preimage는 후보 runtime/verification/preimage-r23-wire-binding에 보존했다. root/donor/WIP/소유권은 유지했다.

input-r24 SOURCE.md SHA5648979872A5A1C24077A5DFAF68CE0E2BECD258DA741ED2881974151B2472D1;inventory26B408C1D233B9AD47EE39FE21EC2816AC62B3ABBB2AACA201EF09508024FE44;2292파일. root lock B2E3D540665FFC8E68D24D0A6C531D2B983658FB19B5ABBA525659BA1ACC371D/mirror lock955DEBA0338A3A32ED64D6B2A500C192CD55517336FE531D6EED374D0B2916A5/AGENTS40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C 유지.

## 실제 검증

로그는 후보 runtime/verification 및 원 source375-datalab-b0-r1-logs 아래1155~1161에 보존했다. 이번 gate 첫 실패는 없다. 이전1146 등 원 실패는 보존한다.

- 1155 중간 support strict 통과. 뒤에 추가한 changed-input assertion은 해당 실행으로 계상하지 않는다.
- 1156 최종 lib no-run terminal exit0. fixed-r24/sdk-source-tests.exe SHA D1C4DB95B9692746330DAAD80AB01FAFAE5BD3FE64893657CE2A846956A6E49F,109399552bytes.
- 1157 정상 CLI exact1 pass/941filtered,5RPC. prepare에서 다른 입력 SHA·변경된 wire token·원 byte budget+1 거절과 budget-1 허용을 확인한 뒤 원 요청1000byte 한도로 실제 offline IPC16byte를 검증·발행했다.
- 1158 취소 CLI exact1 pass/941filtered,7RPC. 같은 실제 scoped binding 경로의 prepare 검사를 거쳐 BodyEnd 뒤 probe pending 취소·원 child 정리·partial 보유/제거·Cancelled/미발행을 확인했다.
- 1159 최종 support all-targets strict,1160 기본 all-targets strict 통과. 1161 fmt는 receipt 재감사 결과exit1이며 당시 통과 보고가 틀렸다. 원1154와 같은 replay.check_request 서명 줄바꿈 불일치다. 원 실패를 보존하고1182 형식 수정 뒤 통과했으며 R24 재실행으로 계상하지 않는다.
- custody verify-current session19225 terminal exit0,2292 content/identity unchanged.

actual2는 기존 독립 C2 고정 driver로 직렬 배정했고, 각 outer selected_one/ownership complete/physical_closed·exit0/Empty/nonforced·raw EOF·first_error null이다. 원 worker는 R15/1101 SHA477ED83B9286BA0F556DD61F97C230FFD6F2C4A8770FFD45B6D161EA63E051F7/1000960bytes, 정상 CLI는 R14/1092 SHA249DD5ECEFFD0A476D3EC7B89B0F7003F9DB5DC13D285AE1126A4B9938DED32B/36405248bytes를 실제 재사용했다. 새 worker/CLI 빌드로 계상하지 않는다. index runtime/verification/source-sdk-integration-r24-index.json은proof=false/SHA BCF4F21CCD8182F03010C8F984A8CB5EC0A3C46E5E02A39775A3E236601DF90A다.

outer 근거는 runtime/verification/source-sdk-wire-cli-{normal,cancel}-os1-r24/outer다. 원 inner는 tmp의 동일 tag 하위 sdk-source-cli-19264-1793100897754220100 / sdk-source-cli-35412-1793100906029006000에 보존했다. 원 source30772/creation134375744979156503 및19240/creation134375745061656550 모두 exit0/Empty/nonforced이며 io_reaped/local_endpoints_closed·Closed/Recorded/raw EOF다. control/data peer EOF=false와 local close를 구별했다. 무거운 실행은 같은 handle terminal 뒤 직렬이며 C-only native/source-sdk-integration-r1/tmp·conservative2·locked/offline/-j1 유지.

## 미완료

이 digest는 wire 상관 식별이고 외부 인증·replay 권한·완료 proof가 아니다. retry 시 attempt가 바뀌는 digest 계약의 별도 종단 행렬, 실제 document owner 이전/원 close ACK, concrete provider policy·실제HTTP/probe·제품 source 등록/전체CLI/GUI는 남는다. 원 browser/policy/probe/body는 합성이며 실제 서비스 성공으로 승격하지 않는다. 이번 입력에서 browser79/지정190/정상bins를 재실행하지 않았고 각각 원 R23/DFB0 근거에 유지한다. 실제 브라우저/HTTP·추론·다운로드·수동GUI·WARP·push·설치는 수행하지 않았다. AI 사용자승인0/9 유지.

기독 modootoday/software-qa/test-fixture-and-mock-build 본문 SHA A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100의 원 input/attempt·negative/positive·원 cleanup 및 fixture/실제OS 경계 구분을 적용했다.
