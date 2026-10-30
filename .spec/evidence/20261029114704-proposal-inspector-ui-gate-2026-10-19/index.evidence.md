---
id: evidence:kinekino/20261029114704
title: 제안 검사기 UI 변경의 별도 컴파일·회귀 검사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제안 검사기 UI 변경의 별도 컴파일·회귀 검사

2026-10-19. UI 담당의 고정된 presentation9개 파일 변경만 기존 프로젝트 기능 checkpoint 위에 반영해 검사했다. jobs 미등록 초안은 root workspace/API/Cargo에 반영하지 않았으며 이 검사에 포함되지 않는다. 근거는 `runtime/quality-review/proposal-inspector-ui-r1/`의 source-at-audit/logs/pins/final-bin이다.

workspace **166 pass/9 ignored**, fmt --all --check, strict workspace all-targets Clippy16.53초, bins/examples build0.74초가 모두 exit0이다. 새 `long_review_keeps_save_and_accept_inside_the_fixed_footer` 회귀가 긴 목록·좁은 검사기에서 저장/채택 footer 배치를 확인했다. 실제 GUI 시각 일치를 이 renderer 검사만으로 완료하지 않는다.

GUI SHA `17b2e93b1f893dfe418eebd2d6ecc0dec50dd76879f231df2f5a1ab61590714e`, CLI `d583d9399baa609e9b5b9f57eb1656cc5a07f30e383bc10106e1418edc8c20a7`를 고정해 UI 담당에게 인계했다. UI는 별도 복사 fixture에서 배치·입력·키보드·footer·typed 상태 연결 UX를 관측한다. 기능 판정은 [공통 SDK/CLI 프로젝트 검증](projects-2026-10-19.md)을 사용하며 동일 파일/CAS/손상 기능을 GUI에서 반복 검증하도록 요구하지 않는다.
