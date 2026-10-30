---
id: sot:kinekino/speech-timing-diarization
title: 시간 정렬과 화자 품질 후속 게이트
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 시간 정렬과 화자 품질 후속 게이트

## 시간과 강제 정렬

기존 관측은 Whisper cue 시간 범위 검증과 ASR 텍스트다. 단어 정답을 제공한 강제 정렬 성공은 아직 아니다. 다음 선택 순서는 기존 Candle timestamp 교정 → 별도 aligner runtime 검토다.

| 후보 | 공식 기능·실행 계보 | 다음 게이트 |
|---|---|---|
| 기존 Candle Whisper | Rust 계산 경로; 이미 음성 입력 실행 관측 있음 | cue 시작/끝/offset·단조성·duration 범위 확인. 단어 경계 정확도는 별도 |
| whisper-rs/whisper.cpp DTW | [whisper-rs](https://github.com/tazz4843/whisper-rs)는 [C/C++ whisper.cpp](https://github.com/ggml-org/whisper.cpp)의 Rust 바인딩 | DTW token timestamp는 주어진 정답 텍스트의 일반 강제 정렬과 구별. 네이티브 라이브러리/가중치 포맷·모델 hash·Windows ABI·메모리 overhead 고정 후 비교 |
| Qwen3-ForcedAligner-0.6B | [공식 카드](https://huggingface.co/Qwen/Qwen3-ForcedAligner-0.6B)는 일본어/한국어 포함 11언어·최대5분 정렬 제시. 공식 사용 예제는 Python qwen-asr/Transformers | [고정 SHA](revisions.md). Qwen 텍스트 decoder 지원만으로 오디오 frontend/align head 지원을 인정하지 않는다. Candle/ORT용 검증 그래프·전처리·token→span decoder가 확보되기 전 Rust 실행은 blocked |

후속 구현 위치 제안: `tools/model-spike/src/domain/alignment.rs`에는 source offset·timebase·token/문자 범위·unmatched/zero-duration을 표현하고, `application/alignment_spike.rs`는 평가 입력/출력, `infrastructure/inference/alignment/`는 엔진별 어댑터를 둔다. GUI/CLI 공유 계약으로 승격하기 전 독립 도구에서 검증한다.

정답 일본어·한국어 원문과 10~25초 원음에 사람이 시작/끝을 붙인 작은 구간표를 먼저 만든다. 정확한 transcript와 ASR 오류 transcript를 분리한다. 음원 없음/무음/오정답 문장·장면 offset·구두점·일본어 무공백을 시험한다. 각 span의 `0 <= start < end <= duration`, 원문 coverage, unmatched 비율, 수동 경계 오차를 기록한다. 0길이 span을 정상 단어로 자동 저장하지 않는다. 강제 정렬이 transcript의 의미 정답을 증명하지도 않는다.

공식 고정 revision의 실제 `model.safetensors`는1,835,544,544 bytes이며 thinker config는 BF16이다. 모델 이름에서 산술한1.2GB 추정은 제거한다. 파일 크기도 peak RAM/VRAM이 아니며 긴 오디오 feature·activation·engine workspace·dtype 승격 포함 실측이 필요하다. 처음부터5분/동시 ASR+aligner를 올리지 말고 하나씩 load/unload한다. 현재 실행 가능한 Rust align CLI는 없음; 모델을 받기 전에 runtime 지원 파일/버전을 먼저 확보한다. 입력 feature·필요 파일은 [실행 준비 계약](execution-readiness.md)에 있다.

## 화자 분리: 모델 교체 전에 정답 평가

기존 `ort + segmentation + WeSpeaker + Rust FBank + greedy clustering`은 실행 관측이지만 군집 품질 미승인이다. 같은 정답 구간 없이 새 모델을 추가하면 개선 여부를 판정할 수 없다.

첫 작업은 실제 원음의 단일 화자, 교대, 겹침, 무음/BGM, 짧은 재등장 사례에 익명 speaker ID와 RTTM을 수작업 작성하는 것이다. 원래 사람의 신원이나 캐릭터 이름으로 매핑하지 않는다. reference와 hypothesis 시간축·collar·overlap 포함 여부·화자 label 최적 대응을 고정한 Rust DER/JER 평가기를 추가한다. collar=0/overlap 포함 결과와 제품 검토용 결과를 따로 보고하고 false speech/missed speech/confusion을 분해한다. 정답이 없으면 DER를 보고하지 않는다.

평가 정의의 1차 참조는 DIHARD 제작자의 [dscore](https://github.com/nryant/dscore#iii-metrics)다. RTTM·DER 구성과 JER 최적 화자 대응을 참고하되 해당 Python 평가기를 이 프로젝트의 새 검증 코드로 추가하지 않는다. Rust 평가기는 완전 일치·label permutation·누락·오검출·겹침의 수계산 작은 fixture와 맞아야 한다. 100% 초과 DER를 잘라내지 않는다.

가벼운 첫 대안은 기존 segmentation/embedding을 유지하고 누적 centroid/AHC·화자 수 상한·unknown/overlap 보존 정책을 비교하는 것이다. 군집 알고리즘 수정과 모델 교체를 한 실험에 섞지 않는다. 작은 embedding 후보는 [공식 sherpa-onnx 화자 모델 문서](https://k2-fsa.github.io/sherpa/onnx/speaker-identification/index.html)의 3D-Speaker/CAM++ 계열을 검토하되 원본 라이선스와 변환 revision을 아직 고정하지 않았으므로 다운로드 승인 목록에 넣지 않는다.

[sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx)는 Windows와 공식 Rust API 예제를 제공한다. 네이티브 C++/ONNX Runtime 실행의 Rust FFI이며 pure Rust 엔진이 아니다. [pyannote-rs](https://github.com/thewh1teagle/pyannote-rs)도 Rust API/ONNX 엔진 경로다. 기존 ORT 어댑터로 동등한 결과가 나오면 새 네이티브 의존성 추가보다 기존 경로를 우선한다.

[Community-1 공식 카드](https://huggingface.co/pyannote/speaker-diarization-community-1)는 CC-BY-4.0 gated 후보이며 공개 benchmark를 로컬 품질로 대체하지 않는다. `config.yaml` 하나만 고정하지 말고 segmentation/embedding·VBx/PLDA·전처리·군집 파라미터 전체를 고정해야 비교 가능하다. 인증·조건 동의·Rust runtime 포팅이 선행되지 않으면 조사 상태로 남긴다. 익명 greedy cluster를 Community-1 또는 VBx 구현이라고 부르지 않는다.

재검증에서 고정 SHA의 config 직접 요청은401이었다. 따라서 위 의존 목록은 확인해야 할 pipeline 책임 목록이며 이 revision의 config 내용을 읽어 확정한 목록이 아니다. 인증을 우회하지 않았고 하위 모델 SHA/파일·dtype·전처리·가중치 라이선스 원문은 미확보다.
