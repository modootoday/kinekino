---
id: plan:kinekino/20261029113834
title: 기본 Native image guard 실증 최소 계획
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 기본 Native image guard 실증 최소 계획

구체적인 순서·영향5·image/config 입장·복원은 [실행 gate 제안](2026-10-24-default-native-file-guard-gate.md) 한곳에서 관리한다. 이 문서는 fixture/관측 계약을 유지하며 실행 배정을 부여하지 않는다.

2026-10-24 SDK fileguard 반환 checkpoint다. 최신 배정은 [최상단 lease](../execution-leases.md)를 따른다. root `c9c48d20abddb110d37a810c08c3520c10e955fc`/library `f85a68df7a6a8fee239d9655895ee15d5eaa5510`에서 effective8 `E3DF2660BB762063BFC155FAD7CF3265981225D44A7CB46931CCB30206D2D94F`의 fmt/양strict/no-run·실제list28/ignored11·영향ordinary5와 별도 승인 filesystem3 각1pass를 확인했다. [실제 fileguard SOT](../../../sots/verification/presentation/default-native-file-guard-2026-10-24.md)를 참조한다. 원 source3964/r2~r4 오류3·r5 pure17은 보존한다. private source/raw8·expected3 8B841C43·기존 HANDOFF는 불변이며 canonical source 채택은 PM 판단이다. 실제 N/C/pipe/GUI/image 실행은0이다.

[기본 창 계획](default-native-window-reachability.md)의 pure17 → 실제 filesystem guard → 정상 N/C 순서를 유지한다. 해당 volume의 실제 A/B FileID·거절/release/recovery·ancestor reparse는 확인했고 정상 N/C는 다음 단계다. 아래 fixture 계약은 유지하며 Save/CAS/모델/전체 CLI 기능을 반복하지 않는다.

## 고정 fixture와 책임

현재 Pin이 조사해 재사용한 기법은 `src/infrastructure/immutable_storage_paths.rs`의 ordinary local path·directory no-delete handle·file OPEN_REPARSE_POINT guard다. `mpv/trusted_executable.rs`, `historical_evidence/mod.rs` 및 `vad_job_files.rs`의 원 파일/부모 교체 거절과 release 후 성공 시험을 참고한다. private 제품 모듈을 공개하거나 mpv.exe 전용 wrapper를 다른 파일에 위장 사용하지 않는다.

현재 **source-only 배정 private 후보 exact2**는 아래와 같다. canonical7은 변경하지 않았고 비수정6·원 단언/owner/Pin 정책을 유지한다. 실제 실행 배정은 아니다.

| 경로 | 제안 책임 |
| --- | --- |
| `tests/native_product_launch/default_launch.rs` 기존1 | 조건부 file_guard 모듈 등록. N/C에서 기존 WindowsFamily::state와 원 identity·Pin snapshot을 연결한 admission witness 기록만 추가 제안 |
| `tests/native_product_launch/default_launch/file_guard.rs` 신규1 | 아래 ignored filesystem3와 전용 fixture의 경로·파일/링크 수명·before/after 근거. 순수 tests.rs와 실제 filesystem 변경을 분리 |

총 후계 입력은 기존 source7+신규1=8이지만 신규 수정 범위는 위2다. 기존 pure17·control/owner·SDK API·Cargo/dependency·기존 OS suite는 변경하지 않는다. 별도 범용 runner/검증 framework를 만들지 않는다.

fixture는 승인된 새 evidence run directory 안에 `workspace/live`, `workspace/other`, `records`만 create-new로 준비한다. 전역 temp 청소나 사용자 폴더/기존 파일/공유 cache를 변경하지 않는다. 입력 A/B는 각각 새 파일로 작성한 3바이트 `abc`이며 SHA256은 `ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad`로 고정한다. 파일명은 probe.bin이고 실행하지 않는다. 두 파일의 실제 FileID가 다름을 확인해 fixture를 입장시킨다. 하드링크·기존 alias를 복원용 사본으로 사용하지 않는다.

