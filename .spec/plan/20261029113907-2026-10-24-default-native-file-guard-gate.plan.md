---
id: plan:kinekino/20261029113907
title: fileguard E3DF2660 실행 gate 제안
status: active
created: 2026-10-29
updated: 2026-10-29
---
# fileguard E3DF2660 실행 gate 제안

Status: active · 2026-10-24 · 계획 자체는 실행권이 아니다. 후속 PM 명시 배정으로 현재 SDK sole-heavy에 phases1~3만 승인됐고 filesystem3/N/C는 별도다. 이 문서는 [fixture 계약·계획](default-native-file-guard-evidence.md)의 실행 상세 정본이고 private GATE-PLAN은 이 문서만 가리킨다. source8·expected3·기존 HANDOFF를 수정하지 않는다.

## Outcome

동결 source8의 타입·실제 등록·직접 영향 순수5와 별도 filesystem3의 관측/오류/복구를 구분한 checkpoint를 만든다. 원 shared preimage와 보호 파일을 복원·감사하고 슬롯을 명시 반환한다. filesystem에서 product image 실행은0이며 N/C·spawn witness 실증은 별도 lease에 남긴다.

## Authority

| 사항 | 결정권·경계 |
| --- | --- |
| heavy와 공유 S/T, canonical 동기화 | PM의 최신 명시 lease. UI 반환 전에 실행0 |
| compiler/list/순수5 | 다음 PM CPU gate 배정에 한정. 아래 명령은 준비된 후보 |
| filesystem3·config·image 적합성 | compiler/순수 gate 뒤 별도 PM OS 배정. 기존 producer 비교만 사용 |
| 코드·기대·권한·API | E3DF2660 exact2 외 확대0. 첫 오류 보존, 정책/단언/예산 완화0 |
| 정리·반환 | SDK 원 runner/session을 유지하며 감사. 미종료면 pending 인계, 강제 종료·새 owner·추가 retry0 |

## Evidence

- 현재 source8 `E3DF2660BB762063BFC155FAD7CF3265981225D44A7CB46931CCB30206D2D94F`, expected3 `8B841C434336B4C6CF020F6CDBFCF618B6670EA7EDC104E3D776545B1E71E9E2`, delta `ED74E09E`, HANDOFF `B464EDC8`. 위치는 `runtime/development/default-native-file-guard-r1/`. private leaf2 rustfmt/check exit0와 diff --check 차이1/진단0만 확인했다. canonical existing7 bytes/mtime·new1 부재는 final-pin-audit8 `95DEE6BA`로 보존했다.
- r5 원 gate는 root76287c43/libraryf85a68d/lock8F2C1D56에서 native no-run JSON·실제25/ignored8·exact17 각1pass다. source8의 실제 결과가 아니다. 첫 fmt/type/Clippy 실패3도 원 receipt로 보존했다. [원 SOT](../../../sots/verification/presentation/default-native-window-pure17-2026-10-24.md)·반환38469809의 S6045/T589/carry136 `E9615564`는 과거 기준이며 다음 최신 UI 반환을 대체하지 않는다.
- 보존 image는 `runtime/verification/default-native-window-r2/produced-aliases/player.exe`, SHA `8D5433A7125DEE7E517ACAE1456B2A13F3B4C2266C6E0785037CA08D11FE872E`, 153160192bytes다. 원 `18-native-producer.stdout.jsonl`의 player/bin/profile.test=false/features=[native-proposal-window]/fresh=false와 `24-produced-alias-producer4.json`의 원 alias↔deps 동일 SHA/FileID 기록에 연결된다. 원 build root76287c43/Mearif85a68d/lock8F2C1D56·A14 sync를 SOT가 고정한다. fresh=false는 이번 생산이며 실행 성공 뜻이 아니다.
- 읽기 전용 `git diff --name-only 76287c43..0a07b96a`의 실제 차이는 r5 test7뿐이다. 따라서 그 두 commit 사이 제품 코드 차이는0이다. 다음 canonical 또는 transitive product/library/feature 입력의 일치는 아직 관측하지 않았다.
- 최신 반환은 UI slot `FEEB0942`, SOT `56261466`, S6081/T589/carry137 SHA `4A929E6972132055E8EC6F893E9E803000F3AD85D1817C20F11C83C08B7E7142`이며 다음 fresh 입장의 비교 기준이다. 최신 root `c9c48d20abddb110d37a810c08c3520c10e955fc`의 diff3를 읽었다. no_media/tests/end_to_end/live_retained.rs는 cfg(test) tests 계층, viewer_native_probe/live_retained.rs는 assembly/meari/mod.rs45의 cfg(all(test,windows,native-proposal-window)) 아래다. 나머지 infrastructure/mod.rs는 `mod immutable_storage_paths`→`pub(crate) mod immutable_storage_paths` 가시성1줄이며 함수/정책/런타임 branch 변화0이다. 따라서 8D image의 기존 runtime 정책 적합성은 소스상 유지된다는 추론을 제시하되, c9에서 새로 생산된 바이너리이거나 byte/codegen 동일함을 입증한 것은 아니다. readonly volume specimen으로의 최종 입장은 phase4에서 PM이 fresh SHA/identity/volume와 함께 판단하고 N/C image 인수로 확대하지 않는다.
- 스킬: modoo.today `spec-authoring/plan` 필요 참조 plan.md SHA `091d2f8d92113d9995ab143ec583529c1ef98413017f3c4d8655406d768a9e2d`, `spec-authoring/plan-authoring` SKILL.md SHA `31459a74d470660d1e32a86f640d49370ec6f0ec61e30468cf39d2a9155e22fb`를 읽고 근거/가정·권한·완료조건·rollback을 분리했다. 같은 세션의 `test-fixture-and-mock-build` 본문 SHA `a393f15f`를 재탐색해 guard 해제 후 동일 작업 성공, 미측정 시간, 실패에도 원 자원 해제를 적용했다. plan-writing의 순서·검증 기준을 적용하되 프로젝트 규칙에 따라 계획은 .spec/plans에 둔다. 기존 fixture 계약을 복제·폐기하지 않는다.

