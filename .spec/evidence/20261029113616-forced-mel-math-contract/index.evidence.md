---
id: evidence:kinekino/20261029113616
title: 정렬 mel 수학 계약 대조
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 정렬 mel 수학 계약 대조

2026-10-18. 기존 한국어 KO1883 및 실패 일본어 JA1516·JA1634의 전체 mel 행렬을, 보존 Rust extractor와 독립 Rust 직접 DFT 참조 사이에서 비교했다. **529152개 원소 모두 사전 허용 절대오차1e-5 안이었고 최대차이는1.1920928955078125e-7이었다.** 새 신경망 추론은0회다. 공식 NumPy/PyTorch를 실행한 결과가 아니며 공식 framework tensor parity나 정렬 품질 통과로 표시하지 않는다.

## 비교 경계

엔진 쪽은 기존 관측 fork의 `feature_extractor.rs`를 변경 없이 직접 import했다. 소스 SHA256 `9687a313e61b4b557b0dcdca736a06fe6df6ffc8268da0b2bca72f3ca2d3bf8a`를 검사하고 빌드 입력으로 함께 동결했다. FFT는 기존 rustfft6.4.1 경로다. streaming extractor나 모델 forward는 이 실제 비교에서 실행하지 않았다.

참조 쪽은 FFT 라이브러리나 원본 extractor의 helper를 호출하지 않는다. Rust 직접 DFT로 각400샘플 창을 계산하고, 공식 NumPy 계약의 complex64 중간 저장·f64 전력·Slaney128필터·log10 후f32·최대값−8범위·(x+4)/4를 구성했다. 중앙 reflect200은 끝점을 반복하지 않고 periodic Hann400/hop160, 마지막 STFT frame 제외를 적용했다. PCM16/32768 변환과 batch1의 all-ones frame mask를 확인했다.

공식 PyTorch 경로는 f32 waveform/window/STFT/필터 연산을 사용한다. 현재 Rust는 f64 FFT 중간값을 사용하며 NumPy 계약 번역은 complex64 저장을 명시한다. 이번 일치는 이 작은 정밀도 차이를 포함한 **Rust 수학 참조 대조**다. 실제 NumPy FFT/BLAS나 PyTorch kernel의 출력 및 encoder tensor/활성값은 관측하지 않았다. 공식 분절·모델 학습 입력·사람 경계 정확도 차이도 배제하지 않는다.

## 실제 입력 결과

| 입력 | mel×frames | 비교 원소 | 최대 절대차 | RMS 차 | 허용치 밖 원소 |
|---|---|---:|---:|---:|---:|
| KO1883 | 128×1248 | 159744 | 1.1920928955078125e-7 | 1.8870813272237346e-8 | 0 |
| JA1516 | 128×1674 | 214272 | 1.1920928955078125e-7 | 1.5765153687026145e-8 | 0 |
| JA1634 | 128×1212 | 155136 | 1.1920928955078125e-7 | 1.5398481289077956e-8 | 0 |

원음 해시와 샘플 수는 [기존 실제 정렬](forced-alignment.md)과 같다. 범위 밖 원소를 삭제하거나 값을 clamp해 비교하지 않았다. shape·유한값·각 row 길이가 다르면 오류다. 별도 합성 회귀에서 reflect 끝점·401샘플의 frame floor·무음floor와 NaN/길이/형상 오류를 확인했다. 합성 결과는 실제 발화나 사람 정답 수에 포함하지 않는다.

## 공식 소스와 보존

다음 원문을 하나의 worker에서 순서대로 취득해 UTF8 텍스트로 보존했다. 참조 코드는 실행하지 않았다. 라이선스 헤더를 유지했다. Transformers4.57.3은 계약 앵커이며 과거 정렬 실행에 사용한 Python 환경 버전이라는 뜻은 아니다.

| 원문 | 바이트 | SHA256 |
|---|---:|---|
| [WhisperFeatureExtractor4.57.3](https://github.com/huggingface/transformers/blob/v4.57.3/src/transformers/models/whisper/feature_extraction_whisper.py) | 16732 | c209e669fae0ea5de92e1846d8790e3d6d66ce28f4cd4071789c2eec88c84a66 |
| [audio_utils4.57.3](https://github.com/huggingface/transformers/blob/v4.57.3/src/transformers/audio_utils.py) | 54284 | c038450307a8dbc9a95eed8e413770f34814fa38058121ca9f2eee8dd0dfa958 |
| [Qwen processor 고정 commit](https://github.com/QwenLM/Qwen3-ASR/blob/7c6daf77a2421100f5fb066495372c00129d39ff/qwen_asr/core/transformers_backend/processing_qwen3_asr.py) | 8545 | a61f1fbfc06e3dfc63a2b8d15e3b29e98b032792c3fb7d0e01c04ca9c2181ffb |

`runtime/ai-spike/results/mel-contract-r1-mel.json`에 소스 원문·preprocessor config·입력·양쪽 tensor의 경로/해시/shape·mask·오차를 기록했다. tensor는 mel-major float32 little endian binary로 전체6개를 저장했다. `report`가 원문/config와 모든 tensor 해시를 검사한 뒤 evidence/artifacts와 snapshot manifest에 편입했다. 스칼라 요약만 남긴 비교가 아니다.

실행 EXE는 `0ab1255bfec101253b2820255096e45af1c31e66ef0a77e0fd47956bd02c48fd`다. [선보존 bundle](../../../evidence/ai/model-spike-builds/0ab1255bfec101253b2820255096e45af1c31e66ef0a77e0fd47956bd02c48fd/build-manifest.json)은 컴파일 입력105개와 EXE1개로 총106개다. 관측 wrapper label은 `mel-contract-r1`, exit0/timeout=false, wall5.0535396초다. admission CPU14.55%/여유RAM38228779008바이트에서 BelowNormal 단일 worker로 실행했다. 모델 가중치를 로드하거나 앱/VM/DPI/캐시 설정을 변경하지 않았다. 네트워크 원문 취득과 DFT가 함께 포함된 wall을 성능 기준으로 쓰지 않는다.

## 보존 수량 표기 정정

기존 preserve-build의 `files`는 소스와 바이너리를 합친 bundle 항목 수였다. 이번 도구는 `compiled_source_inputs`와 `binary_files`를 분리했다. 과거 manifest는 바꾸지 않고 실제 stored_path를 재확인해 Sprite 및 head 관측 정본의 “컴파일 입력” 수를 각각 EXE1개 제외 수로 정정했다. 원래 EXE·소스·측정 해시는 그대로다. 번들 항목 수를 소스 검증 범위나 전이 의존성 전체 검증으로 확대하지 않는다.

## 검사와 잔여 게이트

후속 [실제 원본 NumPy 함수 대조](forced-mel-numpy-actual.md)에서 같은 PCM3개529152원소를 허용오차 내로 확인했다. 이 문서의 독립 Rust 수학 번역 실행과 후속 실제 NumPy 실행은 별도 근거다.

담당 도구 fmt check·test41개·clippy all-targets -D warnings·locked offline build가 통과했다.41개 중2개는 변경 없이 import한 upstream DSP 회귀이며 새로운 신경망 실행이 아니다. 이 수학 참조가 허용치 안이므로 extractor를 바꿔 모델을 재실행하지 않았다. 실제 공식 PyTorch 경로·encoder 입력/활성값·공식 분절과 사람 word 경계 대조는 [잔여 품질 게이트](../../../plans/player/ai-quality-gates.md)로 남는다. 실제 정렬 횟수12·구조3통과/9실패·품질 승인0은 변하지 않는다.
