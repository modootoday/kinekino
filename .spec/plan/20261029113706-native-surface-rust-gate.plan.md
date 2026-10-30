---
id: plan:kinekino/20261029113706
title: Native surface 소스 인벤토리와 Rust gate 계획
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native surface 소스 인벤토리와 Rust gate 계획

## 현재 실행 입력: combined30 r5

최종 고정 입력은 `runtime/coordination/sdk-native-combined-r5/source30.json` SHA4CBABEBFAC60CA7221B688E7CF3CBCA65888A5D9EEF6CBAAB9439B7066534E67이다. product29 r3 E482DDD2D138485295E245C62D1554E52DCF2052DE491AF7EB418F54DF07C1C2와 PM 승인 API unit fixture1을 합쳤다. 원29/19/11과 모든 중간 실패 핀은 보존했다. 아래 combined29는 이전 후보 이력이다.

실제 배정 root는87992b30ea474b4f71e25502d06568ae4d4f3672/Meari d669694a01a00f03f506980df127b8734b6a61d0이며 snapshot의 원98cd 소비 코드와 구분한다. before30은기존8/absent22, active snapshot 보호1262와 root907을 보존한다. 원 snapshot 보호1263은 API test 추가 배정 전 원본으로 유지한다.

원 full native101과 후속 fixture 집중101을 보존하고 전체 COMMANDS 감사를 거쳐 명시 malformed4/Unavailable72 및 실패 조건을 정확 검증하도록 API test1만 수리했다. 2026-10-21 최종 native 전체627pass/29ignored/0fail/166493ms, default 전체523pass/24ignored/0fail/156736ms, 양 strict/fmt0으로 마감했다. receipt29 전부 terminal/기록63PID 부재·input30/보호 snapshot1262/root907 delta0이다. actual 원시 근거·한계는 [combined 검증 정본](../../sots/verification/application/native-surface-observation-combined-2026-10-20.md)과 runtime/quality-review/sdk-native-combined-r1/을 따른다.

독립 parity/snapshot 시험이670ms 겹친 단일 heavy 운영 위반은 원 receipt/시각으로 기록했고 엄격 직렬 증거로 사용하지 않는다. 이후 각 원 handle 회수→terminal receipt→다음 실행 순서를 적용했다. 최종 snapshot을 보존한 채 heavy/shared snapshot 소유권을 PM에 명시 반환하며 추가 Cargo를 시작하지 않는다. GUI/actual OS probe·AI·VM/DPI 보류와 phase2 Unsupported는 유지한다.

## 최신 combined29 후보와 제품 전체 test gate

surface19와 native 수신/조회11은 독립 책임이며 원 핀을 보존한다. 교집합은 mpv/mod.rs 하나로 고유29이고 combined manifest는 `runtime/coordination/sdk-native-combined-r1/source29.json` SHA5BAC21407D206230B5F712F0D08861C8E85FC268E9C27CCA86EFFEBE3C1B4A33이다. 공유 mod는 root98cd865의 기존 내용에 양쪽 등록만 합성했다. 등록 patch9706C24208FEAEB569C0A51A037645E07BB1D9045EAF2A7248C8A609586F590C는 git apply --check0/실제 적용0이다. 다른28개는 각 원본 우선순위(surface19→관측11)의 frozen 바이트다. 아래의 과거 source19 단독 계획은 이력으로 보존한다.

현재 Meari 기존 consumer가 heavy/snapshot을 소유한다. 단계B 반환 전 Cargo/snapshot0이며 최종 실행 root/Meari/보호 manifest는 PM이 확정한다. 새 작성 시험은 surface31과 관측9, 합계40 후보를 분리한다. 기존 outer6/child7/router12, 상태/control/기존 host 회귀는 별도 이름 inventory로 확인한다. host18과 UI capture/shell12는 보호 **소스 범위**이며 이를 시험 수로 더하지 않는다.

최상위 필수 검증은 집중 회귀로 대체하지 않는다. compile/최소 수리/집중 회귀가 끝나고 입력을 마지막 고정한 뒤 **제품 kinekino-player 패키지 전체 cargo test를 default/native-proposal-window 구성으로 각각 한 번** 수행한다. 각 수정마다 전체를 반복하지 않으며 전체 gate 이후 추가 제품 수정이 생기면 영향과 재실행 필요성을 다시 판단한다. workspace의 다른 package·독립 tools 검사는 해당 책임 gate로 분리한다. 이 전체 gate에서 무관 integration target까지 빌드하는 것은 의도한 범위다.

