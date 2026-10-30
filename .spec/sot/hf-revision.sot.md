---
id: sot:kinekino/hf-revision
title: HF 고정 revision과 파일 게이트
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# HF 고정 revision과 파일 게이트

2026-10-18 HF 공식 `/api/models/{repo}` 및 `/api/datasets/{repo}` JSON의 sha/cardData/gated/siblings를 직접 조회했다. 모델 다운로드·load·추론은 하지 않았다. 아래 라이선스 표시는 카드 메타데이터이며 최종 배포 권한 판단을 대체하지 않는다.

| 저장소 | 고정 SHA | 라이선스/파일 확인 |
|---|---|---|
| [google/fleurs](https://huggingface.co/datasets/google/fleurs/tree/70bb2e84b976b7e960aa89f1c648e09c59f894dd) | `70bb2e84b976b7e960aa89f1c648e09c59f894dd` | CC-BY-4.0; `data/ja_jp/dev.tsv`, `data/ja_jp/audio/dev.tar.gz` |
| [Qwen/Qwen3-ForcedAligner-0.6B](https://huggingface.co/Qwen/Qwen3-ForcedAligner-0.6B/tree/c7cbfc2048c462b0d63a45797104fc9db3ad62b7) | `c7cbfc2048c462b0d63a45797104fc9db3ad62b7` | Apache-2.0; `model.safetensors`, config/processor/tokenizer 일체. dtype·파일 크기는 다음 게이트 |
| [Qwen/Qwen3-1.7B-GGUF](https://huggingface.co/Qwen/Qwen3-1.7B-GGUF/tree/90862c4b9d2787eaed51d12237eafdfe7c5f6077) | `90862c4b9d2787eaed51d12237eafdfe7c5f6077` | Apache-2.0; 실제 파일 `Qwen3-1.7B-Q8_0.gguf`. Q4 없음 |
| [Qwen/Qwen3-4B-GGUF](https://huggingface.co/Qwen/Qwen3-4B-GGUF/tree/bc640142c66e1fdd12af0bd68f40445458f3869b) | `bc640142c66e1fdd12af0bd68f40445458f3869b` | Apache-2.0; `Qwen3-4B-Q4_K_M.gguf`, 2,497,280,256 bytes |
| [facebook/EdgeTAM](https://huggingface.co/facebook/EdgeTAM/tree/14d7ecc48c656b94e5184519f698cd5386c5a2bf) | `14d7ecc48c656b94e5184519f698cd5386c5a2bf` | Apache-2.0; `edgetam.pt`; 원본 checkpoint이며 Rust/ONNX 실행 보장은 아님 |
| [pyannote/speaker-diarization-community-1](https://huggingface.co/pyannote/speaker-diarization-community-1/tree/3533c8cf8e369892e6b79ff1bf80f7b0286a54ee) | `3533c8cf8e369892e6b79ff1bf80f7b0286a54ee` | CC-BY-4.0; gated auto. config 의존 모델도 별도 고정 필요 |
| [stabilityai/TripoSR](https://huggingface.co/stabilityai/TripoSR/tree/5b521936b01fbe1890f6f9baed0254ab6351c04a) | `5b521936b01fbe1890f6f9baed0254ab6351c04a` | MIT; `config.yaml`, `model.ckpt`; checkpoint 자동 실행 금지 |
| [tencent/Hunyuan3D-2mini](https://huggingface.co/tencent/Hunyuan3D-2mini/tree/f90a0f7df7d5e6f71109cf333f6a95a0ae3194a6) | `f90a0f7df7d5e6f71109cf333f6a95a0ae3194a6` | API license=`other`; Tencent 독자 라이선스. 한국 제외 조항으로 실행 보류 |
| [onnx-community/depth-anything-v2-small-ONNX](https://huggingface.co/onnx-community/depth-anything-v2-small-ONNX/tree/c3b67641fd837b2368757101311e5d21e511441e) | `c3b67641fd837b2368757101311e5d21e511441e` | Apache-2.0; ONNX 변환자 배포. 기존 실측 manifest의 다른 SHA와 혼동하지 않음 |

4B Q4_K_M의 HF LFS SHA256: `7485fe6f11af29433bc51cab58009521f205840f5b4ae3a32fa7f92e8534fdf5`. 후속 재검증에서 aligner·1.7B Q8·EdgeTAM·TripoSR의 size/hash도 수집해 [실행 준비 계약](execution-readiness.md)에 기록했다. 나머지는 추측하지 않는다. 실행 담당은 고정 SHA의 blobs/tree 메타데이터를 보존하고 다운로드 후 Rust SHA256과 대조한다.

이 표는 후속 조사용이며 기존 스파이크 catalog를 갱신·재취득한 것이 아니다. 모델 카드의 광고성 성능을 로컬 관측으로 전환하지 않는다.
