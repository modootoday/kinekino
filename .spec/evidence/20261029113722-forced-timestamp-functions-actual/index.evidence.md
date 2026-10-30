---
id: evidence:kinekino/20261029113722
title: 원본 타임스탬프 보정 함수 실제 대조
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원본 타임스탬프 보정 함수 실제 대조

2026-10-19. [같은 mel head 대조](forced-head-logits-actual.md)의 저장된 raw class ID를 원본 Torch의 시간 변환과 원본 `Qwen3ForceAlignProcessor.fix_timestamp`·`parse_timestamp`에 전달했다. 한국어1개·일본어2개의 raw F32 시간156개, 보정 정수 ms156개와 최종78단어의 text·F64 시간이 Rust 결과와 모두 일치했다. 일본어 길이0 단어5개도 원본에서 동일하게 재현됐다. 수치 이식의 일치이며 품질 문제 해결이 아니다.

## 실행 계약과 직접 확인 범위

고정된 원본 head 결과3개와 native 결과3개의 SHA256을 먼저 대조하고 expanded IDs 및 선택 클래스가 같은지 확인했다. 기존 사설 Torch2.9.1+cpu·NumPy2.3.2·Transformers4.57.6 및 원본 Qwen package18파일의 전체 manifest 검사를 재사용했다. 새 모델 다운로드·가중치 로드·신경망 forward는0회다.

원본 config SHA256 `d616c65d46c4b90bdc651b0a0963ea932732241140f337f9bb6b0335a9c8ef09`의 `timestamp_segment_time=80.0`을 사용했다. 실제 Torch int64 class tensor×Python float→CPU NumPy F32를 실행하고 little-endian bytes를 확인했다. 실제 processor 인스턴스에서 두 원본 함수를 호출했다. 단위 변환은 표준 `operator.truediv`와 `builtins.round(...,3)`, 출력은 원본 `ForcedAlignItem`·`ForcedAlignResult` 생성자를 Rust/PyO3로 호출했다.

전체 `align()`과 `_to_structured_items()` wrapper는 호출하지 않았다. 시간 연산과 원본 dataclass 생성자를 직접 대조한 범위이며 전체 processor/tokenizer 재생성·일반화된 framework parity를 주장하지 않는다. Python 소스·모델 구현·보정 정책은 수정하지 않았다. domain은 입출력 수치·bits와 길이0/역전 관측만 담당하며 OS·Python 호출은 infrastructure로 분리했다.

## 실제 결과

| 입력 | 단어 | raw F32 | fixed ms | 최종 text·F64 | 길이0 단어 인덱스 |
|---|---:|---|---|---|---|
| KO1883 | 19 | 38/38 일치 | 38/38 일치 | 19/19 일치 | 없음 |
| JA1516 | 28 | 56/56 일치 | 56/56 일치 | 28/28 일치 | 9,10 |
| JA1634 | 31 | 62/62 일치 | 62/62 일치 | 31/31 일치 | 2,5,7 |

인덱스는0부터 시작한다. 최종 시간이 역전된 단어는 세 입력 모두0개다. 아래는 원본과 native 모두에서 관측한 값이다.

| 입력·인덱스 | 단어 | raw 시작→끝 ms | 보정 시작→끝 ms |
|---|---|---|---|
| JA1516·9 | を | 7840→7840 | 7840→7840 |
| JA1516·10 | もっ | 8000→8000 | 8000→8000 |
| JA1634·2 | は | 2240→2240 | 2240→2240 |
| JA1634·5 | 0 | 2720→2800 | 2800→2800 |
| JA1634·7 | 0 | 3040→3040 | 3040→3040 |

4개는 raw에서 이미 동일한 시작/끝 클래스를 선택했고 JA1634·5는 원본 보정 단계에서 collapse됐다. 최소 길이 강제·runner-up 선택·단어 삭제로 통과시키지 않았다. 이 관측은 세 고정 입력에 한정되며 독립 사람 단어 경계는 미확보다. 과거 정렬17회·구조3통과/14실패 이력은 변경하지 않는다.

## 실패 보존과 수정

첫 실행 `timestamp-original-paired-r1`은 exit1/timeout=false다. Rust 어댑터가 정수 `__truediv__`를 직접 호출해 Python의 정상적인 반사 연산자 dispatch를 거치지 못했고 `NotImplementedType`을 round하는 오류가 발생했다. 원본 함수 문제나 수치 불일치로 분류하지 않는다. 표준 `operator.truediv`로 수정한 새 r2에서 세 입력을 완료했다. 실패 소스·EXE·stderr·종료 관측은 r1 그대로 보존했다.

참조 도구7tests·fmt check·clippy all-targets -D warnings·locked offline build를 통과했다. 회귀는 일치하는 길이0 결과를 관측으로 남기고, 최종 F64 bits 변조를 거부하며 fixed ms 차이를 검출한다. 같은 EXE의 기존 기본 JER6개 회귀도 exit0이며 stdout SHA256 `8b516b1ce73187162497b011789a0ee9e4b05b814b06485e7dc766e541b80fba`로 기존 결과와 일치했다.

중앙 검사기에 SDK head 참조 빌드115 compiler inputs·패치7개·입력6개·config·EXE1을 연결했다. 검사기8tests·fmt·strict clippy·release build와 실제 검사를 통과했다. 원본 함수 참조16빌드의 listed source932개·EXE16개도 검증한다. [감사 시점 소스·검사 EXE와 보고서 보존](../../../evidence/verification/timestamp-integrity-2026-10-19-r1/README.md)은 전체 native/transitive attestation과 구별한다.

## 재현 증거와 자원

- [원본 함수·비교 결과](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/timestamp-original-paired-r2/timestamp-result.json), SHA256 `c87084d4fbec0cb4748cfeb8ca2a6112d8d7fbc5f5a447026573e306abdde5c7`
- [감독 관측](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/timestamp-original-paired-r2/observation.json), [원본 stderr](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/timestamp-original-paired-r2/worker.stderr), [실패 r1](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/timestamp-original-paired-r1/observation.json)
- [실행 EXE7e6624…의72 compiler inputs+EXE1](../../../evidence/player/jer-function-reference-builds/7e6624c3a8ac792f714305dc0cfd63cfd39c1ed02e866bed5da30e62fe710607/build-manifest.json), manifest SHA256 `69f1c4ec84791afdf4bc3bff804b4c4968fe0c7f00ba5f584a2097b78a7cd638`
- [기본 JER 회귀](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/timestamp-jer-regression-r1/observation.json)

r2 wall29.895526초, 표본 peak private793063424B·working620191744B,20ms 표본1472개/query failure0, exit0/timeout=false다. import·해시·출력 포함이며 추론 latency가 아니다. 자원 preflight·BelowNormal·private kill-on-close Job·300초 기한을 적용했고 사용자 앱을 종료하지 않았다. 기존 원본 nagisa SyntaxWarning은 숨기거나 수정하지 않았다.

## 다음 게이트

사람 word-boundary reference와 실제 오디오 대조, 언어/다른 작품/길이 확장, 길이0·불확실 정렬에 대한 제품 실패 계약이 남는다. 현재 모델 품질 승인0개·제품 AI capability unsupported 상태를 유지한다.