배정 후 정확 manifest/target 절대 경로를 PM이 고정해 아래 인자를 resource runner conservative2/-j1로 실행한다. RUST_TEST_THREADS=1을 사용하고 실명 경로를 receipt에 남긴다.

```text
cargo test --offline --locked --manifest-path <absolute Cargo.toml> -p kinekino-player --target-dir <assigned absolute target> -j1 --no-run
cargo test --offline --locked --manifest-path <absolute Cargo.toml> -p kinekino-player --target-dir <assigned absolute target> -j1 --features native-proposal-window --no-run
cargo test --offline --locked --manifest-path <absolute Cargo.toml> -p kinekino-player --target-dir <assigned absolute target> -j1 -- --test-threads=1
cargo test --offline --locked --manifest-path <absolute Cargo.toml> -p kinekino-player --target-dir <assigned absolute target> -j1 --features native-proposal-window -- --test-threads=1
```

전체 실행 전 각 구성에서 빌드된 **실제 test binary**의 `--list` 및 `--ignored --list` metadata를 읽고 소스의 ignore 이유/호출 경로와 대조해 실행 명령을 고정한다. list는 시험 실행이 아니다. `--ignored` 전체 실행이나 GUI/미디어 요구 시험의 임의 해제는 금지한다. 현재 계획 단계에서 Cargo list/compile도 수행하지 않았다.

현재 소스 읽기에서 native_gui_endpoint 전체 시나리오와 native_product_launch 네 사례, browser/audio actual, jobs의 실제 provider/category/resolve/episode/native media, preserved DOM/actual-nine, 외부 자막 corpus, FFmpeg/FFprobe 실미디어 및 frame_snapshot_pipeline이 ignore로 분리됨을 확인했다. model_review의 기본 시험은 synthetic review package이며 inference 성공 증거가 아니다. CLI/LocalHost subprocess와 private ignored subprocess fixture를 명시 호출하는 소유권/timeout 회귀는 GUI 실행과 구별한다. bins의 test harness를 빌드하는 것은 player main을 실제 띄우는 실행이 아니다.

이 읽기만으로 향후 변경된 모든 시험의 무GUI를 보장하지 않는다. 최종 고정 입력의 전체 목록·실제 process 생성 경로를 다시 검토하고 비ignored 시험이 GUI/WebView/실미디어·모델을 직접 호출한다면 그 시험을 조용히 skip하지 않고 별도 배정/정확 필터와 미실행 한계를 PM에게 보고해 명령을 확정한다. DPI/VM·모델/다운로드 보류는 유지한다. 전체 pass/failed/ignored 수와 원 실패·receipt·실제 binary pin·보호 SHA/process 종료를 각 구성별로 기록한다. fmt와 default/native all-target strict도 같은 최종 입력에서 필수 인수한다.

2026-10-20 계획이며 실행 배정 전이다. 현재 host17이 heavy를 사용한다. Cargo/OS/GUI 및 공유 snapshot 변경0을 유지한다.

**중앙 상태 갱신:** 현재 root base는 `43e6d9276b912992d818661dd0d4d25eaa634b5c`다. PM 인계상 host SDK/CLI와 자막 fixture18의 고유46pass·양 strict/fmt가 완료됐다. 이 host18을 새 consumer 보호 inventory에 포함한다. 이전 eb626 및 host17 실행 중 표기는 이력이다. 현재 UI capture6+shell6 담당이 유일한 heavy/shared snapshot owner이며 SDK는 실행하지 않는다. root Meari gitlink는494 유지다. source19 및 중앙 producer d669 핀은 불변이며, 최종 실행 base는 UI 후속 commit 뒤 별도 배정에서 다시 고정한다. d669의 전체 revision/manifest는 실행 인수 시 원 근거로 확정하며 축약 ID만으로 producer를 교체하지 않는다.

## 최종 실행 후보: retention r2 포함

현재 실행 인수 후보는 `runtime/coordination/sdk-native-surface-gate-r1/source19.json`, SHA **E90B42ADDE493E00978DF8B2439CFE9FEC9FBD242C8A068268B4E6D2D2BEFC2B**다. probe11 r3 D9B11EAD…7E02 → recovery4 r2 4C6C554C…F20C → retention9 r2 C1B19191125A6E24FD535DC335E41A0C6414C062E169050EDE2415182510CC71 순서로 경로를 합성했고 원 입력은 모두 보존했다. 공통5는 retention이 대체하고 신규4가 추가되어 **고유19**다. 기존 순차 owner의 미변경3을 포함하면 관찰 경로는 **고유22**다. 아래의 15/18과 25/31/50은 retention 전 이력이다.

