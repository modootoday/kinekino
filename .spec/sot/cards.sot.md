---
id: sot:kinekino/cards
title: 카드뉴스 제작
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 카드뉴스 제작

형태·컴포넌트 구성은 [시각 계약](cards-visual.md)을 적용한다.

## 시각화

- [카드 편집과 플랫폼 파생](../../../../artifacts/design/v2/studio/cards.png)

이미지는 배치 참고이며 아래 요소 계약과 공통 필드·상태 규칙이 구현 기준이다.

상태: UI/UX 설계 승인(2026-10-18). 생성 이미지 디테일은 실제 Rust 공통 컴포넌트로 검토한다. 구현 완료와 구분한다. [공통 계약](../sot-020-ui-contract.md)의 프레임·입력 검증·상태·접근성·저장·Rust 계층 규칙을 상속한다. 기본/hover/pressed/selected/disabled/focus는 모든 조작 요소에 적용한다.

## 요소 계약

| 요소 ID | 요소 | 표시와 동작 |
|---|---|---|
| CARD-01 | 시작 | 빈 템플릿/선택 프레임/선택 cue |
| CARD-02 | 목적 | YouTube 커뮤니티/Instagram/사용자 지정, 로컬 저장 |
| CARD-03 | 비율 | 1:1/4:5/3:4/9:16 제안, 플랫폼 공식 한도와 구별 |
| CARD-04 | 순서 | 번호·현재 카드·이동 버튼·드래그 |
| CARD-05 | 조작 | 추가·복제·삭제·undo, 마지막 카드 삭제는 빈 초안 |
| CARD-06 | 템플릿 | 표지/본문/마지막, 기존 내용 유지 범위 확인 |
| CARD-07 | 편집 캔버스 | 레이어 선택·변형·맞춤선·안전영역 |
| CARD-08 | 텍스트 | 제목·본문·페이지 번호·폰트·색·간격 |
| CARD-09 | 넘침 | 줄바꿈·최소 글자 크기·수동 교정, 몰래 잘라내지 않음 |
| CARD-10 | 요약 초안 | 원문 인용/요약 분리·cue 근거·수락/기각 |
| CARD-11 | 피사체 | 마스크·크롭·배경, 원본 재사용 |
| CARD-12 | 플랫폼 파생 | 비율별 layout revision, 이미지 왜곡 확대 금지 |
| CARD-13 | 대체텍스트 | 카드별 초안·사용자 편집 |
| CARD-14 | 저장 | 선택/전체 PNG/JPG, 순번 파일, 게시문 txt·프로젝트 |
| CARD-15 | 출력 검사 | 카드별 넘침·폰트·픽셀·순서·OUT |

## 상태와 오류

카드 수 상한은 플랫폼 현행 규정 재확인 전 고정 보장하지 않는다. 삭제는 초안 수정이고 저장 파일 삭제가 아니다. 요약은 근거 없는 대사·설정을 만들지 않는다. 탭을 떠나도 편집 유지.

## 저장 계약

CardDocument(cards[{id,order,layers,copy,evidence,alt}],layouts_by_preset,theme,revision).

## 인수 기준

한글 긴 문장·비율 변경·순서·중복 파일명·원문 링크·선택/전체 출력.

각 요소 ID의 유스케이스는 application에, 검증 정책/값 객체는 domain에, OS·추론·파일·프로세스는 infrastructure에 분리한다. presentation은 입력과 렌더링을 담당한다. 모델 미선정/encoder·플랫폼 지원 여부 미확정 항목은 완료로 표시하지 않고 해당 기능을 비활성 또는 NeedsInput으로 제공한다.

