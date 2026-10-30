---
id: plan:kinekino/20261029113928
title: 저장 참조 표시의 집중 검증 입력
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 저장 참조 표시의 집중 검증 입력

2026-10-20 PM 배정 대기 계획이다. 아래 명령은 아직 실행하지 않았다. SDK P3 집중5/strict/fmt는 별도 입력에서 완료됐으며 이번 표시10+fixture1 결과로 전이하지 않는다. 새 API·GUI click/OS·mpv·추론·실사이트 검증은 추가하지 않는다.

## 입력과 경계

표시10은 commit `298f023706a540b6910d748d489f2fde463d217d`, inventory `ACD644528C7DF7FE40332C9A59124072C914EDA7A1E4FD02AF24E595B04B73F2`다. fixture1 `tools/player-ui-sdk-bridge/tests/original_review_sdk.rs`는 after SHA `E19500CA19E233067F92CF95C21803114C928ADDD2A6119E2083E7C67466DD2C`이고 commit 슬롯 대기다. 신규 OBJ7/미등록 projection은 포함하지 않는다. Inspector draft3이 현재 등록돼 있는 사실은 baseline에 기록하되 이번 표시 기능 지원의 근거로 삼지 않는다.

검증 전 중앙에서 실제 기준 snapshot과 root/bridge Cargo·lock·Meari/로컬 package·asset 및 SDK P3 parent5 포함 여부를 고정한다. 기존 UI P1a나 SDK 공개 snapshot에 새 소스를 무기록 덮어쓰기하지 않는다. Rust source include를 사용하는 bridge off/on 문맥을 구분한다. 공개 계약/fixture의 현재 경로는 SDK 소유 인계본을 그대로 소비한다.

## 집중 확인

입력 준비 완료: 실제 root cwd는 `%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/public-p3-parent-r1`, bridge cwd는 그 아래 `tools/player-ui-sdk-bridge`다. 기존 snapshot을 ui-historical-display-r1 overlay revision으로 재사용했다. root archive1fd8ca1+기존 공개 overlay4+P3 a50905e의5파일을 유지하고 UI11만 적용했다. Meari344는2c08832 최종 inventory와 차이0, manifest/lock16개와 기존 packages/assets는 상속한다. OBJ/새 SDK wake/격리 Boolean 소스는 복사하지 않았다.

처음 Git blob raw 비교에서769파일 중61개 차이를 발견해 적용을 중단했다. 원archive SHA는 기존 핀과 같았고 .gitmodules 및 TSX/JSON 등61개에 대해 필터 적용 blob은 전부 기준 commit과 같았다. 더 강하게 기존 archive의 해당61 member를 native byte pipeline으로 직접 해시해 snapshot raw bytes와61/61 일치함을 확인했다. Git blob 대비 export archive의 행 끝 표현 차이이며 추출 후 내용 drift가 아니다. 최초 실패/diff는 삭제하지 않았고 PM 인수 후 준비를 재개했다. 줄 끝 재작성·새 archive 생성은 없었다.

`runtime/quality-review/historical-source-display-r1/`의 snapshot-overlay-before/final과 각각 pins는 덮어쓴 정확11개만 보존한다. snapshot-input-ready.json은 실제 cwd/commit/overlay identity를 기록한다. P3 기존5+공개 overlay4의 SHA9개와 P3 commit blob5가 일치했고, Meari344/manifest16 관측은 별도 JSON에 있다. 최종 overlay11은 전부 예정 SHA와 같다. 기존 전체 snapshot archive나 P3/public 원증거를 변경·복제하지 않았다. 실행 배정 전까지 입력동결, Cargo/GUI0이다.

fixture 후속 커밋은 `8d15f6700c8fcbff826716d2f097836fb197f203`이다. exact1/cached check/해시 일치·staged 잔여0을 확인하고 index를 PM에 반환했다. 위 작성 당시의 commit 대기 상태는 해소됐으며 기능 gate는 여전히 미실행이다. 전체 overlay11 inventory SHA는 `EEBA919F40F1F98853A743C461B8607CFD8B1DC8BC044F404E22C4D38AFC0AA0`이다. SDK exact5 `a50905e`가 root 이력에 존재하는 사실과 UI 집중 gate의 base snapshot에 포함되는지는 별도로 기록한다.

1. root feature-on `bootstrap::tests`는 기존 initial 선택/원문·v3 거부 회귀와 새 `historical_source_labels_are_shared_without_claiming_verification`을 실행한다. 새 시험은 세 task×참조 유무6경우의 실제 initial/live builder와 unknown flags/DTO 불변을 대조한다.
2. `original_review_sdk`의 원 Card/Highlight imported graph2개는 공통 ProposalSource의 필수 props 추가와 실제 SDK edit/accept/refresh를 확인한다. 단순 legacy literal fixture가 누락 props로 실패하는지 함께 확인한다.
3. `studio_frame_sdk`의 기존5개는 교정 포함3provider와 같은 document/SDK/atomic callback·실패 복구 영향을 확인한다. 이미 통과한 이전 bytes를 반복하는 목적이 아니라 새 source10이 포함된 graph의 영향을 검증하는 범위다. 기존 layout/OS 검증은 이번 pure display 수정으로 자동 재실행하지 않는다.
4. 최종 snapshot의 fmt/strict 및 프로젝트 필수 test 범위는 PM이 한 번 배정한다. SDK P3 자체 집중5를 UI가 중복 실행하지 않는다.

후보 명령은 순차 실행이다. root는 기존 C native cache, bridge는 기존 D consumer cache를 사용하며 feature/target 문맥을 기록한다. 모든 명령은 PM이 배정한 resource-runner/affinity3/2CPU/BelowNormal/job/단일 heavy를 거치고 RUST_TEST_THREADS=1이다.

```text
cargo test --lib --features native-proposal-window --offline --locked -j1 presentation::web_ui_adapter::proposals::bootstrap::tests -- --nocapture
cargo test --manifest-path tools/player-ui-sdk-bridge/Cargo.toml --features native-proposal-window --test original_review_sdk --test studio_frame_sdk --offline --locked -j1 -- --nocapture
cargo fmt --check
cargo fmt --manifest-path tools/player-ui-sdk-bridge/Cargo.toml --check
cargo clippy --features native-proposal-window --all-targets --offline --locked -j1 -- -D warnings
cargo clippy --manifest-path tools/player-ui-sdk-bridge/Cargo.toml --features native-proposal-window --all-targets --offline --locked -j1 -- -D warnings
```

feature-off 직접 source-include 영향은 `original_review_sdk`의 focused strict만 필요 여부를 PM과 결정한다. off/on 전체 suite·root 전체 suite를 동일하게 무조건 반복하지 않는다. 최종 통합 입력의 프로젝트 필수 검사가 아직 없다면 그 범위를 별도 기록해 누락하지 않는다.

매 실행은 원 handle 종료/terminal receipt/정확 owned family 종료를 확인한 뒤 다음 Cargo로 진행한다. timeout은 재시작 근거가 아니다. 실패하면 원 source/log/receipt를 보존하고 영향 수리를 새 revision으로 만든다. 결과 정본은 historical-source-display-2026-10-20.md이며 미확인 검증/사람 quality를 시험 녹색으로 true로 승격하지 않는다.
