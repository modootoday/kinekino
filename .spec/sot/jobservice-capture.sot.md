---
id: sot:kinekino/jobservice-capture
title: 원 JobService를 사용하는 일괄 입력 capture
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 원 JobService를 사용하는 일괄 입력 capture

최신 검증은 [R126 capture 근거](../../../verification/application/job-batch-capture-20261028.md)를 따른다. 공통 Jobs80시험·제품 기본 no-run·실제 자막 및 합성 거절 capture2시험은 통과했다. 카드 구성 컴파일과 나머지 검증은 진행 중이며 아래 컴파일 전 표기는 작성 당시 기록이다.

R126 구현 중. capture는 원 service의 admission gate 안에서 공통 normalize_request→명시 adapter prepare→source SHA 고정→normalization 재진입 동일성 검사를 수행한다. submit도 같은 normalize_request를 사용하며 기존 key 검사·dedup-before-prepare·Job admission·wake 순서를 유지한다. capture는 Job ID/record를 생성하거나 worker를 깨우지 않고 source blob을 준비할 수 있다. source blob은 완료 산출물이 아니다.

JobExecutor.supports_batch_capture는 기본false다. 첫 opt-in은 기존 자막 convert/export_range와 카드 export_cards이며 registry는 원 adapter로 위임한다. opt-in은 prepare가 안전한 persisted parameters를 검증하고 source.sha를 expected_source_sha256으로 재제출할 수 있다는 계약이다. 원시URL/credential/임의engine·명령을 journal로 쓰지 않으며 다른 engine/model/download 경로를 자동허용하지 않는다. normalization 재진입이 변하면 Unsupported로 거절하고 준비 source 증거를 보존한다.

CapturedJobRequest는 원 core만 생성하는 Rust 값으로 Deserialize/wire constructor가 없다. batch의 외부 생성은 CapturedBatchInput→BatchDocument::from_captured를 사용한다. 원 raw constructor는 job_batches 내부로 제한한다. 초기 journal은 captured source pin을 요구하고 관측 Job의 source SHA도 같은 pin이어야 한다. capture는 admission·실행 성공·출력 부재/충돌 해소·현재파일가용성·AI품질 승인이 아니다. source가 바뀌면 원 prepare가 거절하며 새 pin으로 자동갱신하지 않는다.

preimage-r125-batch-capture에 원 core/API/registry/자막·카드 adapter·journal/tests를 보존했다.2145fmt 종료0,2146snapshot 진행 중. 원 retaining host에서 실제 subtitle capture→Job0/출력없음→source변경거절→원submit/출력검증→source변경후같은key replay→read-onlycapture거절과 원shutdown 시험을 작성했다. 미허용·비멱등 synthetic adapter의 no-queue 거절을 별도로 검증한다. 기존 실제 card Jobs 시험도 capture pin→원submit/replay/reopen으로 보강했다. 아직 컴파일/시험 통과로 계상하지 않는다.

다음은 원 BatchLease 아래 submit/refresh/retry_failed orchestration·actual native event store/scope FileID binding·SDK/CLI/실제부분실패 인수다. typed document fixture의 scope는 합성값이며 실제BatchStore custody를 입증하지 않는다. 최신 완료baseline R125/카드actual R123,기존소유권/WIP/원증거/private overlay·원AI9 NeedsRepair/품질0/9·보류 유지.

기존 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100),code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)을 적용해 원 host/source와 합성 거절 fixture·실제 batch 미구현을 구분한다.
