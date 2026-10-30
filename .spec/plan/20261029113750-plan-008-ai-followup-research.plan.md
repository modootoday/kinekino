---
id: plan:kinekino/20261029113750
title: AI 후속 연구와 다음 실행 게이트
status: active
created: 2026-10-29
updated: 2026-10-29
tags: [player]
---
# AI 후속 연구와 다음 실행 게이트

조사 산출물은 [연구 색인](../../sots/player/infrastructure/inference/research-followups/README.md)에 있다. 기존 [plan-007](plan-007-real-media-model-spikes.md)의 관측을 변경하지 않는다. 조사 세션은 다운로드·추론·빌드를 수행하지 않았고 다음 실행은 원래 실행 담당이 맡는다.

- [x] 공식 HF SHA/라이선스/실재 파일을 조사하고 Rust 계산·native FFI·Python 참조 구현을 구분했다.
- [x] P0 확보: 원래 세션이 `speech-archive` Rust 어댑터로 일본어 dev archive·float32→PCM16·distinct transcript ID3개를 확보했다. 조사 세션은 manifest와 코드를 읽어 참조만 했다. [에셋 문서](../../sots/player/infrastructure/inference/research-followups/speech-assets.md)와 원래 세션 검증 정본을 따른다.
- [ ] P0 후속: 원래 세션의 `asr-japanese` 결과·CER·시간 범위·실패를 검토한다. 서로 다른 transcript ID를 서로 다른 화자 또는 애니메이션 품질 검증으로 표현하지 않는다.
- [ ] P1: 기존 Whisper cue 범위/offset 실패를 재현하고 작은 수작업 경계표를 만든다. Qwen aligner는 Rust runtime·전처리·head 지원이 확인된 뒤만 다운로드한다. word span 없는 token timestamp는 정렬 완료가 아니다.
- [ ] P2: 실제 교대/겹침/재등장 RTTM을 만들고 기존 화자 prototype의 DER/JER·unknown·overlap을 Rust 평가기로 검증한다. 먼저 같은 모델의 군집만 비교한다. Community-1의 gated 동의와 전체 pipeline 고정은 별도다.
- [ ] P3: MobileSAM 동일 좌표 정답 mask와 연속 frame 평가를 먼저 만든다. EdgeTAM은 memory 포함 Rust video runner/ONNX parity가 확보되기 전 보류한다.
- [ ] P4: 기존 mistral.rs0.9.4에서 공식1.7B Q8 또는4B Q4를 실제 파일로 비교한다. 모델 path 선택을 명시적으로 확장한 뒤 text-spike를 실행하며 기존 `next`를 새4B 명령으로 오인하지 않는다. 한국어 의미·근거·schema를 분리 판정한다.
- [ ] P5: 기존 Rust 2.5D GLB round-trip부터 검증한다. TripoSR는 Rust runtime 확인 후8GB shape 실험 후보, Hunyuan3D-2mini는 한국 제외 라이선스로 보류한다.
- [ ] 마지막 검증: 실행 담당은 변경 코드에 `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`를 적용한다. 실제 output·실패·timeout·메모리 표본을 기존 증거 체계에 기록하고 연구 추천과 혼합하지 않는다.

공통 선행조건은 preflight·CPU4/interop1·무거운 worker1개·BelowNormal·오프라인 실행·사용자 앱 보존이다. 새 CLI 인자/adapter는 구현 전 제안이며 아직 실행 명령이 아니다. Pages/VM/DPI는 범위 밖이다.

재검증 완료: [실행 준비 계약](../../sots/player/infrastructure/inference/research-followups/execution-readiness.md)에서 실제 `speech-archive`/`asr-japanese`/`text-spike next`와 미구현 align/track/새4B 선택을 구분했다. `speech-fixture ja_jp`는 존재하지 않는 입력이므로 삭제했다. 실제 파일크기·LFS hash·라이선스 원문·전처리·gated401·추가 의존성의 잔여 게이트를 기록했다. 연구 목표 완료는 후보 추론·제품 통합 완료를 뜻하지 않는다.

