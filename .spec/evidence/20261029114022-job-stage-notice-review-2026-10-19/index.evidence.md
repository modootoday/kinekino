---
id: evidence:kinekino/20261029114022
title: JOB 단계 범위와 정보 안내 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# JOB 단계 범위와 정보 안내 검토

2026-10-19. JOB-05/06. queue-refined.png의 단계(1135,163,420,112)4배 및 정보 banner(1145,305,412,82)4배 확대를 비교했다. 파생물 `runtime/review-runs/job-stepper-r108-4x.png`, `job-info-r109-4x.png`.

## 발견과 책임

기존 system_inspector는 선택 행이 없어도 입력/대기/실행/검증/완료5단계를 그렸다. 시안은 선택된 작업의 상세이므로 모든 작업의 공통 고정 정책으로 사용할 수 없다. 무선택에는 공용 선택 안내만 표시하고 단계/진행/작업 입력 readout을 생성하지 않는다. DesignSample의 선택 행에만 명시적인 배치 예시 caption과 첫 단계 강조를 제공한다. Live의 단계는 작업명/percent로 추측하지 않는다. 실제 stage ID/order/current/status와 attempt/input_revision/progress/ETA/capability는 SDK 세션에 조율 요청했다. application 정책은 변경하지 않았다.

r108 PID18924,1440 demo: Populated의 예시 caption·첫 원 강조와 빈 상태의 무선택 안내/단계 제거를 실제 확인했다. 이 실행에서 공용 Info가 Warning과 같은 황색인 차이를 발견했다. r108은 caption 닫기로 종료했다.

공용 notice는 Kind별 의미 토큰을 사용하도록 수정했다. Info는 파랑 계열 INFO_SURFACE/BORDER/FOREGROUND, Warning은 기존 황색 NOTICE_SURFACE/BORDER/ACCENT다. 구조·여백·radius·border·본문 계층은 공유하고 아이콘·문구도 함께 구분한다. 이는 정보와 경고의 시각적 구분이며 실제 처리 상태를 새로 생성하지 않는다.

## 최종 실행

- r109 PID5704,1440 demo:JOB 현재 단계의 정보 안내가 파란 배경/테두리/아이콘/제목으로 바뀌었고 예시 caption·첫 단계 강조는 유지됐다.760 설정 drawer에서5단계 label이 겹치지 않았고 caption/파란 안내가 표시됐다. Escape로 닫은 후 목록과 선택 행이 유지됐다.1440 PROJ로 이동해 inspector를 스크롤했을 때 Warning은 황색 삼각형/제목/배경을 유지했다.
- r110 PID29288,일반 실행:1180 시청에서 제작 진입 후1440 JOB에서 미지원 목록 안내와 무선택 상세가 파란 정보 카드로 표시됐다.5단계·진행 readout·작업 입력 section을 생성하지 않았다. 등록0이나 실제 실행 성공으로 표시하지 않았다.
- r108/r109/r110은 한 개씩 실행하고 모두 caption 닫기로 종료했다. 후속 player.exe/mpv.exe 조회 결과가 비어 있었다. 기존 가상 모니터 위치3500,100을 사용했고 VM·DPI·모니터 구성은 변경하지 않았다.
- 최종 cargo fmt --check, cargo test(lib70 passed/1 ignored와 CLI/통합), cargo clippy --all-targets -- -D warnings, cargo build --bin player가 통과했다. 외부 미디어 의존 ignored 항목은 실행하지 않았다.

## 남은 범위

typed stage detail·실제 실행·단계별 완료/실패/취소·작업별 glyph와 입력 summary는 미완료다. stepper5단계 예시를 실제 작업 수명으로 일반화하지 않는다. 모든 Info 소비자의 시각 인수·긴 문구·모든 창 크기·접근성은 후속이다.
