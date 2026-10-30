---
id: plan:kinekino/20261029113812
title: 기본 제품 GUI launcher 계약 조사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 기본 제품 GUI launcher 계약 조사

## 실제 launcher7 gate 반환

기준 root a7a9ca4의 UI15를 동기화한 S admission5912에서 원 effective7을 검증했다. 양구성 strict/fmt·default/native producer 및 정확47회(고유26) 모두 통과했고 기존6 bytes/mtime·신규1 부재를 복원했다. [실제 검증 정본](../../../sots/verification/presentation/product-gui-launch-2026-10-24.md)에 BIN/목록/receipt·S5944의 runtime32 귀속·보호 감사·alias 원복 하드링크 오류와 제한 수리를 기록했다. 해당 오류의 원 자료를 보존하며 alias4 FileID 예외와 과거 known2 미귀속 때문에 all_cache_protected=false다. canonical 채택과 실제 GUI 인수는 미완료이고 PM 소유다.

## 후속 private 구현 동결

PM exact6 및 추가 caller1 승인으로 `runtime/development/product-gui-launch-r1/`의 private effective7(existing6/new1)을 작성했다. manifest SHA256 `542A097136DE0857A912CA47DEA68A73A2C88C7582DA35C5D7DBB54F30AE9E9D`, 상세 before/raw/delta·한계·작성시험은 같은 경로 `HANDOFF.md`에 있다. 추가 경로는 `src/assembly/meari/viewer_host/mount_card/tests/capture_readonly/guarded_ipc/product_surface.rs`이며 기존 호출 인수만 typed plan.project로 전환했다. UI14/후속 chooser 시험과 직접 겹침0이다.

featureful 일반실행 NativeViewer/기능없는 일반실행 Player, 명시모드·충돌·프로젝트 단일해석을 작성했다. 원 browser와 validation-position 경로는 Player로 유지한다. 신규 integration 시험10 작성, exact7 파일별 rustfmt와 patch 공백 검사 통과이며 compile/test/Clippy·실제 등록수·CLI/GUI 실행은 미검증이다. 기존 root6 원바이트/신규root부재 보존, Cargo/S/T/GUI/stage/commit/push0. 아래는 구현 전 조사와 전체 인수 계약이며 이 private 후보를 canonical 채택으로 표시하지 않는다.

2026-10-24, 조사 기준 root `c351b5d6e87120d6455944e10ff74bc134bd3171`. 읽기 전용 소스 조사와 이 계획 작성만 수행했다. 제품 코드·UI 소유 경로·Cargo·공유 S/T·GUI 실행 변경은 없다. 아래 변경 경로는 제안이며 구현·검증 완료가 아니다.

## 관측과 판정

| 주장 | 판정·실제 경로 | 확정할 검증 |
| --- | --- | --- |
| Native product-frame에 configured SDK 주입이 없다 | 반증. `src/main.rs`는 SubtitleFiles, frame capture, playback status/control 및 명시 roots의 proposal/project/review store와 read-only jobs를 구성한다. `LocalHost::start(client.clone(), …)` 한 번 뒤 NativeViewer에 같은 client와 원 LocalHost를 넘긴다. | 구성한 store의 알려진 record를 같은 typed client/LocalHost로 조회하고 원 소유 cleanup을 확인한다. |
| 기본 실행이 NativeViewer다 | 반증. `src/application/launch.rs`의 `if !…--ui-demo { return Ok(DesktopMode::Player); }`, `native_viewer.rs`의 명시 flag 검사로 기본은 Player다. main의 나머지 경로는 eframe이다. | 무인자·명시 browser·preview·native proposal·native viewer의 mode 표를 Rust 시험으로 고정한다. |
| 저장 프로젝트 진입이 없다 | 반증. `native_viewer/project.rs`가 ID/revision을 파싱하고 `product_viewer_launch/project.rs`가 `client.inspect_project`와 capture evidence를 준비한다. 현재 명시 native flag가 필요하다. | ID 없음/최신/고정 revision/없는 root/손상 evidence를 구별한다. |
| 무인자 실행이 이미 구성된 기본 저장소를 쓴다 | 반증. main의 root들은 명시 옵션이며 부재 시 None이다. 영속 기본 설정을 읽는 단계는 이 호출 경로에 없다. | root 부재 시 scan·임의 root 생성·첫 record 자동 선택이 없음을 확인한다. 영속 설정 지원은 별도 계약이 필요하다. |
| launcher 변경만으로 완전한 플레이어가 된다 | 미입증. NativeViewer의 초기 NoSelection/NoChild와 captured project 경로는 전체 catalog/provider/media/AI 지원 근거가 아니다. main에는 CLI의 catalog/native/provider 실행 host 조립 전체가 없다. | 기능별 SDK/실제 CLI 선행 근거와 Native 소비자 연결을 각각 확인한다. |

