---
id: evidence:kinekino/20261029114349
title: root의 실제 영화 후반부 프레임 캡처 재검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# root의 실제 영화 후반부 프레임 캡처 재검증

2026-10-18. OCR 자연 입력을 준비하며 공통 SDK production.capture_frame의 기존 전체디코드 방식이 원본820초/default30000ms에서 timeout을 반환했다. [SDK 담당 수정·합성 인수](frame-capture-seek-2026-10-18.md) 후 root가 새 CLI를 보존하고 같은 원본 PTS/기본deadline의 실제 미디어 호출을 수행했다.

## 실제 root 관측

CLI SHA256은 인계와 같은 `dc2b265922b9913e47b92b08699017ebc26a3de8f2cce5cdde78faacf943849c`였다. [root 별도 evidence](../../../evidence/player/frame-capture-seek-root-2026-10-18-r1/)에 EXE·새 request/response·stderr·exit를 보존했다. 기존 timeout request/response와 처음 output 경로는 변경하지 않았다. 새 ID `ocr-credit-seek-r1`·새 PNG `sintel-credits-820-seek-r1.png`를 사용했다.

| 항목 | 실제 반환 |
|---|---|
| exit / response | 0 / ok=true |
| 요청 PTS us | 820000000 |
| SDK가 반환한 실제 PTS us | 820000000 |
| 원본 ticks / time_base | 820000 / 1/1000 |
| PNG | 1280×544,1,657,536bytes |
| timeout | 기본30000ms 유지 |
| PNG SHA256 | 836cb932003eb058aac3905e660b191a8c8c76a3e66e67f2a64d7c8250704943 |

root가 저장된 PNG를 직접 열어 Premium Sponsor/DivX 등의 원본 크레딧 텍스트를 확인했다. 무음/텍스트 렌더를 새로 넣지 않았고 별도 자막 트랙은 제외한다. [보존 이미지](../../../artifacts/spikes/2026-10-18/fixtures/ocr/sintel-credits-820-seek-r1.png)는 자연 영상 내 인쇄 텍스트 자료이며 자연 한국어·일본어 번인 자막과 다르다.

실제 반환 PTS는 SDK receipt의 값이다. root가 이번 실미디어의 전체 ffprobe frame-index/pixel 참조를 별도로 재계산했다고 표시하지 않는다. 담당의 작은 CFR/VFR/offset/keyframe/B-frame 독립 기준 검사는 위 담당 정본에 있고, root는 이 실제 후반부 호출·이미지·EXE 대응을 추가 확인했다. wall시간으로 성능 순위를 만들거나 모든 GOP/컨테이너의30초 성공을 보장하지 않는다.

## 범위와 남은 작업

default/max deadline을 늘리지 않고 source PTS·새 PNG·파일전용/강제 demux·외부 참조 제한·no-overwrite·child cleanup 계약을 유지한 수정이다. 정확한 source 변경·검사는 담당 정본의 범위다. root는 SDK source를 추가 수정하지 않았고 capture 목적의 GUI 실행·앱 종료·VM/DPI·endurance 작업을 하지 않았다.

새 PNG의 OCR detector/recognizer 적용은 별도 실측으로 남긴다. 현재 공통 SDK 지원5개/미지원47개를 이 캡처 성공으로 늘리지 않는다. GUI 현재 프레임 캡처·공통 playback facade 연결·모델/작업 API는 여전히 별도 인수 대상이다.
