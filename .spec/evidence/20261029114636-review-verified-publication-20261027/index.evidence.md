---
id: evidence:kinekino/20261029114636
title: Verified 저장 전이 작업 기록 — 2026-10-27
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Verified 저장 전이 작업 기록 — 2026-10-27

중앙 PM의 진행 중 기록이다. 완료 근거가 아니다. 기존 modoo.today software-qa/code-change-review-verification과 test-fixture-and-mock-build의 실제/합성 구분을 적용한다.

## 구현 범위

Verified archive metadata와 live proof의 권한을 구분한다. 일반 Create/Revise 및 combined create에서 호출자가 제출한 Verified 요약을 거절한다. 명시 VerifyAssociation 요청은 case/association/revision/media/evidence reference/note만 선언하며 서버 evidence resolver가 보유한 proof로 mapping record를 만든다. CAS writer reservation과 원 proof를 publication까지 유지하고 storage가 즉시 publication 직전에 호출하는 guard에서 control/case/media/원 proof와 record 일치를 다시 확인한다. exact committed replay는 현재 media/evidence/control보다 먼저 처리하되 case custody는 계속 요구한다.

저장된 Verified는 역사 기록이다. playback_ready=false를 유지하며 runtime 권한·frame 실제 관측·AI 품질 승인으로 승격하지 않는다.

## SDK·wire·실 파일 후계

revise payload의 선택 timeline_evidence 참조를 명시 VerifyAssociation 요청에 연결했다. 호출자가 timeline_status/mapping record를 직접 제출하면 DTO에서 거절한다. create의 evidence 참조도 거절한다. resolver는 trusted assembly builder로만 주입한다. schema의 입력 Unverified-only association과 출력 archived Verified association을 분리하고 capability는 writable/resolver 주입 상태를 반영한다. 기본 normal CLI 구성은 resolver 미주입으로 새 Verified 요청이 Unsupported이며 JSON에서 도구 pin을 수입하지 않는다.

291 및 보강292 root review_source 영향 시험 각각31pass/1ignored/실패0. 새 SDK 시험은 실제 UserReviewFiles envelope를 닫고 재열어 저장2revision·exact replay·inspect를 확인했다. resolver 부재의 새 요청 Unsupported, readonly+resolver 구성 capability=false/write 거절, 호출자 요약/create evidence 거절을 확인했다. 이 시험의 case/media/evidence authority는 합성이며 실제 봉인 미디어 custody로 계상하지 않는다. 독립 외부 JSON schema validator 검증은 아직 없다.

293 native-proposal-window/all-targets strict 종료0/21548ms, fmt check 통과. 294 정상 review_source_cli no-run 종료0/55756ms. 295 정상 CLI3시험 종료0/22117ms(시험21.64초). 기본 구성의 timeline_evidence 요청은 unknown field가 아니라 Timeline evidence adapter unavailable/Unsupported/data=null/exit3으로 거절됐다. 이후 기존 revision/replay/CAS·readonly·combined 저장/재열기 및 원 증거 불변 검사를 통과했다.

정상 CLI SHA256=C8F0DF8A56C21667D95A6BC2CA55ADA033CB7FB63CE18F41DC7FBC0B258D8C1B. 실제 근거 디렉터리는 runtime/build-cache/tmp/review-source-cli-34204-0이며 01a-verify-unconfigured의 원 PID25088/exit3과 producer SHA를 기록했다. 실패 exit3은 이 명시 미지원 시나리오의 예상 결과다. 실제 봉인 미디어의 Verified 성공을 이 negative CLI로 대체하지 않는다.

SDK/CLI 후계 입력 runtime/verification/source375-verified-sdk-input-r1: SOURCE.md SHA256=480D1F18712D0815D4CB3BD60A61C1684A73F6377B0F90453AA05AC59A981F6A, source-current.tsv SHA256=F6A51A47CDCDADA0503BAB9A0A7FC566E433A910DE52D86DC18F8345D875ECDC,2176파일. 294 이후 snapshot을 고정한 뒤295를 실행했다. 후속 verify-current 종료0으로2176파일의 content/identity 동일을 확인했다.

## 실패·검증

- 287-verified-check: 첫 cargo check 실패. 함수 분리 patch가 checked_receipt에 잘못 적용됐고 application/domain의 timeline_evidence glob 이름이 충돌했다. service 원 실패 소스를 runtime/verification/review-verified-publication-preimage-r1/service-first-check-failure.rs에 보존한 뒤 수정했다. 원 stdout/stderr/receipt는 source375-datalab-b0-r1-logs에 보존한다.
- 288-verified-unit: 시험 코드의 application/domain timeline_evidence 이름 충돌로 컴파일 실패. tests-first-check-failure.rs 사본과 원 로그를 보존하고 명시 import로 수정했다.
- 289-verified-unit: 종료0/66554ms. root review_source 영향30pass/1ignored, 다른 workspace lib는 필터로 실행0이다. 새 계약 fixture2시험에서 proof 기반 저장·호출자 요약 거절·미디어/근거/제어 없이 exact replay 및 저장 직전 취소/근거 무효화 시 미저장·lease 반환을 확인했다. 합성 계약 시험이며 실제 durable 저장 시험이 아니다.
- cargo fmt --all -- --check 통과. 290-verified-strict는 native-proposal-window 구성의 all-targets 검사 종료0/42588ms. dependency blitz-paint의 기존 deprecated 경고는 별도로 남아 있으며 임의 수정하지 않았다.

영향 검사 후 runtime/verification/source375-verified-publication-input-r1에 bounded2176입력을 고정했다. SOURCE.md SHA256=E217DDD4DC50606986E21D4B1A887E886D4438C586AF9E7CD0695E5B39D0D387, source-current.tsv SHA256=0C115AFABB36FC96C4B46D340CF3AF625E0ADD80C6F0A0091E9616AD3AACF061. 검사 전후 전체 동일성을 이 사후 snapshot만으로 주장하지 않는다.

이번 변경은 이전 SOURCE9A9FE935/2176 이후의 WIP다. 이전 190/정상 bins 및 실제 proof 시험을 이번 변경의 검증으로 재사용하지 않는다. 실제 봉인 미디어의 Verified 저장·CLI 성공, 독립 schema validator·전체 회귀는 아직 완료하지 않았다. 수동 GUI·추론·다운로드·WARP·push 보류와 기존 소유권을 유지한다.
