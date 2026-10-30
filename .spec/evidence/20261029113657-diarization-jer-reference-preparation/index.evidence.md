---
id: evidence:kinekino/20261029113657
title: dscore JER 외부 교차 검증 준비
status: active
created: 2026-10-29
updated: 2026-10-29
---
# dscore JER 외부 교차 검증 준비

관측 기준: 2026-10-18 16:12 KST. 범위는 독립 Rust 준비 도구와 합성 검증이다. Python·scorer·모델 실행, 원문 저장/취득, 설치는 수행하지 않았다. `external_jer_parity`, `external_der_parity`, `external_scorer_parity_verified`는 모두 false다. 실제 실행 준비가 끝났다는 의미가 아니다.

## 원문 계약과 발견

원문은 [nryant/dscore 고정 commit](https://github.com/nryant/dscore/tree/e02f949ac6592279300a2c33d03daf9e0c12fd27)이다. [score.py](https://github.com/nryant/dscore/blob/e02f949ac6592279300a2c33d03daf9e0c12fd27/score.py)의 출력은 DER/JER 등을 포함하는 12열 표다. 명시적으로 `--n_digits 6 --table_fmt tsv --step 0.010 --collar 0`을 계획한다. overlap 제외 플래그는 사용하지 않는다. 원문의 UEM 옵션 이름은 `-u,--uem` 한 문자열이므로 계획도 이를 그대로 사용한다.

`--jer_min_ref_dur`에는 type 변환이 없다. 따라서 이 인자를 문자열 `0`으로 넘기지 않고 원문의 숫자 기본값 0.0을 사용한다. 계약에는 min_ref_dur=0을 명시한다. `--step`에는 type=float이 있으므로 0.010을 전달한다. 표 출력 반올림에 대한 허용치는 결과 관측 전에 **0.00000051 percentage points**로 정했다. 실제 grid/구간 변환 차이는 이 허용치를 넓혀 덮지 않는다.

[scorelib/score.py](https://github.com/nryant/dscore/blob/e02f949ac6592279300a2c33d03daf9e0c12fd27/scorelib/score.py)는 JER만 원해도 DER를 호출한다. [metrics.py](https://github.com/nryant/dscore/blob/e02f949ac6592279300a2c33d03daf9e0c12fd27/scorelib/metrics.py)는 `md-eval-22.pl`을 subprocess의 프로그램으로 직접 실행한다. native Windows Python에서 Perl EXE를 지정하는 별도 공식 인자가 없다. 따라서 현재 host에서 무수정 원문이 동작한다고 판정하지 않는다. 원문을 수정하거나 shim을 추가하지 않았다.

## Host 읽기 전용 관측

| 항목 | 관측 | 의미 |
| --- | --- | --- |
| Python | `%USERPROFILE%/AppData/Local/Programs/Python/Python313/python.exe`, 105832 B, PE ProductVersion/FileVersion 3.13.3 | 실행한 버전 stdout이 아니다 |
| Python EXE SHA256 | `d87063e5597f257004c731b66c59c56c91038861c6877b1a3dca6b8c4e919125` | 파일 바이트 식별만 확인 |
| NumPy | METADATA 2.3.2, Requires-Python >=3.11, WHEEL cp313-cp313-win_amd64 | import/수치 동작 미검증 |
| SciPy | METADATA 1.16.0, Requires-Python >=3.11, numpy>=1.25.2,<2.6, 동일 Windows cp313 tag | NumPy와 선언 범위는 맞음; 실제 호환성 미검증 |
| tabulate | METADATA 0.10.0, Requires-Python >=3.10 | 실제 출력 미검증 |
| intervaltree / sortedcontainers | package 및 해당 dist-info 검색 결과 없음 | 필수 의존성 충족 안 됨 |
| site startup | editable/minago/distutils/google/pytest-cov/pywin32 등의 .pth 존재 | 기본 site startup을 통한 코드 실행을 피할 격리 검토 필요 |

[원문 requirements](https://github.com/nryant/dscore/blob/e02f949ac6592279300a2c33d03daf9e0c12fd27/requirements.txt)는 intervaltree>=3.0.0, numpy>=1.16.2, scipy>=0.17.0, tabulate>=0.5.0이다. 최소 범위일 뿐 재현 가능한 lock이 아니다. 호환 환경 선택·실제 버전 증거·누락 의존성 설치 및 정확한 transitive lock 결정은 root에 남긴다. Windows wheel을 POSIX 환경에서 재사용 가능하다고 간주하지 않는다. `-S`와 자식 프로세스에만 적용하는 명시적 dependency search path는 후보 계획이며, import 격리와 원문 동작 검증 전 활성화하지 않는다.

## Rust 준비 도구의 책임

[crate](../../../../tools/diarization-jer-reference/README.md)는 입력·원문·표 출력·비교·보존을 분리한다. frozen5202 report SHA 및 audio/reference/UEM/hypothesis stdout SHA를 다시 검사하고 JER와 frame counts를 동일 Rust API로 재계산한다. serde_json float_roundtrip을 사용하며 기존 DER crate는 수정하지 않는다. source_start_us=60000000 합성 사례에서 reference/UEM을 한 번만 local 시간으로 옮긴다. unknown/overlap unknown은 reference나 UEM을 제외하는 근거로 쓰지 않는다.

원문 13개 파일의 크기와 Git blob SHA1은 `tools/diarization-jer-reference/src/infrastructure/pins.rs`에 고정한다. root 취득 후 `verify-source`가 SHA256 inventory도 생성한다. 단, 임의 추가 import 파일이나 runtime/dependency integrity를 인증하는 기능은 아니다.

`prepare`는 새 디렉터리에 RTTM/UEM·해시·계약·실행 차단 사유·고정 인자 계획을 쓴다. `inspect-output`은 root가 별도로 얻은 bounded stdout/stderr를 보존하고 오류도 기록한다. 12열 TSV, 파일/overall 일치, 6자리 JER 및 원문의 8개 stderr 진행 줄만 허용한다. 임의 stdout은 실행 provenance를 인증할 수 없으므로 수치 일치만 `numeric_jer_parity`에 기록한다.

Python/scorer spawn 자체가 없는 준비 빌드다. 60 s process-tree timeout과 각 stream 1 MiB는 이후 실행 어댑터의 요구 조건이며, 이 빌드가 timeout을 집행한다고 주장하지 않는다. root가 호환 POSIX runtime과 원문/의존성 증거를 확보하고 별도의 bounded process adapter를 검토하기 전 실제 JER parity는 미완료다.

## 검증 및 보존

검증 결과와 최종 EXE/manifest 해시는 [coordination](../../../../runtime/coordination/diarization-jer-reference.md)에 기록한다. 합성 테스트는 실제 frozen5202 실행 증거가 아니다. source·EXE snapshot은 별도 신규 디렉터리에 보존하며 실제 scorer source는 포함하지 않는다.

## root 인계 확인

root가 frozen EXE `123ddcc6771777d4e5174a197f96a9f6e002086a96355bdfb070dc6eba86dbf2`의 verify-build를 실제 실행해 exit0을 확인했다. native SHA 관측도 EXE hash와 manifest `dbf37fe665a3ce585769a6685e3853581ca8076d7ddb9c05c52a3871bebed6e7`가 인계와 일치했다. CLI source의 verify-build 분기가 snapshot verifier를 호출함을 확인했다. 이 검사에서 Python/scorer를 실행하지 않았고 실행 환경/원문 JER parity가 준비됐다고 승격하지 않았다. root의 현재 model-spike·OCR 실제 추론과는 별도 도구다.
