---
id: evidence:kinekino/20261029114104
title: PROJ 검사기 그룹 실제 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# PROJ 검사기 그룹 실제 검토

2026-10-19. PROJ-08/09/10/15. 기준 projection-package.png의 (1115,60,460,830)을2배 확대해 그룹 제목·구분선·간격을 비교했다. 파생물 `runtime/review-runs/projection-inspector-r107-2x.png`.

## 발견과 변경

기존 실제 r106 검사기는 깊이부터 출력까지 단일 목록으로 이어져 시안의 좌표계 및 단위/깊이 정보/행렬 및 UV/출력 설정의 시각 계층이 없었다. 공용 inspector::section으로 네 그룹을 나누고 상대 깊이·기하 단위 설명을 관련 그룹 아래에 배치했다. 패키지는 좌표계부터,2.5D는 깊이 및 투영 설정부터 시작한다. 미연결 값과 disabled 상태를 유지하며 시안의 RH/Y-up/미터 값을 복사하지 않았다. 제품 정책/API는 변경하지 않았다.

## 실제 실행

r107 PID76452, `runtime/review-runs/ui-proj-inspector-20261019-r107/player.exe`, 기존 소유 가상 모니터 위치3500,100.

- 1440×900:2.5D 제목·깊이 필드·좌표계 그룹을 확인했다. 패키지로 이동하면 좌표계 및 단위 → 깊이 정보 → 행렬 및 UV → 출력 설정 순서와 separator가 표시됐다. Inspector 내부 스크롤로 대상 도구와 amber 경고까지 도달했고 검증/저장 footer는 고정됐다.
- 760×480:설정 drawer에서 좌표계 그룹과 설명 줄바꿈을 확인했다. 내부 스크롤로 출력 설정·대상 도구·경고를 읽을 수 있었고 두 footer 버튼은 그대로 남았다. Escape 후 drawer가 닫히고 패키지 탭과 scene이 유지됐다.
- caption 닫기로 종료했다. 종료 뒤 player.exe/mpv.exe 조회가 비어 있었다. VM·DPI·모니터 구성은 변경하지 않았다.

## 검사와 미검증

검사를 겹쳐 실행해 실행 중인 subtitle_conversion 테스트 exe를 linker가 열지 못한 LNK1104가 한 번 발생했다. 두 검사 프로세스 종료를 확인한 후 최종 코드에서 cargo test를 단독 재실행해 통과했다(lib70 passed/1 ignored 및 CLI/통합 검사). cargo fmt --check, cargo clippy --all-targets -- -D warnings, cargo build --bin player도 통과했다. 외부 미디어 의존 ignored 항목은 실행하지 않았다.

시안의 도움말 기호/설명 팝업, 깊이 legend·모드·카메라, 실제 값/실제 도구 검증은 미구현이다.980/1180·긴 문구·전체 keyboard traversal·screen reader·hover/pressed와 실제 미디어는 이번에 검증하지 않았다. 구분선과 그룹 재구성의 확인을 전체 PROJ 시각 일치로 확대하지 않는다.