setup manifest에는 run ID·원 absolute/canonical fixture root·root directory identity·declared mutable paths·A/B bytes/hash/volume/FileID/final path를 보존한다. actual run image를 복사할 volume과 같은 fixture volume을 사용하고 다른 volume으로 일반화하지 않는다. 지원되지 않는 identity/권한/fixture 생성은 미입장 오류로 남기며 통과/조용한 skip으로 바꾸지 않는다.

## exact filesystem gate3 제안

각 이름의 접두사는 `default_launch::file_guard::`다. 별도 OS 배정에서 같은 fixed0AC6B446·실제28/ignored11·--exact/--ignored/--test-threads=1로 세 시험을 각1pass 확인했다. 세부 원 관측과 한계는 위 SOT로 연결한다.

### ordinary_fixture_identity_matches_handle_and_final_path

1. A/B의 고정 bytes/hash는 같고 실제 volume/FileID는 서로 다른지 관측한다. 같거나 identity가 지원되지 않으면 fixture 미입장이다.
2. 실제 Pin::open(A)의 원 file handle snapshot·final path를 기록한다. Pin::verify_bound의 원 guard/reopen identity가 같아야 한다.
3. 같은 A를 final path로 다시 연 identity는 일치하고, B의 실제 identity는 같은 SHA여도 check_binding에서 거부돼야 한다. 이 결과는 실행 이미지 section의 FileID가 아니라 실제 file handle의 결속 증거다.

### file_and_parent_replacement_stays_denied_until_pin_release

1. 새 독립 fixture A의 Pin을 유지한 채 A 파일 rename·write open, live 부모 directory rename을 각각 전용 sibling 목적지에만 시도한다. 원 결과/Windows error를 보존하며 실제 거절을 확인한다.
2. 각 시도 뒤 원 handle과 A reopen의 FileID/final path·bytes/hash가 처음과 같은지 확인한다. 예상과 달리 이동이 성공하면 첫 실패와 실제 새 위치를 보존하고 후속을 멈춘다. 전역 경로나 기존 파일로 대체하지 않는다.
3. 원 Pin과 fixture가 연 모든 probe handle을 명시 해제한 뒤 동일 부모/파일 rename·write를 같은 소유 workspace에서 성공시킨다. 이 대비로 기존 ACL/읽기 전용 파일 때문에 우연히 실패한 것을 guard 수명 효과로 세지 않는다.
4. 새 위치에서도 원 A의 FileID를 확인하고 fixture bytes와 이름만 처음 상태로 복구한다. 이 소유 fixture의 write/rename으로 바뀐 mtime은 기록한다. 사용자 파일/실제 program image/alias에 byte 덮기 복원을 적용하지 않는다.

### ancestor_reparse_fixture_is_rejected_before_spawn

1. 같은 workspace에 ordinary actual directory/A와 그곳만 가리키는 local directory symlink를 Rust `std::os::windows::fs::symlink_dir`로 만든다. 실제 reparse attribute와 정상 lookup 시 A를 가리키는 identity를 확인한다.
2. 링크를 ancestor로 포함한 요청 path에 실제 Pin::open을 호출한다. Stage::Pin 거부와 원 오류가 필요하며 child spawn은 호출하지 않는다. 링크 없는 ordinary A의 앞선 입장 증거와 함께 판정한다.
3. symlink 생성 권한이 없으면 원 setup 오류/guard 미실행을 기록하고 이 proof를 pending으로 둔다. Developer Mode/권한을 바꾸거나 임의 mklink 셸·전역 junction으로 우회하지 않는다. 다른 Rust reparse 생성 기법이 필요하면 정확 경로·책임·조건부터 별도 제안한다.
4. 링크 제거는 이 fixture가 만든 정확 link path의 reparse identity를 재확인한 뒤 link 자체만 수행한다. target을 재귀 삭제하지 않는다. ordinary target A의 identity/hash가 보존돼야 한다.

## 실제 실행 이미지 대조 — 새 product run 추가0

위 filesystem3는 비PE bytes만 사용한다. 실행 이미지 대조는 기존 N/C의 두 product run에 붙이며 별도 공격용 프로그램·추가 player run을 만들지 않는다. 실제 사용 image는 그때의 root/library/feature/artifact와 맞는 고정 GUI BIN이어야 한다. 예전8510096B 사본을 현재 b60/f85 image로 자동 승격하지 않는다.

