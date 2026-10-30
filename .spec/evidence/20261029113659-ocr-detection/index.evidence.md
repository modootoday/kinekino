---
id: evidence:kinekino/20261029113659
title: OCR 텍스트 영역 검출 스파이크
status: active
created: 2026-10-29
updated: 2026-10-29
---
# OCR 텍스트 영역 검출 스파이크

2026-10-18. 기존 [OCR 관측](vision.md)은 수동으로 좁힌 렌더 자막 ROI의 recognition이었다. 이번 후속은 전체 영상 프레임에서 텍스트 위치를 찾는 detection 단계이며 recognition·추적·자막 생성과 구별한다.

후속 현재 상태: [Rust 후처리·한국어 자동crop 연결](ocr-postprocess.md)과 [자연 크레딧30crop 일괄 인식](ocr-natural-credits.md)까지 실제 실행했다. 이 문서의 map2개 관측은 최초 단계이며, 아래 후처리 계획을 현재 구현0회로 해석하지 않는다. 원본 후처리 수치 parity·corpus 품질·제품 OCR API는 여전히 미검증/미구현이다.

## 고정 후보와 입력 계획

공식 HF `PaddlePaddle/PP-OCRv6_small_det_onnx`, revision `28fe5895c24fd108c19eb3e8479f4ab385fbfc62`를 사용한다. `inference.onnx`는9,880,512bytes/LFS SHA256 `d73e0058b7a8086bbd57f3d10b8bcd4ff95363f67e06e2762b5e814fe9c9410e`다. 기존 고정 catalog의 model card는 Apache-2.0을 선언한다. 별도 LICENSE 파일 유무와 제품 재배포 검토는 로컬 실행과 분리한다.

취득은 기존 Rust fetch-record·단일 root worker·CPU<60%/여유12GiB를 사용하고 model/YAML/README를 고정 revision에서 기록한다. 첫label `ocr-det-v6-small-r1`은 CPU100%로 취득 전에 거부됐다. 사용자 앱/작업을 종료하거나 admission을 완화하지 않았고 모델 다운로드·추론0회다. 새label r2 결과는 별도로 기록한다.

입력은1) 기존 자막이 없는 Sintel110초 프레임,2) 기존 렌더 한국어 자막 프레임,3) 원본 영화의 크레딧 텍스트가 있는 프레임을 계획한다. 원본 배포 영상의 크레딧은 자연 영상 내 인쇄 텍스트 입력이며 한국어·일본어 자연 번인 자막의 대체 정답이 아니다. 새 크레딧 프레임은 공통 SDK `production.capture_frame`으로 추출하고 source PTS/receipt를 보존한다. 음성 ASR 정답이나 문서 OCR benchmark로 전용하지 않는다.

## 단계별 게이트

1. 배포 YAML의 resize/channel/scale/mean/std와 실제 ORT 입출력 shape를 확인하고 Rust 전처리를 구현한다.
2. 실제 map 출력과 유효 범위·원본 좌표 복원 정보를 보존한다. graph load나 heatmap 출력만으로 텍스트 검출 기능 성공을 선언하지 않는다.
3. Rust contour·회전 상자·unclip·score filter를 공식 DB postprocess와 대조한다. 외부 Rust 라이브러리를 우선하고 OpenCV/Shapely 원본과의 수치 차이를 별도 검증한다.
4. 검출 상자로 recognition을 연결하고 실제 텍스트·위치·false positive/누락·여러 프레임을 평가한다. rendered positive와 자연 영상 결과를 구분한다.

임계값·회전·문맥·추적을 정답에 맞춰 묵시적으로 변경하지 않는다. 새 실행마다 source/EXE·입력·모델 hash를 고정하고 raw 출력·실패·미검증을 보존한다. GUI/CLI 제품 OCR는 이 standalone spike로 supported가 되지 않는다.

## 취득·구현 준비 관측

새label `ocr-det-v6-small-r2`는 CPU41.015625%/여유37,326,282,752bytes에서 모델·YAML·README를 순차 취득했다. 모델9,880,512bytes의 LFS SHA가 일치했다. [YAML](../../../evidence/ai/ocr-detection-2026-10-18/inference.yml)885bytes SHA256 `193f435274bf9f0b5f71a929bbfbcf148282df7e633b34e7c373e8f44741b516`, [model card](../../../evidence/ai/ocr-detection-2026-10-18/model-card.txt)16076bytes SHA256 `e9403667f0f92055d8ce9e73a2beb1002fc489d9342697db74039eb7178bb5fb`를 기록·보존했다. 이 단계는 실제 추론0회다.

