---
id: sot:kinekino/subject-mask-read
title: 피사체 마스크 읽기 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 피사체 마스크 읽기 계약

2026-10-18 SDK 세션과 조율한 UI 수직 구현 범위다. `application/workspace_read/mask.rs`를 정본 타입으로 사용하며 application 명령이나 실제 모델 실행을 추가하지 않는다. [UI 시각 계약](../presentation/studio/subjects-visual.md), [GPU 확장 설계](../presentation/shared/rendering.md)를 연결한다.

WorkspaceSnapshot.mask는 optional이다. MaskSnapshot은 원본 width_px/height_px, source_asset, source_pts_us(DecimalU64), subjects, origin, state를 제공한다. SubjectMask는 stable track_id, optional polygon_px, optional alpha_asset, review_state를 제공한다. polygon은 원본 전체 canvas의 정수 pixel 좌표이고 alpha도 전체 canvas에 정렬된다. bbox crop이나 UI zoom 좌표를 넣지 않는다.

ReadState는 데이터 준비 여부, MaskReviewState는 Unreviewed/NeedsReview/Confirmed다. Ready는 검토 확정이나 추론 정확도를 뜻하지 않는다. 현재 Live 미연결과 Empty는 None이며 DesignSample만 수동 silhouette를 제공한다. 이후 live adapter의 asset resolver·허용 경로·revision 보강은 별도다. UI는 asset identity와 PTS가 현재 프레임과 일치할 때만 mask를 표시하고 embedded allowlist를 우회하지 않는다.

샘플 원본은 forest-stream-v1.png(1672×941), PTS52000000이다. forest-subject-v1.svg는 사람이 직접 근사한 경계이며 polygon fixture와 같은 좌표를 사용한다. 테스트는 SVG/points 일치·범위·원본asset·PTS·DesignSample origin·Live/Empty 부재를 확인한다. 정밀 머리카락/반투명/가림·실제 추론·저장 검증을 대신하지 않는다.
