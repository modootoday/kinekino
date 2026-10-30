---
id: evidence:kinekino/20261029113825
title: WebView 숨김과 키보드 포커스 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# WebView 숨김과 키보드 포커스 검증

2026-10-18. 실제 조작은 UI 담당 세션이 수행했고, 원래 세션은 브라우저 인프라 소스 조사·수정과 통합 정적/테스트 검증을 담당했다. 아직 키보드 문제 전체의 해결을 확정하지 않는다.

## 관측과 가설

UI 담당은 실제 창에서 탐색·설정 마우스 클릭은 동작하지만 일부 순서에서 Esc/Down/Tab/End가 화면에 반영되지 않는다고 보고했다. 이후 같은 이전 빌드에서도 자막 TextEdit 클릭 후 문자 입력과 Tab으로 다음 SRT 버튼 이동이 성공했다고 보완했다. 따라서 모든 키보드 입력이 항상 차단된다는 설명은 근거보다 넓다. native HWND focus, egui 내부 위젯 focus, 입력 자동화 전달을 분리해서 비교해야 한다.

로컬 Wry0.57 WebView2 구현의 `set_visible`는 ShowWindow/SetIsVisible만 수행하며 `focus_parent`는 별도로 부모 HWND에 SetFocus를 호출한다. builder의 focused 기본값은 true다. 숨김만으로 부모 focus가 회복된다고 가정할 수 없다.

## 수정과 한계

브라우저 인프라 `Browser::new`에 `with_focused(false)`를 적용했다. 가시성 상태를 Cell로 보관하고 실제 visible→hidden 전환 후에만 기존 Wry `focus_parent`를 호출한다. Studio의 100ms 반복 렌더에서 무조건 포커스를 재설정하지 않는다. presentation에 새 플랫폼 API를 넣지 않았다.

최종 가시성 전환 보완 소스에서 `cargo fmt --check`, SDK target의 일반 테스트47개와 Clippy `--all-targets -- -D warnings`를 통과했다. 외부 자막 corpus test1은 기본 ignored다. 이 검사들은 Win32 실제 키보드 수신을 증명하지 않는다.

UI 담당에 수정 빌드로 탐색 클릭→설정→Esc, 방향키/Home/End, TextEdit 입력→Tab, 파일 대화상자 취소/복귀 및 다른 창 전환 순서를 비교하도록 인계했다. 재현 빈도와 실제 수신 여부 확인이 남았다. [UI 구현 기록](native-shell-implementation-2026-10-18.md)과 담당 세션의 후속 실제 검증을 함께 읽는다. 기존 성공/실패 관측을 수정 빌드의 결과로 바꾸지 않는다.