최종 소스는 이전15 경로와 신규 `win32/{construction_guard,construction_guard_tests}.rs`, `recovery/{probe_host,probe_host_tests}.rs`를 포함한다. retention 조정 기존5는 win32/{mod,owner,release}.rs와 recovery/{mod,registry}.rs다. 원 input5를 별도 overlay하지 않는다.

중앙 root와 합성할 기존 등록2의 대조는 `registration-review.diff` SHA **36FC40EA12462291D6F2FF77ABC6BF3456185245859AD5731968F57A4705C334**다. 기준은 a611f791이며 review용 경로를 가진 diff이므로 root에 자동 apply하지 않는다. mpv/mod.rs의 parent_surface/공개 export와 playback_surface/mod.rs의 recovery 등록만 중앙 최신 원문에 합친다. 신규 하위 모듈의 win32/mod.rs·recovery/mod.rs 등록은 frozen19 안에 포함한다.

실제 작성 시험 목록은 input9 + 원 Win32 seam8 + guard phase2 + recovery8 + host4 = **새31**이다. 기존 순차 owner6 포함37, 기존 child7/router12 추가 시 고유56 후보다. `parent_surface::`는19, `playback_surface::recovery::`는12를 예상하며 순차 owner6과 겹치는 넓은 필터는 중복 합산하지 않는다. 실제 실행 stdout/이름/0-test 여부를 정본으로 삼는다. native compile·시험·strict·OS 실행 결과는 아직 없다.

실행 배정 순서는 중앙 host17→capture6 이후 producer/SDK compile·집중 회귀 후보이며 실제 owner 배정 전에는 runner/Cargo/snapshot을 건드리지 않는다. 검증 producer2699/source403과 중앙 최신 base/보호 inventory를 새로 묶어 후보19를 인수한다. 필수 Cargo fmt/test/strict는 아래 절차를 유지하며 compile 실패는 원 로그를 보존한 별도 최소 revision으로 수리한다.

새31은 OS 성공을 검증하지 않는다. actual partial fault·callback·Destroy·parent loss, 강제 host Drop, probe와 engine unique lease 분리, whole outer lifecycle proxy 및 실제 SDK/CLI·제품 조립은 남은 제품 통합 gate다. source-only 검토를 완료 증거로 대체하지 않는다.

## 고정 입력과 중첩

| 입력 | 고정 manifest | 파일 수 | 역할 |
| --- | --- | --- | --- |
| probe r3 | runtime/coordination/sdk-win32-probe-r1/source11-r3.json, SHA D9B11EADCC2EE7666D0774BCDC8145A2DA776ACE7DD49F3E30B053FAE2E07E02 | 11 | 입력 r2 5 + Win32 6 |
| recovery r2 | runtime/coordination/sdk-surface-recovery-r1/source4-r2.json, SHA 4C6C554C23E93055045F97E288B5B0F10A0AFFAB4BE705CC90E20D5488A6F20C | 4 | 기존 playback_surface 등록1 + registry3 |
| 최초 input r1 | runtime/coordination/sdk-surface-input5-r1/pins.json, SHA 3B76BDF3CB749916631840AF30B57C28D5F5D487C6CA15436C4DCAE9D9F6A0AB | 5 | 과거 a77 의존 이력, overlay하지 않음 |
| 기존 순차 owner | root a611f791bad209f156a95d1f2e26cc2dce8499ec / source4 r2 9AAF1E25354AC7B6933A94FBA37FD9FAC69E31024F2DBDFBD4B3AB971EACD4DD | 4 | 기존 generic E→S cleanup 정책 |

새 overlay는 고유15개(11+4)다. input5는 probe11 안의 같은 경로이므로 더하지 않는다. 기존 순차 owner4까지 관찰하는 전체 경로는 고유18개이며 recovery와 기존 playback_surface/mod.rs 한 개가 겹친다. 기존 assembly/meari/mod.rs의 feature 등록과 cleanup.rs/tests.rs는 그대로 보존한다. recovery의 등록은 기존 cleanup 등록/공개 export에 `pub mod recovery`만 추가한다.

정확한 새15 경로는 다음과 같다.

