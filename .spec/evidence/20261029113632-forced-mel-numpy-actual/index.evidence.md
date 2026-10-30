---
id: evidence:kinekino/20261029113632
title: 원본 NumPy mel 함수 실제 대조
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원본 NumPy mel 함수 실제 대조

2026-10-18. [사설 환경 준비](forced-mel-numpy-runtime.md)에 이어 변경하지 않은 Transformers4.57.3의 `WhisperFeatureExtractor._np_extract_fbank_features`를 Rust/PyO3 C-API에서 실제 호출했다. [기존 엔진 tensor](forced-mel-math-contract.md)의 총529152개 원소가 모두 사전에 정한 절대오차1e-5 이내였다. **공식 NumPy feature 함수의 이 입력3개 대조가 통과한 것이며 PyTorch/kernel/encoder·분절·word 경계·모델 품질 승인은 아니다.**

## 실제 입력과 관측

| 입력 | 출력 shape | 원소 수 | 최대 절대차이 | 허용오차 초과 |
|---|---|---:|---:|---:|
| ko1883 | 1×128×1248 | 159744 | 1.1920928955078125e-7 | 0 |
| ja1516 | 1×128×1674 | 214272 | 1.1920928955078125e-7 | 0 |
| ja1634 | 1×128×1212 | 155136 | 1.1920928955078125e-7 | 0 |

기존 `mel-contract-r1-mel.json` SHA256 `63260547a885d8f5b20201177c9a70ef265ae747feece8351aa2c0147d80f866`과 WAV3개·기존 엔진 배열의 해시를 확인했다. PCM16 mono16k를 Rust에서32768로 나눈 float32로 만들고 NumPy float32 batch1에 전달했다. feature128/FFT400/hop160/16k/dither0을 명시한 원본 생성자와 원본 NumPy 함수를 사용했다. 전체 float32 출력 shape·finite와 기존 엔진 배열의 모든 원소를 검사했다. 허용오차를 결과에 맞춰 조정하거나 extractor를 수정하지 않았다.

독립 Rust DFT 번역을 다시 실행한 결과가 아니다. 실제 NumPy2.3.2의 FFT를 포함한 원본 함수 경로다. Torch 부재를 확인했으며 패키지의 Torch/TensorFlow/Flax 미설치 경고는 보존했다. 신규 모델 가중치 로드·신경망 실행0회, 정렬12회·구조3통과/9실패·품질 승인0은 유지한다.

## 격리와 재현 근거

`jer-function-reference --mel mel-numpy-functions-r1`로 실행했다. 기존 JER 명령은 유지하고 mel application·외부 함수 adapter·순수 수치 비교 domain을 분리했다. 공통 supervision의 새 private Windows Job에 worker를 할당한 후 stdin으로 시작을 허용했다. BelowNormal/BLAS·OMP·MKL4threads,60초 기한, stdout/stderr 각각1MiB 상한이었다. 관측15.9117589초·exit0·timeout=false·출력 초과false다. 이 시간은 사설 파일3719개 해시 검사·import·FFT·저장도 포함하며 성능 benchmark가 아니다.

preflight CPU19.670542635658915%·free RAM36250963968B로 수치 admission을 통과했다. 별도 player.exe가 존재해 일반 preflight ready는false였다. ready를 전체 앱 종료 조건으로 사용하지 않았고 사용자 프로세스나 캐시·VM·DPI를 변경하지 않았다. 재생 부하와 동시 실행 성능을 승인한 결과도 아니다.

실행 전에 환경 manifest 및3719개 파일을 재검증했다. CPython은 isolated/no_site/no_user_site/ignore_environment/dont_write_bytecode/safe_path를 실제 확인했다. 검색 경로는 사설 표준 라이브러리 zip·python·site-packages·동일 site-packages4항목뿐이며 호스트 site를 추가하지 않았다. 마지막 경로 중복은 기존 공통 초기화 어댑터의 명시적 source 인자를 같은 검증된 root로 지정한 것이다. 실제 DLL은 사설 `python313.dll`이고 SHA256 `69fd86ea29370697c203f7e12830084f920f490766a8e3045af52c036a9ad529`다. 원본 Python 코드 추가·수정·eval·stub은 없다.

