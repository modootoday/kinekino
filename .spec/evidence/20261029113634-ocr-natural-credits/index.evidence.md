---
id: evidence:kinekino/20261029113634
title: 자연 크레딧 프레임 OCR 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 자연 크레딧 프레임 OCR 검증

2026-10-18. [SDK 실제 캡처](../application/root-frame-capture-seek-2026-10-18.md)로 확보한 Sintel 원본820초1280×544 PNG를 사용한다. SHA256은 `836cb932003eb058aac3905e660b191a8c8c76a3e66e67f2a64d7c8250704943`다. 입력에 새 글자를 렌더링하거나 텍스트 위치를 수동 crop하지 않는다. 기존 [렌더 한국어 한 장 연결](ocr-postprocess.md)과 별도 자연 영상 내 인쇄 텍스트 사례다. 한국어·일본어 자연 번인 자막이나 전체 corpus 평가가 아니다.

## 실행 전 범위

같은 PP-OCRv6 detector revision/FP32·BGR 전처리·min736/nearest32·배포 YAML threshold0.2/box0.45/unclip1.4/max3000을 유지한다. 새 입력의 hash만 명시 allowlist에 추가하고 source/EXE를 다시 동결한다. 저장된 f32 map의 Rust DB-style 후처리를 같은 값으로 적용한다. 원본과 overlay/crop을 확인하고 자동 crop을 Korean PP-OCRv5 mobile의 영어 지원 경로로 순차 인식한다. 임계값/상자를 출력에 맞춰 수동 조절하지 않는다.

원본에서 읽을 수 있는 여러 제목·이름을 assistant가 시각 검토할 수 있지만, 이는 인간이 별도로 검수한 gold transcript/polygon이 아니다. 제한된 명확한 문자열 일치·누락/병합 관측과 품질 승인을 구분한다. 정확한 transcript/line polygon 검수 전 전체 CER/precision/recall을 승인 수치로 발표하지 않는다. 모델 card의 영어 지원 선언은 이 입력의 실제 인식 정확도 보장이 아니다.

실제 detector/후처리/recognition 실행·EXE·자원·실패·원문 출력은 아래에 추가한다. 제품 OCR API를 supported로 바꾸거나 GUI/CLI 공통 jobs가 구현됐다고 표시하지 않는다.

## 실제 detector와 자동 crop

30tests/fmt/Clippy/build 통과 후 EXE `3ddf551520a241c55e4ed4d50a7ee348aae0e2a704986b1d976e23324c49c4db`의85파일을 선보존했다. manifest SHA256 `a3396472eaf8b0a2bd89710a81eb09787883dd88f7409a8213158e5cd6ccf2c9`다. label `ocr-det-natural-credits-r1`은 CPU22.37487733071639%/여유37,712,273,408bytes에서 단일worker로 success=true/exit0/timeoutfalse 완료했다. map→Rust 후처리 `natural-credits-r1`은 새 모델 추론 없이30개 region/crop을 만들었다.

raw f32 map SHA256은 `1eb40ce446575ee752ca45f9e93fb764b4e318b906d6e622ad62d6a6e594a4f6`, 후처리 report SHA256은 `1d40b3652315313a545f15ab2dccd2d940b2df65dc71304b19f53721f5834045`다. root가 overlay를 실제 열어 제목과 두 열의 이름들이 줄 단위 후보로 분리됐음을 확인했다. 수동 상자를 추가하거나 원본을 crop하지 않았다. 이 시각 관측만으로 누락0/recall100%를 계산하지 않는다.

## 일괄 recognition 계약

새 `ocr-batch`는 위 고정 report hash·동일 recognizer/dict·crop 경계/각SHA/크기를 검사한 후 단일 ORT session에서 순차 인식한다. crop 최대128·region index 중복/범위 오류를 거부하고 전체 준비 입력을 확인한 뒤 모델을 호출한다. 출력은 각 region 좌표·crop SHA·원시 recognition을 유지한다. geometry의 행 정렬은 두 열의 의미적 읽기 순서 추론이 아니다.

관측 worker1개와 내부30개 crop 호출을30개 새 모델이나30개 워커로 집계하지 않는다. 실제 실행 전에31tests/fmt/Clippy/build·새source/EXE 동결을 완료하고 자원 gate를 적용한다. 실제 recognition 결과는 이어서 추가한다.

## 실제 recognition 결과

31tests/fmt check/Clippy all-targets-Dwarnings/locked offline build 통과 후 EXE `9744ecd9358f491efd627326ad6b76c36bb1e92991f725b93ab2c9a1f1ad1386`의86파일을 보존했다. manifest SHA256은 `c423eb2e0ee3348792c2b39ccb08ff4568cd408ca663312ab678d5e41c407ea0`다. 실제 label `ocr-natural-credits-rec-batch-r1`은 CPU50%/여유36,927,827,968bytes에서 BelowNormal/CPU4/interop1/오프라인·한worker/한session으로30개 crop을 순차 처리했다. success=true/exit0/timeoutfalse로 종료했다.

반환은 crop_count30/model_run_count30이며 각 region에 원본 quad·crop SHA·텍스트·전처리 폭·내부 추론시간을 기록했다. 실제 원문 예시는 다음과 같다. assistant가 출력 확인 후 이름을 수정하거나 대소문자를 정규화해 원문을 덮어쓰지 않았다.

| region | 실제 원문 |
|---:|---|
| 0 | PREMIUM SPONSOR |
| 1 | DivX, Inc. |
| 3 | THP Games |
| 5 | CoreEL Technologies |
| 7 | CG Cookie |
| 14 | BlenderDay/09 |
| 22 | clipland.com |
| 23 | Champlain College FIREHOSE |
| 26 | KommKino Nuremberg |
| 29 | Daniel Coik |

모든30개 item의 원문은 [실행 stdout](../../../evidence/ai/model-spikes-2026-10-18/results/ocr-natural-credits-rec-batch-r1.stdout), worker/자원/EXE 정보는 [실행 JSON](../../../evidence/ai/model-spikes-2026-10-18/results/ocr-natural-credits-rec-batch-r1.json)에 둔다. [자연 crop/overlay](../../../artifacts/spikes/2026-10-18/ocr-boxes/natural-credits-r1/)와 [원본 이미지](../../../artifacts/spikes/2026-10-18/fixtures/ocr/sintel-credits-820-seek-r1.png)를 함께 검토한다.

sampled private peak52,674,560bytes/working set62,873,600bytes였다. performance_comparison_eligible=false이며 wall7.1188724초를 다른 모델·수동ROI·노트북 속도 순위로 사용하지 않는다. 종료 후 전체 시스템CPU89.35546875%였고 사전 admission은 전체호스트 hard cap이 아니다. 사용자 player/ChatGPT/Terminal을 종료하거나 OS/caches를 변경하지 않았다.

이 결과는 원본 자연 크레딧1프레임의 실제 자동 영역 추출→일괄 인식 연결이다. 인간 gold·전체 CER/precision/recall·이름 철자 최종 승인·두 열의 의미적 읽기 순서·시간적 자막 병합·다른 언어/장면·모든 회전/곡면·원본 OpenCV/Pyclipper parity는 미검증이다. 누락0 또는30/30 정확도로 표시하지 않으며 제품 OCR/AI jobs를 supported로 승격하지 않는다.
