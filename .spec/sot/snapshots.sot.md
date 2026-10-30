---
id: sot:kinekino/snapshots
title: 스냅샷과 썸네일
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 스냅샷과 썸네일

하위화면의 형태·컴포넌트 구성은 [시각 계약](snapshots-visual.md)을 적용한다.

## 시각화

- [원본 프레임](../../../../artifacts/design/v2/studio/raw-frame.png)
- [썸네일 편집](../../../../artifacts/design/v2/studio/snapshot-refined.png)

이미지는 배치 참고이며 아래 요소 계약과 공통 필드·상태 규칙이 구현 기준이다.

상태: UI/UX 설계 승인(2026-10-18). 생성 이미지 디테일은 실제 Rust 공통 컴포넌트로 검토한다. 구현 완료와 구분한다. [공통 계약](../sot-020-ui-contract.md)의 프레임·입력 검증·상태·접근성·저장·Rust 계층 규칙을 상속한다. 기본/hover/pressed/selected/disabled/focus는 모든 조작 요소에 적용한다.

## 요소 계약

| 요소 ID | 요소 | 표시와 동작 |
|---|---|---|
| FRAME-01 | 모드 | 원본 프레임/썸네일 편집 |
| FRAME-02 | 프레임 고정 | 디코딩 PTS·해상도·회전·색공간 기록 |
| FRAME-03 | 이전/다음 프레임 | 실제 프레임 단위, 정지 미디어에서 비활성 |
| FRAME-04 | 자막 포함 | 분리 자막만 끄기 가능, burn-in 제외 불가 안내 |
| FRAME-05 | 원본 비교 | 원본과 편집을 같은 좌표로 전환 |
| FRAME-06 | 크롭/맞춤 | 16:9/9:16/자유 비율·resize·fit/fill |
| FRAME-07 | 텍스트 | 내용·폰트·크기·정렬·색·외곽선·배경 박스 |
| FRAME-08 | 레이어 | 순서·숨김·잠금·복제·삭제·undo |
| FRAME-09 | 피사체 | 검토 mask 참조, 원본 보존 |
| FRAME-10 | 글꼴 대체 | 누락 글꼴/한글 줄바꿈·넘침 표시 |
| FRAME-11 | 색 | sRGB 출력 기본 제안, HDR tone-map 별도 확인 |
| FRAME-12 | 원본 저장 | PNG 별도 파일, UI/OSD 포함 금지 |
| FRAME-13 | 편집본 저장 | JPG/PNG·크기·품질, OUT |
| FRAME-14 | 문구 제안 | 선택 cue 근거·초안 표시·수락/되돌리기 |

## 상태와 오류

프레임 미준비는 고정/저장 비활성. 광고/브라우저 캡처를 원본 영상 프레임으로 저장하지 않는다. 원본 프레임과 편집본을 분리하며 잘못된 폰트·텍스트 넘침은 저장 전 경고한다.

## 저장 계약

FrameAsset(source_pts,dimensions,color,rotation,subtitle_mode), ThumbnailDocument(layers,crop,text,preset).

## 인수 기준

정확 PTS·자막 선택·HDR·회전·UI 미포함·한글 넘침·원본 해시 보존.

각 요소 ID의 유스케이스는 application에, 검증 정책/값 객체는 domain에, OS·추론·파일·프로세스는 infrastructure에 분리한다. presentation은 입력과 렌더링을 담당한다. 모델 미선정/encoder·플랫폼 지원 여부 미확정 항목은 완료로 표시하지 않고 해당 기능을 비활성 또는 NeedsInput으로 제공한다.

