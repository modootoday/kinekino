---
id: sot:kinekino/execution-readiness
title: 실행 준비 계약과 재검증 기록
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 실행 준비 계약과 재검증 기록

2026-10-18 재검증. 이 문서는 공식 메타데이터/소스 조회와 로컬 코드·manifest 읽기 결과다. 모델/미디어 다운로드·추론·빌드는 하지 않았다. 이 표에 있는 실제 명령도 이 조사에서는 실행하지 않았다. 파일·코드는 다른 세션이 갱신할 수 있으므로 실행 담당은 실행 시 hash를 다시 고정한다.

## 실제 로컬 CLI와 제안의 경계

소스 기준은 [application.rs](../../../../../../tools/model-spike/src/application.rs), [speech_archive.rs](../../../../../../tools/model-spike/src/infrastructure/speech_archive.rs), [speech_fixture.rs](../../../../../../tools/model-spike/src/infrastructure/speech_fixture.rs), [japanese_asr.rs](../../../../../../tools/model-spike/src/application/japanese_asr.rs), [text_spike.rs](../../../../../../tools/model-spike/src/application/text_spike.rs)다. 다음 명령은 저장소 루트에서 기존 바이너리가 해당 소스로 빌드되어 있을 때의 실제 인자다. 여기서는 빌드하지 않았다.

| 실제 명령 | 입력·전제와 출력 | 실행 시 주의 |
|---|---|---|
| `tools/model-spike/target/debug/player-model-spike.exe speech-archive` | 고정 FLEURS dev archive/TSV → `runtime/ai-spike/assets/speech-ja-archive.json` | 다운로드가 발생할 수 있다. 원래 세션이 이미 실행했으므로 재실행보다 manifest 검토 우선 |
| `tools/model-spike/target/debug/player-model-spike.exe asr-japanese` | 위 manifest의3개 WAV와 기존 Candle Whisper → `runtime/ai-spike/results/asr-japanese.json` | 무거운 추론이므로 원래 세션만 실행. timestamps=false 텍스트이며 aligned_cues_accepted=false |
| `tools/model-spike/target/debug/player-model-spike.exe asr-japanese bounded` | 같은3개 WAV·수정된 기존 Candle 예제 → `runtime/ai-spike/results/asr-japanese-bounded.json` | 원래 세션의 원본 길이 제한 재검증 명령. 초기 결과를 덮어쓰지 않는 label/집계 경로다. 이 조사에서 실행 안 함 |
| `tools/model-spike/target/debug/player-model-spike.exe speech-fixture ja` | `source/fleurs-ja-rows.json`의 첫 row→단일 WAV | row API 실패를 해결하는 archive 명령이 아니다. 허용 언어는 `ja`/`ko`; `ja_jp`/`ko_kr`는 오류 |
| `tools/model-spike/target/debug/player-model-spike.exe text-spike` | `runtime/ai-benchmark/q4-model.json`의 hash 고정 모델, 기존 cue7개·ASR → text-suite-clean | mistralrs-v0.9.4 CPU, context2048, thinking=false. 신규10~20cue fixture 평가는 아직 제안 |
| `tools/model-spike/target/debug/player-model-spike.exe text-spike next` | `download-text-next-q4.json` 첫 GGUF → text-suite-next | 현재 manifest는 unsloth/Qwen3.5-0.8B-GGUF SHA `6ab461498e2023f6e3c1baea90a8f0fe38ab64d0`; 공식4B 선택 명령이 아님 |

`catalog`, `fetch KEY FILE...`, `infer ROLE MODEL INPUT`도 dispatcher에는 존재한다. 그러나 `fetch`는 domain catalog에 등록된 key만 허용하고 catalog manifest가 필요하다. 조사한 공식1.7B/4B가 자동으로 등록됐다고 가정하지 않는다. `infer`라는 일반 명령이 있다고 Qwen aligner/EdgeTAM/TripoSR adapter가 존재하는 것도 아니다. 새 모델 선택·align·DER/JER·video tracking·TripoSR 명령은 미구현 제안이다. 구체적 CLI를 지어내지 않고 구현과 `--help`/인자 검증을 완료한 뒤 이 표를 갱신한다.

## 파일과 라이선스의 교차 확인

고정 SHA는 [revision 목록](revisions.md)을 따른다. metadata byte/hash는 서버가 보고한 값이며 이 조사에서 로컬 가중치 hash를 계산한 결과가 아니다. 다음은 `api/models/REPO/revision/SHA?blobs=true` 조회다. 일반 Git 파일은 lfs.sha256이 없으므로 LFS hash를 만들어 넣지 않는다.