- src/infrastructure/mpv/mod.rs
- src/infrastructure/mpv/parent_surface/{mod,input,geometry,tests}.rs
- src/infrastructure/mpv/parent_surface/win32/{mod,preflight,owner,window_events,release,tests}.rs
- src/assembly/meari/playback_surface/mod.rs
- src/assembly/meari/playback_surface/recovery/{mod,registry,tests}.rs

공유 root의 mpv/mod.rs와 playback_surface/mod.rs는 통째 덮지 않고 중앙 최신 원문과 등록 diff를 합성한다. recovery 등록 patch는 SHA883D1BA3D788DA9F78E39F7B982F042B4AC7FD9607C85AF12D265D8882482F4C다. 기존 CLI/PNG/자막/manual skip 변경은 인벤토리 밖 보호 대상으로 포함한다.

producer 인수 후보는 실제 gate가 끝난 2699a114342738c48a3a6a308b7daad179027098/source403 manifest8D45A4D47BFBBC5FC7428A70B7DF40E7B84C52D83ECCC9194A4EB7BB7140F942다. c21 대비 API 서명은 같으나 SDK source가 이 producer를 실제 컴파일 소비한 적은 아직 없다. dependency 인수 revision을 별도 기록하고 producer 403을 제품15와 중복 합산하지 않는다. root Meari gitlink를 임의 변경하지 않는다.

## 실행 배정 후 순서

1. 중앙 최신 base·producer·새15/기존 owner3의 실제 SHA·등록 patch·보호 파일 inventory를 확정한다. producer의 parse fixture 수정과 실제 window gate 근거를 보존한다. resource runner/profile/CPU mask/target/최대 jobs는 새 배정대로만 사용한다.
2. 실제 native feature lib compile에서 오류를 먼저 보존한다. 직접 포맷 통과를 Cargo compile로 대체하지 않는다. 첫 실패는 원 stdout/stderr/receipt와 source pin을 보존하고 최소 수리 revision으로 분리한다.
3. native feature `parent_surface::` 필터는 input9+Win32 fault seam8, `playback_surface::recovery::`는8, 기존 순차 owner는6을 예상한다. 새25/합계31은 작성 inventory일 뿐 실제 stdout의 시험 수·이름과 0-test 여부를 정본으로 삼는다. 기존 child7/router12까지 요청 시 고유50 후보이며 중복 필터 합산을 피한다.
4. 필수 `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`를 배정된 workspace/target으로 수행한다. default 검사와 native feature 검사 결과를 분리한다. feature 없이 통과해도 이 Windows native 코드를 검증한 것으로 하지 않는다. 전체 test가 중앙에서 미배정이면 집중 gate만 통과한 상태로 남기고 전체 인수를 주장하지 않는다.
5. source/보호 SHA 일치, terminal receipt, owned process 종료/기존 process 보호, 실제 test binary pin을 확인하고 heavy/snapshot을 반환한다. API/CLI 제품 동작 변화가 없는 이 gate는 actual OS Create/Destroy 성공을 증명하지 않는다.

## 후속 adapter 제안 범위

기존 r3/r2를 고정한 상태에서 제안할 최소 후속은 `parent_surface/win32/{mod,owner}.rs` 조정2 + `construction_guard.rs` 신규1, 그리고 `playback_surface/recovery/{mod,registry}.rs` 조정2 + `probe_host.rs` 신규1, 합계6 후보와 별도 회귀 파일2 후보(최대 exact8)다. 아직 구현 배정이 아니므로 생성하지 않는다.

먼저 borrowed 원 입력/parent scope 안에서 partial resource guard를 중앙에 맡기는 계약이 필요하다. OS 생성이 panic해도 registry의 Constructing tombstone만 남는 데 그치지 않고 생성된 class/HWND/callback 참조의 실제 소유자를 보존해야 한다. guard의 해체는 실제 WM_NCDESTROY/detach/class receipt에 연결하고 알 수 없는 raw Arc를 추측 회수하지 않는다.

host retention은 원 registry의 incomplete shutdown 반환을 strong host에 재보관하고 정상 parent 종료를 차단하는 경계다. registry 자체를 새 cleanup driver로 만들지 않는다. 실제 E unique lease와 probe input의 분리, 기존 PlaybackSurfaceChild 전체의 entry 보유 및 유일 lifecycle proxy는 추가 별도 조립 gate다. generic fake 시험과 실제 Windows callback/Destroy/parent loss 실험을 구별한다.
