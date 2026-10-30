---
id: evidence:kinekino/20261029113648
title: 강제 정렬 토크나이저 정규식 입력 계약 실제 대조
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 강제 정렬 토크나이저 정규식 입력 계약 실제 대조

2026-10-18. [계약 감사](../../player/infrastructure/inference/research-followups/forced-alignment-contract-audit.md)의 확정 String/Regex 불일치를 별도 Rust 진단에서 비교했다. **기존 JA1516·JA1634·KO prompt의 전체 토큰 ID는 세 건 모두 동일했다.** 따라서 이 변경을 기존 0길이 문제의 해결로 표시하지 않고 신경망 재실행을 생략했다. 실제 추론 횟수10·구조3통과/7실패는 변하지 않는다.

## 변경 경계

`tools/tokenizer-contract`는 standalone Rust 도구다. application은 보고서 입력·대조를 조정하고, domain은 위치·길이 계산, infrastructure는 고정 tokenizers0.21.4와 원본 tokenizer 생성 어댑터를 담당한다. 제품 SDK·UI·기존 관측 엔진이나 frozen source는 변경하지 않았다.

A는 frozen719 tokenizer 소스의 원래 BPE 생성자를 사용한다. B는 생성된 A 직렬화에서 첫 Split의 pattern 객체를 `String`에서 같은 문자열의 `Regex`로 한 곳만 바꾸고 재로딩한다. vocab·merges·added token·normalizer·ByteLevel·사전·단어 분절·prompt는 동일하다. 양쪽 전체 serialization을 보존했다. 변경은 이 진단 후보 안에서 적용됐으며 기존 엔진 fork의 수정·재빌드 완료로 표현하지 않는다.

로컬 upstream 사본은 원본 SHA256 `993480389256cd82e1d57ad156b832376f5c81c310e9cb866a1ece3d55b2b9f4`에 diagnostic accessor와 rustfmt만 추가했다. 재배포 라이선스 확인 전까지 해당 사본을 `.gitignore`로 공개 표면에서 제외했다. 네트워크 로더는 보존된 원본 모듈에 있으나 실제 진단 진입점은 로컬 BPE 파일만 연다. 새 다운로드·모델 가중치 로드·신경망 실행은0회다.

## 실제 입력 결과

| 입력 | A/B 전체 ID | prompt ID 수 | 확장 ID 수 | timestamp 수 | 파생 audio pad 수 | 과거 raw timestamp 위치 대조 |
|---|---|---:|---:|---:|---:|---|
| JA1516 | 동일 | 104 | 321 | 56 | 218 | 56개 위치 모두 일치 |
| JA1634 | 동일 | 100 | 257 | 62 | 158 | 62개 위치 모두 일치 |
| KO baseline | 동일 | 88 | 249 | 38 | 162 | 이전 보고서에 capture 없음 |

기존 저장 `generated_prompt`를 그대로 읽었고 파일 SHA256을 기록했다. 16k mono와 bounded sample count를 확인한 뒤 기존 hop160 frame 수와 encoder output length 공식으로 pad를 파생했다. mel 실제 tensor·encoder 실행을 새로 수행한 것은 아니다. batch1 expanded IDs·all-ones mask·timestamp 위치를 보존했고, raw capture가 있는 일본어 두 입력은 A의 확장 위치가 일치하지 않으면 보고서 출력을 중단하도록 했다. 이 확인은 전체 원본 processor tensor parity가 아니다.

합성 `7000`, `A7000 日本123`, `don't 1234\n日本` 세 문자열도 기록했다. String/Regex pretokenizer 분할은 달라지지만 이 세 문자열의 최종 BPE ID도 동일했다. 합성 문자열은 모델 정확도 정답이나 실제 발화 표본에 넣지 않는다.

## 실행 파일과 결과 보존

실행 전에 컴파일 당시 포함된 소스8개를 현재 소스와 byte 대조한 후 새 디렉터리에 EXE·소스·manifest를 보존했다. 관측 결과의 EXE SHA256과 frozen EXE를 별도로 확인했다.

| 항목 | 경로 또는 SHA256 |
|---|---|
| frozen build | [regex-contract-r1](../../../evidence/player/tokenizer-contract-builds/regex-contract-r1/) |
| EXE SHA256 | `0d9a941c93d39d7010815b80b0ea87e152a0665430fb9b33957c811d9e5ad7ed` |
| manifest SHA256 | `c6ab97869e7a096fc1e7577aa45815b7ebb8b42275138f7868dfe5fa012e9c56` |
| 원본 결과·입력 보고서 사본 | [보존 디렉터리](../../../evidence/ai/forced-tokenizer-contract-2026-10-18/) |
| 결과 SHA256 | `29aba0e6f978f003569e0aeec172176108cc492316ae2c64a9f5d116d46e1800` |
| 결과 크기 | 26,002,107 bytes; 전체 A/B tokenizer serialization 포함 |

`cargo fmt --check`, `cargo test --locked --offline`의3tests, `cargo clippy --locked --offline --all-targets -- -D warnings`, locked offline build를 이 도구 manifest에 실행해 통과했다. MSVC link의 기존 native `LNK4098` 경고는 test/build에서 관측했으며 제거 완료로 표시하지 않는다. central 문서 검사기가 이 새 frozen bundle의 소스 무결성을 자동 검증한다는 주장은 하지 않는다.

## 남은 정렬 게이트

이 불일치는 소스 계약 결함이지만 현재 세 prompt의 토큰 차이를 만들지 않았다. 기존 raw equal4건과 repair collapse1건의 원인은 이 결과로 해소되지 않는다. 다음 검증은 공식 분절과 동일 입력 비교, mel 수치 및 실제 head top2/margin 관측, 사람이 검토한 word 경계다. 단어 길이를 강제로 늘리거나 zero를 삭제해 구조 검사를 통과시키지 않는다. 다른 prompt에서 ID 차이가 관측되면 별도 frozen engine fork와 단일 실제 입력 A/B가 필요하다.
