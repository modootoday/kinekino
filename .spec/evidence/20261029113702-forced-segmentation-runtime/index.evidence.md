---
id: evidence:kinekino/20261029113702
title: 공식 분절 사설 환경 취득·추출
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공식 분절 사설 환경 취득·추출

2026-10-18. [배포물 조사](forced-segmentation-readiness.md)에 이어 미확보 의존성3개를 확인하고 고정 wheel11개를 실제 취득했다. 기존 CPython·NumPy·SciPy archive3개를 재검증해 총14개를 새 프로젝트 내부 경로에 추출했다. **interpreter 실행·native import·실제 분절은 아직 수행하지 않았다.**

## 고정 계획과 실제 취득

Cython3.3.0·narwhals2.27.1·cloudpickle3.1.2를 더한 [metadata r3](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/segmentation-runtime-metadata-r3/result.json) SHA256은 `c9b72ccd884efe09ece3a4a7bc8333bac2620ebcc9022a157eb7f8cb25cff912`이다. Cython과 cloudpickle의 Requires-Dist는null, narwhals 의존성은 관측 metadata에서 extras 조건이었다. 일반 환경과 test/docs/선택 extras를 구별했으며 자동 dependency resolver를 실행하지 않았다.

고정11개는 nagisa0.3.0, soynlp0.0.493, DyNet38 2.2, six1.17.0, psutil7.2.2, scikit-learn1.9.1, joblib1.6.0, threadpoolctl3.7.0, Cython3.3.0, narwhals2.27.1, cloudpickle3.1.2다. native wheel은 일반 CPython313 Windows x64, psutil은cp37-abi3 Windows x64를 선택했다. free-threaded cp313t나 sdist 빌드를 자동 대체하지 않는다.

[고정 계획](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/segmentation-package-plan-r1/result.json) SHA256 `92059e98281abfdf3664a022d119be5d9976cd78d0738c1f2296b4e829148258`을 검토한 뒤 그 해시를 acquire 명령에 전달했다. 파일별 실제 길이와 PyPI SHA256이 모두 일치했다. [취득 결과](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/segmentation-wheels-r1/result.json) SHA256은 `c049825402b716e441b24a8172bb1779e32a5ddf4c628d38d7ca0896f993e9b4`다. 개별 wheel 이름·URL·크기·digest 및 원문 metadata hash는 manifest를 따른다. 준비용 패키지 수를 AI 모델 수나 추론 횟수에 더하지 않는다.

## 사설 환경과 원본 관측

`prepare-segmentation-runtime segmentation-runtime-private-r1 segmentation-wheels-r1 <검토한 manifest SHA>`로 실행했다. preflight CPU25.638506876227897%·free RAM37301063680B에서 기존 자원 기준을 통과했고 exit0이었다. `runtime/ai-spike/source/segmentation-runtime-private-r1`에4355개 파일·273336577B를 추출했다. [결과와 전체 파일 inventory](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/segmentation-runtime-private-r1/result.json) SHA256은 `766dd2db64b219b7e12dcf0128638f27ed62465e69f6cb616c57456e8c10d23c`다.

기존 JER·mel 경로에 덮어쓰지 않았고 호스트 Python/pip·전역 환경·사용자 앱·VM·DPI를 변경하지 않았다. 공통 ZIP 어댑터의 새 경로·Windows 경로/중복·CRC·symlink 거부·파일128MiB/전체768MiB 제한을 공유했다. 배포물의 원본 코드나 사전·가중치를 수정하지 않았다.

원본 `nagisa/__init__.py`는 import 시 `Tagger()`를 초기화하므로 실제 import부터 모델/native 자원 관측이 필요하다. 원본 `soynlp/__init__.py`는 `__license__ = 'GPL v3'`를 선언한다. 앞선 blank PyPI license metadata를 배포 조건 해소로 해석하지 않는다. 이 환경은 비배포 참조 검증용이며 제품 의존성 채택·묶음 배포 승인과 구별한다. 원문 고지와 모델·사전·native 종속성의 재배포 조건은 별도 게이트다.

DyNet38 배포 버전2.2의 원본 `dynet.py`에는 `__version__ = "2.1"`과 별도 gitversion 문자열이 있다. 이는 디스크 원문 관측이며 실제 import에서 반환한 버전이나 호환 실패가 아니다. 이후 실행 보고서에 artifact 버전과 module 속성을 구별해 기록해야 한다.

## Rust 도구와 증거

wheel 검증·취득을 공통 infrastructure `reference_packages`로 분리했고 mel18개·분절11개의 고정 domain 목록을 각각 유지했다. 각 계획/취득/추출은 별도 application 유스케이스다. 검토 manifest 해시·정확한 package/version/filename·HTTPS 배포 host·길이·SHA·cache 내부 경로 조건을 공유하며 임의 wheel 실행이나 fallback은 없다.

실행 EXE SHA256 `d64f2200a2ea5ee6e3456d6a10f7df7d0514bb0ce11e6467cf9037c433f3f711`의 컴파일 입력167개·EXE1개를 [선보존](../../../evidence/ai/model-spike-builds/d64f2200a2ea5ee6e3456d6a10f7df7d0514bb0ce11e6467cf9037c433f3f711/build-manifest.json)했다. 담당 도구48tests·fmt check·clippy all-targets -D warnings·locked offline build를 통과했다. metadata r3 실행의 별도 build `924f7c53…`도 보존했고 이후 소스로 소급하지 않는다.

## 다음 게이트

후속 [원본 tokenizer 실제 대조](forced-segmentation-actual.md)에서 native import와 transcript3개 분절을 실행했다. KO19개는 같고 JP2개는 단위가 달랐다. 아래 사전 identity·정렬 A/B·전체 processor/encoder·사람 경계 게이트는 계속 남는다.

Rust C-API의 격리 경로와 DLL을 확인하고 native import를 실행한다. nagisa 모델 파일과 한국어 사전 identity를 묶어 같은 실제 음성의 고정 transcript3개에 원본 nagisa/soynlp 함수를 호출한다. raw/cleaned 분절과 prompt 전체를 기존 Rust 결과와 비교해야 한다. 원본 Qwen processor 전체 import에는 Torch/backend도 필요하므로 standalone tokenizer 대조를 전체 공식 processor 실행으로 표현하지 않는다. 분절 후 필요한 단일 요인 정렬 A/B·공식 encoder 수치·사람 word 경계는 여전히 남는다.