## Unverified

source8 Cargo/type/Clippy·실제28/ignored11·순수 영향5·filesystem3는 모두0이다. 실행 시간은 not measured다. Windows symlink 권한·현재 image 적합성·다음 최신 S/T/carry·config의 실제 readonly/identity/volume도 미래 입장에서 확인한다. 순수5는 witness 파일 IO 실패 주입이나 실제 process/Job/handle 생존을 입증하지 않는다. filesystem3는 원 spawn admission JSON·process section FileID·메모리 hash·N/C·전체 CLI·시각 인수를 입증하지 않는다. synchronous 파일/OS 호출의 hard deadline 보장도 없다.

## Phases

### 1. 최신 반환 인수·정확 overlay

UI의 최종 반환과 PM 명시 배정 뒤 root/library/lock·runner·S/T·carry 전체 핀/원 owner/writer를 fresh 대조한다. 기존 S=`%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/public-p3-parent-r1`, T=`.../project-9dc296c4/native`를 재사용한다. 새 target·clean·무관 graph sync·다운로드0이다.

승인 canonical delta만 정확 path별 원 S preimage를 백업하고 별도 admission baseline을 고정한다. 다음 source8의 비수정6은 공유 hash 일치만 확인하며 overlay하지 않는다. 기존 `tests/native_product_launch/default_launch.rs`와 신규 `.../default_launch/file_guard.rs` exact2의 실제 shared bytes/mtime·new 부재를 따로 보존하고 변경 bytes만 fresh mtime으로 overlay한다. private before7을 shared 복원 preimage로 쓰지 않는다. source/raw8·expected3와 canonical7 핀이 다르면 자동 덮기 없이 중지한다.

원 fixed carry·alias/known6·root/library/index/manifest/lock의 bytes/mtime/FileID와 독립 backup을 기록한다. 기존 alias에 직접 overwrite 복원하지 않는다. **완료조건:** 최신 lease·반환 일치, 승인 sync baseline과 overlay2/readonly6 보호가 분리돼 있고 writer0이다.

### 2. fmt·양 strict·producer·실제 list

모든 실행은 기존 `runtime/build-cache/resource-runner-topology/debug/resource-runner.exe` SHA `EEA9DEE6E8BCE8950E74A297098044EFF1154CB334D7E6FA2C4C622B4005488D`의 conservative2/CPU2/mask3/BelowNormal/descendant Job으로 수행한다. 각 호출 직전 RAM≥16GiB/GPUfree≥3072MiB/Cfree≥12GiB/writer0, 실제 S cwd·절대 manifest, CARGO_TARGET_DIR=T/CARGO_BUILD_JOBS=1/RUST_TEST_THREADS=1/offline을 기록한다. stdout/stderr/receipt/resource는 새 evidence run에서 create-new로 보존한다.

| 순서 | 준비된 호출 |
| --- | --- |
| fmt | cargo fmt --manifest-path S/Cargo.toml --check |
| default strict | cargo clippy --manifest-path S/Cargo.toml --offline --locked -j1 -p kinekino-player --all-targets -- -D warnings |
| native strict | 위 strict에 --features native-observer-test-support 추가 |
| producer | cargo test --manifest-path S/Cargo.toml --offline --locked -j1 -p kinekino-player --test native_product_launch --features native-proposal-window --no-run --message-format=json |

