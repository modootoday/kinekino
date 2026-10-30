---
id: evidence:kinekino/20261029113655
title: 저장된 OCR map의 Rust 후처리
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 저장된 OCR map의 Rust 후처리

2026-10-18. [실제 detector map](ocr-detection.md)을 다시 읽어 contour·회전 상자·score·unclip·원본 좌표·crop을 만드는 후속 단계다. 모델을 재실행하지 않으며 원시 f32 map을 사용한다. PNG16 미리보기로 수치를 대신하지 않는다.

## 구현과 검증 범위

Rust `imageproc=0.27.0`의 Suzuki contour·polygon rasterization·perspective와 `geo=0.33.1`의 최소 회전 사각형·round buffer를 고정했다. 모델 도구의 독립 Cargo manifest/lock만 변경하고 제품 Cargo/API/UI에는 새 의존성을 넣지 않는다. detector map threshold0.2·box score0.45·unclip ratio1.4·최대3000은 배포 YAML 그대로다. 사각형 area×ratio/perimeter 거리로 확장하고 원본 좌표에 복원한다.

이 경로는 **Rust DB-style prototype**이다. OpenCV contour 순서/픽셀 포함·minAreaRect tie 처리와 Pyclipper buffer의 수치 parity를 아직 증명하지 않았다. 공식 구현과 동등한 검출 품질로 표시하지 않는다. synthetic rectangle의 확장·원본좌표 복원·낮은 score 거절·비정상 map 거절을 검사한 뒤 실제 저장 map에서 결과를 관측한다.

application은 source label·입력/map SHA·파일 경계·크기·new output을 확인하고, infrastructure는 geometry·crop/overlay 인코딩을 담당한다. 외부 source JSON을 프로그램/명령으로 실행하지 않는다. 원본 map·입력 이미지를 변경하지 않는다. 새 output 디렉터리와 INCOMPLETE 표시로 중간 실패를 남기고 같은 label을 덮어쓰지 않는다.

관측 전에 텍스트 bbox를 수동으로 주거나 crop을 좁히지 않는다. 결과 region에서 perspective crop을 만들고 실제 열어 자막/배경 응답을 검토한다. 아직 recognition 재실행·여러 frame 병합·자연 번인 자막·제품 OCR 완료를 의미하지 않는다. 실제 결과와 build identity는 검증 후 추가한다.

## 실제 후처리와 시각 확인

30tests/fmt check/Clippy all-targets-Dwarnings/locked offline build 통과 후 EXE `889cf486d80dfa12fdf00bf9da737aef59081bfcbe426013e1cfac845e090d1b`의85파일을 선보존했다. manifest SHA256 `df7f2cf8364d740e9da38f9d2ad9668b039e026bcd0e3b57229fbacaa41256c6`다. 실제 두 offline 후처리 결과에 EXE/manifest·원시 source stdout/map·입력 이미지 SHA를 기록했다. 신경망 실행은0회다.

| source label | 후처리 label | 후보 수 | 관측 |
|---|---|---:|---|
| ocr-det-rendered-ko-r1 | rendered-ko-r1 | 1 | score0.8419617487386073, quad [[472,495],[807,496],[807,525],[471,524]], perspective crop337×30 |
| ocr-det-plain-frame-r1 | plain-frame-r1 | 0 | 원시 map에는 높은 응답이 있었지만 고정 score/기하 필터 후 후보없음 |

root가 실제 crop과 원본 위 overlay를 열어 자막 줄 전체가 포함됨을 확인했다. crop SHA256은 `f8a796f5be67238b0185e1fc978017bc734d52644996fa8b2a9c5db3c4b0865f`, positive overlay는 `8b60ac81b2a700b1ce2d204bc5fb6acdf769024aacfb77df0b998d848c69b781`다. 수동 ROI가 아니라 detector map에서 자동 추출했으며 임계값을 이 두 결과에 맞춰 바꾸지 않았다. 한 장의 positive/negative는 일반화된 검출 precision/recall이 아니다.

결과는 runtime/ai-spike/results/ocr-postprocess-{label}.json, crop/overlay는 assets/ocr-boxes/{label}에 원본을 보존하고 report로 evidence/artifacts에 복사한다. 이후 한국어 recognition은 이 crop SHA·기존 model/dict SHA를 고정한 별도 관측 role로 검증한다. 해당 모델 입력 전source/EXE는 새 build로 다시 보존한다. 이 후속 추론과 위 offline0회를 혼합하지 않는다.

## 자동 crop의 실제 recognition 연결

고정된 model `monkt/paddleocr-onnx` revision `7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65`의 Korean PP-OCRv5 mobile rec SHA256 `322f140154c820fcb83c3d24cfe42c9ec70dd1a1834163306a7338136e4f1eaa`, dict SHA256 `a88071c68c01707489baa79ebe0405b7beb5cca229f4fc94cc3ef992328802d7`와 위 crop SHA만 허용하는 별도 observation gate를 추가했다. 기존 recognition 전처리/모델/dictionary를 바꾸거나 수동 crop을 추가하지 않았다.

30tests/fmt/Clippy/build가 통과한 EXE `070d881fdb5388ec158b456b7864231f62f226c1459d6c04d81054e3b88c99af`의85파일을 추론 전에 보존했다. manifest SHA256 `cf432304eab07a41f1854539a74bcb1230684dee9b72c9fdef35a039746b85cd`다. 실제 label `ocr-auto-crop-ko-r1`은 CPU29.58984375%/여유36,683,522,048bytes에서 BelowNormal/4threads/interop1/오프라인 단일worker로 success=true/exit0/timeoutfalse 종료했다.

실제 출력은 **`이 칼날에는 어두운 과거가 있다.`**로, 이 렌더 fixture의 문구와 같았다. 입력은 자동 검출된337×30 crop이며 recognition tensor는 height48/width540/RGB다. 실제 ORT graph input은 [-1,3,48,-1]로 확인했다. 배포 config.json의 height32 안내와 차이가 있지만 immutable 외부 파일을 수정하거나 실제 graph를32로 억지 변경하지 않았다. 해당 변환 metadata는 graph 계약과 구별한다.

따라서 이 한 rendered positive/negative에서는 전체프레임→detector map→Rust DB-style 후보/원근crop→Korean recognition을 실제 연결했다. 이 사실을 corpus OCR 정확도·자연 번인·다국어·OpenCV/Pyclipper 수치 parity·연속 영상·제품 API 완료로 확대하지 않는다. recognition 원시 stdout·실행 JSON과 각 단계 SHA는 [보존 결과](../../../evidence/ai/model-spikes-2026-10-18/results/)를 따른다.
