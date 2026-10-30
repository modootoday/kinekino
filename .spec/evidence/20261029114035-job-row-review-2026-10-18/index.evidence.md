---
id: evidence:kinekino/20261029114035
title: JOB 행 높이·상태 열·행 선택 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# JOB 행 높이·상태 열·행 선택 검토

2026-10-18 JOB-02. 기준 queue-refined.png 전체 및 직전 작업의 rows(297,245,805,270)2× 파생물 job-rows-r90-2x.png를 사용한다. r90/r91 실제 관측은 [동작 버튼 기록](job-actions-review-2026-10-18.md)에 있다. 변경 전 일반32행·작업 glyph 없음·상태와 진행의 별도 열 없음·첫 셀에만 focus 외곽선이 있었다.

## 구현과 경계

공용 DataTable Appearance로 최소48행·List SVG20·행 전체 hit/focus를 추가한다. 작업별 typed 종류가 없어 중성 List를 사용하며 이름에서 추측하지 않는다. JOB는 상태112/진행80 기준 고정폭과 나머지 유동폭을 사용한다. 상태 열이 없는 snapshot은 표시용 column만 보완하고 셀은 Missing `—`다. 상단에 실제 상태 연결 필요 안내를 두고 진행률로 실행 상태를 추론하지 않는다. snapshot 원문과 origin은 유지하며 application 계약을 변경하지 않는다.

JOB 첫 선택/무효 선택은 실제 첫 row_id와 맞추고 Empty는 해제한다. 공용 row_id 기반 전체 행 클릭·선택 왼쪽 선·독립 focus2를 사용한다. 기존 표의 기본 높이와 셀 포커스는 유지한다.

## 검증

headless 회귀에서 긴 값의 표 폭,48행·전체 폭 hit 영역을 확인하고 첫 행의 마지막 출력 셀 영역 클릭→Down으로 두 번째 row_id 선택을 검증했다. 첫 Clippy는 중첩 if에 collapsible_if를 보고해 let-chain으로 정리했다. 이후 최종 cargo fmt --check, cargo test(라이브러리66통과/1ignored 및 CLI·통합 통과), cargo clippy --all-targets -- -D warnings, cargo build --bin player가 통과했다. 기존 외부 미디어 ignored는 실행 성공으로 계산하지 않는다.

## 실제 실행 r92

`runtime/review-runs/ui-job-rows-20261018-r92/player.exe`(PID48140)를 기존 가상 모니터3500,100에서 한 개만 실행했다.1440×900에서48행·중성 List glyph·분리된 상태/진행 열과 첫 선택 표시를 확인했다. 두 번째 행의 출력 셀 클릭은 전체 행 focus 외곽선과 프레임 저장 inspector를 표시했고 Down은 세 번째 자산 내보내기 행/inspector로 이동했다. 시안의 작업별 종류 glyph·source 보조 설명과 같다고 판정하지 않는다.

760×480에서 다섯 열이 폭 안에 들어가고 상태 `—`와 예시 진행률이 구분됐다. 첫 행 출력 셀 클릭→End는 세 번째 전체 행 focus와 자동 스크롤을 표시했다. 설정에 세 번째 작업이 표시됐고 Escape 복귀 후 선택과 스크롤이 유지됐다. LongLabels는 작업/source/output 원문을 각 열 안에서 말줄임했고 List glyph와 상태/진행 열을 밀어내지 않았다. Empty는 모든 행/선택을 제거하고 등록된 작업 없음 안내를 표시했다. NeedsInput fixture는 상태와 진행 모두 `—`이며 미상 진행을0%나 실행으로 보충하지 않았다.

r92는 캡션 닫기로 종료했고 후속 CIM 조회에서 player.exe/mpv.exe가 없었다. VM·DPI·모니터 구성은 변경하지 않았다. SDK 세션에 UI 상태 열은 미연결이며 typed kind/state/progress/stage/action 계약이 필요하다는 경계를 전달했다.

## 남은 범위

980/1180, Home/Up, 원문 hover, OS 접근성 실제 판독, 많은 행의 스크롤/선택 유지, 실제 typed 상태/작업 종류/시도·revision·입력 보조 설명·단계·진단/출력 capability는 후속이다. 기존 표 소비자의 전체 GUI 회귀는 이번 실행에서 수행하지 않았다. 전체 JOB 시각 일치와 제품 전체 인수는 미완료다.
