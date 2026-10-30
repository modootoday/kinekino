---
id: plan:kinekino/20261029113818
title: P1b 소비 검증 입력과 집중 gate
status: active
created: 2026-10-29
updated: 2026-10-29
---
# P1b 소비 검증 입력과 집중 gate

2026-10-20 최신 상태: mixed1df 수리 인수 뒤 같은 consumer9 및 최종 on/off strict/fmt를 통과하고 heavy/snapshot을 PM에 반환했다. 최종 UI9r2 pin0DAE1CC3…517B/overlay22r2 0B3C2603…7EB8와 정확 결과는 [검증 정본](../../../sots/verification/presentation/native-inspector-consumer-2026-10-20.md)이 소유한다. root cwd는 `%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/public-p3-parent-r1`, bridge cwd는 하위 tools/player-ui-sdk-bridge였다. 아래 원 r5 실패·mixed 인수 절차와 명령은 각 당시의 계획으로 보존한다. 원 실패 및 이전 입력의 before/final evidence를 덮어쓰지 않았다. 다음은 PM의 gitlink/UI9 맥락별 커밋 배정이며 실제 시각 검토는 별도다.

## 현재 기준선과 mixed 수리 인수 후 재개

- 원 UI9 r5 실패 사본·inventory SHA62ECB687D7A0D99323215FF716050EF830251A04EF90B8852C1502B80C2A1CC4와 미인수 overlay49 D7CB58E9D2DA6C65AE771F38927AC09B71EE73CE53D47BADBD6D662C3A7B20B8를 그대로 보존한다. known Inspector4 실패가 있던 사실을 지우지 않는다.
- 현재 재사용 snapshot에는 이후 SDK NativeChild r3 exact4가 포함됐다. SDK 정본 native-playback-child-2026-10-20.md, source pin `runtime/coordination/sdk-native-child4-r3/`(JSON SHA41AA59916814AE975C81D623AF2186DABD21B7912EDF18A7BAFF0F1AC0663985), `runtime/quality-review/sdk-native-child-r2/final-source4.json`을 입력 인수 때 직접 대조한다. 현재 SDK commit은 아직 index 인계 중이므로 특정 commit을 추정하지 않는다. 이 포트5pass는 UI mixed 실패 해소 근거가 아니다.
- Meari mixed 수리 gate가 인수되고 PM이 입력 변경을 배정한 뒤에만0589→검증된 mixed commit의 exact delta를 확인한다. 원0589 source와 바뀌는 파일의 기존 bytes·핀, 새 사본·핀·추가/삭제 목록을 별도 revision으로 보존해 적용한다. Core/Native/paint/Web의 같은 고정 source를 사용하고 P2/predicate branch를 가져오지 않는다.
- UI9 r5는 caller 수정 없이 유지한다. SDK3a8/UI11/NativeChild r3 exact4 및 Cargo·lock·assets의 현재 identity를 보호하고 mixed library delta 밖의 차이는 중단·보고한다. 입력 핀은 원49를 소급 수정하지 않고 새로운 mixed+UI9 overlay manifest로 고정한다. 새 archive/clone이나 전체 snapshot 사본을 만들지 않는다.
- 현재는 PM 명시 실행 슬롯 전이라 snapshot 수정/Cargo/GUI0이다. 실제 SDK4 pin/새 library commit을 대조하는 단계는 인수 시 수행하며 이 계획 작성만으로 확인 완료라 표시하지 않는다.

## 재개 시 최소 실행 순서

이 gate는 기능·노드·dirty/focus·callback 수명 인수다. 실제 Native/Web 제품 렌더와 시안 확대/1:1 비교는 아직 미실행이며 [후속 시각 계획](visual-fidelity-follow-up.md)의 동일 source/상태/viewport/scale/font/assets 조건으로 중앙 별도 배정 후 진행한다. UI는 제품 시안, Meari는 공통 renderer 동등성을 소유한다. 기능 gate 성공을 시각 일치 완료로 해석하지 않는다.

현재 원 UI9는 기존 typecheck와 순수 state/mailbox3 unit을 통과했다. producer delta가 그 코드/API를 바꾸지 않고 해시도 같으면 해당 unit을 자동 반복하지 않고 원 근거를 연결한다. source/API 변화가 있으면 그 영향만 다시 검사한다. 우선 같은 실제 소비자 target을 한 번 실행해 Inspector4와 기존 frame5를 함께 확인한다. 기존 all-keyed workspace 교체의 호환성도 이5개 범위에 있다. 실패하면 원 receipt를 보존하고 실패한 원인/분기만 집중 재실행한다.

```text
cargo test --manifest-path %USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/public-p3-parent-r1/tools/player-ui-sdk-bridge/Cargo.toml --features native-proposal-window --test studio_frame_sdk --offline --locked -j1 -- --nocapture
cargo fmt --check
cargo fmt --manifest-path %USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/public-p3-parent-r1/tools/player-ui-sdk-bridge/Cargo.toml --check
cargo clippy --features native-proposal-window --all-targets --offline --locked -j1 -- -D warnings
cargo clippy --manifest-path %USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/public-p3-parent-r1/tools/player-ui-sdk-bridge/Cargo.toml --features native-proposal-window --all-targets --offline --locked -j1 -- -D warnings
cargo clippy --manifest-path %USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/public-p3-parent-r1/tools/player-ui-sdk-bridge/Cargo.toml --test original_review_sdk --test document_factory --test bootstrap_preparation --test native_pending_window_sdk --offline --locked -j1 -- -D warnings
```

