---
id: sot:kinekino/state-transitions
title: 공통 상태와 명령 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 공통 상태와 명령 계약

이 문서는 각 탭 요소 계약과 application 유스케이스의 연결 기준이다. OS·추론 호출 구현을 UI에 포함하지 않는다.

| 사용자 입력 | application 명령 | 성공 상태 | 실패/거부 |
|---|---|---|---|
| 소스 선택 | select_media | Draft에 MediaRef 연결, duration/track 갱신 | 재연결/트랙 확인 NeedsInput |
| 필드 수정 | revise_document | 새 Draft revision, undo entry | 필드 오류, 기존 유효 값 보존 |
| 모델 선택 | select_model_for_task | 호환 ModelRef 연결 | 지원 안 함/미설치 표시 |
| 분석 시작 | enqueue_analysis | 입력 snapshot·의존 모델 job 생성 | Preflight 오류, 큐에 불완전 작업 무단 실행 금지 |
| 결과 미리보기 | preview_revision | 선택 결과/원본 PTS 비교 | 미디어 없음/오래된 결과 표시 |
| AI 제안 수락 | accept_suggestion | 원문을 보존한 수정 revision | 근거 revision 불일치면 재검토 |
| 저장 시작 | enqueue_export | 경로·충돌 정책·preset snapshot | 지원/쓰기/여유 공간 확인 실패 |
| 작업 취소 | request_cancel | Cancelling→Cancelled | 완료와 경합하면 완료 영수증 유지 |
| 실패 재시도 | retry_job | attempt 증가·같은 logical item | 이미 Completed면 중복 실행 방지 |
| 회차 전환 | select_episode | 최신 generation 연결, pause_intent 보존 | stale event 무시·해결 동작 표시 |
| 구간 스킵 | skip_verified_segment | seek ack 후 toast·undo token | seek 실패면 현재 위치 유지·수동 재시도 |
| 스킵 되돌리기 | undo_skip | 이전 PTS 이동·같은 구간 auto suppression | 오래된 미디어 token은 적용 금지 |

## 변경과 닫기

일반 편집은 입력이 안정된 뒤 500ms debounce로 로컬 초안 저장을 제안한다. 파일 저장 실패는 “초안 저장 안 됨”으로 표시하고 현재 메모리의 편집을 버리지 않는다. OS 파일 내보내기와 자동 초안 저장은 구분한다. 매 키마다 전체 미디어/이미지를 복제하지 않는다.

다른 탭·시청으로 돌아가기는 초안을 유지한다. 앱을 닫을 때 저장 실패한 초안이 있으면 재시도/버리고 닫기/돌아가기, 실행 중 작업이 있으면 취소 후 닫기/돌아가기를 제공한다. 임의의 상주 서비스나 사용자 앱 종료를 추가하지 않는다. 정상 종료 뒤 복구 시 마지막 초안과 중단 작업을 보여주고 자동 재실행하지 않는다.

## 기본 상태별 표시

| 상태 | 중앙 | 검사기 | 주 버튼 | 큐 |
|---|---|---|---|---|
| 소스 없음 | 입력 선택 빈 상태 | 필수 입력 안내 | 소스 선택 | 기존 작업 유지 |
| 모델 없음 | 수동 편집 가능 | 모델 필요·선택 진입 | 분석 비활성, 모델 선택 활성 | 실행 안 됨 |
| 준비 완료 | 입력 프리뷰 | 요약/수정 | 시작/추가 활성 | 대기 가능 |
| 실행 중 | 현재 초안 편집·이전 결과 구별 | 실행 snapshot 표시 | 새 revision 실행은 별도 작업 | 실제 단계·취소 |
| 입력 필요 | 기존 초안 유지 | 원인과 해결 필드 | 해결 전 비활성 | NeedsInput |
| 실패 | 마지막 유효 결과 보존 | 원인·재시도 가능 여부 | 다시 시도 또는 설정 수정 | Failed |
| 취소 | 취소 전 초안 유지 | partial 산출물 처리 표시 | 다시 시작 | Cancelled |
| 완료 | 검토 가능한 결과 | 원본/결과·revision | 검토·저장/열기 | 검증 영수증 |
| 오래된 결과 | “이전 초안의 결과” | 현재와 차이 | 복사/재검토 선택 | 자동 덮어쓰기 없음 |

## 모델 준비와 출력의 게이트

모델이 준비돼도 source/track·경로·사용 설정이 유효하지 않으면 실행할 수 없다. 결과가 생성돼도 검증 전은 Completed가 아니다. 변환 손실/피사체 소실/미관측 3D 영역은 입력 검토와 품질 경고로 구별하고 실행 엔진 오류로 합치지 않는다.

## 시각화와 구현의 대응

이미지에 보이는 조작은 탭별 ID로 연결한다. 이 문서/필드 규칙에 없는 추가 제어가 생성 이미지에 등장하면 채택하지 않는다. tooltip·accessible name·disabled reason은 동일 동작 문자열을 사용하고 별도 화면마다 새 표현을 만들지 않는다. 사용자 결정 변경 시 공통 계약을 먼저 갱신하고 관련 모든 탭/이미지/Pages를 같은 버전으로 맞춘다.
