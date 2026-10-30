---
id: sot:kinekino/media-preview
title: 공통 미디어 미리보기
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 공통 미디어 미리보기

[r160 실제 실행 검토](../../../verification/presentation/media-overlay-review-2026-10-19.md):1440 SEG/SUB5/OBJ, OBJ 원본→Right 마스크→Right 합성,760 OBJ 스크롤·SUB 변환/OCR·Empty를 확인했다. 좁은 OBJ transport 첫 화면 접근과 공통 transport의 시안 세부 차이, 나머지 실제 미검증은 기록에 남긴다.

## r160 SEG/SUB/OBJ 확장 설계

r158 전체 실제 감사에서 SEG-01a·SUB 영상/ROI·OBJ-01a는 CHROME 빈 측면과 별도 작은 아이콘 toolbar를 사용했다. 기준 skip-settings.png·subtitles.png·ocr-refined.png·subject.png의 영상면/transport 구조와 차이가 있다. 기존 CLIP AI/SAVE/OUT의 공용 media_preview를 확장해 같은 VIDEO_SURFACE·THIN/BORDER·CONTROL radius 프레임과 CHROME transport를 조합한다.

새 `unavailable_with_overlay`는 artboard의 response rect를 fitted_rect로 변환한 실제 contain 영역을 callback에 전달한다. 호출자는 그 영역에서만 OCR ROI, 동일 source/PTS 마스크, 좌측 편집 도구와 우상단 보기 선택을 조합한다. transport는 callback 뒤에 배치해 영상 위 덧그림이 덮지 못하게 한다. 기존 unavailable 호출자는 빈 callback으로 동일 외형을 유지한다. 전체 슬롯을 black surface로 채우고 원본 비율·viewport 높이 예산을 유지한다. 영상 데이터/시간/seek capability를 생성하지 않으며 연결 전 controls는 disabled·시간은 미상으로 둔다.

검증은 SEG/SUB5/OBJ의1440 및760 실제 실행, OCR ROI·OBJ 원본/마스크/합성 좌표와 키보드 focus, Empty의 이전 overlay 제거를 포함한다. 기존 공용 소비자의 크기/도구 줄바꿈 영향도 별도 확인한다. 구현·Rust 테스트만으로 시각 일치를 완료 처리하지 않는다. 별도 mask 예시 안내는 transport 밖에 유지한다.