새 EXE SHA256 `cdb34d91c1ff96401ec201568e7e28adec6069d27b5af1d4b648ffb91fe86a25`의 컴파일 입력51개와 EXE1개를 [선보존 묶음](../../../evidence/player/jer-function-reference-builds/cdb34d91c1ff96401ec201568e7e28adec6069d27b5af1d4b648ffb91fe86a25/build-manifest.json)에 보존했다. 참조 실행기의 기존 명칭이 유지된 보존 경로이며 이 실행에서 JER 함수를 실행했다는 뜻은 아니다. native/transitive 전체 재현 빌드 attestation과 구별한다.

[실제 결과](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/mel-numpy-functions-r1/mel-result.json) SHA256은 `82d25a232e32d16c952604f99edced9f048c3938fc0295969a38d7f9abaef5c1`이다. [supervision 관측](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/mel-numpy-functions-r1/observation.json)·worker stdout/stderr 및 전체 배열3개를 보고서1311 snapshots에 수집했다. 원시 배열은 다음 경로에 보존했다.

- [ko1883](../../../artifacts/spikes/2026-10-18/acquisitions/mel-numpy-functions-r1/ko1883-official-numpy-f32.bin): `a11be61a84c23db6c9e0fe261b14b99142b76760bc21c2d5cd1e9a1bb58b4949`
- [ja1516](../../../artifacts/spikes/2026-10-18/acquisitions/mel-numpy-functions-r1/ja1516-official-numpy-f32.bin): `5c760eb80f244f9ea99db57387de1fe024435c16f63e4e5eed359b9995fb92d8`
- [ja1634](../../../artifacts/spikes/2026-10-18/acquisitions/mel-numpy-functions-r1/ja1634-official-numpy-f32.bin): `74510774b41a55ff689ae43a273b2e42629b87dd0aac444cbdf722dc84acca59`

担当 Rust 도구는2tests·fmt check·clippy all-targets -D warnings·locked offline build를 통과했다. 신규 순수 domain 회귀는 오차 초과 수, 비유한 값, 비어 있거나 길이가 다른 배열의 거부를 검사한다. 기존 출력 상한 회귀도 유지한다.

## 남은 판단

새 실행기에서 기본 JER 명령의 회귀도 요청했으나 `mel-jer-default-regression-r1`은 CPU95.23346303501945%·RAM31882514432B, r2는69.41176470588235%·31287926784B로 admission에서 종료했다. 두 시도 모두 worker 시작·JER 실행이 없으며 기존6개 대조 성공을 새 실행기의 재검증으로 소급하지 않는다. preflight 및 stderr를 별도 보존했다. 자원 여유 시 이 기본 명령 회귀를 우선 재개한다.

후속 `mel-jer-default-regression-r3`는 같은 EXE `cdb34d91…`에서 CPU55.46875%·free RAM31286358016B로 admission을 통과해36.0487493초·exit0·timeout=false였다. 기본 JER 원본6개 함수 대조의 corrected counts·scored frames·교차 행렬이 모두 일치하고 `external_jer_function_parity_verified=true`였다. [새 기본 경로 원시 출력](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/mel-jer-default-regression-r3/worker.stdout) SHA256은 `8b516b1ce73187162497b011789a0ee9e4b05b814b06485e7dc766e541b80fba`다. 이로써 새 mel 분기 추가 후 기본 경로 실제 회귀를 완료했다. 전체 dscore CLI·모델 품질false와 새 신경망 실행0은 유지한다. 앞의 자원 대기2건을 삭제하거나 실제 실행으로 계산하지 않는다.

중앙 검사에서 보존 실행기5개·목록상 소스237항목·바이너리5개와 immutable105개 해시를 확인했다. 문서303개·정본 링크 오류0·보존 upstream 오류2였다. 새51항목은 이전186항목에 더한 목록 수이며 고유 소스 수나 전이 의존성 전체 attestation이 아니다.

현재 입력에서 mel 수학/NumPy 경로의 큰 차이는 관측되지 않아 이 부분을 임의 수정하거나 같은 정렬을 재실행하지 않는다. 다음은 공식 분절·PyTorch/encoder activation·사람 word 경계와 cue 검토 저장이다. 작은 mel 오차가 모델 내부에서 영향이 없다는 인과 증명이나 다른 입력/장치/구현의 동등성으로 확대하지 않는다. [품질 게이트](../../../plans/player/ai-quality-gates.md)를 따른다.
