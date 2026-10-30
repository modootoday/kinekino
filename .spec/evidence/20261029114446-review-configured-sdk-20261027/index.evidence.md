---
id: evidence:kinekino/20261029114446
title: 동일 구성의 source·evidence·playback SDK
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 동일 구성의 source·evidence·playback SDK

후속 [같은 입력 통합](source375-review-configured-sdk-regression-20261027.md)은 선택190/정상2bins/저장CLI3·전후2197파일 동일/종료0까지 통과했다.

readonly/writable 관측은 user-review source 저장소 범위다. model review의 기존 decision 계약과 사용자 AI 품질 승인 권위는 바꾸지 않았다.

2026-10-27 중앙 PM. assemble_sdk는 기존 SDK의 review reader/source API/playback 세 adapter만 동일한 trusted Rust 구성으로 교체하고 다른 기능을 유지한다. root layout guard를 공통화해 evidence/user/runtime의 분리를 writable user 초기화 전에 확인한다. readonly는 생성하지 않는다. 임의 manifest/저장 Verified를 새 authority로 사용하지 않았으며 admission은 Rust-only다.

입력 runtime/verification/source375-review-configured-sdk-input-r1/SOURCE.md SHA256 `9F602A06C1D46F71D8F83484AD4F3E119ECB04D653726AFB7AD77A45F81F56BC`; inventory `3CD23AF02F68D1D834665426E079C0FF20E08598549222EE5FD118D17467DD0F`,2197파일. preimage는 runtime/verification/review-configured-sdk-preimage-r1. 원375 overlay/첫실패·이전E28 통합·CE23 fixture영향은 별도다.

402 초기 strict/403 공통21/404 최종 all-target native strict/405 actual no-run/406 actualCLI40(41.14초)/407 별도host process2(actual1,21.98초)가 통과했다. overlap의 writable 초기화 전 거절·readonly 무생성 신규2시험도 포함한다. 원 로그/receipts는 source375-datalab-b0-r1-logs 각 번호다.

406은 새 빈 configured-user root를 사용했다. 정상 CLI가 새 association cli-configured를 create(rev1)→실제 sealed evidence revise(Verifiedrev2)→exact replay→foreign proof RevisionConflict→inspect(rev2보존)했다. 이후 그 CLI-created association을 open/다섯 제어/관측 경계 pause/close 대상으로 사용했다. Verified publication의 playback_ready=false를 유지하고 실제 open/관측 근거와 분리했다. Unverified child미생성·재시작 scope 상호 거절·pipe/sdk Joined/3owner 물리 완료도 보존했다.

406 결과 runtime/verification/review-configured-sdk-cli-actual-datalab-r1/RETURN.json SHA256 `406A3355B47FE5C2B12417EA15FF820F5D1204E9A87E58E286E1F3F5BCBDACB2`; CLI producer `058751BA9E63DB0B1CC662214B3D266B38109F0D0C1EFEF5908B0C5B7A7489A1`. 이 정상CLI client를 기본 serve CLI 제품 host 완성으로 계상하지 않는다.

407은 readonly assemble_sdk를 사용하는 별도OS hostPID37084/33932·실제 미디어2 직렬에서 EOF→pipe/sdk Joined/owner1물리완료·exit0/forced=false를 확인했다. 같은 root에서 restart·새 endpoint/scope·이전 session RevisionConflict·동일 key/ordinal 새 host session을 확인했다. PROCESS-HOST-RETURN SHA256 `F6C90AF2FA45AEFEC557284D9482D61E680380C71BB0075BD2120D6D5080F399`. publication-only RETURN과 뒤따르는 별도 decode witness/return은 각 범위로 유지한다.

제품 CLI serve startup과 실제 producer의 trusted admission 공급·crash/입출력 실패 전수·전체CLI·AI품질/GUI시각은 남는다. actual9_binding/quality_approved=false. 기존 modoo.today QA/cache/shared-checkout의 정확한 출처·판단은 선행 원 worker/host 검증 문서에 유지했다. 기존 소유권/WIP·원 증거·C 캐시/locked/offline/jobs1/CPU2/BelowNormal/heavy1 유지. 수동GUI·추론·다운로드·WARP·push0.