| 후보 | 필요 파일·서버 byte·LFS SHA256 | 라이선스 근거와 남은 게이트 |
|---|---|---|
| Qwen aligner | `model.safetensors` 1,835,544,544; `47831d0e82f96b20e9034dba01a075ee06436654719f6a68289e49f1b65ce0e7`; `config.json`, `preprocessor_config.json`, `generation_config.json`, `tokenizer_config.json`, `chat_template.json`, `merges.txt`, `vocab.json` | 고정 HF README의 Apache-2.0 표기. HF 별도 LICENSE 파일은 목록에 없음. 공식 [실행 코드 LICENSE](https://github.com/QwenLM/Qwen3-ASR/blob/7c6daf77a2421100f5fb066495372c00129d39ff/LICENSE)는 Apache-2.0. 가중치 권리 근거는 카드와 구분 보존 |
| Qwen1.7B Q8 | `Qwen3-1.7B-Q8_0.gguf` 1,834,426,016; `061b54daade076b5d3362dac252678d17da8c68f07560be70818cace6590cb1a`; README/LICENSE | 고정 [LICENSE 원문](https://huggingface.co/Qwen/Qwen3-1.7B-GGUF/blob/90862c4b9d2787eaed51d12237eafdfe7c5f6077/LICENSE) HTTP200·Apache 확인. GGUF tokenizer/chat template metadata와 기존 엔진 load gate 잔여 |
| Qwen4B Q4 | `Qwen3-4B-Q4_K_M.gguf` 2,497,280,256; hash는 revision 표 | 고정 [LICENSE 원문](https://huggingface.co/Qwen/Qwen3-4B-GGUF/blob/bc640142c66e1fdd12af0bd68f40445458f3869b/LICENSE) HTTP200·Apache 확인. 기존0.9.4 실행·한국어 품질 미확인 |
| EdgeTAM | `edgetam.pt` 56,116,523; `ed2d4850b8792c239689b043c47046ec239b6e808a3d9b6ae676c803fd8780df`; HF repo에는 config가 없음 | HF Apache 표기와 공식 [코드 LICENSE](https://github.com/facebookresearch/EdgeTAM/blob/7711e012a30a2402c4eaab637bdb00a521302c91/LICENSE) 확인. YAML/config·source commit·checkpoint 연결과 tensor dtype·ONNX export는 별도 확보 |
| TripoSR | `model.ckpt` 1,677,246,742; `429e2c6b22a0923967459de24d67f05962b235f79cde6b032aa7ed2ffcd970ee`; `config.yaml` 987 bytes | HF MIT 표기와 [공식 MIT 원문](https://github.com/VAST-AI-Research/TripoSR/blob/107cefdc244c39106fa830359024f6a2f1c78871/LICENSE). DINO config·배경 제거·mesh 의존성까지 고정 전 완전 재현 패키지가 아님 |
| Community-1 | HF public metadata의 SHA/gated/card만 확인 | 고정 config raw 요청401. 카드 CC-BY-4.0와 [라이선스 법문](https://creativecommons.org/licenses/by/4.0/legalcode) 참조; gated 가중치 원문·의존 파일·dtype는 미확보. 사용자 동의/권한 후 확인 |
| FLEURS | archive/TSV·WAV hash는 원래 세션 manifest | 고정 [README](https://huggingface.co/datasets/google/fleurs/blob/70bb2e84b976b7e960aa89f1c648e09c59f894dd/README.md)의 CC-BY-4.0 표기와 위 CC 법문. attribution/변형 고지 필요 |

Hunyuan은 [고정 원문](https://huggingface.co/tencent/Hunyuan3D-2mini/blob/f90a0f7df7d5e6f71109cf333f6a95a0ae3194a6/LICENSE)의 한국 제외 때문에 실행 후보에서 보류한다. 파일 정밀도/하위 모델 목록 수집보다 권리 게이트가 먼저다. 기존 depth ONNX는 신규 대안이 아니라 기존 측정 경로이므로 기존 download manifest의 dtype/hash/라이선스를 재사용하며 이 연구 SHA와 혼합하지 않는다.

## 전처리·엔진·메모리 잔여 게이트

아래 수치는 공식 config/소스 사실과 제안 실험 조건을 구분한다. 제안 조건은 품질 기준 승인이나 실측 수치가 아니다.

- Qwen aligner: 고정 [config](https://huggingface.co/Qwen/Qwen3-ForcedAligner-0.6B/blob/c7cbfc2048c462b0d63a45797104fc9db3ad62b7/config.json)는 qwen3_asr architecture, thinker BF16, audio128mel, timestamp_segment_time80을 선언한다. 마지막 값의 단위/복원은 공식 decoder 소스를 따라야 하며 임의 초 환산하지 않는다. [processor](https://huggingface.co/Qwen/Qwen3-ForcedAligner-0.6B/blob/c7cbfc2048c462b0d63a45797104fc9db3ad62b7/preprocessor_config.json)는 feature128, hop160, FFT400, n_samples480000, chunk30, right-pad0이다. 16kHz mono 정규화·mel filter/log scaling·chunk stitching·문자/word segmentation parity는 Rust 포팅 전 확인해야 한다. 기존 Whisper80mel을 복사하지 않는다. 가중치1.84GB에 activation/feature/dtype 승격을 더해 측정하고 첫 입력은10~25초로 제한하는 제안이다. 현재 Rust 지원 확인 실패이며 공식 Python 예제를 실행하지 않았다.
- EdgeTAM: 고정 [transforms](https://github.com/facebookresearch/EdgeTAM/blob/7711e012a30a2402c4eaab637bdb00a521302c91/sam2/utils/transforms.py)는 ToTensor→정사각 resize→mean(0.485,0.456,0.406)/std(0.229,0.224,0.225) normalize를 사용한다. resolution은 config 입력이므로 MobileSAM letterbox나 임의1024를 가져오지 않는다. 좌표·원본 mask 복원·메모리 state parity를 검증한다. 56MB checkpoint가 영상 peak 메모리를 뜻하지 않으며 frame수/object수/memory horizon을 고정한10/30초 비교는 제안이다. Rust ORT는 네이티브 ONNX Runtime 어댑터이고 export만 있으면 자동 동등해지는 것은 아니다.
- TripoSR: 고정 [config](https://huggingface.co/stabilityai/TripoSR/blob/5b521936b01fbe1890f6f9baed0254ab6351c04a/config.yaml)는 cond_image_size512와 facebook/dino-vitb16 config 의존을 선언한다. [run.py](https://github.com/VAST-AI-Research/TripoSR/blob/107cefdc244c39106fa830359024f6a2f1c78871/run.py)의 기본 foreground ratio0.85·MC grid256·texture2048과 배경 제거는 소스 사실이다. [image tokenizer](https://github.com/VAST-AI-Research/TripoSR/blob/107cefdc244c39106fa830359024f6a2f1c78871/tsr/models/tokenizers/image.py)의 normalization과 DINO config fetch를 고정해야 한다. 기본 rembg 의존을 피하려면 이미 검증한 Rust mask로 RGB/alpha 합성과 foreground resize parity를 만들고 no-remove-bg 경로와 비교한다. rembg 모델을 몰래 추가 다운로드하지 않는다. 공식 약6GB는 보장치가 아니며8GB 호스트에서 geometry/texture/renderer 동시 peak를 별도 측정한다. native export/Rust model·mesh extraction은 미구현이다.
- 텍스트: 기존 adapter는 CPU2048 context·paged attention off·thinking false를 사용한다. 최신 mistral.rs 문서와 기존0.9.4 인자를 혼합하지 않는다. 1.7B/4B GGUF의 embedded tokenizer/template·EOS·한국어 cue 입력과 schema를 맞춘 뒤2K baseline을 먼저 측정한다. 4K와 GPU는 adapter 변경 후 별도 실험 제안이며 현재 명령으로 자동 수행되지 않는다.
- 화자: 기존 segmentation/WeSpeaker의16kHz/window/FBank 조건은 [원래 세션 검증](../../../../verification/ai/audio.md)을 참조한다. Community-1과 CAM++의 feature/embedding 차원을 아직 확인하지 않았으므로 교체 가능한 drop-in으로 쓰지 않는다. 새 경량 모델은 원본·변환 SHA와 라이선스가 고정되기 전 조사 후보에 머문다.

모든 신규 후보는 실행 승인/품질 통과 상태가 아니다. runtime이 미확보라면 파일 다운로드보다 작은 Rust adapter의 graph/입출력/decoder 계약 검토가 먼저다. 공통 preflight·단일 worker·unload·샘플 peak 한계는 기존 [스파이크 프로토콜](../spikes/protocol.md)을 따른다.
