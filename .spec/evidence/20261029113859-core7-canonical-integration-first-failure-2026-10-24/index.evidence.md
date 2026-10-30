---
id: evidence:kinekino/20261029113859
title: Core7 채택·통합 CPU 첫 실패
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Core7 채택·통합 CPU 첫 실패

PM 배정으로 SDK selector3와 UI I4exact4를 canonical에 채택했다. `runtime/verification/core7-canonical-integration-cpu-r1/`의 `02-canonical-{SDK,UI}-receipt*.json`, `04-canonical-combined7.json`이 정확 경로·후보 핀을 연결한다. 선택 Meari 진단2/gitlink·기존 WIP는 수정하지 않았다. canonical7은 **통합 미검증 WIP**이며 stage/commit0이다.

fmt는 terminal0/6175ms. 기본 제품 package cargo test는 terminal101/528461ms, 완료17 harness 합계924pass/4fail/45ignored다. 라이브러리는742pass다. `tests/media_download_owned.rs:337`의 별도 fixture BIN 요구로 nonignored4가 실패했다. 원 `08-test.stdout.jsonl`, `08-test.stderr`, `08-test.receipt.json`, `10-test-summary.json`을 보존했다. 이후 harness·Clippy/default/native strict·등록은 미실행이며 전체 통합 성공이 아니다. fixture 임의 공급·시험 ignore 추가·재시도·수리0이다.

S exact7의 원 SHA/bytes/mtime/FileID와 임시 file_guard1의 원부재를 복원했다(`09-*`). canonical7 후보·부모5·기존 기타5·과거 실패12를 유지했고 보호203delta0이다. Cargo가 바꾼 normal producer를 JSON에 귀속해 보존한 뒤 alias4만 unlink/독립교체로 원 bytes/mtime를 복원했다(FileID 변경 유지, deps 새 producer4 불변). `10-product-artifacts.json`, `12-test-executables-provenance.json`은 실행파일38의 정확 경로·SHA·길이·profile/features/fresh와 원 producer를 연결한다. 추가 전체 binary 복사/전수 cache 보호를 주장하지 않는다.

`13-EOF4.json`·`13-audit.json`에서 원 runner 로그4의 exclusive EOF·기록 PID4 부재·관련 writer/child0을 확인했다. carry156은 원150+이번 고정 시험2+normal producer 독립4이며 SHA01F0F923955BFE077789D19F6916C2CA08781F3601CDFB141B7A9596342865CA다. 알려진 alias/known 예외와 all_cache_protected=false를 유지한다.

실제 Native/미디어/모델/D-B/staging·수동 시각0이다. 기존 개별 I4/N-C 근거를 새 비계측 N/C·core7 통합 성공으로 승격하지 않는다. modoo.today frontend-first-journey 및 test-fixture-and-mock-build의 원 fixture 요구/첫 실패·검증 범위 분리를 적용했다. 후계 fixture/시험 범위와 커밋은 PM 소유다.
