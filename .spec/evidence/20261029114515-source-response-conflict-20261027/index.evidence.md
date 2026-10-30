---
id: evidence:kinekino/20261029114515
title: 원 응답 충돌의 metadata lease 무효화 — R18
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 응답 충돌의 metadata lease 무효화 — R18

## 실제 결함과 변경

원 Browser 담당 metadata7을 최신 private SDK에 연결한 R17 후 replay issuer 경로를 대조했다. RequestLedger::observe_at는 같은 current scope/request URL의 두 번째 Response를 모두 무시했다. 서로 다른 status/MIME 등 응답이 관측돼도 원 lease가 유효한 결함을 1119 단일 재현 시험으로 확인했다(unwrap_err가 Ok를 받아 실패). 원 실패 stdout/stderr/terminal receipt를 보존했다.

격리 후보 `runtime/development/source-sdk-integration-r1/src/application/browser_network/request_ledger.rs`만 수정했다. scope/loader/frame/URL이 맞는 기존 응답과 정확히 같은 반복은 무시하고 lease를 유지한다. 같은 현재 식별의 충돌 응답은 원 response를 덮어쓰지 않고 record.retired=true로 원 lease·재발급을 차단하며 relevant=true로 충돌 관측을 원 evidence에 남긴다. 다른 scope의 늦은 응답을 현재 응답 충돌로 처리하지 않는다. metadata는 replay 허가가 아니다.

원 preimage는 후보 runtime/verification/preimage-r17-response-conflict/request_ledger.rs에 보존했다. 제품 root·donor·lock·소유권·기존 WIP 및 보류는 유지했다.

## 실제 검증

로그 경로는 후보 runtime/verification이다.

- 1119 최초 재현:1 failed/930 filtered,terminal exit1.
- 1120 중간 수정:browser_network36pass/895filtered. 충돌 관측 보존을 추가하기 전 입력으로 구분한다.
- 1121 최종 수정:browser_network36pass/895filtered,terminal exit0. 새 시험은 동일 반복 current 유지·충돌 후 SourceExpired·재발급 거절·관련 관측 반환을 확인한다.
- 1122 all-targets clippy --locked --offline -j1, test-support feature, -D warnings 통과.
- 1123 browser_provider33pass/898filtered,terminal exit0.
- 1124 fmt --all --check 통과.
- custody verify-current 원 실행 session26692 terminal exit0:2287 content/identity unchanged.

input-r18은2287파일, SOURCE.md SHA256 `077D3E1D94AB7954A587DDD7F83717BB2ABCE54C8443D9D28365830A5607364A`, inventory `DA3D79EE102974ECC67D40B34C42244DCBD7AD277F6EC220E49666EFB370E8E6`다. 최종 request_ledger SHA256 `5193392D0275278584099031DC3FF5F98FCD33E710330D2A78CC02939AEFAC85`다.

## 남은 발급 공백

현재 extra_info.rs는 Referer-only projection으로 Cookie/Authorization/associatedCookies를 의도적으로 폐기한다. RequestHeaderEvidence는 referer/수신 시각만 보존한다. 따라서 빈 referer/관측 header로 인증 부재를 증명할 수 없다. 다음 issuer는 명시적인 provider별 replay 정책과 원 request의 검증된 credential 상태를 필요로 한다. secret 원문을 evidence에 보존하거나 GET200으로 허가를 추정하지 않는다. 기존 BrowserWork clone은 native activate/frame 명령의 전송 전 후보 검증에 쓰이므로 무조건 제거하지 않는다. 원 authority가 후보 clone/스냅샷에서 재발급되지 않는 계약도 후속 issuer 시험 범위다.

실제 브라우저/HTTP·추론·다운로드·GUI·push는 실행하지 않았다. DFB0 지정190/정상bins와 AI 사용자승인0/9는 원 상태를 유지한다. 전체 제품 완료로 계상하지 않는다.

modootoday/software-qa/test-fixture-and-mock-build(기독 본문 SHA A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100)의 실제 실패 재현·원 수명·fixture/서비스 경계 규칙을 적용했다.
