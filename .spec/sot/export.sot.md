---
id: sot:kinekino/export
title: 내보내기와 파일 충돌
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 내보내기와 파일 충돌

구성·상태별 형태는 [요소별 시각 계약](export-visual.md)을 적용한다.

## 시각화

- [파일 충돌과 최종 저장](../../../../artifacts/design/v2/studio/export.png)

이미지는 배치 참고이며 아래 요소 계약과 공통 필드·상태 규칙이 구현 기준이다.

상태: UI/UX 설계 승인(2026-10-18). 생성 이미지 디테일은 실제 Rust 공통 컴포넌트로 검토한다. 구현 완료와 구분한다. [공통 계약](../sot-020-ui-contract.md)의 프레임·입력 검증·상태·접근성·저장·Rust 계층 규칙을 상속한다. 기본/hover/pressed/selected/disabled/focus는 모든 조작 요소에 적용한다.

## 요소 계약

| 요소 ID | 요소 | 표시와 동작 |
|---|---|---|
| OUT-01 | 요약 | 소스·범위·revision·출력 개수·형식·변환 손실 |
| OUT-02 | 경로 | OS 선택기·쓰기 가능·절대 대상 검증 |
| OUT-03 | 파일명 | 예약이름·구분자·확장자·길이 검사 |
| OUT-04 | 충돌 | 새 이름 권장/건너뛰기/명시적 덮어쓰기, 파일별 적용 범위 |
| OUT-05 | 용량 | 알려진/추정/미상, 여유 공간·임시 공간 |
| OUT-06 | 색/트랙 | 실제 선택 오디오·자막·alpha·color summary |
| OUT-07 | 저장 시작 | 프리플라이트 통과 후 고정 snapshot 큐 |
| OUT-08 | 검증 | 길이·형식·트랙·frame/파일 수·manifest 참조 |
| OUT-09 | 확정 | .part→검증→원자적 이름 확정, 기존 원본 보존 |
| OUT-10 | 취소/실패 | 소유 임시파일만, 최종 완료파일 보존 |
| OUT-11 | 결과 | 경로·크기·검증 요약, 폴더 열기와 다시 실행을 분리 |
| OUT-12 | 게시 | 로컬 저장과 구별, 외부 게시 버튼 없음 |

## 상태와 오류

닫기는 작업 취소와 별개다. 경로가 읽기전용·없음·디스크 부족·동명 충돌이면 해결 전 시작 불가. 파일쓰기 성공만으로 완료라고 표시하지 않고 형식 검증을 거친다.

## 저장 계약

ExportRequest(project_revision,items,preset,destination,collision_policy), ExportReceipt(outputs,verification,completed_at), 비밀 없는 provenance.

## 인수 기준

형식 mismatch·손상/partial·덮어쓰기 범위·경로 traversal·Windows 예약이름·원본/완료 보존.

각 요소 ID의 유스케이스는 application에, 검증 정책/값 객체는 domain에, OS·추론·파일·프로세스는 infrastructure에 분리한다. presentation은 입력과 렌더링을 담당한다. 모델 미선정/encoder·플랫폼 지원 여부 미확정 항목은 완료로 표시하지 않고 해당 기능을 비활성 또는 NeedsInput으로 제공한다.