배포 YAML은 BGR·1/255·mean[.485,.456,.406]/std[.229,.224,.225]·CHW와 DetResizeForTest 기본값을 사용한다. 현재 [공식 resize 구현](https://github.com/PaddlePaddle/PaddleOCR/blob/main/ppocr/data/imaug/operators.py)의 기본 min736/max4000/nearest32를 참고해 명시했고 ties-even과 가로세로 비율 경계를 Rust 단위 검사로 확인했다. upstream main 관측은 해당 ONNX export 당시 commit·OpenCV 수치 parity의 증거가 아니다. Rust image Triangle resize와 OpenCV bilinear의 수치 대조는 미검증이다.32px 미만 입력은 이번 role에서 명시 거부한다.

새 `ocr-det-map` role은 원시 확률 map의 little-endian f32와PNG16 시각화·shape·범위·좌표크기를 보존하는 첫 단계다. `inferred-map-only` 상태이며 contour·상자·unclip·recognition을 구현했다고 표시하지 않는다. 두 고정1280×544 입력의 resize는1728×736이며 threshold는 YAML0.2 그대로다. box_thresh0.45/unclip1.4/max_candidates3000의 후처리는 다음 별도 단계다.29tests/fmt/Clippy/build와source/EXE선보존 후 새label로 실제 실행한다.

원본820초 크레딧 캡처는 공통 SDK기본30000ms에서 timeout으로 실패했고 PNG를 게시하지 않았다. [원시 응답](../../../evidence/ai/model-spikes-2026-10-18/results/ocr-natural-credit-capture-response.json)을 보존하고 SDK 세션에서 정확한 원본PTS를 유지하는 input-seek 개선을 분리 진행한다. 실패를 낮은해상도·완화된PTS·더긴deadline으로 숨기지 않는다. 이번 첫 추론 입력은 기존 원본 프레임과 렌더 한국어 자막 프레임이며 자연크레딧·자연번인 결과로 표시하지 않는다.

## 실제 map 추론

29tests/fmt check/Clippy all-targets-Dwarnings/locked offline build 통과 후 EXE `e3e85ca9aac63176a70a974e0901cfd4e059834f638da3f44ad0fb130983f698`의82파일을 실제 추론 전에 보존했다. manifest SHA256 `d572cf6e32a91c5c3fb8978d56bf9502cf77a9f33895aa522125aae9bee05ebb`다. 새 source는 domain resize 규칙·infrastructure ORT/영상 전처리/원시 산출물·application 고정 관측 gate를 분리한다.

| 관측 | 렌더 한국어 자막 | 원본 자막 없는 프레임 |
|---|---:|---:|
| label | ocr-det-rendered-ko-r1 | ocr-det-plain-frame-r1 |
| admission CPU % | 17.94374393792435 | 17.67578125 |
| admission 여유RAM bytes | 37,658,677,248 | 37,884,776,448 |
| 입력/출력 map 크기 | 1728×736 | 1728×736 |
| 실제 output shape | [1,1,736,1728] | [1,1,736,1728] |
| map 최소/최대 | 0 / 0.9997515678405762 | 0 / 0.9039666652679443 |
| YAML0.2 초과 픽셀 수 | 6845 | 8315 |
| raw f32 SHA256 | 8f3cde7e0e1a69d62a1140b86e9e1c5dc83dce9cc58d74bdd6a9da7ffeb51eba | bce58199d004a378652546b5f8d099cabe0b55527d7d8bea75414dffb3750442 |
| 표본 private peak bytes | 328,536,064 | 321,495,040 |

두 worker는 순차1개/BelowNormal/CPU4interop1/오프라인으로 success=true/exit0/timeoutfalse 종료했다. 원 ready=false/player1개도 유지했고 다른 앱을 종료하지 않았다. performance_comparison_eligible=false이므로 시간·메모리 순위로 사용하지 않는다. 두 입력은 원본1280×544이고 SHA256은 원본 `691c0c6224fe1ab4ba2e3f38efac75761951386e2b30670584791535e2c7b9c7`, 렌더 `cf4c9a1473423bc659f9f1f00bc4ca600b5abf5f8fb2cc690c1e95aeaf414977`다.

root가 원본·렌더·두 확률 PNG를 실제 열어 확인했다. 렌더 입력에서는 하단 자막 줄에 강한 연속 응답이 보였다. 자막 없는 동일 장면은 상단/왼쪽의 무기·배경 질감 쪽에 분산 응답이 있었다. 원본 쪽의 threshold 초과 픽셀 수가 더 많으므로 단순 픽셀 수·최댓값을 텍스트 존재나 검출 정확도로 사용하지 않는다. 이 검토는 정답 polygon과 계산한 precision/recall/IoU가 아니다.

실행/stdout은 [보존 결과](../../../evidence/ai/model-spikes-2026-10-18/results/), f32와PNG16은 [검출 map 자산](../../../artifacts/spikes/2026-10-18/ocr-detection/)에 둔다. 원시 float를 PNG16 양자화 이미지로 대체하지 않는다. 다음은 저장된 map의 contour·회전상자·score/unclip·원본좌표 복원과 recognition 연결이다. 아직 전체 OCR 검출 기능·자연 번인 자막·제품 API 지원은 미완료다.
