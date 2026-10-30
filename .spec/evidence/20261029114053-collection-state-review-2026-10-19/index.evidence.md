---
id: evidence:kinekino/20261029114053
title: MODEL·JOB 목록 확인 상태 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# MODEL·JOB 목록 확인 상태 검토

2026-10-19. r99 일반 MODEL에서 후보(0)를 관측한 후 application WorkspaceSnapshot을 확인했다. LiveWorkspaceProvider는 ReadOrigin::Live/ReadState::Unsupported와 빈 TableSnapshot을 반환한다. 조회 성공의 빈 결과가 아니므로 후보0/등록 작업 없음으로 표시한 것은 잘못된 의미였다. 원본 관측은 [제목줄 검토](studio-title-context-review-2026-10-18.md)에 있다.

## 공용 계약과 구현

CollectionState는 Live Unsupported/Unknown/NeedsInput을 각각 확인 불가/확인 필요/입력 확인 필요로 투영한다. Ready 또는 명시적 DesignSample만 표시 행 수를 사용한다. MODEL accordion heading과 내부 안내, JOB 목록 아래 안내가 같은 공용 notice/토큰을 조합한다. 설치/실행 가능 상태나 새 application 정책을 만들지 않는다. 실제 read adapter는 여전히 미구현이다.

모델/대기열 기준 시안과 기존 확대 근거는 각 models-visual/jobs-visual을 유지한다. 미확인 상태는 시안의 설치/완료/빈 결과를 복제하지 않고 공용 정보 banner로 표시한다. registered0과 미확인 안내의 시각 계층을 실제 실행에서 구분한다.

회귀 검사는 실제 LiveWorkspaceProvider의 MODEL/JOB 빈 행 배열을 Unsupported/Unknown/NeedsInput에 따라0으로 승격하지 않는지, Ready와 explicit Empty fixture만0을 표시하는지 확인한다. application 파일과 데이터 상태는 수정하지 않는다.

## 실행 검증

최종 cargo fmt --check, cargo clippy --all-targets -- -D warnings, cargo build --bin player, cargo test가 통과했다. 라이브러리70통과/1ignored 및 CLI·통합 통과, 기존 외부 미디어 ignored 유지. 모든 목록 소비자의 상태 표시 이관 완료로 확대하지 않는다.

## 실제 실행

모두 기존 가상 모니터3500,100에서 한 개씩 실행하고 캡션 닫기로 종료했다. VM·DPI·모니터 구성은 변경하지 않았다.

- r100(PID117348), `runtime/review-runs/ui-collection-state-20261019-r100/player.exe`: 일반 실행에서 MODEL 후보0 대신 확인 불가 heading·정보 안내, JOB 등록 없음 대신 목록 확인 불가 안내를 확인했다. JOB에는 metadata 부재로 상태 header만 남는 문제를 발견해 통과로 두지 않고 다섯 열 heading 보완을 추가했다.
- r101(PID115024), `runtime/review-runs/ui-collection-state-20261019-r101/player.exe`: demo1440×900 Populated의 후보(3) 유지, Empty의 후보(0)·등록 모델 없음, JOB Empty의 다섯 heading·등록 작업 없음 안내를 확인했다. 디자인 빈 표본과 실제 미지원이 다른 문구/계층을 사용하는 범위다.
- r102(PID79308), `runtime/review-runs/ui-collection-state-20261019-r102/player.exe`: 일반 JOB1440×900에서 작업/소스/상태/진행/출력 다섯 heading 골격과 확인 불가 banner를 함께 확인했다. 행·진행값·등록0을 새로 만들지 않았다.

후속 CIM에서 player.exe/mpv.exe가 없었다. SDK 세션에 UI 투영이 read 상태를 보존하며 application 데이터/정책을 바꾸지 않았다는 경계를 전달했다.

## 남은 범위

일반 Unknown/NeedsInput/Ready empty의 실제 adapter 실행,760/980/1180 안내 줄바꿈, 동적 로딩/오류 복구, 실제 catalogue/queue 계약은 미검증/미구현이다. 다른 화면의 빈 결과·개수 표시 전수 이관, JOB 선택 없음의 단계 골격과 상세 상태는 후속이다. 이 기록은 MODEL/JOB의 해당 문구/골격 범위이며 전체 시각 인수는 미완료다.
