---
id: sot:kinekino/8gb
title: 추적·마스크와 8GB 에셋 제작 후속
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 추적·마스크와 8GB 에셋 제작 후속

## 마스크와 추적

기존 관측은 Candle MobileSAM 단일 frame과 point→PNG/overlay다. 정답 mask IoU나 video ID 안정성 검증이 아니다. 먼저 같은 source hash에서 10~30초 구간을 고르고 시작·가림·재등장·장면 전환 frame에 수작업 정답을 만든다. 원본 좌표/resize/pad/음성 point의 변환을 고정하고 IoU·경계 누락·배경 유출·ID switch·장면 reset을 각각 평가한다.

[EdgeTAM 공식 구현](https://github.com/facebookresearch/EdgeTAM)은 PyTorch 기반 SAM2 계열 video memory 모델이다. [HF 원본](https://huggingface.co/facebook/EdgeTAM)은 Apache-2.0 `edgetam.pt`를 제공한다. 공식 모바일 성능 주장은 다른 장치/실행기 결과이며 이 Windows 호스트의 속도가 아니다. [Transformers 문서](https://huggingface.co/docs/transformers/model_doc/edgetam) 지원도 Rust 지원 증거가 아니다.

다음 실행 게이트는 image encoder·prompt/mask decoder뿐 아니라 memory encoder·memory attention·object pointer·frame index·reset을 처리하는 Rust 실행 경로 확보다. 검증된 고정 ONNX export가 있으면 ORT 어댑터를 우선 비교하되 export provenance·dynamic shape·연속 frame parity가 필요하다. 현재 이 조사에서 검증된 Rust EdgeTAM video runner는 확보하지 못했다. 공식 checkpoint만 받은 뒤 inference 성공으로 표시하지 않는다.

그 전에는 기존 MobileSAM을 keyframe마다 다시 실행하고 사용자가 마스크를 수정하는 경로를 완성하는 편이 현실적이다. 재실행을 temporal tracking으로 표시하지 않는다. feature matching/optical flow의 Rust 구현도 장면 전환·가림에서 drift할 수 있으므로 동일 정답 구간을 통과해야 한다. 모델 memory는 frame 수에 따라 늘 수 있어 단일 frame peak로 video 예산을 보장하지 않는다.

## 2.5D 우선, 완전 3D 별도

| 순서 | 산출물 | 실행·예산·권리 게이트 |
|---|---|---|
| A | 기존 mask+상대 depth로 plane/layer/depth projection GLB | 기존 Rust image/glam/gltf-rs와 ORT V2 depth 재사용. 배경 구멍·edge stretch·alpha·UV·한글 metadata·GLB import/render round-trip 검사. 미터 깊이/가려진 뒷면 복원으로 표시하지 않음 |
| B | TripoSR 단일 이미지 mesh | [공식 README](https://github.com/VAST-AI-Research/TripoSR)는 기본 단일 입력 약6GB VRAM 제시. [HF](https://huggingface.co/stabilityai/TripoSR)는 MIT. 실제8GB 호스트 적합은 미실측. 공식 Python/PyTorch+torchmcubes 경로이며 검증 Rust runtime 없음 |
| C | Hunyuan3D-2mini shape | [공식 원본](https://huggingface.co/tencent/Hunyuan3D-2mini)은0.6B shape 후보. [공식 저장소](https://github.com/tencent-hunyuan/hunyuan3d-2)의 일반6GB shape/16GB shape+texture 안내를 mini 전체 pipeline 실측으로 일반화하지 않음. 한국 적용 라이선스 게이트로 실행 보류 |

중요 권리 발견: 고정 revision의 [Hunyuan LICENSE](https://huggingface.co/tencent/Hunyuan3D-2mini/raw/f90a0f7df7d5e6f71109cf333f6a95a0ae3194a6/LICENSE)는 한국·EU·영국을 허용 Territory에서 제외한다(서두와1.l,5.c). 한국 사용자/배포 경로에서 기본 허용 후보로 취급하지 않는다. 별도 유효 권한이 확인되기 전 다운로드·실행·번들 계획에서 보류한다. 메타데이터의 open weights나 작은 모델 크기로 이 게이트를 대체하지 않는다.

TripoSR는 권리 면에서 먼저 검토할 완전3D 후보지만 실행기는 아직 차단점이다. Rust 전처리/추론/mesh extraction 포팅 또는 고정 native export를 확보하고 참조 parity를 입증해야 한다. Python 참조 코드를 새 프로젝트 정책/검증 코드로 추가하지 않는다. 단순 외부 프로세스 wrapper만으로 기존 Python 코드 전체를 Rust 구현이라고 부르지 않는다.

로컬 코드 재확인: `infrastructure/asset_package.rs`의 기존 `asset-package`는 foreground/background PNG와 `projection.gltf`·`plane.bin`을 만들고 gltf-rs로 파싱한다. 깊이 메시·GLB·렌더 round-trip을 이미 구현했다는 뜻은 아니다. 위 A의 depth/GLB 작업은 후속 제안이다. 신규 모델별 실제 파일·전처리·소스 commit은 [실행 준비 계약](execution-readiness.md)을 따른다.

8GB 실험 계획은 작업1개, 해상도/geometry resolution/texture resolution 고정, 각 단계 종료 후 unload, texture 생성 별도다. weights+activation+geometry grid+mesh extraction+렌더러 동시 점유를 측정한다. 우선 shape 또는 vertex color까지만 비교하고, texture까지8GB라고 약속하지 않는다. 원본 입력 권리·생성 결과 재배포·허상 뒷면/토폴로지/면수/법선/재질·GLB 재열림을 별도 승인 게이트로 둔다. 이번 조사에서3D 모델을 실행하거나 결과를 만든 적은 없다.
