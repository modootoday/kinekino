---
id: evidence:kinekino/20261029113718
title: 공식 encoder 참조 의존성 고정과 사설 환경 구성
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공식 encoder 참조 의존성 고정과 사설 환경 구성

2026-10-18. [원본 API 조사](../../player/infrastructure/inference/research-followups/forced-encoder-pytorch-readiness.md)와 [Torch archive](forced-encoder-torch-runtime.md)에 이어 나머지26개 wheel을 실제 취득했다. encoder 실행·수치 비교·모델 품질 승인은 아직 미검증이다.

## 고정과 실제 취득

domain/encoder_packages.rs가26개의 package/version/ABI/bytes/SHA를 소유한다. PyPI metadata가 이 pin과 다르면 계획 단계에서 실패하며, 다운로드는 봉인된 계획 SHA와 pin을 다시 확인한다. 기존64MiB wheel 한도와 files.pythonhosted.org HTTPS 경로 제한을 유지했다. 변경 버전 또는 같은 길이의 다른 SHA를 거부하는 domain 회귀를 추가했다.

- [계획26개](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/encoder-packages-plan-r1/result.json), SHA `347f2e05576b67820159112df68755c5d669f68d850698b960f900e3e28fe48c`
- [실제 취득26개·총86423669B](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/encoder-packages-wheels-r1/result.json), SHA `be53ca0e8b72bb27ddf8a0b3b116daa7f4e07b45a0ebc8246aea9e769d32df6d`
- [취득 build](../../../evidence/ai/model-spike-builds/6b58d799dbf789ea30c6b752bda3a2507e5c14a5aec93f4dae7edb493969f6a9/build-manifest.json), compiled inputs183+EXE1
- [환경 구성 build](../../../evidence/ai/model-spike-builds/2275badd5f3a6034d8820a92b7db5f34075758efaa01a74729ff786e3258029b/build-manifest.json), compiled inputs184+EXE1

Transformers4.57.6·nagisa0.2.11을 원본 pin으로 선택했다. 기존4.57.3/0.3.0 환경은 보존하고 새 환경에서만 교체한다. librosa0.11·numba0.63.1·llvmlite0.46·soundfile0.13.1과 Torch의 sympy/mpmath/networkx/jinja2/MarkupSafe/setuptools 등을 포함한다. 전체 pin과 배포 metadata/license 요약은 계획에 보존했다. source-import profile이며 qwen-asr 전체 web/CLI distribution 의존성 충족 또는 배포 승인으로 해석하지 않는다.

## 환경 조립 경계

새 encoder-reference-private-r1은 CPython3.13.3·NumPy2.3.2·SciPy1.16.0 고정 archive3개, 과거 mel 의존성17개와 분절 의존성10개, 새26개, Torch1개로 총57개다. 과거 manifest SHA와 각 파일 bytes/SHA를 다시 확인하고 구버전 Transformers/nagisa archive를 선택에서 제외한다. 기존 wheel 또는 source를 수정하지 않는다.

Torch를 포함하는 전용 profile만 단일파일512MiB·전체2GiB·총파일50000 한도를 받는다. 첫 CPython archive SHA·정확한 Torch archive SHA를 확인하고 공통 안전 추출기의 Windows 경로/중복/CRC 검사를 유지한다. 일반 reference 추출 제한은 유지한다. 실패 시 부분 경로와 오류를 보존하며 기존 환경에 덮어쓰지 않는다.

실제 조립은 exit0으로 종료했다. [환경 manifest](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/encoder-reference-private-r1/result.json)는22624파일·929957910B, SHA `65a578e79bb71dad7fdd6009f08a30bcfe019f16a9965d8bcbbc86eec9fbc7d9`다. 시작CPU14.746%·빈RAM36474486784B였다. 원본 package tree·모든 import origin·DLL 경로·eval/no_grad·CPU F32 strict weight load·동일 full mel encoder 관측은 별도 게이트다. 준비·취득·추출만으로 원본 framework 동작을 승인하지 않는다.

담당55tests·fmt·clippy·locked offline build가 통과했다. 후속 원본source 취득 코드의56tests·fmt·clippy는 통과했지만 당시 같은 target EXE를 환경 조립에 사용 중이어서 Windows의 EXE 교체 거부가 발생했다. 조립 종료 후 locked offline build를 재실행해 통과했으며 이 오류를 모델/환경 실패로 집계하지 않는다.

## 원본 package 보존

QwenLM/Qwen3-ASR commit7c6daf77a2421100f5fb066495372c00129d39ff의 전체 qwen_asr package와 LICENSE·pyproject·README를 Git tree로 열거하고 각 파일을 원본 Git blob SHA1과 대조했다. modeling/configuration/processing의 조사된 blob pin3개도 일치했다. 원본에 없는 init 파일·stub·wrapper를 추가하지 않았다.

[원본 package manifest](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/qwen-original-package-r1/result.json)는18파일·569874B, SHA `fc6f484466c2383b2d8aeffb39341e373e7d544b404f5b6d4765306faa04901d`다. 원본 bytes는 [별도 source 보존](../../../evidence/ai/model-spikes-2026-10-18/encoder-reference/original-source/LICENSE)과 연결한다. [취득 build](../../../evidence/ai/model-spike-builds/50151299d09b87c65e9499d2cad89b370fc900f6730cae90fc774ee310fdb975/build-manifest.json)는compiled inputs186+EXE1이며56tests·fmt·clippy·build 통과다. 원본 import의 실제 결과와 허용되지 않은 origin·추가 의존성은 별도 관측으로 기록한다.

보존 README의 finetuning/ 참조는 이번 encoder package 취득 범위 밖이다. 고정 upstream tree에는 실제 directory가 있다. checker는 정확한 README SHA27b5d6e31c3dc10c9b4b86e226f215baacca57bede504994779ad334a67275a0와 tree metadata SHAb7b4bddaa7419364250373208e850e72a6f6d39cc0cd49c124e6ef584bdb9248 및 해당 tree entry를 확인한 경우에만 preserved_upstream_reference_outside_source_bundle로 구분한다. 로컬 정상 링크 또는 upstream 대상 부재로 표현하지 않으며 원본 README를 고치거나 가짜 directory를 만들지 않는다. 기존 upstream 자체 부재2건과 이 subset 참조1건은 status/note로 구분한다.
