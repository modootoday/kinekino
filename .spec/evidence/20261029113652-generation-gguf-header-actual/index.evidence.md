---
id: evidence:kinekino/20261029113652
title: 경량 생성 모델 GGUF 실제 취득과 헤더 검사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 경량 생성 모델 GGUF 실제 취득과 헤더 검사

2026-10-19. Gemma270M·Llama1B의 고정 공개 GGUF를 실제 취득했고 기존 Qwen0.6B와 함께 Rust로 헤더를 검사했다. 파일명·repo·revision·bytes·SHA를 명시하며 첫 GGUF 선택이나 다른 모델 fallback은 없다. 이 문서는 파일·metadata 관측이며 추론 결과는 [Llama 실제 smoke](generation-llama1b-structured-smoke.md)와 [Gemma 준비 실패](generation-gemma270-loader-failure.md)로 분리한다.

## 고정 artifact와 접근 범위

| profile | repo와 revision | 파일 bytes | 실제 SHA256 |
|---|---|---:|---|
| gemma270-q4 | unsloth/gemma-3-270m-it-GGUF@c90975dbd40c0c7b275fefaae758c3415c906238 | 253115424 | b1baabd6b729e4041822220d3e648e00d99cac5df86b10dffb77bcccf0688e39 |
| llama1b-q4 | bartowski/Llama-3.2-1B-Instruct-GGUF@067b946cf014b7c697f3654f621d577a3e3afd1c | 807694464 | 6f85a640a97cf2bf5b8e764087b1e83da0fdb51d7c9fab7d0fece9385611df83 |
| qwen06-q4 | unsloth/Qwen3-0.6B-GGUF@50968a4468ef4233ed78cd7c3de230dd1d61a56b | 396705472 | ac2d97712095a558e31573f62f466a3f9d93990898b0ec79d7c974c1780d524a |

새 두 repo는 gated=false이며 metadata license는 각각 gemma·llama3.2였다. 취득 시 고정 HF metadata의 LFS size/SHA와 파일 bytes를 대조하고 README의 Git blob도 검사했다. 원본 gated weight·tokenizer 파일에 접근하거나 계정/수락 UI를 변경하지 않았다. 공개 변환본 접근을 원본 동의·완전한 변환 계보·재배포 승인으로 취급하지 않는다. [Gemma 약관](https://ai.google.dev/gemma/terms)과 [Llama 고정 원본 라이선스](https://huggingface.co/meta-llama/Llama-3.2-1B-Instruct/blob/9213176726f574b556790deb65791e0c5aa438b6/LICENSE.txt)의 제품 배포 검토는 별도다.

Gemma r1은 CPU90.2415%로 자원 admission에서 중단됐다. 파일 취득이나 모델 실행은0회였고 failure/preflight를 보존했다. r2는 CPU25%에서 취득 완료, Llama r1은20.6660%에서 완료했다. 순차 취득이며 사용자 앱·서비스·메모리 cache·VM·디스플레이를 변경하지 않았다.

## 실제 inventory

| profile | GGUF architecture | tensor | 저장 tensor 원소 합계 | 실제 tensor type별 개수 |
|---|---|---:|---:|---|
| Gemma270 | gemma3 | 236 | 268098176 | F32 109, Q4_K 27, Q5_0 81, Q6_K 9, Q8_0 10 |
| Llama1B | llama | 147 | 1235814432 | F32 34, Q4_K 96, Q6_K 17 |
| Qwen0.6B | qwen3 | 310 | 596049920 | F32 113, Q4_K 168, Q6_K 29 |

집계는 저장된 tensor shape의 곱이며 활성 파라미터·상주 메모리·alias나 논리 parameter count를 입증하지 않는다. Llama 원본의 앞선 BF16 inventory1235814400과32원소 차이도 임의로 제거하지 않았다. Gemma `token_embd.weight`는640×262144/Q8_0/167772160원소다. Q4_K_M 파일을 모든 weight가4bit인 것으로 표현하지 않는다.

Gemma BOS2/EOS106, add_bos=true/add_eos=false, vocab의 `<bos>`2·`<eos>`1·turn 시작105/끝106을 관측했다. Llama BOS128000/EOS128009이며 add_bos/add_eos key는 헤더에서 미상이다. Gemma 원본 context metadata32768·sliding512, Llama131072를 그대로 기록한다. runtime cap2048 적용 성공과 구별한다.

임베디드 chat template SHA256: Gemma `af95fbef33b76a50e5f463ff9766f85ce84f5849d915c4bd6c1619d852ac3231`, Llama `5816fce10444e03c2e9ee1ef8a4a1ea61ae7e69e438613f3b17b69d0426223a4`, Qwen `5da44855ab7e0641774d5ed0ff0b4f89671d01df7d316e54d06557a962211cb4`다. 원본 tokenizer/template byte parity·실제 rendered prompt/token IDs는 아직 확인하지 않았다.

## 검사 구현과 보존

Rust bounded reader는 [GGUF 공식 규격](https://github.com/ggml-org/ggml/blob/master/docs/gguf.md)의 little-endian v3 헤더·metadata type·tensor info를 읽는다. 헤더64MiB, string8MiB, tensor10000개, metadata2048개, array2000000개·전체 value4000000개·깊이8 미만으로 제한한다. 중복 key/name·비정상 bool·nonfinite·길이/곱 overflow·파일 밖 시작 offset을 거부한다. payload를 디코딩하거나 각 tensor의 마지막 byte extent/양자화 수치를 검사한 것은 아니다. 큰 metadata 배열은 원소 수와 decoded canonical JSON SHA로 보존하며 원본 직렬화 byte hash와 구별한다.

새 취득·검사 build `a33b20bbd156f827278a8a00dc181cab2561d769337999d0993f2a4902d87046`의197 compiler inputs+EXE1을 [선보존](../../../evidence/ai/model-spike-builds/a33b20bbd156f827278a8a00dc181cab2561d769337999d0993f2a4902d87046/build-manifest.json)했다.61tests·fmt check·clippy all-targets -D warnings·locked offline build를 통과했다. 실제 파일 전 단계에 truncated/과대 길이/파일 밖 offset/version2 거부 회귀를 실행했다.

- [Gemma 취득](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/generation-gemma270-acquisition-r2/result.json), SHA `ded61232e642535916cca19c12aaa28c52ec339ff84af006064445c384ac1f7d`; [첫 자원 차단](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/generation-gemma270-acquisition-r1/failure.json)
- [Llama 취득](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/generation-llama1b-acquisition-r1/result.json), SHA `81b1848261fb8e9be725839fa92f37a3b428feecdcb2d6e61fac651c3b666364`
- [Gemma 전체 헤더 관측](../../../evidence/ai/model-spikes-2026-10-18/results/generation-gemma270-header-r1-gguf-header.json), [Llama](../../../evidence/ai/model-spikes-2026-10-18/results/generation-llama1b-header-r1-gguf-header.json), [Qwen](../../../evidence/ai/model-spikes-2026-10-18/results/generation-qwen06-header-r1-gguf-header.json)

모델 가중치는 runtime 사설 cache에 두며 공개 Git·문서 증거에 복제하지 않는다. README는 동일 bytes를 model-card.txt로 보존하며 repo 내부 상대링크를 로컬 문서의 유효 링크로 오인하지 않는다. 변환 원본 commit·quantizer/calibration 전체 계보·다른 양자화·원본 framework parity·제품 배포 고지는 후속 게이트다.
