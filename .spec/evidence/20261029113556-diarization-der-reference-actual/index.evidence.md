---
id: evidence:kinekino/20261029113556
title: 실제 원본 DER 평가기 교차 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 원본 DER 평가기 교차 검증

2026-10-18. root가 [준비된 Rust adapter](diarization-der-reference-preparation.md)의 frozen EXE `aed71a64a8524f68b34c6938f46d6942b59af3eaba8301dfc6162ed0896f2745`로 AMI60초의 기존/control 및 local-anchor 출력을 수정 없는 원본 md-eval-22.pl과 실제 대조했다. 새 모델 추론은0회이며 기존 hypothesis를 그대로 사용했다.

## 원문과 실행 경계

고정 dscore commit `e02f949ac6592279300a2c33d03daf9e0c12fd27`의 원본 script와 LICENSE를 root가 순차 취득했다. Rust adapter가 길이·Git blob SHA1·실제 SHA256을 확인한 후에만 실행했다.

| 파일 | SHA256 |
|---|---|
| md-eval-22.pl | 872aa955cbc3d57e6d4fd6fe2e699791739c1cb716cc89105bb4717415b9400d |
| LICENSE | 5df07460c4c14cf5937c097a3fd335d48a97c88e74abbd6c0c7f36231a0413a0 |
| 기존 Git Perl EXE | 8679db320f05c37d004c70ee5196d0bf8e9029aea49b9abc2f5cab8e9da810c5 |

실제 `--version`은 v5.42.2/x86_64-cygwin-thread-multi다. 원본 stdout을 request hash와 함께 기록했다. 새 Perl/Python 코드를 작성하거나 interpreter를 설치하지 않았다. 실행은 고정 Rust Command argv·collar0·overlap 포함·전체 UEM·60초 timeout·각 stream1MiB cap이며 전역 설정이나 사용자 앱을 변경하지 않았다.

첫 시도는 결과 부모 디렉터리가 없어 scorer spawn 전에 os error2로 종료했다. 부모를 명시적으로 만든 뒤 같은 요청으로 처음 결과 디렉터리를 생성해 실행했다. 이전 scorer 실행이나 실패 결과를 덮어쓴 것이 아니다. Cygwin은 실제 Windows forward-slash 경로를 열어4회 모두 exit0/timeout=false/cap=false/stderr empty로 종료했다.

## 실제 결과

| 결과 디렉터리 | 대응 hypothesis label | Rust DER % | 원본 인쇄 DER % | 네 component 차이 us | 판정 |
|---|---|---:|---:|---|---|
| nist-fp32-control-r1 | diarize-ami-fp32-control-r4 | 77.54037968441814 | 77.54 | 0/0/0/0 | 통과 |
| nist-int8-control-r1 | diarize-ami-int8-control-r2 | 77.88769723865877 | 77.89 | 0/0/0/0 | 통과 |
| nist-fp32-local-r1 | diarize-ami-fp32-local-r2 | 69.24772682445759 | 69.25 | 0/0/0/0 | 통과 |
| nist-int8-local-r1 | diarize-ami-int8-local-r1 | 67.97645463510848 | 67.98 | 0/0/0/0 | 통과 |

component는 scored/MISS/FA/CONF 순서다. 사전 고정 기준 DER차이≤0.0051 percentage point·각 component차이≤1us를 바꾸지 않았다. 모든 결과에서 실제 차이는0us다. 각 요청의 기존5202 보고서·WAV·RTTM·UEM·stdout 해시를 확인하고 Rust 재평가가 기존 보고서와 일치해야 원본 scorer를 실행한다. null/overlap을 reference에서 제외하거나 oracle 화자 수를 주입하지 않았다.

## 보존과 해석 범위

결과는 [.spec/evidence/ai/diarization-der-reference-runs](../../../evidence/ai/diarization-der-reference-runs/)의 각 디렉터리에 원문 script/LICENSE·Perl version·생성 RTTM/UEM·execution-plan·raw stdout/stderr·comparison.json을 보존한다. 모델 및 기존5202 출력은 변경하지 않았다. adapter verify-build도 root가 실행해 통과했다.

`external_der_parity=true`는 이4개 exact known-hyp/UEM/collar0/overlap 포함 비교에만 해당한다. `external_jer_parity=false`와 전체 `external_scorer_parity_verified=false`를 유지한다. 원본 JER·다른 corpus·모든 경계 사례·전체 도구 동일성 또는 모델 품질 승인은 아니다. [local-anchor 개선](diarization-local-anchor.md) 후에도 오류가 높으며 held-out·전처리·miss·창 간 clustering·overlap 대응 개선이 남아 있다.

후속으로 [60~120초 시간 분리2쌍](diarization-temporal-holdout.md)의 원본DER도 확인해 누적6개 exact 쌍이 통과했다. nonzero source crop offset을 실제 검증했으며 components차이는 모두0us다. FP32 입력의 [JSON1ULP 회귀](diarization-json-precision.md)를 고친 새136a5773 어댑터와 기존 aed71a64의 증거 범위를 구분한다. 이전4쌍 결과를 새 EXE 실행으로 소급 변경하지 않았다.
