---
id: sot:kinekino/shorts
title: 숏폼 제작
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 숏폼 제작

하위 화면별 세부형태·컴포넌트 책임은 [시각 계약](shorts-visual.md)을 따른다.

## 시각화

- [수동 제작](../../../../artifacts/design/v2/studio/short.png)
- [AI 추천](../../../../artifacts/design/v2/studio/short-recommend-refined.png)
- [회차 일괄 제작](../../../../artifacts/design/v2/studio/short-batch.png)

이미지는 배치 참고이며 아래 요소 계약과 공통 필드·상태 규칙이 구현 기준이다.

상태: UI/UX 설계 승인(2026-10-18). 생성 이미지 디테일은 실제 Rust 공통 컴포넌트로 검토한다. 구현 완료와 구분한다. [공통 계약](../sot-020-ui-contract.md)의 프레임·입력 검증·상태·접근성·저장·Rust 계층 규칙을 상속한다. 기본/hover/pressed/selected/disabled/focus는 모든 조작 요소에 적용한다.

## 요소 계약

| 요소 ID | 요소 | 표시와 동작 |
|---|---|---|
| CLIP-01 | 하위 탭 | 수동/AI 추천/일괄, 동일 ClipDocument 사용 |
| CLIP-02 | 원본 | 선택 미디어·범위, 타임라인 PTS |
| CLIP-03 | in/out | 프레임 경계 확인, 길이 표시·범위 검사 |
| CLIP-04 | 추천 설정 | 후보 개수·최소/최대 길이·근거 종류·중복 제외 |
| CLIP-05 | 후보 | 회차·시간·추천 이유·근거 cue, 선택/기각/수정 |
| CLIP-06 | 일괄 | 명시적 회차 묶음·공통 템플릿·출력 요약 후 시작 |
| CLIP-07 | 캔버스 | 9:16 기본 제안, 1:1/16:9/사용자 지정 |
| CLIP-08 | 크롭 | 맞춤/채우기·원본/결과 비교·정규화 crop 좌표 |
| CLIP-09 | 추적 | 검토된 피사체 선택·고정 크롭 대안·컷 초기화 |
| CLIP-10 | 자막 | cue 범위·글꼴·배경·안전영역·원본과 합성 구별 |
| CLIP-11 | 오디오 | 선택 트랙·원음0~100·음소거, 시간 동기 |
| CLIP-12 | 편집 | 클립 순서·추가·제거·undo, 다중 클립은 구현 단계 표시 |
| CLIP-13 | 출력 프리셋 | 크기·fps·codec·container, 실제 가용 encoder만 |
| CLIP-14 | 컷 방식 | 정확 컷 재인코딩/지원 시 키프레임 빠른 컷, 오차 표시 |
| CLIP-15 | 미리보기 | 저장과 별도, 확대·자막 충돌·피사체 잘림 검사 |
| CLIP-16 | 큐 추가 | Preflight와 OUT, 회차별 실패만 재시도 |

## 상태와 오류

원본에 없는 대사를 AI가 추가하지 않는다. 추천은 수정 가능한 초안이다. 원본 길이 초과·encoder 없음·출력 비율 충돌·추적 소실은 NeedsInput. 출력이 원본보다 고해상도면 확대를 표시한다. 플랫폼 프리셋은 현재 규정 보장과 구별한다.

## 저장 계약

ClipDocument(source_refs,ranges,crop_keys,subject_ref,cues_ref,audio,preset,revision), BatchPlan(items,idempotency_keys).

## 인수 기준

30초/60초 경계·VFR·정확 컷·추적 흔들림·자막 안전영역·취소·일괄 중복 방지.

각 요소 ID의 유스케이스는 application에, 검증 정책/값 객체는 domain에, OS·추론·파일·프로세스는 infrastructure에 분리한다. presentation은 입력과 렌더링을 담당한다. 모델 미선정/encoder·플랫폼 지원 여부 미확정 항목은 완료로 표시하지 않고 해당 기능을 비활성 또는 NeedsInput으로 제공한다.

