---
id: evidence:kinekino/20261029114530
title: 설정 포커스 수정의 별도 SDK 검사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 설정 포커스 수정의 별도 SDK 검사

2026-10-19. 순수 패키지 분리 실행본과 구분하여 UI 담당이 수정한 `src/presentation/studio/mod.rs`를 고정했다. 프로젝트 기능 소스 추가 전 검사다. 근거는 `runtime/quality-review/focus-sdk-r1/`의 source 사본·diff·pins·logs다.

workspace 테스트 **156 pass/9 ignored**, `cargo fmt --all --check`, workspace strict all-targets Clippy, bins/examples build가 모두 exit0이다. 새 `settings_escape_returns_focus_to_trigger_for_keyboard_reentry` 회귀는 Escape 이후 동일 트리거 focus와 Enter 활성화를 확인했다. 파일 사본과 현재 source SHA는 `fcd16e5ba2a195a82e89a5e361d1849048a75f1883d58abff52a46dc012a9183`으로 같다. diff 기록의 git diff --no-index exit1은 차이 존재 결과이며 cargo 실패가 아니다.

고정 GUI SHA는 `cc4b1d3208ab73820dd75435b0a446101622028c3e512f28e1c6c18dab952071`, CLI는 `357a6c399af2aab709b6ca14deba2fa1ce4cf0c4c700fd2a3678b829b6923456`이다. UI 담당에게 별도 복사 fixture의 실제 Escape→Enter·drawer 닫기 관측용으로 인계했다. 실제 GUI 검토와 시각 일치는 이 검사만으로 완료하지 않는다. 순수 분리 고정본과 근거는 [이전 checkpoint](package-extraction-2026-10-19.md)에 보존한다.
