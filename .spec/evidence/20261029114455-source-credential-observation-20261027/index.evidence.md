---
id: evidence:kinekino/20261029114455
title: 원 요청의 비밀값 없는 인증 관측 — R19
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 요청의 비밀값 없는 인증 관측 — R19

## 구현 범위

중앙 PM은 최신 private SDK 후보의 browser_network6파일(extra_info,protocol_events,request_headers,request_ledger,capture_state,mod)만 변경했다. 원 담당 donor/root·WIP·소유권은 유지하며 preimage는 후보 runtime/verification/preimage-r18-credentials에 보존했다. 기존 Referer-only 관측이 버리던 credential의 값은 계속 버리고 존재/불충분 상태만 원 session/request header association→ledger→metadata lease로 전달한다.

CredentialObservation은 Unknown(default),Present,NoneObserved다. Cookie/Authorization/Proxy-Authorization 헤더 이름은 대소문자와 무관하게 값이 비어 있어도 Present다. associatedCookies 비어있지 않으면 blockedReasons와 무관하게 보수적으로 Present다. 전체 header 값이 문자열이고 associatedCookies가 명시 빈 배열인 경우에만 NoneObserved다. 누락/null/잘못된 관측은 Unknown이다. NoneObserved는 모든 인증의 부재나 public replay 허가가 아니다. 비밀 헤더 값·연관 cookie 값은 event/ledger evidence에 보존하지 않는다. historical RequestHeaderEvidence에 새 필드가 없으면 Unknown이며 과거 증거를 허가로 승격하지 않는다.

## 고정 입력과 실제 결과

input-r19 SOURCE.md SHA256 B42EEFBCA2F8EAADB886743CE6AD6077687032538CA99525DBB6BF7C62ADEAB3; inventory A0C4DF5E06D87B090041AE78E735E4B6944A218FBBC9C751F53C20E566447608;2287파일.

원 로그는 runtime/development/source-sdk-integration-r1/runtime/verification에 있다.

- 1125 중간 all-targets strict 통과. 뒤에 추가한2시험은 이 입력의 실행으로 계상하지 않는다.
- 1126 최종 browser_network39pass/895filtered:8개 인증 관측 조합·비밀 문자열 비노출, historical default Unknown, original stream의 sanitized credential 보존을 새3시험으로 확인했다.
- 1127 최종 all-targets clippy -D warnings 통과.
- 1128 browser_provider33pass/901filtered.
- 1129 fmt --all --check 통과.
- custody verify-current session85838 terminal exit0,2287 content/identity unchanged.

각 compiler/시험 실행은 동일 resource-runner/conservative2·heavy1·stable MSVC·locked/offline/-j1, C-only native/source-sdk-integration-r1와tmp를 사용했다. root lock B2E3D540665FFC8E68D24D0A6C531D2B983658FB19B5ABBA525659BA1ACC371D와AGENTS40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C 유지. 이번 gate 첫 실패는 없다. R18/1119 원 실패는 보존했다.

## 미완료

Provider별 명시 replay policy·원 response/session/generation의 typed 발급·일회소비·원 owner 수명과 실제 source 연결은 아직 미완료다. 관측 enum은 해당 계약의 입력일 뿐 ReplayAdmission 성공 구현이나 제품 source 등록이 아니다. Unknown/Present를 허가하지 않으며 NoneObserved만으로 허가하지 않는다. 실제 브라우저/HTTP·추론·다운로드·수동GUI·WARP·push·설치를 수행하지 않았다. root DFB0의 지정190/정상2bins를 새 입력 재실행으로 계상하지 않는다. AI0/9·GUI 시각 인수와 제품 전체CLI는 남는다.

기독 modootoday/software-qa/test-fixture-and-mock-build 본문 SHA A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100을 재사용하여 missing/invalid/positive 관측과 실제 서비스 사실을 분리하고 fixture의 secret 비노출·원 lease 전달·과거 evidence 기본값을 검증했다.
