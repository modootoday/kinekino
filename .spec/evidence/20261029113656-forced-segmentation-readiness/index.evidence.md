---
id: evidence:kinekino/20261029113656
title: 공식 강제 정렬 분절 참조 환경 조사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공식 강제 정렬 분절 참조 환경 조사

2026-10-18. [실제 NumPy mel 대조](forced-mel-numpy-actual.md) 다음으로 공식 분절의 원본과 Windows 배포물을 조사했다. **현재는 원문·메타데이터 취득이며 nagisa/soynlp 설치·import·분절 실행은 하지 않았다.** 공식 분절과 Rust 분절의 동일성은 미검증이다.

## 공식 원본 계약

Qwen3-ASR commit `7c6daf77a2421100f5fb066495372c00129d39ff`의 [원본 processor](https://github.com/QwenLM/Qwen3-ASR/blob/7c6daf77a2421100f5fb066495372c00129d39ff/qwen_asr/inference/qwen3_forced_aligner.py)를 Rust HTTP adapter로 그대로 보존했다.16411B/SHA256 `fb586127507c5980d11c6d848b530e652afad3ae0707cba6eb0c90377b2a2111`이다. [보존 사본](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/segmentation-runtime-metadata-r2/qwen3_forced_aligner.py)은 변경하거나 실행하지 않았다.

일본어는 `nagisa.tagging(text).words`, 한국어는 공식 사전 단어마다 score1.0을 부여한 `soynlp.tokenizer.LTokenizer`를 사용한다. 이후 Unicode L/N 범주와 ASCII apostrophe를 남기고 빈 token을 제거해 timestamp prompt를 만든다. 현재 Rust의 Lindera/IPADIC·merge 방식은 별도 구현이며 공식 nagisa를 실행했다는 뜻이 아니다. 한국어 사전의 실제 바이트와 생성 규칙도 함께 대조해야 한다. 이 원본 모듈은 최상위에서 Torch와 Qwen backend도 import하므로 standalone tokenizer 함수 실행과 원본 processor 전체 import는 서로 다른 게이트다. 원문을 잘라 실행하거나 모듈 stub으로 통과시키지 않는다.

## 실제 PyPI 관측

공식 [nagisa 메타데이터](https://pypi.org/pypi/nagisa/json)와 [soynlp 메타데이터](https://pypi.org/pypi/soynlp/json), 직접 확인한 의존성6개의 메타데이터를 원문으로 저장했다. 조회 시점의 최신값을 보존한 것이며 제품 기본값이나 호환성 승인으로 채택하지 않는다.

| 패키지 | 관측 버전 | 관측한 Windows/공통 wheel | 남은 의존성·판단 |
|---|---|---|---|
| nagisa | 0.3.0 | cp313-cp313-win_amd64,21530912B | NumPy·six·DyNet38; 기본 모델/사전 고지 확인 |
| soynlp | 0.0.493 | py3-none-any,416753B | NumPy·SciPy·psutil·scikit-learn |
| DyNet38 | 2.2 | cp313-cp313-win_amd64,1299290B | Cython·NumPy, 실제 native import 미검증 |
| six | 1.17.0 | py2.py3-none-any,11050B | metadata상 runtime 의존성 없음 |
| psutil | 7.2.2 | cp37-abi3-win_amd64,137737B | 별도 cp313-cp313t는 free-threaded용이므로 현 CPython313용으로 자동 선택하지 않음 |
| scikit-learn | 1.9.1 | cp313-cp313-win_amd64,8237244B | NumPy·SciPy·joblib·narwhals·threadpoolctl |
| joblib | 1.6.0 | py3-none-any,306115B | cloudpickle; test/docs extras와 구별 |
| threadpoolctl | 3.7.0 | py3-none-any,26362B | metadata상 runtime 의존성 없음 |

Cython·narwhals·cloudpickle의 고정 배포물/의존성과 전체 closure는 아직 확보하지 않았다. wheel 존재는 현재 NumPy2.3.2 ABI/native DLL 로드·기본 사전 파일·추론 성공의 증거가 아니다. nagisa는 신경망 분절기이므로 실제 tagging은 별도 AI worker 실행으로 계수하고 자원을 관측해야 한다. MIT metadata와 blank license 필드를 개별 원문에 보존했으며 blank를 무라이선스로 단정하지 않는다. 모델·사전·native 종속성의 원문 고지와 배포 조건은 별도 확인한다.

## 증거와 검사

최초2개 패키지 조사 [r1](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/segmentation-runtime-metadata-r1/result.json) SHA256 `0a2257355f4255cf91e6fe6e257e7b965593567737c0059a5ca03c2626921c97`, 직접 의존성6개를 더한 [r2](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/segmentation-runtime-metadata-r2/result.json) SHA256 `9bf2d5347eefe7c602e07fc335fd89fa41bb64c7dcc7b30d2355ab72a33b9f10`이다. 각 result는 원문 metadata SHA, Requires-Python/Requires-Dist와 배포물명·크기·URL·PyPI digest를 기록한다. metadata read 상한4MB·공식 source256KiB이며 분포 파일 취득은false다.

r2 실행 EXE `e0ae8461fbcb9e06e03e134f72bfbe41d28d87cf9520362f8a9e8a0a3339388f`의 컴파일 입력165개·EXE1개를 [선보존](../../../evidence/ai/model-spike-builds/e0ae8461fbcb9e06e03e134f72bfbe41d28d87cf9520362f8a9e8a0a3339388f/build-manifest.json)했다. r1 EXE `ebcd163a…`는 별도 보존했으며 최신 소스로 소급하지 않는다. 담당 도구 Rust48tests·fmt check·clippy all-targets -D warnings·locked offline build를 통과했다. 메타데이터 조사 도구가 Python 패키지를 설치하거나 사용자 앱을 종료하지 않았다.

## 다음 실행

후속 [고정 wheel 취득·사설 추출](forced-segmentation-runtime.md)에서 미확보3개 metadata·wheel11개 취득·총14archive 추출을 완료했다. 아래 import·실제 분절 대조는 아직 남아 있다. 이 문서의 최초 조사 시점과 후속 완료 범위를 구별한다.

미확보 직접 의존성3개를 포함해 고정 wheel 계획을 검토하고 hash-bound 순차 취득·새 사설 경로 추출을 수행한다. 기존 JER/NumPy 환경에 패키지를 덮어쓰지 않는다. 실제 native import와 nagisa 모델 파일·KO 사전 identity를 확인한 후 동일 transcript3개의 원시/cleaned 분절과 timestamp prompt를 Rust 기준 출력과 대조한다. 분절이 다르면 같은 입력의 단일 요인 정렬 A/B 여부를 판단한다. 사람 word 경계 정답과 별개이며 분절 일치를 정렬 품질 승인으로 취급하지 않는다.