각 원 terminal0 뒤 다음으로 진행한다. compiler-artifact의 package/target/profile/features/executable/filenames/fresh를 확인하고 테스트 BIN을 새 독립 fixed 사본으로 고정해 producer와 SHA/bytes·서로 다른 FileID를 기록한다. 그 BIN의 --list 및 --list --ignored만 실행해 **실제 전체28/ignored11·원 ordinary17·expected3 정확 이름/분류**를 대조한다. default 등록0은 별도 producer/list 근거 없이 실제라고 쓰지 않는다. **완료조건:** 양 strict·producer terminal0, fixed pin·실제 목록 일치. 첫 오류에서 후속0, 원 source/진단 보존이다.

### 3. 직접 영향 ordinary5

같은 fixed BIN에서 아래 접두사 `default_launch::tests::`의 정확5만 각각 --exact --test-threads=1로 실행한다. 실제 ordinary 등록1회·각1pass/0failed/0ignored·receipt terminal0을 요구한다.

| 이름 | 직접 영향과 한계 |
| --- | --- |
| pin_binding_rejects_substituted_file_or_final_path_even_when_sha_matches | 새 actual fixture가 소비하는 원 binding 비교; 실제 FileID/교체 proof는 아님 |
| pin_requires_ordinary_local_input_and_window_checks_do_not_reresolve_it | fixture/config local 입력과 guarded spawn path 소비; 실제 reparse proof는 아님 |
| mismatched_or_invalid_image_digest_is_a_pin_error | 새 config/image SHA Pin 소비 |
| original_io_and_process_error_objects_survive_classification | 새 witness IO/fixture 오류가 사용하는 원 Failure 보존; 실제 witness IO fault 주입은 아님 |
| noforce_records_denial_without_terminating_or_consuming_the_original_family | 새 witness 실패도 기존 borrowed capture/retained 경로 소비; fake family/panic counter 범위 |

정확 명단 metadata는 `runtime/development/default-native-file-guard-r1/GATE-impact5.json` SHA `45404119BA73C09E13068DD18B53B692A5192A2BCD8277F6FF32B2ABFFDEA0B1`이다. 변경 없는 control ACL/replay/log, argv/probe, window admission/report/finish 관련 나머지12를 일괄 재실행하지 않는다. 새 상태/기능 정책 변경이 확인되면 영향 범위를 PM과 다시 고정한다. **완료조건:** 고유5 exact pass 또는 첫 실패 checkpoint. 같은 fixed BIN·원 lease를 유지한 채 PM에 결과와 phase4의 readonly image/config 입장 근거를 보고한다. 별도 OS 배정 전 fixture 실행0이다. witness의 실제 내용은 N/C 별도 검증으로 남는다.

### 4. 별도 filesystem lease·image/config 입장

앞 단계 통과만으로 OS3를 실행하지 않는다. PM 별도 배정 시 새 전용 evidence run 아래 inputs와 fixtures-parent를 만들고 fixtures-parent의 절대 경로·ordinary root→ancestor·실제 root identity를 고정한다. 기존/global/user/cache 파일은 교체 대상으로 받지 않는다.

첫 image 후보는 원8D5433 **독립 보존 사본**이다. 현재 bytes/hash/mtime/FileID를 fresh 확인하고 원 producer JSON/feature/root/library/lock·제품 transitive inputs와 최신 승인 build graph를 대조해 `source-compatible-for-readonly-fixture` 또는 `mismatch/unverified`를 기록한다. 위 c9의 cfg(test)2+내부 가시성1줄 분석을 근거로 제시하며 실제 producer root76287을 c9로 다시 표기하지 않는다. 다른 product/library 변화나 근거 없는 codegen 동등성이 필요하면 image 입장을 중지해 PM에 판단을 맡긴다. 이미 단계2가 만든 새 player image가 있어도 자동 대체/추가 compile하지 않고 별도 source/feature/producer pin 검토를 거친다. 이 phase에서 image 실행0, 기존 image의 attr/bytes/mtime 변경0이다.

새 inputs/file-guard.json은 ≤16KiB/create-new, deny_unknown_fields의 `{run_id,parent,image,image_sha256}`만 갖는다. run_id는 새 ASCII alnum/hyphen≤64자, parent는 위 절대 fixtures-parent, image는 입장된 독립 사본이다. config는 own readonly 파일로 고정하고 hash/bytes/mtime/FileID·별도 독립 backup, 실제 non-reparse 경로/ancestor를 기록한다. 환경 `NATIVE_DEFAULT_FILE_GUARD_CONFIG`/`_SHA256`는 해당 실행 process tree에만 주며 전역 환경/권한을 바꾸지 않는다. image/config/부모 보호는 OS3 전후 대조한다. A/B와 root·image의 같은 volume 및 다른 FileID는 실제 fixture setup이 다시 확인해야 하며 예상으로 통과시키지 않는다. **완료조건:** 명시 OS lease, current image 적합성·readonly config·절대 ordinary parent·fresh case3 부재·resource 입장 모두 확인.

