---
id: evidence:kinekino/20261029113640
title: EmbeddingGemma 2 텍스트 파일 취득과 실제 Session 규격
status: active
created: 2026-10-29
updated: 2026-10-29
---
# EmbeddingGemma 2 텍스트 파일 취득과 실제 Session 규격

후속 [실제33입력 FP32 실행](embeddinggemma2-fp32.md)을 완료했다. 아래 추론 전·0회 문구는 취득/로드 단계의 범위로 보존한다.

2026-10-18. [Rust 실행 경계 연구](../../player/infrastructure/inference/research-followups/embeddinggemma2-rust-runtime.md)의 후보를 실제 취득·로드 단계로 진행했다. 고정 text export6개를 확보하고 ORT CPU Session에서 입력/출력 규격을 확인했다. **Session.run은 호출하지 않았고 새 신경망 추론0회다.** tokenizer 실행·검색 품질·양자화 비교·원본 framework parity는 아직 미검증이다. 기존20개 실행 모델 묶음이나 품질 승인0개를 늘리지 않는다.

## 고정 취득

[onnx-community/embeddinggemma-2-ONNX](https://huggingface.co/onnx-community/embeddinggemma-2-ONNX/tree/daa72c51243991dfcaf9f9137d2c573d8f7790c0) revision `daa72c51243991dfcaf9f9137d2c573d8f7790c0`를 HF API로 재확인했다. gated=false, card Apache-2.0 선언이며 원본 Google 가중치와 export/quantizer의 완전한 변환 계보 확인이나 재배포 승인과 구분한다. 기존 global catalog를 바꾸지 않고 별도 fixed entry를 생성했다. metadata SHA256은 `2ba73dd6aa0b55b9b265b0a45b9d2e1fe9a64716869fd07c67ecaf88b9085c46`다.

| 파일 | 실제 bytes | 실제 대조 SHA256 |
|---|---:|---|
| onnx/model.onnx | 421421 | `bc47de15f81208a5c99e5ab10f746d5e33b51ea228b7dc0bef9c133a94f1c1c3` |
| onnx/model.onnx_data | 1084170240 | `9fd452bfc6916e5a92f080f8dada35598ca4512b21c65897d4c5974d5f9630a2` |
| tokenizer.json | 32170510 | `4d777ef5bdc1aa36227abdfb77c3e49e7b9c892d16e1b6bda41c393504828be4` |

이3개는 LFS SHA와 실제 파일을 대조했다. 추가 README.md28720B·config.json5031B·tokenizer_config.json1599B는 Session 준비 단계에서 실제 Git blob SHA1을 각각 `ddfe751d31ecbfe6c1ba691a5cb270ab2b5de28a`, `b267dcb270ba41f67a8d14001bf470f23a9525fb`, `a9afd12fb3ca2c418fd405320ace85ad46748d03`과 대조했다. 작은 원문3개의 SHA256도 inspection 결과에 있다. vision/audio encoder 가중치는 취득하지 않았다.

`embeddinggemma2-text-fp32-r1`은 자원 검사 CPU48.59903381642512%·free RAM35514228736B에서 순차 취득 후exit0이었다. [취득 manifest](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/embeddinggemma2-text-fp32-r1/result.json)의 SHA256은 `6dac2b8644175fb2081066e609cc233f26c36abacc87b2056f80b7269f6a7b01`. 기본1GiB를 약10MB 넘는 data 파일은 정확한 repo/revision/name/SHA tuple에만1100000000B 상한을 허용했다. 다른 조합은 기본 상한으로 남으며 request timeout180초는 유지했다.

취득 전 EXE `2b152447ef2bb17b2468d7e329c67f6c1c58d302974b8a9edd2eacc2854ea57a`의 컴파일입력154개와EXE1개를 [보존](../../../evidence/ai/model-spike-builds/2b152447ef2bb17b2468d7e329c67f6c1c58d302974b8a9edd2eacc2854ea57a/build-manifest.json)했다. 모델 파일은 runtime cache에 두며 공개 Git이나 `.spec`에 가중치를 복제하지 않는다.

## 실제 CPU Session 규격

`embeddinggemma2-text-inspection-r1`은 자원 검사 CPU40.234375%·free RAM36121739264B에서 exit0으로 완료됐다. ORT Rust crate2.0.0-rc.14, CPU intra4/inter1, telemetry off로 실제 Session을 생성했다.

| 이름 | 실제 dtype | 실제 shape |
|---|---|---|
| input_ids | Int64 | [batch, sequence] |
| attention_mask | Int64 | [batch, sequence] |
| image_features | Float32 | [num_image_tokens,512] |
| video_features | Float32 | [num_video_tokens,512] |
| audio_features | Float32 | [num_audio_tokens,512] |
| last_hidden_state 출력 | Float32 | [batch,sequence,768] |
| sentence_embedding 출력 | Float32 | [batch,768] |

입력5개 이름·타입·rank와 feature 폭512를 코드로 검증했다. 출력은 metadata에서 실제 읽었으며 sentence head 존재를 확인했다. 입력 폭512와 출력768을 혼동하지 않는다. raw metadata에는 ORT 내부 주소도 포함되어 있으므로 debug 문자열의 byte 동일성을 수치 parity로 사용하지 않는다. FP32라는 export 이름과 입력/출력 타입 확인을 모든 initializer dtype/opset/operator inventory 확인으로 승격하지 않는다.

[검사 결과](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/embeddinggemma2-text-inspection-r1/result.json) SHA256 `c5a98d8bba3cc16026dc66c79f36d64e1419023aebd0f931936a074465fec288`. 기록된 Session load0.7930624초는 전체 명령·모델 추론 시간이나 성능 비교 결과가 아니다. 실행은 직접 준비 명령이며 BelowNormal·peak memory 표본·process-tree deadline으로 측정한 추론 worker 결과를 보유한 것으로 표현하지 않는다.

검사 EXE `e8799abbdfcb72e939b69a2f8748566dcdf5a1f82fc35965d2243b89f518c1a0`의 컴파일입력157개와EXE1개를 [선보존](../../../evidence/ai/model-spike-builds/e8799abbdfcb72e939b69a2f8748566dcdf5a1f82fc35965d2243b89f518c1a0/build-manifest.json)했다. 테스트47개·fmt check·clippy all-targets -D warnings·locked offline build를 통과했다. 초기 ORT builder의 비Send/Sync 오류 변환 컴파일 문제는 기존 어댑터 방식대로 오류 문자열 변환으로 수정했다. 실패한 build를 실행 성공에 포함하지 않았다. 보고서 snapshot1127개에 이번 취득·검사 결과를 포함했다.

## 다음 실제 추론

기존 실제 자막33입력과 query/document 구분을 유지하고 Gemma 전용 prefix·BOS/EOS·Rust tokenizer 결과·truncation을 고정한다. batch1/최대256 tokens와 text-only 빈 Float32[0,512] features3개로 실제 Session.run을 검증해야 한다. 이미 sentence embedding인 출력을 다시 token pooling하지 않는다. raw token 출력과 sentence head의 원본 수치 계약도 별도 확인한다.

FP32 실제 출력과 finite/shape/norm/rank를 먼저 관측한 뒤 같은 입력의 q4 또는q8 한 조건을 추가한다. MRL 차원 절단·재정규화와 가중치 양자화, 벡터 저장량과 상주 메모리는 별도다. 원본 framework oracle·변환 계보·인간 relevance·무관 질의 보정·GPU 및 노트북 전력 조건·제품 SDK job 연결은 남아 있다.