실행 tool workdir를 snapshot 절대경로로 지정하고 별도 cwd guard를 확인한다. 복사·포맷·핀 작성 호출과 Cargo 실행 호출은 분리한다. 모든 명령은 고정 resource-runner/기존 C native·D consumer cache/단일 heavy 예산을 사용하고 원 handle 종료·receipt terminal·owned 종료 뒤에만 다음 명령으로 간다. 전체 root test/이전UI11/bootstrap/SDK독립5/GUI·OS는 이 재개 목록에 자동 추가하지 않는다. 최종 source·보호 입력·결과·미완료·heavy/snapshot 반환은 UI 정본에 남긴다.

## 입력 전환

SDK3a commit3a754b4fee555562db6db144eed2e6b8b45f1b57의 exact8 및 기존 UI11 총19개가 준비 전 핀과 일치했다. old Meari344도2c 고정 inventory와 일치했다. Meari2c→0589의 git name-status 정확40개를 meari-exact40-diff.json에 기록하고, 변경 전 존재 여부/bytes/핀과 새40개 사본/핀을 보존해 적용했다. 삭제 목록은 meari-exact-deletions.json=[]다. Core/Native 변경과 같은0589의 paint·Cargo/lock을 함께 소비하며 paint의 변경 없는 소스도 old344 핀으로 보호했다. 격리 P2 또는 SDK 후속assembly4는 복사하지 않았다.

UI source9는 inventory SHA088937C80F57DCC19BB4B47CAA0E5D9468442F8B09D3D2F4476C1DFB6F29E147와 일치하는 동결 사본만 적용했다. 새 mailbox1파일의 과거 부재와 기존8파일 bytes/pin을 snapshot-ui-before-pins.json 및 snapshot-ui-before/에 보존했다. constructors인 initial_panel/product_window는 기존 FrameComposition 전달을 재사용해 변경0이며, opt-in with_inspector를 호출하는 테스트 writer studio_frame_sdk.rs가9개 inventory에 포함됐다. 원 TSX Dock/frame·공통styles·SDK·OBJ는 새 source 목록에 없다.

최종 Meari40+UI9 overlay 차이0, 보호 SDK8+겹치지 않는 UI11+manifest/lock16 차이0, Meari 변경 목록 밖 old344 차이0이다. 기록은 `runtime/quality-review/native-inspector-consumer-r1/`의 snapshot-input-ready.json과 각 차이 JSON에 있다. 새 Cargo/GUI0이다.

## 배정 후 후보 순서

기존 C native cache/root와 D consumer cache/bridge를 각각 재사용하고 같은 window/debug 문맥으로 진행한다. 처음 type check는 새 Boolean exhaustive match와 constructor/API 문맥 차이를 빠르게 드러내는 범위다. Core 자체 회귀나 이전 UI11/bootstrap 전체를 자동 반복하지 않는다.

```text
cargo check -p kinekino-player --lib --features native-proposal-window --offline --locked -j1
cargo test -p kinekino-player --lib --features native-proposal-window --offline --locked -j1 presentation::web_ui_adapter::proposals::inspector_ -- --nocapture
cargo test --manifest-path tools/player-ui-sdk-bridge/Cargo.toml --features native-proposal-window --test studio_frame_sdk --offline --locked -j1 -- --nocapture
cargo fmt --check
cargo fmt --manifest-path tools/player-ui-sdk-bridge/Cargo.toml --check
cargo clippy --features native-proposal-window --all-targets --offline --locked -j1 -- -D warnings
cargo clippy --manifest-path tools/player-ui-sdk-bridge/Cargo.toml --features native-proposal-window --all-targets --offline --locked -j1 -- -D warnings
cargo clippy --manifest-path tools/player-ui-sdk-bridge/Cargo.toml --test original_review_sdk --test document_factory --test bootstrap_preparation --test native_pending_window_sdk --offline --locked -j1 -- -D warnings
```

root unit 기대 범위는 기존 InspectorViewState2+신규 mailbox1이다. bridge target은 신규 Inspector 실제SDK3+atomic rollback1 및 기존 frame SDK5, 총9개다. 마지막 off strict는 source-include의 module/type identity와 Boolean enum 영향을 집중 확인하며 off 전체 suite를 무조건 반복하지 않는다. 모든 기대 개수는 실제 결과로 확인하며0 test 실행을 성공 근거로 쓰지 않는다.

고정 runner/mask3/2CPU/BelowNormal/descendant Job/offline locked j1/RUST_TEST_THREADS1·단일 heavy이며 원 handle 종료/terminal/정확 owned family 확인 뒤 다음 실행으로 간다. 실패 source/log/receipt를 보존하고 예상 밖 ABI/feature 차이는 소리 없이 manifest를 바꾸지 않고 인계한다. full13/GUI/OS/native 영상·P2 navigation 인수는 이 gate에 포함하지 않는다.
