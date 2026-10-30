---
id: evidence:kinekino/20261029113801
title: JOB 상태 필터 r159 실제 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# JOB 상태 필터 r159 실제 검토

2026-10-19. r158 전체 화면 감사에서 발견한 ‘여섯 상태 탭이 같은 목록을 보여주는 문제’의 후속이다. 작업 상태/filter capability가 없는 현재 SDK 계약에 맞춰 전체만 선택 가능하게 했다. 진행률로 상태를 추정하지 않는다.

## 구현과 기준

- JOB-01: 페이지 탭과 상태 필터를 분리했다. 공용 Rust `filter_bar`는 Taffy FlexWrap, TAB_MIN72/CONTROL32, THIN/BORDER, CONTROL radius, ACCENT 선택 윤곽과 별도 내부 focus를 사용한다. 기준은 queue-refined.png 및 jobs-visual.md의 JOB-01 계약이다.
- 카탈로그는 작업 대기열 한 페이지다. 여섯 상태 버튼 중 전체만 활성화하며 나머지는 연결 필요 사유를 제공한다. 목록 위에도 항상 전체 목록 표시 사유를 둔다.
- 비활성 항목을 건너뛰는 키보드 선택은 원래 항목 인덱스로 복원한다. 회귀 테스트에서 비활성 클릭 거부, 비연속 enabled 인덱스, Home/End 및 잘못된 선택 해제를 확인했다.

## 검사와 실행 식별

단일 NVMe target의 순차 검사 session24807 exit0: cargo fmt --check, cargo test(141 passed,9 ignored; lib96 passed/1 ignored), cargo clippy --all-targets -- -D warnings, player build. clippy8.69s/build0.64s.

검토본: `runtime/review-runs/ui-job-filter-20261019-r159/player.exe`, SHA256 `a9c1ff2e2b92f29e6f2375b34e19a0b4343637da645b45acb5684c346611e69a`, 소유 PID65144. --ui-demo --record-validation --validation-position=3500,100으로 기존 가상 모니터에서 실행했다. 모니터/DPI 설정은 변경하지 않았다.

## 실제 관측

- 1440×900 Populated: 전체 선택 윤곽, 다섯 비활성 버튼, 연결 필요 안내와 기존 세 행을 확인했다. 실행 클릭은 선택을 바꾸지 않았다. 전체 클릭 후 내부 focus를 확인했고 Right 입력에도 전체가 유지됐다.
- 760×480 Populated: 여섯 버튼이 한 줄에 들어가며 본문 안내와 표가 유지된다. 실행 클릭은 비활성 상태를 유지했다. 두 번째 행 출력 셀 클릭으로 프레임 저장 예시가 선택되고, 설정 drawer 제목도 같은 작업으로 바뀌었다. Escape 후 전체 필터와 두 번째 행 선택이 유지됐다.
- 760×480 Empty: 이전 세 행과 선택 윤곽이 제거되고 다섯 heading·등록된 작업 없음·전체 필터와 미연결 안내가 유지됐다.
- 캡션 닫기 후 별도 fresh list_windows 조회에서 해당 검토 경로 창0개를 확인했다.

## 남은 차이와 미검증

실제 상태별 조회는 SDK 계약 미연결이다. 실제 필터 결과/작업 실행을 검증한 것이 아니다. 이번 실제 입력에서는 End/Tab 및 disabled hover tooltip을 확인하지 않았으며, 줄바꿈이 필요한 더 좁은 폭·긴 번역문도 미검증이다. 760에서는 줄바꿈이 발생하지 않았다. 복합 작업 셀·목록 바닥면·상단 동시 실행 배치 등 전체 시안 차이는 남아 있다. 공통 미디어 미리보기, MODEL 기본 접힘, PROJ viewport 등 r158 감사의 나머지 우선순위는 별도 후속이다.