## 최소 계약

기존 SDK 구성과 단일 LocalHost/ExecutionOwner 수명을 재사용한다. 새 SDK factory, 두 번째 LocalHost, 화면별 worker 또는 기능 정책을 추가하지 않는다. root·historical verifier의 기존 검증과 원 outcome/cleanup 보고를 유지한다. 원 cleanup의 Closed 확인 없이 성공으로 처리하지 않는다.

하나의 typed launch plan이 effective mode와 `Option<InspectProject>`를 결정하도록 제안한다. 프로젝트 옵션의 문법 파싱과 effective mode 검증을 분리해 `desktop_mode → project_selection → desktop_mode` 재귀를 만들지 않는다. 조립 계층은 raw args를 다시 해석하지 않고 이미 검증된 선택을 소비한다. 선택 부재는 inspect/scan 없이 None, 선택 존재는 SDK inspect와 evidence 검증을 LocalHost/Native loop 시작 전에 수행한다.

기능을 포함한 제품 빌드의 일반 실행을 Native product-frame으로 보내는 변경과 전체 기본 플레이어 교체 인수는 구분한다. `--native-viewer` 호환과 중복·값·충돌 거부를 유지한다. `--ui-demo`, 명시 native proposal, `--start-url`/`--smoke-test`의 기존 의도를 정적으로 버리지 않는다. browser 인자의 기존 경로를 보존할지 대체 경로를 지원할지는 모드 표에 명시한 뒤 구현한다. Native 기능 없는 빌드의 명시 native 요청은 현재 Unsupported를 유지하며 조용한 fallback을 성공으로 표시하지 않는다.

현재 Cargo 기본 기능에는 `native-proposal-window`가 없다. 제품 배포 빌드의 기능 지정과 일반 Cargo 개발 빌드를 구분해야 한다. 이번 최소 소스 후보에 Cargo 변경은 포함하지 않는다. 기본 feature 변경이 필요하면 CLI까지의 영향과 패키징 계약을 별도 제안한다.

명시 root만 있는 현재 계약에서는 무인자 Native 창을 열어도 configured 저장소가 생기지 않는다. 사용 가능한 기능과 NeedsInput/Unsupported를 정직하게 표시해야 한다. 영속 기본 root·설정 마이그레이션·초기 설정 UX는 미설계이며 cwd 탐색/자동 생성으로 대체하지 않는다. chooser opt-in과 반환 후 재결합은 UI 소유 계획의 책임이다.

## 정확한 소스 후보와 소유권

| 경로 | 제안 책임 |
| --- | --- |
| `src/application/launch.rs` | typed effective launch plan·모드 결정과 기존 inline 시험 |
| `src/application/launch/native_viewer.rs` | 명시 flag 호환·기능 가용성·충돌 검사와 inline 시험 |
| `src/application/launch/native_viewer/project.rs` | 옵션 문법과 effective mode 검증 분리·revision 정책 유지 |
| `src/main.rs` | 한 번 결정된 plan dispatch·기존 configured SDK/LocalHost 전달 |
| `src/assembly/meari/product_viewer_launch/project.rs` | raw args 대신 typed 선택 소비·시작 전 준비 |
| `tests/product_gui_launch.rs` (신규 후보) | 실제 configured store/typed client 및 launcher 오류 경계의 Rust 통합 시험. 실행 가능한 fixture에 필요한 최소 범위만 작성 |

기존5+신규1 후보이며 구현 lease 전이다. `viewer_native_launch.rs`, `viewer_host/mod.rs`, `viewer_host/viewing_chooser.rs`, `return_viewing.rs`, product-frame document/TSX 및 UI14의 나머지 경로를 수정하지 않는다. assembly의 `product_viewer_launch/project.rs`는 UI의 `viewer_native_launch.rs`와 다른 경로다. UI의 chooser 등록·표시·반환 작업은 [UI 연결 제안](viewing-chooser-reachability-proposal.md)에 맡긴다.

## CLI 우선 전제와 실제 검증 순서

