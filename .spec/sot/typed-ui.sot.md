---
id: sot:kinekino/typed-ui
title: 저장 프로젝트의 얇은 typed 조회 UI
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 저장 프로젝트의 얇은 typed 조회 UI

최신 사용자 기준은 SDK/API/CLI 우선·저비용 작업 우선이며 사전 수동 GUI 검토와 시각 폴리시는 보류다. 이 문서는 `studio/projects`의 읽기 전용 연결 범위만 정의한다. [SDK 프로젝트 계약](../../application/api/projects.md)의 ApplicationClient::inspect_project를 사용하고 파일·OS API·별도 검증/저장 정책은 UI에 구현하지 않는다.

프로젝트 ID 조회는 명시 입력 후 비동기 typed 호출한다. SDK가 반환한 title/identity/revision/latest_revision/SHA/work_state와 전체 proposal snapshot ID·task를 그대로 표시한다. 최대32개 snapshot을 task별한개 편집buffer에 자동 덮어쓰지 않고 사용자가 선택한 항목만 읽기 전용으로 보여준다. 원문·생성결과·working_state·기존 acceptance는 각각 SDK값으로 구분한다. 조회 오류는 이전 표시 결과를 유지하고 현재 표시 중인 project ID를 명시한다. capability 부재는 연결 필요로 표시하며 성공 자료를 생성하지 않는다.

저장된 watching locator를 표시해도 URL/프로그램/미디어를 실행하지 않는다. stored working draft는 captured acceptance나 현재 live proposal을 대체하지 않는다. 이 첫 연결에는 create/save/rebind/edit/accept 버튼을 추가하지 않는다.

SDK 담당이 확인한 현재 미구현 경계: historical snapshot→live proposal의 원자 rebind/guard 계약이 없다. ID와revision이 같다는 이유만으로 backend/source/raw 동일성을 판단하지 않는다. 향후 기대capture/sourceSHA+revision을 mutation에서 SDK가 원자대조하는 계약 전에는 저장 snapshot을 live editor로 활성화하지 않는다. UI에서 current inspect 비교만으로 안전성을 판정하는 별도 정책도 만들지 않는다.

소스 범위는 studio/mod.rs와studio/projects/{mod,view,worker,tests}.rs다. 단독 포맷 검사는 통과했으며 컴파일/전체필수검사는 다음 중앙 조율 묶음에 포함한다. 기능 정확성은 SDK/CLI 프로젝트 정본을 사용하고 수동 GUI 클릭/스크린샷은 재개하지 않는다. 시각 미검증은 유지하며 구현과 검증 전 완료로 표시하지 않는다.

정적 입력경계 검토에서 조회창이 열려있는 동안 Escape는 조회창을 먼저 닫고 제작workspace를 유지하도록 연결했다. popup이 열려있으면 기존popup닫기우선순위는유지한다. 이 동작의 최소headless 회귀1개를 projects/tests.rs에추가했으며 실행전에는통과로보고하지않는다. 기능정책·프로젝트저장상태전이를새로구현한것이아니다.

후속 소스 검토에서 WorkingState::Absent 문구를 `별도 작업본 없음`으로 정정했다. 저장 snapshot 안의 working draft 부재를 현재 live 편집의 미저장 여부로 추정하지 않는다. 원문 전체가 길 때 조회창의 상세 높이·스크롤과 키보드 도달성은 아직 실제 실행 미검증이다. 이 조회창은 전체 시각 인수 완료 근거가 아니다.

조회 실행 버튼을 공용 Button의 Secondary/compact 조합으로 통일하고 조회창 전체의 수직 스크롤을 활성화했다. 목록만 스크롤하던 구성에서 긴 원문·원생성 결과도 창 본문으로 스크롤할 수 있도록 한 소스 보완이다. 실제 화면의 높이 제한·중첩 스크롤·포커스 도달성 검증은 여전히 남아 있다. 컴파일 및 회귀는 다음 중앙 gate에 포함한다.

위 후속 수정까지 포함한 5파일에 `rustfmt --edition 2024 --config skip_children=true --check`를 실행해 종료 코드 0을 확인했다. 이 검사는 포맷 범위이며 cargo fmt/test/strict Clippy 및 실제 GUI 인수를 대신하지 않는다. 별도 root cargo는 실행하지 않았다.

검증 로그 `runtime/quality-review/jobs-package-draft-r3/logs/root-check.log`에서 `Checking kinekino-player`와 `Finished dev profile ... in 8.61s`를 확인했다. SDK 실행 기록은 `cargo check --workspace --all-targets --offline --locked -j1` 종료 0이다. 이 선행 checkpoint의 컴파일 증거를 이후 launcher/vscroll/문구 수정까지 포함하는 증거로 확대하지 않는다. 최신 5파일의 test/strict Clippy 및 후속 컴파일은 계속 대기 상태다.