### 5. filesystem ignored3 — 첫 실패에서 중지

동일 fixed BIN에서 expected3 `8B841C43` 순서대로 각1회 `--exact --ignored --test-threads=1`을 수행한다. 접두사는 `default_launch::file_guard::`다.

| exact 이름 | 필요한 원 관측 |
| --- | --- |
| ordinary_fixture_identity_matches_handle_and_final_path | 원 Pin handle/final-path reopen=A, 실제 동SHA B≠FileID·binding 거절 |
| file_and_parent_replacement_stays_denied_until_pin_release | 파일 rename/write-open/부모 rename 원 IO 거절, 매번 A 불변; Pin 해제 뒤 동일 작업 성공·원 FileID/bytes/이름 복구·mtime 관측 |
| ancestor_reparse_fixture_is_rejected_before_spawn | 실제 directory reparse attrs·link identity·lookup=A, 원 Stage::Pin ancestor 거절, 정확 link 제거·target A 보존 |

각 stdout/stderr/원 terminal과 exact1pass/0failed/0ignored, 해당 case-root의 events.jsonl·원 failure/partial state·root/A/B/image volume/FileID·Pin 해제/recovery를 읽는다. 계획의 argv가 실제 argv/receipt와 같아야 한다. 각 test 전 resource 입장을 반복한다. baseline `fixture_admitted=true`만으로 reparse proof가 된 것은 아니다. symlink 생성 권한 부족(예: 원 OS1314)은 harness 실패·reparse 자극 미입장/guard 미실행으로 기록해 pending으로 둔다. pass/skip·Developer Mode/권한 변경·mklink 대체·자동 새 run_id 재시도0이다. journal 실패는 최초 오류와 secondary를 구분하며 기록 부재를 성공으로 대체하지 않는다. 미종료면 원 session/runner를 유지하고 PM에 인계한다.

**완료조건:** 실제 고유3 각1pass와 원 관측/정리 또는 첫 원 실패 checkpoint. Config/고정 image와 case밖 파일의 불변도 확인한다. 작은 case workspace/raw·실패한 link/부분 경로는 보존하며 broad cleanup0이다.

### 6. 최종 감사·SOT·명시 반환

원 overlay existing1 bytes/mtime·new1 부재와 비수정6·승인 sync baseline, root/library/index/manifest/lock·fixed carry·config/image·alias-known 예외를 대조한다. S 신규 runtime/변경과 T producer/new는 원 Cargo JSON·실제 PID/receipt/같은 FileID 연결로 귀속한다. fixed1을 최신 carry에 추가하되 실제 hash/bytes/mtime/FileID를 다시 고정한다. 원 receipt/PID의 terminal·생성시각/재사용과 writer0/pending0을 구분한다. SOT에는 source·pure5·filesystem3·미입장·witness/N-C actual0을 분리한다. **완료조건:** 불일치가 해결되거나 정확 pending으로 인계되고 원 owner가 정리돼 PM에 slot receipt를 명시 반환한다. canonical 채택/stage/commit은 PM 소유다.

## Rollback

1. 단계1 승인 sync는 승인 baseline으로 유지하고 overlay2만 실제 shared preimage로 원복한다. snapshot·before·새 부재 기록은 삭제하지 않는다.
2. 단계2/3 실패 때 다음 호출0, source/원 진단/고정 BIN을 보존한다. 새 Cargo 생산물은 cache 전체 rollback/clean 대상으로 삼지 않는다. alias가 바뀌면 **복원 전** exe/pdb exact 생산자 사본·hash/mtime/FileID를 보존한 뒤 [alias 보존 준칙](../cache-alias-preservation.md)에 따라 원 bytes/mtime의 새 파일로 exact alias unlink/replace한다. 직접 overwrite0, 생산자 bytes/mtime/FileID 확인·alias FileID 변경/known2 과거 예외를 유지한다.
3. 단계4는 config/새 parent/input과 원 image 사본을 보존하고 실행 process 환경만 범위 종료한다. 이미지나 config를 실패를 감추려고 덮지 않는다. 기존 readonly attr·권한·글로벌 환경은 바꾸지 않는다.
4. 단계5 실패는 code의 원 Pin 해제와 첫 오류·실제 A/live/link 위치를 보존한다. 성공 경로만 fixture 자체 이름/abc와 정확 link 제거를 완료한다. 남은 link/부분 디렉터리는 원 identity·추가 조치를 pending으로 인계하며 자동 삭제/재귀 정리하지 않는다. forced recovery나 새 fixture를 성공 대체로 만들지 않는다.
5. 실제 실행과 원 오류는 취소할 수 없는 관측 이력이다. 단계6은 그 기록을 보존하고 불일치/pending을 명시한다. 이 계획의 작성·검토는 실행권·OS 완료·새 validation 코드 승인이 아니다.
