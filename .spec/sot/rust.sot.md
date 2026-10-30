---
id: sot:kinekino/rust
title: Rust 우선 도구 선정과 관측
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Rust 우선 도구 선정과 관측

동등한 기능·품질·Windows 지원·유지보수 검증이면 Rust를 우선한다. Rust API wrapper와 순수 Rust 계산 엔진을 혼동하지 않는다. 새 오케스트레이션·가공·측정·평가 코드는 tools/model-spike의 계층별 Rust다.

| 도구 | 역할·상태 |
|---|---|
| Candle0.11.0 | 순수 Rust tensor/model 실행. 실제 Whisper/MobileSAM CPU 테스트. 배포 예제 Symphonia API 충돌을 재현해 공식 수정 커밋으로 격리 사본 복구 |
| ort2.0.0-rc.14 + ORT1.31 | Rust API와 네이티브 ONNX Runtime. 실제 VAD/OCR/YAMNet/화자/깊이 CPU. Rust-only 엔진이라고 부르지 않음 |
| mistral.rs0.9.4 | Rust 텍스트 실행기. 기존 고정 portable/weights를 재사용한 실제 요청. 초기 Qwen0.6B 교정/한국어 카드 문구는 품질 미달 |
| kaldi-fbank0.1.0 | 순수 Rust FFT/FBank. 실제오디오에서 knf-rs0.3.2 네이티브 기준과 동일 Povey 조건을 비교해 최대오차0.00006294 <0.001 확인. HF WeSpeaker용 Hamming은 별도 옵션 |
| knf-rs0.3.2 | native Kaldi wrapper를 비교 기준으로 사용. 고정 window 기본값이 모델 전처리와 달라 생산 경로는 설정 가능한 Rust FBank로 전환 |
| rusty-chromaprint0.3.0 | 순수 Rust 반복 오디오 fingerprint 후보. 음악 이벤트/OP·ED 의미 분류와 다른 역할 |
| Symphonia0.6 | 순수 Rust 디코더 후보. 입력 AC3 포함 영화의 완전 대체 지원은 별도이며 모든 컨테이너/코덱을 지원한다고 주장하지 않음 |
| FFmpeg/ffprobe6.1 설치본 | 실제 AC3→PCM·프레임 추출·자막 렌더/포맷 가공. 외부 검증 도구 adapter이며 자체 비Rust 코드 작성 아님. 배포는 최신 portable 보안/라이선스·실행 파일 hash를 다시 검증 |
| image0.25/hound3.5/reqwest0.12/SHA256 | Rust 이미지·WAV·HF 전송·무결성. HTML/임의 프로그램을 자산으로 실행하지 않음 |
| gltf-rs1.4/glam·Rust 이미지/글꼴 도구 |2D 레이어/2.5D·projection 패키지 후보. GLB import/render round-trip과 한글 layout 검증은 별도 |

Candle 복구 근거: [공식 commit48fb1719f0f013947e226580e2c63b4946741135](https://github.com/huggingface/candle/commit/48fb1719f0f013947e226580e2c63b4946741135). 원본/수정 audio.rs와 commit JSON을 runtime/ai-spike/source에 보존하고, vendor 사본에만 적용했다. 추가 실험용 변경은 SAM raw mask PNG 저장, Whisper timestamp 모드를 명시적으로 끌 수 있는 CLI 설정이다. 시스템 패키지/원본 Cargo registry는 변경하지 않았다.

[kaldi-fbank](https://github.com/ryaningli/kaldi-fbank), [pyannote-rs](https://github.com/thewh1teagle/pyannote-rs), [usls](https://github.com/jamjamjon/usls)를 조사했다. usls는 다양한 모델의 Rust 어댑터지만 현재 alpha와 자동 모델 관리가 고정 revision 통제에 미치는 영향을 검증해야 한다. 이번에는 직접 고정 파일을 여는 작은 어댑터를 사용했다. pyannote-rs의 접근법을 baseline으로 참고하되 현재 greedy 군집은 community1/VBx 성능의 검증이 아니다.

외부 도구 선택은 UI 책임이 아니다. 전송·process·GPU·decoder·파일 포트는 infrastructure, 작업/예산/검토는 application, 시간·화자·변환 정책은 domain에 둔다.
