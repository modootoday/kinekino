---
id: evidence:kinekino/20261029113709
title: 공식 NumPy mel 참조 환경 준비
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공식 NumPy mel 참조 환경 준비

2026-10-18. [독립 Rust 수학 대조](forced-mel-math-contract.md) 다음 단계로, 공식 Transformers 4.57.3의 변경하지 않은 NumPy mel 함수를 실행하기 위한 사설 환경을 준비한다. 이 버전은 기존 비교 소스의 고정 기준이며 최신 제품 추론 엔진 권고가 아니다. **환경 취득·추출과 실제 함수 실행·수치 대조는 별도 게이트다.** 현재 신경망 실행과 품질 승인은 추가하지 않는다.

## 고정 입력과 취득

Rust 도구가 공식 PyPI 버전 메타데이터에서 정확한 wheel 이름·버전·플랫폼을 선택했다. 메타데이터18개를 원문 보존하고 yanked 배포물, 잘못된 digest, 64MiB 초과 파일을 거부한다. 검토된 계획 manifest의 SHA256을 명령에 전달한 후 files.pythonhosted.org의 정확한 HTTPS wheel만 순차 취득했다. 파일별 실제 길이와 SHA256이 일치한18개를 보존했다. 의존성 import/해결 성공은 아직 확인하지 않았다.

- [계획 manifest](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/mel-numpy-package-plan-r1/result.json): `3ae0eb25da82401b2bda7366312687687cf351d82b635b8278d79ef4399d2932`
- [취득 manifest](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/mel-numpy-wheels-r1/result.json): `e1526d73f6f7d155558195a993529e6071628d13724f2116b4934442b0cee1f1`

18개는 transformers4.57.3, huggingface-hub0.36.0, filelock3.20.0, fsspec2025.10.0, packaging25.0, pyyaml6.0.3, regex2025.11.3, requests2.32.5, charset-normalizer3.4.4, idna3.11, urllib3 2.5.0, certifi2025.10.5, tokenizers0.22.1, safetensors0.6.2, tqdm4.67.1, colorama0.4.6, typing-extensions4.15.0, hf-xet1.2.0이다. 라이선스 메타데이터는 각 원문/manifest에 있으며 전체 재배포 검토 완료를 뜻하지 않는다.

## 사설 추출 계약

기존 [검증된 배포물](diarization-jer-runtime-archives.md)에서 CPython3.13.3 embedded와 NumPy2.3.2만 선택하고18개 wheel을 더한다. SciPy·intervaltree·sortedcontainers는 새 환경으로 추출하지 않는다. 프로젝트 내부 새 경로에만 기록하며 호스트 Python/pip/site-packages를 변경하지 않는다. 기존 Rust ZIP 어댑터의 경로·중복·symlink·CRC·파일/총량 제한을 공유한다.

추출 후 `transformers/audio_utils.py` SHA256 `c038450307a8dbc9a95eed8e413770f34814fa38058121ca9f2eee8dd0dfa958`와 `transformers/models/whisper/feature_extraction_whisper.py` SHA256 `c209e669fae0ea5de92e1846d8790e3d6d66ce28f4cd4071789c2eec88c84a66`을 기존 비교 anchor와 대조한다. 원본 패키지는 수정하지 않는다.

실행 전 도구의 컴파일 입력163개·EXE1개를 [보존](../../../evidence/ai/model-spike-builds/0f99701e39d3a9d2200a4dc86c8b8ab446166bd9bfe1fcff843a6745b6703403/build-manifest.json)했다. EXE SHA256은 `0f99701e39d3a9d2200a4dc86c8b8ab446166bd9bfe1fcff843a6745b6703403`이다. Rust48 tests, fmt check, clippy all-targets -D warnings, locked offline build를 통과했다.

## 실제 추출 결과

첫 `mel-numpy-runtime-r1`은 preflight CPU65.72265625%로 기존60% 미만 기준을 넘겨 추출 전에 종료했다. RAM30807326720B와 preflight의 일반 ready 값만으로 acquisition 허용을 판단하지 않았다. [실패 기록](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/mel-numpy-runtime-r1/failure.json)을 보존했으며 기준 완화나 앱 종료는 없었다.

새 `mel-numpy-runtime-r2`는 CPU47.36328125%·free RAM31170895872B에서 실행해 exit0이었다.20개 archive에서3719개 파일·139266082B를 추출했고 두 공식 소스 hash가 모두 일치했다. [결과 manifest](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/mel-numpy-runtime-r2/result.json) SHA256은 `69e6e663ca9b1b57162fd3ab40ee180dba3d402c58b470d0c853cf4f722bf3d6`이다. 실제 경로는 `runtime/ai-spike/source/mel-numpy-runtime-r2`이며 interpreter 실행false·원본 archive 수정false다. 보고서1298 snapshots에 계획 원문·취득·추출·실패·소스 근거를 수집했다. 추출 파일 inventory를 보존한 것이 모든 파일을 별도 증거 디렉토리에 복제했다는 뜻은 아니다.

## 다음 실행 게이트

후속 [원본 NumPy 함수 실제 대조](forced-mel-numpy-actual.md)에서 아래 PCM3개 전체 tensor 대조를 완료했다. 이 문서는 취득·추출 시점의 증거와 계약을 보존한다.

Rust C-API 어댑터의 격리 경로와 실제 import를 검증하고 원본 `_np_extract_fbank_features`에 고정 PCM3개를 전달한다. 전체529152원소를 기존 실제 엔진 tensor와 대조하고 원시 배열·shape·dtype·최대 오차를 보존해야 한다. 허용오차1e-5를 결과에 맞춰 변경하지 않는다. NumPy 함수 대조가 통과하더라도 PyTorch/kernel/encoder·공식 분절·사람 word 경계와 정렬 품질 승인은 별도로 남는다.
