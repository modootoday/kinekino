---
id: evidence:kinekino/20261029114145
title: 자막 범위 job 실제 출력·취소 — 2026-10-27
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 자막 범위 job 실제 출력·취소 — 2026-10-27

제품 정책 코드는 유지하고 기존 jobs range_export의2시험을 별도로 실행했다. 취소 시험만 subtitles.convert 외 subtitles.export_range의 발행 예약 전/후를 추가했다. GUI·영상 다운로드·모델 추론·WARP·push 없음. 원본/WIP/소유권·private overlay·기존 증거 유지. 등록190의 jobs6은 이 실제종단과 다르다.

## 실제 결과

로그는 `runtime/verification/source375-datalab-b0-r1-logs/777~782-range-*`. 원 actual CLI/요청/stdout/stderr/exit·파일·host기록은 `runtime/verification/range-jobs-actual-r1/`에 있다. fixture가 원/복사 CLI SHA256=1DF8ADD0F6210B4E7C38CEC46943D0EA6647530086902DCB752752D4C735378D를 확인했다. 원 Cargo773 jobs producer로777시험을, 새 Cargo778 jobs producer로779시험을 실행했다.

- 777 range_export2/2, CLI19회:94바이트 원 SRT를2..5초 범위로 clipping/offset해98바이트 VTT를 발행했다. 원 source SHA=2840bb1f0ba39a911fe040b9bb79861486628f0f2bb2e7a5bbd57432782017aa, output SHA=f85e351d515582490a97eb64a3febd6479851fd1993c5bee268262718125288c. job 출력과 동기 SDK/CLI 출력의 전체 바이트가 일치했고 한국어/일본어 원문을 유지했다.
- 동일 요청 idempotent job 재사용, 범위/operation 변경 충돌, 원 source SHA 불일치 SourceExpired/미생성, 미세시간 정밀도 손실 NeedsInput/미생성, allow_loss 명시 후 별도job 완료와 losses기록을 확인했다. 기존 output은 revision_conflict/NeedsInput·원 SHA 유지, 빈 범위는 invalid_argument/NeedsInput·미생성, range 누락은 입력 거절이었다. 원 source SHA를 마지막에 재확인했다. 정상 host stop은 pipe/sdk joined·completed_total1이다.
- 778 no-run exit0, 779 취소1/1 내부4사례/CLI12회: 기존 convert2와 새 range2를 실제 executor/파일/IPC로 실행했다. reserve_publication 전 gate에서 취소하면 requested/cancelling→Cancelled/output없음, 예약 성공 뒤 gate에서 취소하면 publication_in_progress/running→Completed/실제output SHA일치다. 두 경우 gate 중 output은 아직 존재하지 않는다. 이는 파일 저장 완료 뒤 취소 시험이 아니다. 원 source는 모든 사례에서 동일했다.
- 780 해당 Native jobs strict·781 fmt exit0. 시험 수행에서 첫 실패는 없고 과거 실패는 원 기록에 보존했다.

## 입력·범위

783 전후 검증은 exit0/39913ms로 종료했다. 전체2244 입력 content/identity 동일이며 아래 진행 중 표기는 종료 전 이력이다.

782 고정 완료: SOURCE.md SHA256=942117833E0F48D6419212A7B76476C83C4813A0DA44F6ADCDE980084A01ECB2, inventory SHA256=DD231E01019C2FE0CB041FB83AA67099FA03D8AA7033CE5A2D06CB6094880385, files2244. `runtime/verification/source375-range-job-cancel-input-r1/`에 보존했다. 783 전후 동일 확인은 진행 중이다. 아래 고정 진행 중은 이전 기록이다.

`tests/jobs/cancel.rs` 새 SHA256=672A8CAE2A06D0CDC20D14BF2CF2493ECCAEDBBB30E32BB5AA7FA8D6A026B7A3, preimage `runtime/verification/range-job-cancel-preimage-r1/cancel.rs` SHA256=B5F81C727600626DD59656896CD21A4321E21270F9F0682BE557FCACEC4B7B61. 777은 SOURCE28B8A1E9에서,779는 이 시험 전용 확장 후 실행했다. 782 새 입력 고정은 진행 중이다. 전체strict/190/정상bins를 이 시험 변경 뒤 재실행한 것으로 계상하지 않는다.

기존 modoo.today test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100)의 원본/합성·실제 파일·통제 gate 범위를 적용했다. 생성한 자막 fixture이며 실제 영상 PTS/화자 추론/GUI/사용자 품질·모든IO fault를 입증하지 않는다. 전체 CLI와 나머지 제작 executor 인수는 미완료다.