별도 provenance 목표: [6개 실행 모델군의 권리 조사](../../sots/player/infrastructure/inference/research-followups/provenance-licenses.md)를 완료했다. 출시 전 남은 작업은 lmz 변환 원본/고지, OCR 사전·원본 checkpoint 대응, YAMNet 가중치 원문·class map 권리, segmentation gated LICENSE·INT8 계보, WeSpeaker 직접 원본·CC attribution 확보다. 모델 다운로드·추론·설치는 하지 않았다. 원문 미확인/계보 불명을 통과로 바꾸지 않으며 기존 실행 검증 완료와 별도 추적한다.

최신 Rust ASR 목표: [고정 소스 실행 경로](../../sots/player/infrastructure/inference/research-followups/rust-asr-runtime.md)에 alan0.2.2 CPU safetensors와 voxora0.6.3 동일 엔진 adapter를 선정했다. Rust ASR 구현 부재라는 차단은 해소됐으며 Windows build·tokenizer.json 준비/parity·실제 입력 검증이 다음 작업이다. Alkd GGUF 카드 명령은 현행 source와 불일치하므로 보류한다. ForcedAligner classify head/시간 복원은 조사 후보에서 미확인으로 ASR과 분리한다. 본 연구는 작은 소스·metadata 조회만 했고 빌드/추론/모델 다운로드는 하지 않았다.

추가 독립 정렬 목표: [Rust ForcedAligner 재조사](../../sots/player/infrastructure/inference/research-followups/rust-forced-aligner-runtime.md)에서 lumosimmo·eclipse·QwenASR·valle의 실제 classify head/시간 복원을 확인했다. 앞선 미확인은 당시 후보 범위로 한정한다. JP/KO 우선 후보는 lumos(분절/권리/Windows 게이트 선행), 영어 통제군은 valle(JP/KO 명시 거절)다. eclipse는 구현·내장 nagisa 모델 권리 미확인, QwenASR는 Unix직접 의존·JP/KO문자분절로 보류한다. source읽기만 했고 clone/weights다운로드/build/inference는 하지 않았다.

lumos 사전 보충 목표: [KO/JP dictionary readiness](../../sots/player/infrastructure/inference/research-followups/lumos-dictionary-readiness.md)에서 lock의 Lindera2.2.0→IPADIC20250920 URL·expected MD5·HEAD12,907,073 bytes와 COPYING, KO 사전의 Qwen 원본과 동일 Git blob, JP merge6개 유래 불명을 구체화했다. 다운로드/build/inference 금지는 이 연구 세션의 담당 분리이며 root는 기존 승인 범위에서 단일 worker로 실행한다. 새 사용자 승인을 요구하지 않는다. 현행 forced-aligner feature는 KO-only 입력에도 IPADIC build fetch를 유발할 수 있다. 로컬 비배포 검증의 사용권/고지 보존과 최종 제품 배포 고지 패키징은 별도 게이트이며 후자의 미완성만으로 전자를 동일하게 막지 않는다.

실제 zero-duration 후속 원인 목표: [lumos JP zero 분석](../../sots/player/infrastructure/inference/research-followups/lumos-zero-duration-analysis.md)은 root의 KO19개 구조통과·JA1516 zero2/28·JA1634 zero3/31을 읽고 config80ms·일본어 숫자분할·비감소 LIS repair와 연결되는 증거/가설을 분리했다. raw class/pre-repair 값이 없어 실제 생성 단계를 확정하지 않았다. 다음 단일 변수는 진단 raw/fixed trace, 분절 중간 단계, JA1634의 user term7000 한 항목 비교 순서다. 연구 세션은 source/출력 읽기만 했고 새 추론/보정/코드 변경은 없다. 구조 실패 needs_review와 품질 미승인을 유지한다.