승인된 실제 BIN을 소유 workspace의 image-A에 독립 복사하고, image-B에도 같은 hash의 별도 copy를 준비해 FileID가 다름을 관측한다. 실제 spawn 대상은 A 하나다. 원 source/private inputs·producer JSON·copy hash/bytes/FileID/mtime·최종 path를 연결하고 guard를 원 WindowsFamily 수명 내내 유지한다. B는 대조용 byte/file identity 사본이며 실행/교체 공격에 쓰지 않는다. 실제 실행 파일이나 공유 debug alias를 파괴적으로 교체하지 않는다.

현재 owned-process::spawn은 **원 process handle**로 creation time과 QueryFullProcessImageName을 확인하고 canonical requested path와 다르면 resume 전에 오류로 반환한다. Running은 그 admission을 통과한 뒤만 반환된다. 이 기존 readback을 재사용해 반환된 family.state/원 identity, guard-derived spawn path, spawn 전후 실제 file handle identity를 N/C의 raw witness로 연결하는 최소 기록을 제안한다. PID를 다시 열어 새 owner를 만들지 않는다.

기록은 `requested_final_path`, `pin_before/after`, `original_process_identity`, `returned_launch_state`, `original_spawn_readback_checked`, `process_section_file_id_observed=false`를 구분한다. cached ProcessIdentity.executable을 독립적인 새 image query라고 이름 붙이지 않는다. guard3의 namespace 수명 증거와 원 spawn admission 경로를 결합한 비교이며 process memory hash/section object의 직접 FileID attestation은 아니다.

만약 독립된 실제 OS image path의 재조회 값을 그대로 보존하는 것이 필수 인수 조건이면 현재 public WindowsFamily에는 그 raw query 반환 port가 없다. 그때만 `packages/owned-process/src/infrastructure/windows/mod.rs`의 **원 handle을 빌리는 readonly image observation**과 현재 NoForce adapter/launch witness의 정확 변경을 별도 제안해야 한다. 현재 이 제품/공통 API 경로 변경은 배정하거나 구현하지 않는다.

## 소유·실패·정리 기준

- 모든 mutable 대상은 create-new manifest의 workspace 안 ordinary 파일/전용 링크다. 절대 경로와 root identity를 검증하고 전역·사용자·cache paths를 교체 대상으로 받지 않는다.
- filesystem3에는 child/SDK worker/window가 없다. 원 Pin/probe handle을 유지한 관측과 해제 뒤 복구를 별도로 기록한다. 실패 시 fixture/raw/부분 이동 상태를 보존하고 소유 handle을 명시 해제한다. evidence를 지우거나 성공처럼 원복하지 않는다.
- 성공해도 작은 workspace/records를 보존한다. 재귀 정리나 glob 삭제를 수행하지 않는다. 남은 link/부분 파일은 정확 identity와 다음 조치 대상으로 명시하며 다른 경로를 탐색·삭제하지 않는다.
- N/C에는 기존 host가 원 WindowsFamily/Supervisor/Incomplete를 유지한다. 첫 실패에서 후속을 중지하고 PM이 원 live control 주소로 State/정리 결정을 확인한다. Finish는 원 carrier가 남으면 거부하고 별도 명시 Abort를 normal close로 세지 않는다.
- source13+4의 순수 통과, 실제 filesystem3, N/C의 창/LocalHost/Closed는 서로 다른 근거다. 합산해 제품 기능 전체·시각 일치·CLI 미지원 완료로 표시하지 않는다.

적용 스킬은 같은 세션에 읽은 modoo.today `test-fixture-and-mock-build`의 원 fixture 수명/해제 후 복구·한계 기록과 `code-change-review-verification`의 코드상 readback/미입증 actual 구분이다. 정확 출처/SHA와 실제 적용은 `runtime/development/default-native-file-guard-r1/HANDOFF.md` 및 위 실제 SOT에 기록했다. UI B4는 최초 source-only 작성 시점 배정 이력이며 현재 자원은 최상단 lease를 따른다. Rust 정책은 유지한다.
