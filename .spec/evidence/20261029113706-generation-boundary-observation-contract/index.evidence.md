---
id: evidence:kinekino/20261029113706
title: 내부 경계 관측 공통 Rust 계약 구현
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 내부 경계 관측 공통 Rust 계약 구현

2026-10-19. [내부trace 연구](../../player/infrastructure/inference/research-followups/gemma270-internal-boundary-trace.md)를 바탕으로 `tools/boundary-observation` 라이브러리를 추가했다. domain의 캡처 DTO·U32LE codec, application의 순수 검증기, infrastructure의 SHA 어댑터를 분리했다. 엔진과 관측 워커가 같은 Rust 타입 및 판정 규격을 공유하기 위한 구성요소다. 기존 모델·엔진 source/EXE·기존 실제 결과는 변경하지 않았다.

## 후속 raw schema2 예비 검사

06시36분 후속. [raw→DTO 매핑 계약](../../player/infrastructure/inference/research-followups/gemma270-raw-observation-mapping.md)을 따라 raw Journal/Plan/Event 타입, bounded JSON/file decoder, 순수 accounting 예비 검사를 별도 책임으로 추가했다. 필수진단 필드 누락·타입 오류·unknown top/event field를 decoder가 거부하고, accounting은 incomplete/invalid/dropped·저장실패·flush부재·mutation revision·전역 ordinal·중복/누락 request·close·completed membership·prepared-row event수를 검사한다. stage별 payload 의미·전체 join·외부 HTTP/worker provenance는 아직 이 예비 검사 범위가 아니다.

fmt·전체13개 시험·all-targets strict Clippy가 통과했다. 이 중 신규3개는 합성 accounting envelope의 변조 거부 시험이며 전체 실제 trace 성공이 아니다. `check_journal` 예제를 원래 R1 실제 journal에 실행한 결과 exit1/missing mutation_revision으로 거부했다. R1을 schema2로 읽기 위해 누락 필드를0으로 채우지 않았다. 기존 R1 complete=false/invalid2 바이트는 변경하지 않았다. 이 결과는 schema1→DTO 변환 완료가 아니다.

[소스 사본·Cargo·예제 EXE·시험/Clippy/거부 원문](../../../evidence/ai/boundary-observation/preflight-r1/)을 보존했다. 예제 EXE SHA256은 `b441da33c9eee2038768f86b7e58039d4f41ec9bbbebe43ffae2a5d8e45d74208`, 실제 거부 응답 SHA256은 `11d2b0fd2ecc8a7c9cd077a7aa961010071ad4369656957f0bfbd4f47442c4b5`다. source-at-audit이며 전체 전이 compiler 입력 증명이 아니다. 새 R2 actual/최종 gate와 외부 provenance 확인 후 필드별 DTO 변환·sidecar·pure verify 연결이 남아 있다. 예비 검사 성공도 dto_conversion_executed/shared_contract_verified/provenance_verified=false로 출력한다.

## 공통 Observation 검증 범위

검증기는 명시한 engine/model/3request identity, max_tokens2·seed42·temperature0·ignore_eos=true·truncatefalse·n_choices1·maxseq1·prefixcache0·stream/speculativeoff를 요구한다. complete와 invalid/dropped event수, 서로 다른 internal request ID와3case를 검사한다. rendered UTF8/BOS 실제분기 기록과 U32LE ID길이, accepted/rejected admission·HTTP/usage를 구분한다.

accepted2047은 동일 sequence에서 실제 forward 입력IDs가 prompt의 연속 구간과 일치해야 한다. CPU/pagedoff/packedoff/text-only, native32768/runtime2048,18개 layer의 actual RoPE positions가0..2046을빠짐없이 덮고 entry→layer→return→sampling→close순서를 만족해야 한다. 분할prefill은 허용한다. raw token vector의2047→2048, 생성1개, 구체 ModelLength 전/후/최종reason을 요구하며 외부length만으로 통과할 수 없다. rejected2048/2049는 sequence없음·forward/sampling없음과 **완결된case accounting**이 함께 필요하다.

## 실제 실행과 한계

fmt check,10개 synthetic 단위시험, strict all-targets clippy, all-targets cargo check가 exit0이다. 시험에는 누락accounting, GenerationBudget→ModelLength 혼동, 잘못된position/duplicate layer, alteredforwardIDs, 거부case실행, 중복request, BOS오류/dropped events, little-endian절단, 분할prefill누락, 품질budget/암묵truncate혼입, forwardreturn전sampling을 거부하는 회귀가 포함된다. Rust1.99 clippy의 chunks_exact_to_as_chunks 지적은 as_chunks::<4>()로 수정했으며 실패r1/r2로그도 보존한다.

이는 **검증기 구현과 합성시험**이다. supplied값의 계약 검사는 실제capture의진위나compiler provenance를 증명하지 않는다. 해당10시험 체크포인트에서는 엔진 hook이나 새 모델추론을 실행하지 않았다. 이후 [실제 trace](generation-gemma270-internal-trace-actual.md)를 수집했지만 invalid2/completefalse로 실패했으며 DTO 변환·off/on 대조는 남아 있다. 품질승인0개·실제역할23묶음은 바꾸지 않는다.

[10시험 stdout](../../../evidence/verification/boundary-observation-contract-2026-10-19-r1/boundary-observation-tests-r4.stdout) SHA256은 `1e753923b620d5fec41611c3f1953278a94c7711dcd006b4a084d84aefdf5f38`, 보존 Cargo.lock은 `8fa865f18e131dfe0c65a28de6d7492a9e63eca402ca8b22693340bd009ebcf3`다. 같은 폴더에 source-at-check와 [Clippy stderr](../../../evidence/verification/boundary-observation-contract-2026-10-19-r1/boundary-observation-clippy-r4.stderr)를 보존했다. 이는 검사 시점 source 사본이며 compiler 입력 전체 attestation이나 실제 엔진 EXE 보존으로 확대하지 않는다.

## 다음 연결

고정된 [연구 호출점](../../player/infrastructure/inference/research-followups/gemma270-internal-boundary-trace.md)의 recorder R2 최종 gate와 새 engine/worker 선보존·실제 재수집을 이어 간다. 그 뒤 raw plan/request/HTTP/worker controls·source/EXE/model provenance를 확인하고 DTO 변환과 sidecar를 보존한 경우에만 순수 검증기의 실제 결과를 기록한다. 이 라이브러리의 source/Cargo.lock도 새 변환 worker의 snapshot에 포함해야 한다. API 결과를 조합해 미캡처 필드를 채우지 않는다.
