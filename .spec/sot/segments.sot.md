---
id: sot:kinekino/segments
title: 오프닝과 엔딩
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 오프닝과 엔딩

배치·세부형태·컴포넌트 책임은 [시각 계약](segments-visual.md)을 따른다.

## 시각화

- [수동과 자동 설정](../../../../artifacts/design/v2/studio/skip-settings.png)

이미지는 배치 참고이며 아래 요소 계약과 공통 필드·상태 규칙이 구현 기준이다.

상태: UI/UX 설계 승인(2026-10-18). 생성 이미지 디테일은 실제 Rust 공통 컴포넌트로 검토한다. 구현 완료와 구분한다. [공통 계약](../sot-020-ui-contract.md)의 프레임·입력 검증·상태·접근성·저장·Rust 계층 규칙을 상속한다. 기본/hover/pressed/selected/disabled/focus는 모든 조작 요소에 적용한다.

## 요소 계약

| 요소 ID | 요소 | 표시와 동작 |
|---|---|---|
| SEG-01 | 소스/구간 | 현재 회차, 전체/선택 범위, 미디어 ID·fingerprint |
| SEG-02 | 감지 방식 | 음악 후보 / 반복 오디오·영상 / 결합, 지원 모델이 있는 방식만 활성 |
| SEG-03 | 모델 | 작업 종류·revision·정밀도, 미선택 시 분석 비활성 |
| SEG-04 | 분석 시작 | 사용자 시작, 재생 중 무거운 분석은 큐 대기 기본 |
| SEG-05 | 후보 목록 | 종류·in/out·길이·후보/확인 표시, 확률 임의 생성 금지 |
| SEG-06 | 수동 지정 | 현재 시각으로 시작/끝, 숫자 ms 입력, ±1초 이동 |
| SEG-07 | 미리보기 | 구간 전후3초, 길이 밖 clamp, 복귀 위치 보존 |
| SEG-08 | 확인/해제 | 자동 스킵에 쓸 수 있는 verified 상태를 명시적 갱신 |
| SEG-09 | 삭제 | 초안 annotation 삭제, 되돌리기 가능 |
| SEG-10 | OP 모드 | 끔/수동/자동 각각 저장, 기본 수동 제안 |
| SEG-11 | ED 모드 | 끔/수동/자동 각각 저장, 기본 수동 제안 |
| SEG-12 | 자동 예고 | 시작3초 전, 이번에는 보기, 일시정지 시 정지 |
| SEG-13 | 되돌리기 | 자동/수동 이동 후5초 제안, 재진입 시 자동 반복 방지 |
| SEG-14 | 다음화 | 엔딩과 별도 기본 끔, 마지막화·극장판 비활성 |
| SEG-15 | 회차 저장 | 현재 회차만, 다른 회차 복제는 별도 검토 |
| SEG-16 | 작품 예외 | 공통 설정 덮어쓰기·초기화의 적용 범위 표시 |

## 상태와 오류

음악만으로 확정하지 않는다. 미확인·겹침·잘못된 경계·길이 미상은 자동 스킵 비활성이다. 취소는 이미 재생 중인 영상을 종료하지 않는다. 콜드 오픈·배경음·가변 OP/ED·쿠키 영상을 별도 사례로 둔다.

## 저장 계약

SegmentAnnotation(id, media_id, fingerprint, kind, in_us, out_us, origin, evidence_refs, verified_revision), SkipPolicy(op,ed,next_episode,series_override).

## 인수 기준

구간 겹침·되돌리기·탐색 재진입·일시정지 카운트다운·늦은 분석 결과·회차 혼선·쿠키 보존.

각 요소 ID의 유스케이스는 application에, 검증 정책/값 객체는 domain에, OS·추론·파일·프로세스는 infrastructure에 분리한다. presentation은 입력과 렌더링을 담당한다. 모델 미선정/encoder·플랫폼 지원 여부 미확정 항목은 완료로 표시하지 않고 해당 기능을 비활성 또는 NeedsInput으로 제공한다.

