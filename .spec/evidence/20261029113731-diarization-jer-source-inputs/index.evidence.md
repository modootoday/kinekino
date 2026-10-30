---
id: evidence:kinekino/20261029113731
title: JER 원문과 실제 출력 기반 평가 입력 확보
status: active
created: 2026-10-29
updated: 2026-10-29
---
# JER 원문과 실제 출력 기반 평가 입력 확보

2026-10-18. 고정 dscore 원문13개를 취득하고 기존 실제 모델 출력6개의 RTTM/UEM 입력을 새로 준비했다. 원음·reference·UEM·원시 hypothesis·frozen5202 보고서의 해시와 기존 Rust JER/frame count 재계산이 모두 통과했다. **외부 scorer 및 Python은 아직 실행하지 않았고 새 신경망 실행도0회다.** 독립 JER 일치·화자 품질 승인은 여전히 미완료다.

## 원문 취득

[nryant/dscore](https://github.com/nryant/dscore/tree/e02f949ac6592279300a2c33d03daf9e0c12fd27) commit `e02f949ac6592279300a2c33d03daf9e0c12fd27`의 LICENSE·requirements·score.py·scorelib 원문과 md-eval을 취득했다. 파일별 바이트·Git blob SHA1을 기존 `diarization-jer-reference::infrastructure::pins::SOURCES`와 대조하고 SHA256 inventory를 생성했다. 핀 목록을 새 도구에 복제하지 않았다. 기존 md-eval 디렉터리나 frozen 증거는 변경하지 않았다.

- 원문 폴더: `runtime/ai-spike/source/dscore-dscore-reference-r1`.
- source manifest: `runtime/ai-spike/results/dscore-reference-r1-source.json`, SHA256 `e2b653c6c209edc74a6babab5d1cdf6930ad8773eae6a80f4c6a79895dd5c226`.
- 수집 EXE: `fa79d9f79e8180d011c1a592c6b906bc5417df9d328bd93bfa18f01c1e9804d4`. 컴파일 입력147+EXE1=148개 bundle 선보존.
- admission CPU37.99%, 여유RAM34269503488바이트, BelowNormal 단일 worker, exit0/timeout=false, wall5.0788093초. 열린 player와 사용자 앱을 보존했다. 성능 비교가 아니다.

inventory의 `execution_authorized=false`는 원래 준비 도구가 원문 취득을 scorer 실행 증거로 승격하지 않는 고정 메타데이터다. 별도 사용자 승인 요청이나 이번 source 검증 실패를 뜻하지 않는다. source_verified와 실제 scorer 실행 provenance는 별개다.

## 실제6개 평가 입력

기존 공개 Rust `inputs::load`를 재사용해 각 원본 보고서와 실제 자료의 동일성을 검사하고 local 시간의 reference.rttm/hypothesis.rttm/scoring.uem을 신규 파일로 저장했다. unknown·overlap unknown 때문에 reference나 UEM을 제거하지 않았다. 첫4개는 같은 AMI 첫60초, 마지막2개는 같은 녹음의60~120초이며 독립 회의6개가 아니다.

| 실제 출력 label | source 시작 us | 기존 Rust JER % | scored frames |
|---|---:|---:|---:|
| diarize-ami-fp32-r1 | 0 | 77.78615375785219 | 6000 |
| diarize-ami-int8-r1 | 0 | 82.88452876260921 | 6000 |
| diarize-ami-fp32-local-r2 | 0 | 71.72540450264229 | 6000 |
| diarize-ami-int8-local-r1 | 0 | 75.88525753651477 | 6000 |
| diarize-holdout-fp32-pcm16-r1 | 60000000 | 52.10018262457605 | 6000 |
| diarize-holdout-int8-pcm16-r1 | 60000000 | 51.718722523920356 | 6000 |

위 표의 값은 기존 Rust 재계산이며 원본 dscore 값이 아니다. source offset은 reference crop에서 한 번만 적용했다. 원문 transcript나 hypothesis를 고쳐 수치를 맞추지 않았다. manifest에는 화자 이름별 frame counts와 원본 보고서/오디오/reference/UEM/stdout 해시 연결을 보존했다.

- fixture manifest: `runtime/ai-spike/results/dscore-jer-inputs-r1-fixtures.json`, SHA256 `ac82afa732eaac8c4df56a802313530a66601a358c9d4bad14efcab73b3c40c4`.
- 준비 EXE: `916b674cc9dd850b4a06f5f08771c56f0889248b65cb97326b8328df3543d391`, 컴파일 입력149+EXE1=150개 선보존.
- [원문 사본](../../../evidence/ai/model-spikes-2026-10-18/dscore-reference/source/)과 [6개 fixture](../../../evidence/ai/model-spikes-2026-10-18/dscore-reference/fixtures/)를 report가 SHA256 재검사 후 snapshot manifest에 포함했다.

root 도구가 기존 JER library를 직접 의존하게 되면서 serde_json의 float_roundtrip feature도 합쳐졌다. 이는 frozen5202의 binary64 재계산 검증에 필요하다. 새 실행의 Cargo.lock·build inputs에 남겼으며 과거 실행 소스나 출력에 소급하지 않았다. fmt check·test41·clippy all-targets -D warnings·locked offline build를 통과했다.

## 다음 실행 경계

후속 [런타임 배포물5개 확보](diarization-jer-runtime-archives.md)는 완료했다. 아래 미보유 항목 중 archive 기대/실제 hash 연결은 후속 근거로 보충되지만 추출 DLL·격리 실행·외부 함수 parity는 여전히 미완료다.

원문 `scorelib.score.turns_to_frames`, `metrics.contingency_matrix`, `metrics.jer`는 별도 함수다. 전체 CLI는 여전히 DER의 직접 Perl 실행을 포함한다. Windows에서 무수정 전체 CLI가 가능하다고 바꾸지 않았다. [Rust 함수 호출 경계 결정](../../../adrs/player/adr-jer-function-reference-boundary.md)에 따라 원문 함수를 Rust C-API 어댑터로 실제 호출하는 추가 검증을 준비하되, 전체 CLI/DER/parity 승인으로 대체하지 않는다.

필요한 실제 인터프리터·NumPy/SciPy·intervaltree/sortedcontainers 버전과 wheel/DLL 해시, 격리 import/실행·deadline·스트림 증거는 아직 없다. `external_jer_parity`, `external_der_parity`, `external_scorer_parity_verified`는 false를 유지한다. 기존 [준비 단계 한계](diarization-jer-reference-preparation.md)와 [남은 품질 게이트](../../../plans/player/ai-quality-gates.md)를 따른다.
