---
id: evidence:kinekino/20261029113548
title: 원본 JER 함수 실제 대조와 경계 계산 수정
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원본 JER 함수 실제 대조와 경계 계산 수정

후속 [교차 행렬78개 원소 대조](diarization-jer-matrix-actual.md)도 완료했다. 아래 행렬 미대조 문구는 이 최초 함수 대조 단계의 범위로 보존한다.

2026-10-18. 사설 CPython3.13.3·NumPy2.3.2·SciPy1.16.0에서 변경 없는 dscore 원본 함수를 Rust C-API 어댑터로 실제 호출했다. **기존 실제 출력6개에서 수정한 Rust JER/frame counts가 원본과 일치했다.** 이는 전체 dscore CLI 실행이나 화자 분리 모델 품질 승인과 다르다. 새 모델 추론0회이며 같은 AMI의 두 시간 구간으로 독립 작품6개가 아니다.

## 발견한 오류와 수정

최초 `jer-functions-actual-r1`의 앞4개는 reference FEE013 frame 수가 원본941, 기존 Rust940이었다. 원본 RTTM loader는 decimal onset과 duration을 각각 binary64로 읽은 뒤 더해 offset을 만들지만 기존 Rust는 정수 마이크로초 endpoint를 직접 binary64로 바꿨다. 원본 `0.010*k` grid의 searchsorted 경계에서 차이가 발생했다. UEM은 원래 explicit endpoint이며 duration 방식으로 바꾸지 않았다.

`diarization-evaluation::domain::metrics::evaluate`의 JER speaker framing을 RTTM onset+duration으로 수정했다. DER의 연속 시간 계산은 변경하지 않았다. 새 보고서 schema는 `diarization-evaluation/0.2.0`이며 sampling 설명도 분리했다. 과거 frozen5202 보고서의 재현에는 명시적 `evaluate_legacy_endpoint_grid`를 사용하고, reference 입력 검증은 기존 값의 exact binary64 재현과 새 corrected 값을 함께 보유한다. 과거 raw/report/표를 새 값으로 덮어쓰지 않았다.

## 같은 입력 실제 수치

| 고정 출력 label | 과거 Rust JER% | 원본 및 수정 Rust JER% |
|---|---:|---:|
| diarize-ami-fp32-r1 | 77.78615375785219 | 77.79852854181782 |
| diarize-ami-int8-r1 | 82.88452876260921 | 82.89690354657483 |
| diarize-ami-fp32-local-r2 | 71.72540450264229 | 71.74159650441752 |
| diarize-ami-int8-local-r1 | 75.88525753651477 | 75.9015157970689 |
| diarize-holdout-fp32-pcm16-r1 | 52.10018262457605 | 52.10018262457605 |
| diarize-holdout-int8-pcm16-r1 | 51.718722523920356 | 51.71872252392035 |

수정한 Rust와의 최대 차이는1.4210854715202004e-14 percentage points다. 사전 허용치0.00000051을 변경하지 않았다. 각각6000 scored frames이며 reference/hypothesis의 모든 frame counts가 일치했다. 교차 행렬도 원본 출력으로 보존했지만 Rust 교차 행렬의 모든 원소를 별도 대조한 증거는 아직 없다. 위 표 local INT8의 Rust 값은75.90151579706888이며 표시 반올림으로만 일치한 것으로 해석하지 않는다.

## 실행과 격리 관측

Rust에서 원문의 `load_rttm/load_uem/turns_to_frames/contingency_matrix/jer`를 호출했다. 새 Python source·eval·wrapper는 작성하지 않았다. 원문13개, runtime2500파일, fixture manifest와 각RTTM/UEM, 원래 audio/reference/hypothesis 해시를 재검증했다.

실제 flags는 isolated=1, no_site=1, no_user_site=1, ignore_environment=1, dont_write_bytecode=1, safe_path=true였다. sys.path는 사설 표준 library ZIP·Python·site-packages 및 고정 dscore source의4개 경로만 canonical 대조했다. sitecustomize/usercustomize가 import되지 않았고 주요6개 module의 실제 경로/hash를 관측했다. 모든 native DLL의 실제 적재 목록을 수집한 결과와는 구분한다.

실제 CPython DLL은 사설 runtime 안에서 적재됐고 SHA256은 `69fd86ea29370697c203f7e12830084f920f490766a8e3045af52c036a9ad529`다. build에서 `PYO3_CONFIG_FILE`로 기존 Python 실행 probe를 피하고 host의 python313.lib만 link import library로 사용했다. host interpreter는 실행하지 않았고 PATH 변경은 실행 shell/child에만 한정했다. 설정 파일은 로컬 ABI 정보로 git 공개 대상에서 제외했다.

## 재현 증거와 범위

주 실행은 `jer-functions-corrected-r3`다. EXE SHA256 `db4e79640d4f290e3aafd5aaddbd6e0a0817562cdeafe9bdfc46437b984b7d5f`이며 [보존 build](../../../evidence/player/jer-function-reference-builds/db4e79640d4f290e3aafd5aaddbd6e0a0817562cdeafe9bdfc46437b984b7d5f/build-manifest.json)에 컴파일 source48개와 EXE를 보존했다. test source 및 모든 transitive/native 의존성의 완전한 attestation은 아니다.

[원시 출력](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/jer-functions-corrected-r3/worker.stdout) SHA256은 `a1b0dd176aa6c0581b9ef4a32ff8f4639cb580aee9246ce0cc58040981009f44`다. [감시 결과](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/jer-functions-corrected-r3/observation.json)는 exit0,37.7412439초, timeout=false, stdout/stderr 상한 초과false였다. BelowNormal·4threads·사설 kill-on-close Job·60초 기한·각stream1MiB를 설정하고 Job 할당 전 worker는 stdin gate에서 기다린다. 이번 실행에서 timeout 고장 주입이나 peak memory 실측을 한 결과는 아니다.

`jer-functions-import-r1`은 실제 격리/import만 수행했다. [최초 불일치](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/jer-functions-actual-r1/worker.stdout)는 기존JER 비교, [수정r2](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/jer-functions-corrected-r2/worker.stdout)와r3는 같은6case 재검증이다. 이를 새 모델 추론이나 독립corpus로 합산하지 않는다.

## 검사와 남은 작업

평가기20tests, reference준비4tests, C-API실행기1test, root spike46tests를 통과했다. 각 변경 crate의fmt check와clippy all-targets -D warnings를 수행했다. CRC/stream cap 및 합성 boundary 검사를 실제 모델 품질 증거로 승격하지 않는다.

`external_jer_function_parity_verified=true`는 이6개 출력의 추가 함수 경로만 뜻한다. full_dscore_cli_parity_verified, 기존external_jer_parity, model_quality_approved는false다. Windows 전체CLI의Perl 직접 실행 문제, Rust 교차 행렬 모든 원소 대조, 별도입력/overlap/경계corpus, host IPC·제품AI job 통합은 남아 있다. [잔여 품질 게이트](../../../plans/player/ai-quality-gates.md)를 따른다.
