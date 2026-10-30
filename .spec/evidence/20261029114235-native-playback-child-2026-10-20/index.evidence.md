---
id: evidence:kinekino/20261029114235
title: Native playback cleanup adapter focused 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native playback cleanup adapter focused 검증

2026-10-20. [조립 계획](../../../plans/player/native-playback-child-assembly.md)의 첫 exact4 cleanup adapter만 검증했다. 실제 OS parent/lease 생성, NativeSurfaceSlot, main, mpv/media, GUI event 소비는 미검증이다.

입력 snapshot은 `%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/public-p3-parent-r1`이다. Meari `0589b58a9e8b40be1ff10b8ad27b3dfd27f68d61` + 기존 SDK `3a754b4fee555562db6db144eed2e6b8b45f1b57`/UI11 + UI9 r5를 이어받았다. UI9 pin은 `62ECB687D7A0D99323215FF716050EF830251A04EF90B8852C1502B80C2A1CC4`, 미인수 overlay49 pin은 `D7CB58E9D2DA6C65AE771F38927AC09B71EE73CE53D47BADBD6D662C3A7B20B8`이다. 이 base에는 Inspector runtime4의 알려진 mixed-slot preflight 실패가 남아 있다. 이번 renderer 비경유 SDK 통과는 그 실패를 해소하거나 성공으로 재분류하지 않는다.

## 입력과 수리

`runtime/quality-review/sdk-native-child-r2/`에 exact4 before 존재 여부/원본, 입력핀, 보호57/19, stdout/stderr/receipt와 최종 상태를 보존했다. 신규3파일은 이전 부재였고 상위 mod 원 bytes도 보존했다. r2 동결 소스/pin은 `runtime/coordination/sdk-native-child4-r2/`이며 pin JSON SHA256은 `36B318EE936ED8734BF488438C926A090C0E8B7D61D3CF6D0AA038030AFD83A6`이다.

r2 focused5는 통과했지만 all-target strict에서 새 register closure3곳의 `result_large_err`가 발생했다. CleanupMachine의 직접 소유 task 때문에 ChildCreationFailure가 480bytes였다. `attempt: Option<Box<Attempt>>`와 Box 생성의 두 표현만 수리했다. r2 원본과 실패 기록은 보존했고 r3 원본/pin은 `runtime/coordination/sdk-native-child4-r3/`에 별도 저장했다. r3 pin JSON SHA256은 `41AA59916814AE975C81D623AF2186DABD21B7912EDF18A7BAFF0F1AC0663985`이다. 범위는 신규 playback_child mod/cleanup/tests와 상위 mod의 기능 등록 exact4 그대로다.

## 관측 결과

| 입력·검사 | 결과 | receipt elapsed_ms |
|---|---|---:|
| r2 feature-on focused | 5 passed, 0 failed/ignored | 90937 |
| r2 feature-on all-target strict | 101, 새 large Err 경고3 | 47943 |
| r3 feature-on focused | 5 passed, 0 failed/ignored, 392 filtered | 63696 |
| r3 feature-on all-target strict | 0 | 33064 |
| r3 default lib/bins strict | 0 | 21386 |
| r3 cargo fmt --check | 0 | 1759 |

focused 명령은 `cargo test -p kinekino-player --offline --locked -j 1 --features native-proposal-window --lib assembly::meari::playback_child::tests -- --test-threads=1`이다. strict는 feature-on `--all-targets`, default `--lib --bins`에 `-- -D warnings`를 적용했다. 실제 cwd guard와 절대 manifest를 사용했다. 기존 canonical Rust resource runner의 SHA256 `692C22751FAC7F9999F2D833571DAB6605AFD58E66711370494E62DD8A62C463`, affinity3/논리CPU2/BelowNormal/descendants job/threads1을 각 receipt로 확인했다.

공개 WindowLifecycle/CompletionSignal과 fake engine으로 joined 실패의 명시 retry, broker epoch 불일치, gated worker의 실제 Waker channel 통지 뒤 coordinator 재관측, 동일 attempt 중복 및 foreign/stale 입력, closed queue 거부의 owner 보존, incomplete typed receipt의 retry 후 flags/진단 보존을 확인했다. 테스트는 private NativeParentLease를 만들어내지 않았다. Outer !Send 소유와 lease/broker의 trusted assembly 연결 계약은 소스 구조이며 실제 OS 연관 검증 결과가 아니다. 명시 retry는 private 이전 task/진단을 교체하고 이미 반환된 오류 clone은 typed receipt를 보존한다.

## 최종 상태와 한계

최종 live/snapshot/source4 해시 차이0, 보호 base delta0이다. 이전 UI 입력과 최종 UI9가 겹치는 보호19 항목은 최종 overlay49 핀과 대조했으며 이전 UI 해시로 되돌리지 않았다. 6개 원실행 receipt가 모두 terminal이고 12개 launcher/child PID가 부재다. `final-owned.json=[]`, `final-protected-differences.json=[]`와 `final-source4.json`에 기록했다. root index는 사용하지 않았다.

전체 cargo test나 전체 제품 인수를 수행한 결과가 아니다. 초기 in-flight open→revoke의 이 조립 회귀, 실제 GUI parent-ready/slot/표면 종료, 실제 mpv 및 품질·시각 검증은 후속으로 남는다. DPI·VM 보류는 유지한다.
