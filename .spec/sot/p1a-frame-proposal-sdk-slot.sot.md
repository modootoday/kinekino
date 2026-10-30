---
id: sot:kinekino/p1a-frame-proposal-sdk-slot
title: P1a 원 frame·proposal SDK slot 소비
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# P1a 원 frame·proposal SDK slot 소비

2026-10-20 P1a 영향 검증 checkpoint 완료. [계획](../../../../plans/player/presentation/studio-frame-sdk-slots.md)과 Meari `node-slots.md` 최종 P1a 계약을 따른다. 전체 공통 frame/P1b Dialog/P2 route·기본 launcher·영상·전체13 시각 완료가 아니다.

`proposals/frame_composition.rs`는 원 StudioFrame+WindowChrome graph와 실제 editor/provider graph를 합쳐 검증하고, 공개 mount_with_node_slots/factory.component로 같은 초기 ModuleProgram에 editor를 발급한다. graph collision·다른 frame source는 거부한다. provider hash에는 공통 frame source도 포함한다. workspace 외 navigation/inspector/queue는 명시 Empty이며 production application7의 unsupported 표현을 삭제/우회하지 않는다.

기존 initial 준비/fullref/target·canonical editor props 대조·명시 ACK·실제 root scope·ReviewSession·registry/context·동일 SDK worker·실패/recovery/close 경로를 재사용한다. `prepare_product_frame_window`와 `InitialPanelHandoff::mount_in_frame`은 동일 내부 경로에 composition만 전달한다. 별도 문서/SDK/registry/loop를 node로 이식하지 않는다. 기존 prepare_product_window/mount의 nonframe 동작은 None composition으로 유지한다.

live projection은 기존 session.reproject_with의 새 callback export 준비 안에서 같은 Native document.node_slot→root props→atomic callback export 교체로 진행한다. callback 교체 실패는 이전 export와 document를 유지한다. same placement/key/reference의 갱신은 기존 state/useId/dirty/focus/DOM을 유지하고 `replace_frame_workspace`는 다음 checked commit에서 명시 key 교체를 검증한다. rawContent/foreign/unused old generation은 라이브러리 authority 검사를 그대로 소비한다. 별도 node ABI/registry를 만들지 않는다.

`operation_pending`은 실제 SDK transport의 active ticket 조회다. domain pending token은 inspect 실패 후 사용자의 명시 Refresh를 기다리며 남을 수 있으므로 두 상태를 동일시하지 않는다. 정상 상태의 Refresh는 원 TSX의 hidden={!refreshRequired}를 유지한다. 기존 정책을 변경하거나 숨긴 버튼을 강제 실행하지 않는다.

공통 studio.css의 workspace는 grid-column2를 명시한다. Empty navigation에서 원 auto placement가1440폭 x0/width180으로 탐색 열에 들어가는 Native layout을 재현한 뒤 수정했다. 수정 후1440은x180/w960,1180은x180/w1000,760은x48/w712이며 viewport 우측 overflow가 없다. nav/inspector 폭·breakpoint·토큰은 바꾸지 않았다. 원 CSS/fixed10 핀·SDK 이전 binary를 새 스타일 인수로 소급하지 않는다. full navigation/Dialog 조합의 실제 인수는 P1b/P2에 남는다.

실제 SDK Card/Highlight/Correction의 edit→save→inspect 전달실패→명시 Refresh→accept→inspect6명령, source/generated·자동적용false·same session·old callback 거부·workspace 교체·join은 Rust consumer3개에서 통과했다. 전달 실패는 실제 SDK inspect 실행 뒤 시험 client가 실패 응답을 전달하는 trusted seam이며 mutation 재실행은 없다. foreign/raw/옛 generation atomic 거부와 export 유지, 초기/등록 후 실패 복구는 별도 영향 시험으로 기록한다. 최종 회차/핀/대표 OS 및 미검증은 검증 정본에 이어 기록한다.
