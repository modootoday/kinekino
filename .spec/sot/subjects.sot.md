---
id: sot:kinekino/subjects
title: 피사체 인식
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 피사체 인식

배치·세부형태·컴포넌트 책임은 [시각 계약](subjects-visual.md)을 따른다.

## 시각화

- [탐지와 분할 추적](../../../../artifacts/design/v2/studio/subject.png)

이미지는 배치 참고이며 아래 요소 계약과 공통 필드·상태 규칙이 구현 기준이다.

상태: UI/UX 설계 승인(2026-10-18). 생성 이미지 디테일은 실제 Rust 공통 컴포넌트로 검토한다. 구현 완료와 구분한다. [공통 계약](../sot-020-ui-contract.md)의 프레임·입력 검증·상태·접근성·저장·Rust 계층 규칙을 상속한다. 기본/hover/pressed/selected/disabled/focus는 모든 조작 요소에 적용한다.

## 요소 계약

| 요소 ID | 요소 | 표시와 동작 |
|---|---|---|
| OBJ-01 | 소스/프레임 | 현재 프레임 PTS·범위 선택, 원본 비교 |
| OBJ-02 | 작업 | 탐지/분할/추적을 별도 선택, 지원 모델만 제공 |
| OBJ-03 | 피사체 목록 | 사용자 라벨·색·track ID, 신원 인식처럼 표시 금지 |
| OBJ-04 | 선택 | 클릭/박스, 새 피사체 추가·삭제·이름 변경 |
| OBJ-05 | 포함/제외 점 | 좌표 원본 보존, 점 삭제·undo |
| OBJ-06 | 브러시/지우개 | 크기1~원본 짧은 변 길이, 미리보기, 확대 좌표 변환 |
| OBJ-07 | 표시 | 원본/마스크/합성, opacity0~100, checkerboard |
| OBJ-08 | 경계 | feather 원본 pixel 단위, 미리보기와 원본 비교 |
| OBJ-09 | 추적 범위 | in/out, 기준 프레임, 장면 컷에서 재검토 |
| OBJ-10 | 추적 시작/취소 | 준비 모델 확인, 작업 revision 고정 |
| OBJ-11 | 실패 프레임 | 가림·소실·컷 구간 표시, 수동 재선택 |
| OBJ-12 | 검토 확정 | 구간별 승인, 미검토 결과 사용 경고 |
| OBJ-13 | 활용 | 숏폼 크롭·썸네일·카드·2D/투영으로 참조 전달 |
| OBJ-14 | 저장 | mask PNG/시퀀스+PTS manifest, OUT 검증 |

## 상태와 오류

가려진 피사체를 새 개체로 무조건 교체하지 않는다. 추적 소실은 경고 구간이고 전체 성공으로 표시하지 않는다. 실제 모델 분류 라벨과 사용자 별명을 구별한다. 원본 변경 시 마스크 재연결 필요.

## 저장 계약

SubjectTrack(track_id,user_label,media_ref,frames[{pts,box,mask_ref,review}],transforms,model_ref).

## 인수 기준

머리카락·반투명·애니메이션 선화·가림·빠른 이동·컷·회전·마스크 좌표 왕복.

각 요소 ID의 유스케이스는 application에, 검증 정책/값 객체는 domain에, OS·추론·파일·프로세스는 infrastructure에 분리한다. presentation은 입력과 렌더링을 담당한다. 모델 미선정/encoder·플랫폼 지원 여부 미확정 항목은 완료로 표시하지 않고 해당 기능을 비활성 또는 NeedsInput으로 제공한다.