| 기능 | 기존 근거·현재 한계 | launcher 인수에서 필요한 것 |
| --- | --- | --- |
| proposal 목록/Open | [목록 SDK/CLI](../../../sots/verification/application/proposal-list-sdk-cli-2026-10-24.md), [Open headless](../../../sots/verification/application/proposal-chooser-open-2026-10-24.md). 목록 metadata는 target authority가 아니며 Open48은 실제 GUI 도달성 근거가 아니다. | 기존 SDK 재사용·동일 client/store 연결. 목록·Open 정책을 launcher에서 복제하지 않는다. |
| 프로젝트 저장·조회 | [프로젝트 SDK/CLI](../../../sots/verification/application/projects-2026-10-19.md). 명시 root·CAS/과거 조회와 선택 revision을 구별한다. | 선택 없음/최신/고정 revision과 저장소 오류를 시작 전에 판정한다. 기존 저장 기능 검사를 GUI에서 반복하지 않는다. |
| 회차 resolve/native playback | [native CLI](../../../sots/verification/application/native-resolution-cli-2026-10-19.md)의 r3 실제 회차 검증과 이전 미디어 없는 EOF를 구분한다. CLI의 명시 owned host가 GUI main에 자동 설치되는 것은 아니다. | 실제 media 기능을 기본 Native 지원으로 승격하기 전에 해당 SDK/CLI host와 생산 child·surface 연결 근거를 따로 확보한다. |
| 다운로드/jobs | [공통 다운로드 CLI](../../../sots/verification/production/download-product-sdk-cli-2026-10-23.md)는 로컬 fixture CLI5이며 실제 provider source host 지원이 아니다. main jobs는 read-only다. | read-only 기록과 executor 지원을 구별한다. 기본 provider 다운로드 완료로 표시하지 않는다. |
| catalog·광고차단·외부 provider·자막·제작·AI | 제품 전체 선행 SDK/CLI 범위를 유지한다. 위 일부 근거로 전체 인수를 선언하지 않는다. | 실제 지원·오류·취소·저장 증거를 기능별로 채운다. 모델 추론/다운로드와 자원 보류는 중앙 배정을 유지한다. |

1. 배정 후 먼저 Rust 모드 표 시험: 무인자, roots만, 각 명시 모드, 중복/잘못된 값, 프로젝트 ID/revision, 기능 유무의 오류와 기존 browser 의미를 고정한다. 영속 설정 부재를 fixture에서 명시한다.
2. configured SDK/실제 임시 store와 원 LocalHost를 쓰는 headless 통합으로 같은 데이터가 보이는지, 프로젝트 준비 실패가 worker 시작 전인지, 선택 부재가 inspect를 호출하지 않는지 검증한다. UI chooser의 기능 시험은 소유 gate를 참조한다.
3. 변경 범위에 맞는 SDK/CLI 회귀와 `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`를 중앙 자원 lease 아래 수행한다. default/native feature 조합, 실행 binary/hash·입력·최초 실패·수리·실제 테스트 이름을 분리 보존한다. 기존 gate를 새 실행으로 합산하지 않는다.
4. 기능 통합 뒤 별도 OS 실행 배정으로 실제 featureful `player.exe`를 무인자/명시 roots/명시 프로젝트로 시작해 선택된 route, 같은 host, 표시된 결과·오류, 원 close/cleanup을 확인한다. GUI 고유 입력·포커스·접근성·레이아웃은 이 단계에서 검토한다. headless pass를 창/시각 pass로 표시하지 않는다.
5. 전체 기본 전환은 catalog→작품→회차→provider→재생 및 자막·제작·AI의 미지원/보류 목록과 기존 eframe 기능 보존 방안을 함께 인수한다. proposal workspace 도달성만으로 제품 범위를 축소하지 않는다.

## 적용 스킬·관측 경계

`%USERPROFILE%/.codex/skills/architecture/SKILL.md`와 `context-discovery.md`를 읽고 기존 주입·수명 재사용과 최소 변경 경계를 적용했다. 요구/소유/자원 제약은 기존 사용자·PM 배정으로 확인했다.

modoo.today에서 이 작업의 코드 주장 검증 도메인으로 탐색한 `modootoday/software-qa/code-change-review-verification` 본문과 `references/verdict-table.md`를 읽었다. 본문 SHA256 `9f73865f96ec0cefcf3a4023b294400258473640df3b1d2add03dcca8192d64b`, 참조 SHA256 `9d9619a829ae24d73645ef5f7c8126c8257caeddac80800632866257008ad2a9`. 첫 표에 입력·호출 경로·반증/미입증·확정 시험을 적용했다. 이 계획의 계약 제안은 현재 구현 판정과 구별했다. 새 제품 diff가 없으므로 diff가 만든 stale 문서 판정은 없다. 과거 [기본 route 조사](proposal-default-route-gap-audit.md)의 예전 root 관측은 이후 Open/선택 구현으로 소급 수정하지 않는다.
