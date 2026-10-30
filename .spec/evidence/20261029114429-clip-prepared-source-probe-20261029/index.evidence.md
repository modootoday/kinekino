---
id: evidence:kinekino/20261029114429
title: 원 source probe 준비 소유권 — R195
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 source probe 준비 소유권 — R195

## R196 원 Jobs source admission 연결

`clip_render_jobs/source_admission.rs`의 부모 보유자가 원 captured source pin마다 준비한 probe를 보관한다. 기존 application `prepare_media`를 그 보유자 안에서 실행하고, 정상 인수한 원 lease는 PreparedClipMedia로 넘긴다. pin/clock/관측의 도메인 검증을 별도로 복제하지 않는다. 실패·unwind 시 부모가 같은 probe들을 정리한 뒤 원 코드/메시지를 유지하거나 Internal unwind 실패를 반환하며, 실제 정리 관측을 details에 남긴다. panic payload는 정리가 끝날 때까지 유지한다.

이 연결은 원 Jobs의 source admission 단계에 실제 적용했으며 encoder/decoder/overlay/verifier/publication 전체의 상위 owned-session 연결을 뜻하지 않는다. 모든 단계 연결 전 owned_executor opt-in은 하지 않는다. 현재 legacy 호출은 정리 중 원 부모를 유지하며 pending을 완료 ACK로 바꾸지 않는다.

수정 전 `preimage-r195-job-source-owner/` 보존. `input-r196`2456파일 inventory SHA256 `3E54C760914F1C3755128302CD4399D240982CB37EDA1C0E3A5F84FAB1027B97`, SOURCE.md SHA256 `025425224A5A3FC419BDE0FC1B6E79B1214AEC1FF0313F0041F51FB9F859AA51`. `2756`fmt/`2757`native strict/`2758`default strict/`2759`lib no-run 종료0. 고정 lib exe/PDB는 mirror `fixed-r196/native-core/`, exe SHA256 `F363CC16A7BD9258FFBACECA2C1C8D12D4C5EE7D17B36C7AC5B7C8D0208CCE6D`.

`2760-source-sdk-r196-job-source-owner` 실제1시험 통과. 부모 보유자 안에서 launch 뒤 주입 panic의 원 프로세스 child_exit124/family_empty=true/receipt_recorded=true/forced=true를 정리한 뒤 Internal을 반환하고, 실패 source 재인수를 거절했다. 정상 lease가 부모 보유자를 벗어나도 원 미디어 guard를 유지함도 확인했다. root `runtime/verification/source-sdk-r196-job-source-owner-r1/`의 failure/facts/verification 및 원 process 증거를 보존한다.

`2761` 원 Jobs 선택 자막 렌더·발행·동일key재사용·재열기, `2762` 취소·정상 재기동·수동 attempt2/입력·부분 증거 보존 각1 실제 회귀도 통과했다. root `runtime/verification/source-sdk-r196-jobs-subtitles-r1/`, `source-sdk-r196-jobs-retry-r1/`에 산출물을 보존한다. `2763-source-sdk-r196-custody` 종료0,2456파일 content/identity 동일 확인. 신규 SDKCLI/정상bins/190 통합은 아직 수행하지 않았다. source 이외 단계와 전체 owned-attempt opt-in은 미완료다.

private overlay의 `clip_sources/probe.rs`는 원 미디어/ancestor guard·도구 Arc·원 source SHA·deadline·준비된 native operation을 함께 보유한다. `prepare_source_probe`는 파일 확인/해시와 process evidence 준비까지만 수행하고 엔진을 launch하지 않는다. 상위 세션이 이 값을 보관한 다음 `run`을 호출할 수 있다. 기존 `inspect_source` SDK 경로도 이 객체를 사용한다.

동일 객체의 재실행은 거절한다. 결과 검증 뒤 원 guard를 NativeClipSourceLease에 한 번만 이전하고, `poll_close`는 원 operation을 정리하며 receipt 실패가 있으면 pending 소유권을 유지한다. close 전 launch가 없으면 나중 launch도 거절한다. callback panic 시 상위에서 준비 객체를 보관하고 catch한 뒤 원 객체를 정리하는 경로를 시험으로 작성했다. 기존 동기 API 전체의 panic/drop 안전 또는 Jobs owned-attempt 연결 완료로 표시하지 않는다.

- 수정 전 `runtime/verification/preimage-r194-source-probe/`에 원 mod와 operation 파일 보존.
- `input-r195`2455파일 inventory SHA256 `AEFAF81973D322833E46D0505FA88C6ADD03061F8AFFFB9D28351B63764A6EEC`, SOURCE.md SHA256 `3E349B07C4D6DE2AE177C53AF3858AF22EF191332DD0E782F4FC9E77AF695142`.
- 로그 루트 `runtime/verification/source375-datalab-b0-r1-logs/`. `2748`fmt/`2749`native strict/`2750`default strict/`2751`lib no-run 종료0. 고정 lib exe/PDB는 mirror `fixed-r195/native-core/`, exe SHA256 `F507B905128CA7F81FFED265C13968DBE59A9DDBAF585077752615EAFA9127D8`.
- `2752-source-sdk-r195-prepared-probe` 실제1시험 통과. launch 전 guard 유지·close 뒤 run 거절, launch 뒤 callback panic을 catch한 상태에서 원 guard 유지·같은 operation 정리를 확인했다. 실제 unwind-close는 child_exit124/family_empty=true/receipt_recorded=true/forced=true이며 정상 후보로 인수하지 않았다. 별도 정상 probe는 원 lease에 한 번 인수하고 source/tool 쓰기 거절을 유지했다. root `runtime/verification/source-sdk-r195-prepared-probe-r1/`의 raw/facts/verification/unwind-close와 원 process 증거를 보존한다. stderr의 expected callback panic은 명시 주입 fault이며 시험은 통과했다.
- `2753` 기존 실제 source/project admission·변경 pin 거절, `2754` MP4 검증/발행/해제 각1회귀도 통과했다. root `runtime/verification/source-sdk-r195-probe-regression-r1/`, `source-sdk-r195-render-r1/`에 근거를 보존한다. `2755-source-sdk-r195-custody` 종료0,2455파일 content/identity 동일 확인.
- 정상bins/190/SDKCLI 최신 통합은 R193이며 R194 강제 종료 정책·정리 관측은 별도 부분 근거로 유지한다. R195도 Jobs owned-attempt 상위 소유권 연결·전체 중단/복구 인수는 미완료다.
- 기존 test-fixture-and-mock-build/code-change-review-verification을 적용해 실제 엔진·원 guard와 명시 시험 fault를 구분한다. 공통 source clock/duration/관측 SHA·tool pin과 ownership/보류/private overlay를 유지한다.
