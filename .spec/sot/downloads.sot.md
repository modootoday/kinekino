---
id: sot:kinekino/downloads
title: 영상 저장
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 영상 저장

형태·컴포넌트 구성은 [시각 계약](downloads-visual.md)을 적용한다.

## 시각화

- [소스와 트랙 저장](../../../../artifacts/design/v2/studio/download-refined.png)

이미지는 배치 참고이며 아래 요소 계약과 공통 필드·상태 규칙이 구현 기준이다.

상태: UI/UX 설계 승인(2026-10-18). 생성 이미지 디테일은 실제 Rust 공통 컴포넌트로 검토한다. 구현 완료와 구분한다. [공통 계약](../sot-020-ui-contract.md)의 프레임·입력 검증·상태·접근성·저장·Rust 계층 규칙을 상속한다. 기본/hover/pressed/selected/disabled/focus는 모든 조작 요소에 적용한다.

## 요소 계약

| 요소 ID | 요소 | 표시와 동작 |
|---|---|---|
| SAVE-01 | 대상 | 현재/명시적 회차 묶음, 전체 회차 자동 선택 금지 |
| SAVE-02 | 소스 확인 | 정상 세션 handle 재확인, HTML/blob 미디어 오인 금지 |
| SAVE-03 | 품질 | 실제 variant·해상도·codec, unavailable 표시 |
| SAVE-04 | 트랙 | 영상·오디오·자막 선택, 없는 트랙은 없음 |
| SAVE-05 | 방식 | 원본/리먹스/호환 변환, 자동 열화 전환 금지 |
| SAVE-06 | container | 트랙 codec 호환 검사, 불가 이유·대안 |
| SAVE-07 | 변환 | 실제 encoder·크기·품질, 원본 저장에서는 숨김 |
| SAVE-08 | 예상 용량 | 알려진 bytes/추정/미상 구분 |
| SAVE-09 | 경로/파일명 | OUT, Windows 예약 이름·길이·충돌 |
| SAVE-10 | 재개 | Range/ETag 등 지원 시만, HLS는 segment 단위 상태 |
| SAVE-11 | 만료 | 정식 재리졸브·미디어/트랙 일치 확인 후 재개 |
| SAVE-12 | 큐 추가 | 지원 매트릭스·출력 여유 공간·트랙 확인 |
| SAVE-13 | 취소 | 소유 .part 처리 선택, 완료 파일 유지 |
| SAVE-14 | 완료 | 파일 열림·트랙·길이 확인 후 확정 |

## 상태와 오류

라이브·암호화·DRM·키 접근 등은 지원 여부 판정 전 사용 가능으로 표시하지 않는다. 외부 브라우저 쿠키/임의 프로그램 실행으로 우회하지 않는다. 실패 시 retryable 원인과 사용자 입력 필요 원인을 구별한다.

## 저장 계약

DownloadPlan(media_id,variant,tracks,mode,preset,destination,session_handle), resume validator and segment journal separate from project secret-free history.

## 인수 기준

Range 객체 변경·토큰 만료·HLS 불연속·네트워크 취소·part 복구·트랙/길이 검증.

각 요소 ID의 유스케이스는 application에, 검증 정책/값 객체는 domain에, OS·추론·파일·프로세스는 infrastructure에 분리한다. presentation은 입력과 렌더링을 담당한다. 모델 미선정/encoder·플랫폼 지원 여부 미확정 항목은 완료로 표시하지 않고 해당 기능을 비활성 또는 NeedsInput으로 제공한다.

