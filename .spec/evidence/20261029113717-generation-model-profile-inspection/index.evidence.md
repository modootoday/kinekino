---
id: evidence:kinekino/20261029113717
title: 경량 생성 모델의 명시적 profile·artifact 검사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 경량 생성 모델의 명시적 profile·artifact 검사

2026-10-18. [후속 연구](../../player/infrastructure/inference/research-followups/lightweight-generation-spike-readiness.md)의 고정 후보를 Rust domain profile로 표현하고 기존 로컬 파일의 크기·SHA와 설치 engine SHA를 실제 재검증했다. **이 변경은 inspector이며 structured_text의 모델 선택 실행 경로 이관·새 모델 load/run은 아직 미완료다.**

## 구현과 실제 검사

`inspect-generation-model PROFILE`은 balanced2b-q4, qwen06-q4, gemma270-q4, llama1b-q4만 받는다. domain은 repo/revision/정확한 파일명/bytes/SHA를 소유하고 infrastructure는 명시한 사설 cache 경로·실제 파일 크기·SHA·engine identity를 확인한다. 첫 GGUF 선택이나 알 수 없는 profile의 기본값 fallback은 없다. 파일을 취득하거나 engine을 실행하지 않는다.

| 실제 요청 | 결과 | 모델 bytes |
|---|---|---:|
| qwen06-q4 | 기존 artifact·설치 engine SHA 일치 | 396705472 |
| balanced2b-q4 | 기존 artifact·설치 engine SHA 일치 | 1280835840 |
| gemma270-q4 | 사설 cache 디렉터리 부재로 시작 전 오류 | 미취득 |

Llama profile의 실제 파일 검사는 실행하지 않았다. Gemma/Llama의 공개 변환 artifact와 원본 접근·약관·완전 변환 계보·배포 검토를 동일한 승인으로 취급하지 않는다. inspector 성공은 GGUF header/template/tensor inventory, architecture load, JSON/의미 품질 검증이 아니다.

Qwen0.6 profile은 기존 HF snapshot symlink를 canonicalize해 같은 사설 hub의 blob으로 해석됐다. 두 성공 모두 모델 cache 내부 실제 bytes를 검사했으며 설치 mistral.rs0.9.4 EXE SHA `59ee8ff009a06cee8c39890cf6fd90d74f4617e46e88ac9b46a28a11538a9d73`도 일치했다. native 실행·tokenizer/template parity·변환 계보·품질은 모두false로 기록했다.

## 보존과 후속

- [Qwen0.6 실제 inspector 출력](../../../evidence/ai/model-spikes-2026-10-18/results/generation-model-qwen06-r1.json), SHA256 `1dacac460344b3579b33858ade7c6331cf5fc8c9e747639b96a3ea3a5e79f534`
- [기존2B 실제 inspector 출력](../../../evidence/ai/model-spikes-2026-10-18/results/generation-model-balanced2b-r1.json), SHA256 `a8b2090d5e135b31e73ac2d5ee95b867fd146f9e48cc39e078e376605a343c85`
- [Gemma 시작 전 오류](../../../evidence/ai/model-spikes-2026-10-18/results/generation-model-gemma270-r1.stderr): OS error3, 모델 다운로드·대체 실행 없음

실행 EXE `7c28f25e0f0a6b8374253cefdbdcf1d9a9832908b47003bb7a141ec90bf77bf4`의 컴파일 입력173개·EXE1개를 [선보존](../../../evidence/ai/model-spike-builds/7c28f25e0f0a6b8374253cefdbdcf1d9a9832908b47003bb7a141ec90bf77bf4/build-manifest.json)했다. 해당 단계50tests·fmt check·clippy all-targets -D warnings·locked offline build가 통과했다. 후속 정렬 admission 추가 build와 이 실제 검사 build를 구별한다. 새 신경망 실행0회다.

다음은 이 고정 profile을 LocalTextServer 유스케이스에 연결하고 동일 prompt/schema·context2048/max256/temperature0 조건의 최소 smoke를 수행하는 것이다. 기존2B18요청·제약9/9를 새 후보3요청과 같은 표본으로 표현하지 않는다. 원본/embedded template와 실제 rendered prompt를 확인하고 구조·원문 보존·누락/환각·자원·거절을 구분한다. 정렬 A/B root 단일 worker가 끝난 뒤 생성 worker를 실행한다.
